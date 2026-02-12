/// XLSX 分批异步导出功能模块
///
/// 提供大数据量表格的分批处理功能，避免阻塞主线程
/// 采用两阶段策略：分批读取 DOM 数据 + 同步生成 XLSX
use crate::core::{
    MergeRange, RowSpanTracker, TableData, create_and_download_xlsx, find_table_element,
    get_cell_span,
};
use crate::utils::{is_element_hidden, yield_to_browser};
use rust_xlsxwriter::{Format, Workbook};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlTableCellElement, HtmlTableRowElement, HtmlTableSectionElement};

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
        if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0)) {
            web_sys::console::warn_1(&e);
        }
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
) -> Result<TableData, JsValue> {
    // 复用 extract_table_data_batch_with_offset
    // 默认进度范围为 0.0 - 80.0 (DOM 读取阶段)
    let progress_info = progress_callback.as_ref().map(|cb| (cb.clone(), 0.0, 80.0));

    extract_table_data_batch_with_offset(
        table_id,
        tbody_id,
        batch_size,
        exclude_hidden,
        &progress_info,
    )
    .await
}

/// 同步生成 XLSX 文件并触发下载
///
/// 内存中的数据写入 XLSX 非常快，通常 < 500ms
fn generate_and_download_xlsx(
    table_data: TableData,
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
            worksheet
                .write_string(i as u32, j as u16, cell_text)
                .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
        }

        // 定期报告进度（XLSX 生成阶段占 80% - 95%）
        if let Some(callback) = progress_callback
            && (i % 100 == 0 || i == total_rows - 1)
        {
            let progress = 80.0 + ((i + 1) as f64 / total_rows as f64) * 15.0;
            if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress)) {
                web_sys::console::warn_1(&e);
            }
        }
    }

    // 应用合并单元格（需要传入首单元格文本，因为 merge_range 会覆盖内容）
    let merge_format = Format::new();
    for merge in &table_data.merge_ranges {
        let text = table_data
            .rows
            .get(merge.first_row as usize)
            .and_then(|row| row.get(merge.first_col as usize))
            .map(|s| s.as_str())
            .unwrap_or("");
        worksheet
            .merge_range(
                merge.first_row,
                merge.first_col,
                merge.last_row,
                merge.last_col,
                text,
                &merge_format,
            )
            .map_err(|e| JsValue::from_str(&format!("合并单元格失败: {}", e)))?;
    }

    // 报告合并单元格完成进度
    if let Some(callback) = progress_callback {
        if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(98.0)) {
            web_sys::console::warn_1(&e);
        }
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

/// 多工作表分批异步导出配置项（从 JS 对象解析）
struct BatchSheetConfig {
    /// 表格元素 ID
    table_id: String,
    /// 可选的 tbody ID
    tbody_id: Option<String>,
    /// 工作表名称（可选，默认为 "Sheet{idx+1}"）
    sheet_name: Option<String>,
    /// 是否排除隐藏行列
    exclude_hidden: bool,
}

