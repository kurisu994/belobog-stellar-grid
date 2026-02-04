/// XLSX 分批异步导出功能模块
///
/// 提供大数据量表格的分批处理功能，避免阻塞主线程
/// 采用两阶段策略：分批读取 DOM 数据 + 同步生成 XLSX
use crate::resource::UrlGuard;
use crate::utils::is_element_hidden;
use crate::validation::{ensure_extension, validate_filename};
use rust_xlsxwriter::{Format, Workbook};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, HtmlAnchorElement, HtmlTableCellElement, HtmlTableElement, HtmlTableRowElement,
    HtmlTableSectionElement, Url,
};

/// 合并单元格区域信息
#[derive(Debug, Clone)]
struct MergeRange {
    /// 起始行索引（0-based）
    first_row: u32,
    /// 起始列索引（0-based）
    first_col: u16,
    /// 结束行索引（0-based，inclusive）
    last_row: u32,
    /// 结束列索引（0-based，inclusive）
    last_col: u16,
}

impl MergeRange {
    /// 创建新的合并区域
    fn new(first_row: u32, first_col: u16, last_row: u32, last_col: u16) -> Self {
        Self {
            first_row,
            first_col,
            last_row,
            last_col,
        }
    }
}

/// 分批导出用的表格数据结构
#[derive(Debug)]
struct BatchTableData {
    /// 二维字符串数组，表示表格数据
    rows: Vec<Vec<String>>,
    /// 合并单元格区域列表
    merge_ranges: Vec<MergeRange>,
}

impl BatchTableData {
    #[allow(dead_code)]
    fn new() -> Self {
        Self {
            rows: Vec::new(),
            merge_ranges: Vec::new(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            merge_ranges: Vec::new(),
        }
    }
}

/// 单元格跨度信息
struct CellSpan {
    /// 单元格文本内容
    text: String,
    /// 列跨度（colspan 属性值）
    colspan: u32,
    /// 行跨度（rowspan 属性值）
    rowspan: u32,
}

/// 获取单元格的跨度信息
fn get_cell_span(cell: &HtmlTableCellElement) -> CellSpan {
    let text = cell.inner_text();
    let colspan = cell.col_span().max(1);
    let rowspan = cell.row_span().max(1);
    CellSpan {
        text,
        colspan,
        rowspan,
    }
}

/// 分批异步导出 HTML 表格到 XLSX 文件
///
/// 采用两阶段策略避免阻塞主线程：
/// 1. 分批读取 DOM 数据（异步，可 yield 让出控制权）
/// 2. 同步生成 XLSX 文件（内存操作，快速）
///
/// # 参数
/// * `table_id` - 要导出的 HTML 表格元素的 ID
/// * `tbody_id` - 可选的数据表格体 ID（用于分离表头和数据）
/// * `filename` - 可选的导出文件名（默认为 "table_export.xlsx"）
/// * `batch_size` - 每批处理的行数（默认 1000）
/// * `exclude_hidden` - 可选，是否排除隐藏的行和列（默认为 false）
/// * `progress_callback` - 进度回调函数，接收进度百分比 (0-100)
///
/// # 返回值
/// * `Promise<void>` - 异步操作的 Promise
///
/// # 示例
/// ```javascript
/// import { export_table_to_xlsx_batch } from './pkg/belobog_stellar_grid.js';
///
/// await export_table_to_xlsx_batch(
///     'my-table',
///     'my-tbody',  // 可选的 tbody ID
///     'data.xlsx',
///     1000,  // 每批 1000 行
///     false, // 不排除隐藏行
///     (progress) => {
///         console.log(`进度: ${progress}%`);
///     }
/// );
/// ```
#[wasm_bindgen]
pub async fn export_table_to_xlsx_batch(
    table_id: String,
    tbody_id: Option<String>,
    filename: Option<String>,
    batch_size: Option<u32>,
    exclude_hidden: Option<bool>,
    progress_callback: Option<js_sys::Function>,
) -> Result<JsValue, JsValue> {
    // 输入验证
    if table_id.is_empty() {
        return Err(JsValue::from_str("表格 ID 不能为空"));
    }

    let batch_size = batch_size.unwrap_or(1000) as usize;
    let exclude_hidden = exclude_hidden.unwrap_or(false);
    if batch_size == 0 {
        return Err(JsValue::from_str("批次大小必须大于 0"));
    }

    // 报告初始进度
    if let Some(ref callback) = progress_callback {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0));
    }

    // 阶段一：分批读取 DOM 数据（0% - 80% 进度）
    let table_data = extract_table_data_batch(
        &table_id,
        tbody_id.as_deref(),
        batch_size,
        exclude_hidden,
        &progress_callback,
    )
    .await?;

    // 阶段二：同步生成 XLSX 文件（80% - 100% 进度）
    generate_and_download_xlsx(table_data, filename, &progress_callback)?;

    Ok(JsValue::UNDEFINED)
}

