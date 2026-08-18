/// CSV 导出模块
///
/// 提供 CSV 格式的表格导出功能
use crate::resource::{trigger_blob_download, trigger_bytes_download};
use crate::utils::report_progress;
use csv::Writer;
use std::io::Cursor;
use wasm_bindgen::prelude::*;

/// 生成 CSV 内容字节（不触发下载）
///
/// 仅生成 CSV 格式的字节数据，供 Worker 等场景使用。
///
/// # 参数
/// * `table_data` - 表格数据（二维字符串数组）
/// * `progress_callback` - 可选的进度回调函数
/// * `strict_progress` - 是否启用严格进度回调模式
/// * `with_bom` - 是否添加 UTF-8 BOM
///
/// # 返回值
/// * `Ok(Vec<u8>)` - 生成的 CSV 字节
/// * `Err(JsValue)` - 生成失败
pub fn generate_csv_bytes(
    table_data: Vec<Vec<String>>,
    progress_callback: Option<&js_sys::Function>,
    strict_progress: bool,
    with_bom: bool,
) -> Result<Vec<u8>, JsValue> {
    let total_rows = table_data.len();

    // 报告初始进度
    if let Some(callback) = progress_callback {
        report_progress(callback, 0.0, strict_progress)?;
    }

    // 创建一个 CSV 写入器
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

    // 写入所有数据，并报告进度
    for (index, row_data) in table_data.into_iter().enumerate() {
        // 转义 CSV 注入字符后写入，Cow::Borrowed 时零拷贝
        let safe_row: Vec<_> = row_data
            .iter()
            .map(|cell| crate::utils::escape_csv_injection(cell))
            .collect();
        wtr.write_record(safe_row.iter().map(|s| s.as_ref()))
            .map_err(|e| JsValue::from_str(&format!("写入 CSV 数据失败: {}", e)))?;

        // 定期报告进度（每10行或最后一行）
        if let Some(callback) = progress_callback
            && (index % 10 == 0 || index == total_rows - 1)
        {
            let progress = ((index + 1) as f64 / total_rows as f64) * 100.0;
            report_progress(callback, progress, strict_progress)?;
        }
    }

    // 安全地完成 CSV 写入
    wtr.flush()
        .map_err(|e| JsValue::from_str(&format!("完成 CSV 写入失败: {}", e)))?;

    // 获取 CSV 数据
    let csv_data = wtr
        .into_inner()
        .map_err(|e| JsValue::from_str(&format!("获取 CSV 数据失败: {}", e)))?;

    let raw = csv_data.into_inner();
    if raw.is_empty() {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    // 如果需要 BOM，拼接到头部
    if with_bom {
        let mut result = Vec::with_capacity(3 + raw.len());
        result.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
        result.extend(raw);
        Ok(result)
    } else {
        Ok(raw)
    }
}

/// 导出为 CSV 格式（生成文件并触发下载）
///
/// # 参数
/// * `table_data` - 表格数据（二维字符串数组）
/// * `filename` - 可选的导出文件名
/// * `progress_callback` - 可选的进度回调函数
/// * `with_bom` - 是否添加 UTF-8 BOM
/// * `strict_progress` - 是否启用严格进度回调模式
///
/// # 返回值
/// * `Ok(())` - 导出成功
/// * `Err(JsValue)` - 导出失败，包含错误信息
pub fn export_as_csv(
    table_data: Vec<Vec<String>>,
    filename: Option<String>,
    progress_callback: Option<js_sys::Function>,
    with_bom: bool,
    strict_progress: bool,
) -> Result<(), JsValue> {
    let bytes = generate_csv_bytes(
        table_data,
        progress_callback.as_ref(),
        strict_progress,
        with_bom,
    )?;

    // 创建并下载文件（BOM 已在 bytes 中处理）
    create_and_download_csv(&bytes, filename)
}

/// 创建 CSV Blob 并触发下载
///
/// # 参数
/// * `data` - CSV 数据字节（可能已包含 BOM）
/// * `filename` - 可选的导出文件名
pub(crate) fn create_and_download_csv(
    data: &[u8],
    filename: Option<String>,
) -> Result<(), JsValue> {
    trigger_bytes_download(
        data,
        "text/csv;charset=utf-8",
        filename,
        "table_export.csv",
        "csv",
    )
}

/// 从多个 Blob 片段拼接 CSV 并触发下载
pub(crate) fn create_and_download_csv_parts(
    parts: &js_sys::Array,
    filename: Option<String>,
    default_name: &str,
) -> Result<(), JsValue> {
    trigger_blob_download(
        parts,
        "text/csv;charset=utf-8",
        filename,
        default_name,
        "csv",
    )
}