/// 从 JsValue 数组解析分批导出的工作表配置列表
fn parse_batch_sheet_configs(sheets: &JsValue) -> Result<Vec<BatchSheetConfig>, JsValue> {
    // 验证输入是否为数组
    if !js_sys::Array::is_array(sheets) {
        return Err(JsValue::from_str("工作表配置必须是数组"));
    }

    let array = js_sys::Array::from(sheets);
    let length = array.length();

    if length == 0 {
        return Err(JsValue::from_str("工作表配置列表不能为空"));
    }

    let mut configs = Vec::with_capacity(length as usize);

    for i in 0..length {
        let item = array.get(i);

        // 提取 tableId（必填）
        let table_id = js_sys::Reflect::get(&item, &JsValue::from_str("tableId"))
            .ok()
            .and_then(|v| v.as_string())
            .unwrap_or_default();

        if table_id.is_empty() {
            return Err(JsValue::from_str(&format!(
                "第 {} 个工作表配置缺少 tableId",
                i + 1
            )));
        }

        // 提取 tbodyId（可选）
        let tbody_id = js_sys::Reflect::get(&item, &JsValue::from_str("tbodyId"))
            .ok()
            .and_then(|v| v.as_string());

        // 提取 sheetName（可选）
        let sheet_name = js_sys::Reflect::get(&item, &JsValue::from_str("sheetName"))
            .ok()
            .and_then(|v| v.as_string());

        // 提取 excludeHidden（可选，默认 false）
        let exclude_hidden = js_sys::Reflect::get(&item, &JsValue::from_str("excludeHidden"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        configs.push(BatchSheetConfig {
            table_id,
            tbody_id,
            sheet_name,
            exclude_hidden,
        });
    }

    Ok(configs)
}

/// 多工作表分批异步导出 HTML 表格到 XLSX 文件
///
/// 将页面上多个 HTML 表格分批异步提取后导出到同一 Excel 文件的不同工作表中
///
/// # 参数
/// * `sheets` - JS 数组，每个元素为 `{ tableId: string, tbodyId?: string, sheetName?: string, excludeHidden?: boolean }`
/// * `filename` - 可选的导出文件名（默认为 "table_export.xlsx"）
/// * `batch_size` - 每批处理的行数（默认 1000）
/// * `progress_callback` - 进度回调函数，接收进度百分比 (0-100)
///
/// # 返回值
/// * `Promise<void>` - 异步操作的 Promise
///
/// # 示例
/// ```javascript
/// import { export_tables_to_xlsx_batch } from './pkg/belobog_stellar_grid.js';
///
/// await export_tables_to_xlsx_batch(
///   [
///     { tableId: 'table1', sheetName: '订单列表', excludeHidden: true },
///     { tableId: 'table2', tbodyId: 'tbody2', sheetName: '商品列表' },
///   ],
///   'report.xlsx',
///   1000,
///   (progress) => console.log(`进度: ${progress}%`)
/// );
/// ```
#[wasm_bindgen]
pub async fn export_tables_to_xlsx_batch(
    sheets: JsValue,
    filename: Option<String>,
    batch_size: Option<u32>,
    progress_callback: Option<js_sys::Function>,
) -> Result<JsValue, JsValue> {
    // 解析工作表配置
    let configs = parse_batch_sheet_configs(&sheets)?;

    let batch_size = batch_size.unwrap_or(1000) as usize;
    if batch_size == 0 {
        return Err(JsValue::from_str("批次大小必须大于 0"));
    }

    let total_sheets = configs.len();

    // 报告初始进度
    if let Some(ref callback) = progress_callback {
        if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0)) {
            web_sys::console::warn_1(&e);
        }
    }

    // 阶段一：逐个表格分批提取数据（0% - 80% 进度）
    let mut all_sheets_data: Vec<(String, TableData)> = Vec::with_capacity(total_sheets);

    for (sheet_idx, config) in configs.iter().enumerate() {
        // 计算当前 sheet 在阶段一中的进度范围
        let sheet_progress_start = (sheet_idx as f64 / total_sheets as f64) * 80.0;
        let sheet_progress_range = 80.0 / total_sheets as f64;

        // 使用带偏移的进度回调
        let sheet_callback = progress_callback
            .as_ref()
            .map(|cb| (cb.clone(), sheet_progress_start, sheet_progress_range));

        let table_data = extract_table_data_batch_with_offset(
            &config.table_id,
            config.tbody_id.as_deref(),
            batch_size,
            config.exclude_hidden,
            &sheet_callback,
        )
        .await?;

        let sheet_name = config
            .sheet_name
            .clone()
            .unwrap_or_else(|| format!("Sheet{}", sheet_idx + 1));

        all_sheets_data.push((sheet_name, table_data));
    }

    // 阶段二：同步生成多工作表 XLSX 文件（80% - 100% 进度）
    generate_and_download_xlsx_multi(all_sheets_data, filename, &progress_callback)?;

    Ok(JsValue::UNDEFINED)
}