/// 分批异步提取表格数据
///
/// 从 DOM 中分批读取数据，每批之间让出控制权给浏览器
async fn extract_table_data_batch(
    table_id: &str,
    tbody_id: Option<&str>,
    batch_size: usize,
    exclude_hidden: bool,
    progress_callback: &Option<js_sys::Function>,
) -> Result<BatchTableData, JsValue> {
    // 安全地获取全局的 window 和 document 对象
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

    // 1. 获取主表格（通常包含表头）
    let table_element = document
        .get_element_by_id(table_id)
        .ok_or_else(|| JsValue::from_str(&format!("找不到 ID 为 '{}' 的表格元素", table_id)))?;
    let table = table_element
        .dyn_into::<HtmlTableElement>()
        .map_err(|_| JsValue::from_str(&format!("元素 '{}' 不是有效的 HTML 表格", table_id)))?;
    let table_rows = table.rows();
    let table_row_count = table_rows.length() as usize;

    // 2. 获取数据表格体（如果有）
    let mut tbody_rows_collection = None;
    let mut tbody_row_count = 0;

    if let Some(tid) = tbody_id
        && !tid.is_empty()
    {
        let tbody_element = document
            .get_element_by_id(tid)
            .ok_or_else(|| JsValue::from_str(&format!("找不到 ID 为 '{}' 的 tbody 元素", tid)))?;

        let tbody = tbody_element
            .dyn_into::<HtmlTableSectionElement>()
            .map_err(|_| {
                JsValue::from_str(&format!("元素 '{}' 不是有效的 HTML 表格部分(tbody)", tid))
            })?;

        let rows = tbody.rows();
        tbody_row_count = rows.length() as usize;
        tbody_rows_collection = Some(rows);
    }

    let total_rows = table_row_count + tbody_row_count;

    if total_rows == 0 {
        return Err(JsValue::from_str("表格为空，没有数据可导出"));
    }

    let mut table_data = BatchTableData::with_capacity(total_rows);

    // 用于追踪被 rowspan 占用的位置
    let mut rowspan_tracker: HashMap<(usize, usize), String> = HashMap::new();

    // 跟踪实际输出的行索引（因为隐藏行可能被跳过）
    let mut output_row_idx: u32 = 0;

    // 分批处理数据
    let mut current_row = 0;
    while current_row < total_rows {
        let batch_end = std::cmp::min(current_row + batch_size, total_rows);

        // 处理当前批次
        for i in current_row..batch_end {
            let row_element = if i < table_row_count {
                table_rows.get_with_index(i as u32)
            } else if let Some(ref rows) = tbody_rows_collection {
                rows.get_with_index((i - table_row_count) as u32)
            } else {
                None
            };

            let row = row_element
                .ok_or_else(|| JsValue::from_str(&format!("无法获取第 {} 行数据", i + 1)))?;

            let row = row
                .dyn_into::<HtmlTableRowElement>()
                .map_err(|_| JsValue::from_str(&format!("第 {} 行不是有效的表格行", i + 1)))?;

            // 如果需要排除隐藏行
            if exclude_hidden && is_element_hidden(&row) {
                continue;
            }

            let mut row_data = Vec::new();
            let cells = row.cells();
            let cell_count = cells.length();

            let mut col_idx: usize = 0;

            for cell_idx in 0..cell_count {
                // 处理被上方 rowspan 占用的列
                while let Some(text) = rowspan_tracker.remove(&(i, col_idx)) {
                    row_data.push(text);
                    col_idx += 1;
                }

                let cell = cells.get_with_index(cell_idx).ok_or_else(|| {
                    JsValue::from_str(&format!(
                        "无法获取第 {} 行第 {} 列单元格",
                        i + 1,
                        cell_idx + 1
                    ))
                })?;

                let cell = cell.dyn_into::<HtmlTableCellElement>().map_err(|_| {
                    JsValue::from_str(&format!(
                        "第 {} 行第 {} 列不是有效的表格单元格",
                        i + 1,
                        cell_idx + 1
                    ))
                })?;

                // 如果需要排除隐藏列
                if exclude_hidden && is_element_hidden(&cell) {
                    continue;
                }

                let span = get_cell_span(&cell);

                // 记录合并区域（仅当 colspan > 1 或 rowspan > 1 时）
                if span.colspan > 1 || span.rowspan > 1 {
                    table_data.merge_ranges.push(MergeRange::new(
                        output_row_idx,
                        col_idx as u16,
                        output_row_idx + span.rowspan - 1,
                        (col_idx + span.colspan as usize - 1) as u16,
                    ));
                }

                // 处理 rowspan
                if span.rowspan > 1 {
                    for r in 1..span.rowspan as usize {
                        for c in 0..span.colspan as usize {
                            let fill_text = if c == 0 {
                                span.text.clone()
                            } else {
                                String::new()
                            };
                            rowspan_tracker.insert((i + r, col_idx + c), fill_text);
                        }
                    }
                }

                // 处理 colspan
                row_data.push(span.text);
                for _ in 1..span.colspan {
                    row_data.push(String::new());
                }

                col_idx += span.colspan as usize;
            }

            // 处理行尾残留的 rowspan 占位
            while let Some(text) = rowspan_tracker.remove(&(i, col_idx)) {
                row_data.push(text);
                col_idx += 1;
            }

            table_data.rows.push(row_data);
            output_row_idx += 1;
        }

        current_row = batch_end;

        // 报告进度（DOM 读取阶段占 0% - 80%）
        if let Some(callback) = progress_callback {
            let progress = (current_row as f64 / total_rows as f64) * 80.0;
            let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress));
        }

        // 在批次之间让出控制权
        if current_row < total_rows {
            yield_to_browser().await?;
        }
    }

    Ok(table_data)
}

