/// 分批异步导出功能模块
///
/// 提供大数据量表格的分批处理功能，避免阻塞主线程。
/// 采用分块 Blob 片段策略：每个批次生成独立的 CSV 字节片段，
/// 最后拼接成单个 Blob 下载，降低内存峰值。
/// 支持合并单元格（colspan/rowspan）
use crate::core::{RowSpanTracker, TableRowSources, process_row_cells};
use crate::utils::{is_element_hidden, report_progress, yield_to_browser};
use csv::Writer;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

/// 分批异步导出 HTML 表格到 CSV 文件
///
/// 这个函数将表格数据分批处理，在批次之间让出控制权给浏览器事件循环，
/// 从而避免在处理大量数据时阻塞主线程导致页面卡死。
/// 支持合并单元格（colspan/rowspan）的正确处理。
///
/// **内存优化**：采用分块 Blob 片段策略，每个批次的 CSV 字节在转为
/// `Uint8Array` 后立即释放 Rust 侧内存，峰值仅为一个批次大小。
///
/// # 参数
/// * `table_id` - 要导出的 HTML 表格元素的 ID
/// * `tbody_id` - 可选的数据表格体 ID（用于分离表头和数据）。**注意**：此 ID 应指向**不在** `table_id` 所指表格内部的独立 `<tbody>` 元素。如果传入的 `tbody` 在 `table` 内部，会导致该部分数据被重复导出（一次作为 table 的一部分，一次作为独立 tbody）。
/// * `filename` - 可选的导出文件名（可选，默认为 "table_export.csv"）
/// * `batch_size` - 每批处理的行数（默认 1000）
/// * `exclude_hidden` - 可选，是否排除隐藏的行和列（默认为 false）
/// * `progress_callback` - 进度回调函数，接收进度百分比 (0-100)
/// * `with_bom` - 可选，是否添加 UTF-8 BOM（默认为 false）
/// * `strict_progress_callback` - 可选，是否严格报告进度（默认为 false）。如果为 true，则每次进度更新都会触发回调；如果为 false，则可能跳过一些更新以提高性能。
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
#[allow(clippy::too_many_arguments)]
pub async fn export_table_to_csv_batch(
    table_id: String,
    tbody_id: Option<String>,
    filename: Option<String>,
    batch_size: Option<u32>,
    exclude_hidden: Option<bool>,
    progress_callback: Option<js_sys::Function>,
    with_bom: Option<bool>,
    strict_progress_callback: Option<bool>,
) -> Result<JsValue, JsValue> {
    // 输入验证
    if table_id.is_empty() {
        return Err(JsValue::from_str("表格 ID 不能为空"));
    }

    let batch_size = batch_size.unwrap_or(1000) as usize;
    let exclude_hidden = exclude_hidden.unwrap_or(false);
    let with_bom = with_bom.unwrap_or(false);
    let strict = strict_progress_callback.unwrap_or(false);
    if batch_size == 0 {
        return Err(JsValue::from_str("批次大小必须大于 0"));
    }

    let sources = TableRowSources::open(&table_id, tbody_id.as_deref())?;
    let total_rows = sources.total_rows();

    if total_rows == 0 {
        return Err(JsValue::from_str("表格为空，没有数据可导出"));
    }

    // 报告初始进度
    if let Some(ref callback) = progress_callback {
        report_progress(callback, 0.0, strict)?;
    }

    // 用于追踪被 rowspan 占用的位置: (row, col) -> cell_text
    let mut tracker = RowSpanTracker::new();

    // 收集 Blob 片段（分块策略，降低内存峰值）
    let blob_parts = js_sys::Array::new();

    // 第一个片段包含 BOM（如果需要）
    if with_bom {
        let bom = js_sys::Uint8Array::from(&[0xEF_u8, 0xBB, 0xBF][..]);
        blob_parts.push(&bom);
    }

    // 分批处理数据，每个 batch 生成一个 CSV 片段
    let mut current_row = 0;
    while current_row < total_rows {
        let batch_end = std::cmp::min(current_row + batch_size, total_rows);

        // 创建当前批次的 CSV Writer
        let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

        // 处理当前批次
        for i in current_row..batch_end {
            let row = sources.get_row(i)?;

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

        // 完成当前批次的 CSV 写入
        wtr.flush()
            .map_err(|e| JsValue::from_str(&format!("完成 CSV 写入失败: {}", e)))?;

        let csv_data = wtr
            .into_inner()
            .map_err(|e| JsValue::from_str(&format!("获取 CSV 数据失败: {}", e)))?;

        let raw = csv_data.into_inner();

        // 将当前批次字节转为 Uint8Array 片段（此后 raw 被 drop，释放 Rust 侧内存）
        if !raw.is_empty() {
            let uint8_array = js_sys::Uint8Array::from(raw.as_slice());
            blob_parts.push(&uint8_array);
        }

        current_row = batch_end;

        // 报告进度
        if let Some(ref callback) = progress_callback {
            let progress = (current_row as f64 / total_rows as f64) * 100.0;
            report_progress(callback, progress, strict)?;
        }

        // 在批次之间让出控制权
        if current_row < total_rows {
            yield_to_browser().await?;
        }
    }

    // 用所有 Blob 片段创建 CSV 文件并触发下载
    crate::core::export_csv::create_and_download_csv_parts(
        &blob_parts,
        filename,
        "table_export.csv",
    )?;

    Ok(JsValue::UNDEFINED)
}
