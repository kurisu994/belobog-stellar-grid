/// 分批异步导出功能模块
///
/// 提供大数据量表格的分批处理功能，避免阻塞主线程
/// 支持合并单元格（colspan/rowspan）
use crate::core::{
    RowSpanTracker, create_and_download_csv, get_table_row, process_row_cells, resolve_table,
};
use crate::utils::{ensure_external_tbody, is_element_hidden, report_progress, yield_to_browser};
use csv::Writer;
use std::io::Cursor;
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::HtmlTableSectionElement;

/// 分批异步导出 HTML 表格到 CSV 文件
///
/// 这个函数将表格数据分批处理，在批次之间让出控制权给浏览器事件循环，
/// 从而避免在处理大量数据时阻塞主线程导致页面卡死。
/// 支持合并单元格（colspan/rowspan）的正确处理。
///
/// # 参数
/// * `table_id` - 要导出的 HTML 表格元素的 ID
/// * `tbody_id` - 可选的数据表格体 ID（用于分离表头和数据）。**注意**：此 ID 应指向**不在** `table_id` 所指表格内部的独立 `<tbody>` 元素。如果传入的 `tbody` 在 `table` 内部，会导致该部分数据被重复导出（一次作为 table 的一部分，一次作为独立 tbody）。
/// * `filename` - 可选的导出文件名（可选，默认为 "table_export.csv"）
/// * `batch_size` - 每批处理的行数（默认 1000）
/// * `exclude_hidden` - 可选，是否排除隐藏的行和列（默认为 false）
/// * `progress_callback` - 进度回调函数，接收进度百分比 (0-100)
/// * `with_bom` - 可选，是否添加 UTF-8 BOM（默认为 false）
///
/// # 返回值
/// * `Promise<void>` - 异步操作的 Promise
///
/// # 示例
/// ```javascript
/// import { export_table_to_csv_batch } from './pkg/belobog_stellar_grid.js';
///
/// await export_table_to_csv_batch(
///     'my-table',
///     'my-tbody',  // 可选的 tbody ID
///     'data.csv',
///     1000,  // 每批 1000 行
///     false, // 不排除隐藏行
///     (progress) => {
///         console.log(`进度: ${progress}%`);
///     },
///     true // 添加 BOM
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
    with_bom: Option<bool>,
) -> Result<JsValue, JsValue> {
    // 输入验证
    if table_id.is_empty() {
        return Err(JsValue::from_str("表格 ID 不能为空"));
    }

    let batch_size = batch_size.unwrap_or(1000) as usize;
    let exclude_hidden = exclude_hidden.unwrap_or(false);
    let with_bom = with_bom.unwrap_or(false);
    if batch_size == 0 {
        return Err(JsValue::from_str("批次大小必须大于 0"));
    }

    // 1. 获取主表格（支持直接的 table 或包含 table 的容器）
    let table = resolve_table(&table_id)?;
    let table_rows = table.rows();
    let table_row_count = table_rows.length() as usize;

    // 2. 获取数据表格体（如果有）
    let mut tbody_rows_collection = None;
    let mut tbody_row_count = 0;

    if let Some(tid) = tbody_id
        && !tid.is_empty()
    {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

        let tbody_element = document
            .get_element_by_id(&tid)
            .ok_or_else(|| JsValue::from_str(&format!("找不到 ID 为 '{}' 的 tbody 元素", tid)))?;

        // 运行时校验 tbody 不在目标 table 内部，防止数据重复导出
        ensure_external_tbody(&table, &table_id, &tbody_element, &tid)?;

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
        report_progress(callback, 0.0, false)?;
    }

    // 用于追踪被 rowspan 占用的位置: (row, col) -> cell_text
    let mut tracker = RowSpanTracker::new();

    // 分批处理数据
    let mut current_row = 0;
    while current_row < total_rows {
        let batch_end = std::cmp::min(current_row + batch_size, total_rows);

        // 处理当前批次
        for i in current_row..batch_end {
            let row = if i < table_row_count {
                get_table_row(&table_rows, i as u32)?
            } else if let Some(ref rows) = tbody_rows_collection {
                get_table_row(rows, (i - table_row_count) as u32)?
            } else {
                return Err(JsValue::from_str(&format!("无法获取第 {} 行数据", i + 1)));
            };

            // 如果需要排除隐藏行
            if exclude_hidden && is_element_hidden(&row) {
                continue;
            }

            let proc_result = process_row_cells(&row, i as u32, &mut tracker, exclude_hidden)?;

            // CSV 需要转义注入字符
            let safe_row: Vec<_> = proc_result
                .row_data
                .iter()
                .map(|cell| crate::utils::escape_csv_injection(cell))
                .collect();

            // 写入 CSV 记录
            wtr.write_record(safe_row.iter().map(|s| s.as_ref()))
                .map_err(|e| JsValue::from_str(&format!("写入 CSV 记录失败: {:?}", e)))?;
        }

        current_row = batch_end;

        // 报告进度
        if let Some(ref callback) = progress_callback {
            let progress = (current_row as f64 / total_rows as f64) * 100.0;
            report_progress(callback, progress, false)?;
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

    // 创建并下载 CSV 文件
    create_and_download_csv(csv_data.get_ref(), filename, with_bom)?;

    Ok(JsValue::UNDEFINED)
}