/// 同步生成 XLSX 文件并触发下载
///
/// 内存中的数据写入 XLSX 非常快，通常 < 500ms
fn generate_and_download_xlsx(
    table_data: BatchTableData,
    filename: Option<String>,
    progress_callback: &Option<js_sys::Function>,
) -> Result<(), JsValue> {
    let total_rows = table_data.rows.len();

    // 创建工作簿与工作表
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 写入所有数据
    for (i, row_data) in table_data.rows.iter().enumerate() {
        for (j, cell_text) in row_data.iter().enumerate() {
            // 检测公式：以 = 开头且长度大于 1 的内容视为公式
            if cell_text.starts_with('=') && cell_text.len() > 1 {
                worksheet
                    .write_formula(i as u32, j as u16, cell_text.as_str())
                    .map_err(|e| JsValue::from_str(&format!("写入 Excel 公式失败: {}", e)))?;
            } else {
                worksheet
                    .write_string(i as u32, j as u16, cell_text)
                    .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
            }
        }

        // 定期报告进度（XLSX 生成阶段占 80% - 95%）
        if let Some(callback) = progress_callback
            && (i % 100 == 0 || i == total_rows - 1)
        {
            let progress = 80.0 + ((i + 1) as f64 / total_rows as f64) * 15.0;
            let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress));
        }
    }

    // 应用合并单元格
    let merge_format = Format::new();
    for merge in &table_data.merge_ranges {
        worksheet
            .merge_range(
                merge.first_row,
                merge.first_col,
                merge.last_row,
                merge.last_col,
                "", // 使用空字符串，因为内容已经写入了
                &merge_format,
            )
            .map_err(|e| JsValue::from_str(&format!("合并单元格失败: {}", e)))?;
    }

    // 报告合并单元格完成进度
    if let Some(callback) = progress_callback {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(98.0));
    }

    // 将工作簿写入内存缓冲区
    let xlsx_bytes = workbook
        .save_to_buffer()
        .map_err(|e| JsValue::from_str(&format!("生成 Excel 文件失败: {}", e)))?;

    if xlsx_bytes.is_empty() {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    // 创建并下载文件
    create_and_download_xlsx(&xlsx_bytes, filename)
}

/// 创建 Excel Blob 并触发下载
fn create_and_download_xlsx(data: &[u8], filename: Option<String>) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

    // 创建 Excel Blob 对象
    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");

    let array = js_sys::Array::of1(&js_sys::Uint8Array::from(data));
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_property_bag)
        .map_err(|e| JsValue::from_str(&format!("创建 Blob 对象失败: {:?}", e)))?;

    // 创建下载链接
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| JsValue::from_str(&format!("创建下载链接失败: {:?}", e)))?;

    // 使用 RAII 模式确保 URL 资源释放
    let _url_guard = UrlGuard::new(&url);

    // 设置文件名
    let final_filename = filename.unwrap_or_else(|| "table_export.xlsx".to_string());

    // 验证文件名安全性
    if let Err(e) = validate_filename(&final_filename) {
        return Err(JsValue::from_str(&format!("文件名验证失败: {}", e)));
    }

    let final_filename = ensure_extension(&final_filename, "xlsx");

    // 创建下载链接元素
    let anchor = document
        .create_element("a")
        .map_err(|e| JsValue::from_str(&format!("创建下载链接元素失败: {:?}", e)))?;
    let anchor = anchor
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| JsValue::from_str("创建的元素不是有效的锚点元素"))?;

    anchor.set_href(&url);
    anchor.set_download(&final_filename);
    anchor.click();

    Ok(())
}

/// 让出控制权给浏览器事件循环
async fn yield_to_browser() -> Result<(), JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("无法获取 window 对象");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0);
    });

    JsFuture::from(promise).await?;
    Ok(())
}
