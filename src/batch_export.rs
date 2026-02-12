/// 分批异步导出功能模块
///
/// 提供大数据量表格的分批处理功能，避免阻塞主线程
/// 支持合并单元格（colspan/rowspan）
use crate::core::find_table_element;
use crate::resource::UrlGuard;
use crate::utils::is_element_hidden;
use crate::validation::{ensure_extension, validate_filename};
use csv::Writer;
use std::collections::HashMap;
use std::io::Cursor;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    Blob, HtmlAnchorElement, HtmlTableCellElement, HtmlTableRowElement, HtmlTableSectionElement,
    Url,
};

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

/// 分批异步导出 HTML 表格到 CSV 文件
///
/// 这个函数将表格数据分批处理，在批次之间让出控制权给浏览器事件循环，
/// 从而避免在处理大量数据时阻塞主线程导致页面卡死。
/// 支持合并单元格（colspan/rowspan）的正确处理。
///
/// # 参数
/// * `table_id` - 要导出的 HTML 表格元素的 ID
/// * `tbody_id` - 可选的数据表格体 ID（用于分离表头和数据）
/// * `filename` - 可选的导出文件名（可选，默认为 "table_export.csv"）
/// * `batch_size` - 每批处理的行数（默认 1000）
/// * `exclude_hidden` - 可选，是否排除隐藏的行和列（默认为 false）
/// * `progress_callback` - 进度回调函数，接收进度百分比 (0-100)
///
/// # 返回值
/// * `Promise<void>` - 异步操作的 Promise
///
/// # 示例
/// ```javascript
/// import { export_table_to_csv_batch } from './pkg/excel_exporter.js';
///
/// await export_table_to_csv_batch(
///     'my-table',
///     'my-tbody',  // 可选的 tbody ID
///     'data.csv',
///     1000,  // 每批 1000 行
///     false, // 不排除隐藏行
///     (progress) => {
///         console.log(`进度: ${progress}%`);
///     }
/// );
/// ```
#[wasm_bindgen]
pub async fn export_table_to_csv_batch(
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

    // 安全地获取全局的 window 和 document 对象
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

    // 1. 获取主表格（支持直接的 table 或包含 table 的容器）
    let table_element = document
        .get_element_by_id(&table_id)
        .ok_or_else(|| JsValue::from_str(&format!("找不到 ID 为 '{}' 的元素", table_id)))?;
    let table = find_table_element(table_element, &table_id)?;
    let table_rows = table.rows();
    let table_row_count = table_rows.length() as usize;

    // 2. 获取数据表格体（如果有）
    let mut tbody_rows_collection = None;
    let mut tbody_row_count = 0;

    if let Some(tid) = tbody_id
        && !tid.is_empty()
    {
        let tbody_element = document
            .get_element_by_id(&tid)
            .ok_or_else(|| JsValue::from_str(&format!("找不到 ID 为 '{}' 的 tbody 元素", tid)))?;

        // 尝试转换为 HtmlTableSectionElement (tbody)
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

    // 创建 CSV 写入器
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

    // 报告初始进度
    if let Some(ref callback) = progress_callback {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0));
    }

    // 用于追踪被 rowspan 占用的位置: (row, col) -> cell_text
    let mut rowspan_tracker: HashMap<(usize, usize), String> = HashMap::new();

    // 分批处理数据
    let mut current_row = 0;
    while current_row < total_rows {
        let batch_end = std::cmp::min(current_row + batch_size, total_rows);

        // 处理当前批次
        for i in current_row..batch_end {
            let row_element = if i < table_row_count {
                // 从主表格读取
                table_rows.get_with_index(i as u32)
            } else {
                // 从 tbody 读取
                if let Some(ref rows) = tbody_rows_collection {
                    rows.get_with_index((i - table_row_count) as u32)
                } else {
                    None
                }
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

            // col_idx: 实际输出列位置（考虑 colspan/rowspan 后的逻辑位置）
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

                // 处理 rowspan: 将当前单元格内容预填到后续行的对应位置
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

                // 处理 colspan: 当前单元格内容 + 空白填充
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

            // 安全地将行数据写入 CSV
            wtr.write_record(&row_data)
                .map_err(|e| JsValue::from_str(&format!("写入 CSV 数据失败: {}", e)))?;
        }

        current_row = batch_end;

        // 报告进度
        if let Some(ref callback) = progress_callback {
            let progress = (current_row as f64 / total_rows as f64) * 100.0;
            let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress));
        }

        // 在批次之间让出控制权
        if current_row < total_rows {
            yield_to_browser().await?;
        }
    }

    // 安全地完成 CSV 写入
    wtr.flush()
        .map_err(|e| JsValue::from_str(&format!("完成 CSV 写入失败: {}", e)))?;

    // 获取 CSV 数据
    let csv_data = wtr
        .into_inner()
        .map_err(|e| JsValue::from_str(&format!("获取 CSV 数据失败: {}", e)))?;

    if csv_data.get_ref().is_empty() {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    // 安全地创建 Blob 对象
    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("text/csv;charset=utf-8");

    let array = js_sys::Array::of1(&js_sys::Uint8Array::from(&csv_data.get_ref()[..]));
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_property_bag)
        .map_err(|e| JsValue::from_str(&format!("创建 Blob 对象失败: {:?}", e)))?;

    // 安全地创建下载链接
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| JsValue::from_str(&format!("创建下载链接失败: {:?}", e)))?;

    // 使用 RAII 模式确保 URL 资源释放
    let _url_guard = UrlGuard::new(&url);

    // 设置文件名（默认为 table_export.csv）
    let final_filename = filename.unwrap_or_else(|| "table_export.csv".to_string());

    // 验证文件名安全性
    if let Err(e) = validate_filename(&final_filename) {
        return Err(JsValue::from_str(&format!("文件名验证失败: {}", e)));
    }

    let final_filename = ensure_extension(&final_filename, "csv");

    let anchor = document
        .create_element("a")
        .map_err(|e| JsValue::from_str(&format!("创建下载链接元素失败: {:?}", e)))?;
    let anchor = anchor
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| JsValue::from_str("创建的元素不是有效的锚点元素"))?;

    anchor.set_href(&url);
    anchor.set_download(&final_filename);

    // 触发下载
    anchor.click();

    Ok(JsValue::UNDEFINED)
}

/// 让出控制权给浏览器事件循环
///
/// 使用 setTimeout(0) 创建一个微任务，允许浏览器处理其他事件
async fn yield_to_browser() -> Result<(), JsValue> {
    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let window = web_sys::window().expect("无法获取 window 对象");
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0);
    });

    JsFuture::from(promise).await?;
    Ok(())
}