/// 分批异步提取表格数据（带进度偏移）
///
/// 与 extract_table_data_batch 类似，但支持进度映射到指定区间
async fn extract_table_data_batch_with_offset(
    table_id: &str,
    tbody_id: Option<&str>,
    batch_size: usize,
    exclude_hidden: bool,
    progress_info: &Option<(js_sys::Function, f64, f64)>,
) -> Result<TableData, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

    // 获取主表格（支持直接的 table 或包含 table 的容器）
    let table_element = document
        .get_element_by_id(table_id)
        .ok_or_else(|| JsValue::from_str(&format!("找不到 ID 为 '{}' 的元素", table_id)))?;
    let table = find_table_element(table_element, table_id)?;
    let table_rows = table.rows();
    let table_row_count = table_rows.length() as usize;

    // 获取数据表格体（如果有）
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

    let mut table_data = TableData::with_capacity(total_rows);
    let mut tracker = RowSpanTracker::new();
    let mut output_row_idx: u32 = 0;

    let mut current_row = 0;
    while current_row < total_rows {
        let batch_end = std::cmp::min(current_row + batch_size, total_rows);

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

            if exclude_hidden && is_element_hidden(&row) {
                continue;
            }

            let mut row_data = Vec::new();
            let cells = row.cells();
            let cell_count = cells.length();
            let mut col_idx: usize = 0;
            let u_row_idx = i as u32;

            for cell_idx in 0..cell_count {
                while let Some(text) = tracker.pop(u_row_idx, col_idx) {
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

                if exclude_hidden && is_element_hidden(&cell) {
                    continue;
                }

                let span = get_cell_span(&cell);

                // 计算实际覆盖的可见行数 (effective_rowspan)
                let mut visible_rows_covered = 0;
                if span.rowspan > 1 {
                    for r in 1..span.rowspan as usize {
                        let next_row_idx = i + r;
                        let next_row = if next_row_idx < table_row_count {
                            table_rows.get_with_index(next_row_idx as u32)
                        } else if let Some(ref rows) = tbody_rows_collection {
                            rows.get_with_index((next_row_idx - table_row_count) as u32)
                        } else {
                            None
                        };

                        if let Some(next_row) = next_row {
                            #[allow(clippy::collapsible_if)]
                            if let Ok(next_row_el) = next_row.dyn_into::<HtmlTableRowElement>() {
                                if !exclude_hidden || !is_element_hidden(&next_row_el) {
                                    visible_rows_covered += 1;
                                }
                            }
                        }
                    }
                }

                let last_row = output_row_idx + visible_rows_covered;
                let last_col = (col_idx + span.colspan as usize - 1) as u16;
                if last_col > 16383 {
                    return Err(JsValue::from_str("列数超过 Excel 限制 (16384)"));
                }

                // 记录合并区域（仅当范围覆盖多个单元格时）
                if last_row > output_row_idx || last_col as usize > col_idx {
                    table_data.merge_ranges.push(MergeRange::new(
                        output_row_idx,
                        col_idx as u16,
                        last_row,
                        last_col,
                    ));
                }

                tracker.add(u_row_idx, col_idx, &span);

                row_data.push(span.text);
                for _ in 1..span.colspan {
                    row_data.push(String::new());
                }

                col_idx += span.colspan as usize;
            }

            while let Some(text) = tracker.pop(u_row_idx, col_idx) {
                row_data.push(text);
                col_idx += 1;
            }

            table_data.rows.push(row_data);
            output_row_idx += 1;
        }

        current_row = batch_end;

        // 报告进度（使用偏移映射）
        if let Some((callback, start, range)) = progress_info {
            let local_progress = current_row as f64 / total_rows as f64;
            let progress = start + local_progress * range;
            if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress)) {
                web_sys::console::warn_1(&e);
            }
        }

        if current_row < total_rows {
            yield_to_browser().await?;
        }
    }

    Ok(table_data)
}

/// 同步生成多工作表 XLSX 文件并触发下载
fn generate_and_download_xlsx_multi(
    all_sheets_data: Vec<(String, TableData)>,
    filename: Option<String>,
    progress_callback: &Option<js_sys::Function>,
) -> Result<(), JsValue> {
    if all_sheets_data.is_empty() {
        return Err(JsValue::from_str("没有可导出的工作表数据"));
    }

    let total_sheets = all_sheets_data.len();

    let mut workbook = Workbook::new();
    let merge_format = Format::new();

    for (sheet_idx, (sheet_name, table_data)) in all_sheets_data.iter().enumerate() {
        let worksheet = workbook.add_worksheet();

        worksheet
            .set_name(sheet_name)
            .map_err(|e| JsValue::from_str(&format!("设置工作表名称失败: {}", e)))?;

        let total_rows = table_data.rows.len();

        // 写入数据
        for (i, row_data) in table_data.rows.iter().enumerate() {
            for (j, cell_text) in row_data.iter().enumerate() {
                worksheet
                    .write_string(i as u32, j as u16, cell_text)
                    .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
            }

            // 报告进度（XLSX 生成阶段占 80% - 95%，按 sheet 均分）
            if let Some(callback) = progress_callback
                && total_rows > 0
                && (i % 100 == 0 || i == total_rows - 1)
            {
                let sheet_progress_start = 80.0 + (sheet_idx as f64 / total_sheets as f64) * 15.0;
                let sheet_progress_range = 15.0 / total_sheets as f64;
                let row_progress = (i + 1) as f64 / total_rows as f64;
                let progress = sheet_progress_start + row_progress * sheet_progress_range;
                if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress)) {
                    web_sys::console::warn_1(&e);
                }
            }
        }

        // 应用合并单元格（需要传入首单元格文本，因为 merge_range 会覆盖内容）
        for merge in &table_data.merge_ranges {
            let text = table_data
                .rows
                .get(merge.first_row as usize)
                .and_then(|row| row.get(merge.first_col as usize))
                .map(|s| s.as_str())
                .unwrap_or("");
            worksheet
                .merge_range(
                    merge.first_row,
                    merge.first_col,
                    merge.last_row,
                    merge.last_col,
                    text,
                    &merge_format,
                )
                .map_err(|e| JsValue::from_str(&format!("合并单元格失败: {}", e)))?;
        }
    }

    // 报告完成进度
    if let Some(callback) = progress_callback {
        if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(98.0)) {
            web_sys::console::warn_1(&e);
        }
    }

    let xlsx_bytes = workbook
        .save_to_buffer()
        .map_err(|e| JsValue::from_str(&format!("生成 Excel 文件失败: {}", e)))?;

    if xlsx_bytes.is_empty() {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    create_and_download_xlsx(&xlsx_bytes, filename)
}
