/// 核心导出协调模块
///
/// 提供统一的导出接口，协调各个导出模块
mod export_csv;
mod export_xlsx;
mod table_extractor;

use export_csv::export_as_csv;
use export_xlsx::export_as_xlsx;
use table_extractor::extract_table_data;
use wasm_bindgen::prelude::*;

/// 导出格式枚举
#[wasm_bindgen]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ExportFormat {
    /// CSV 格式（默认）
    #[default]
    Csv,
    /// Excel XLSX 格式
    Xlsx,
}

/// 统一的表格导出函数（带进度回调）
///
/// 支持导出为 CSV 或 Excel 格式，通过 format 参数控制，支持进度回调
///
/// # 参数
/// * `table_id` - 要导出的 HTML 表格元素的 ID
/// * `filename` - 可选的导出文件名（不包含扩展名时会自动添加）
/// * `format` - 导出格式（Csv 或 Xlsx），默认为 Csv
/// * `progress_callback` - 可选的进度回调函数，接收 0-100 的进度值
///
/// # 返回值
/// * `Ok(())` - 导出成功
/// * `Err(JsValue)` - 导出失败，包含错误信息
///
/// # 示例
/// ```javascript
/// import init, { export_table, ExportFormat } from './pkg/excel_exporter.js';
/// await init();
///
/// // 导出为 CSV（默认，无进度回调）
/// export_table('my-table');
/// export_table('my-table', '数据.csv');
///
/// // 导出为 CSV（带进度回调）
/// export_table('my-table', '数据', ExportFormat.Csv, (progress) => {
///     console.log(`进度: ${progress.toFixed(1)}%`);
/// });
///
/// // 导出为 Excel（带进度回调）
/// export_table('my-table', '报表', ExportFormat.Xlsx, (progress) => {
///     document.getElementById('progress').style.width = `${progress}%`;
/// });
/// ```
#[wasm_bindgen]
pub fn export_table(
    table_id: &str,
    filename: Option<String>,
    format: Option<ExportFormat>,
    progress_callback: Option<js_sys::Function>,
) -> Result<(), JsValue> {
    let format = format.unwrap_or_default();

    // 输入验证
    if table_id.is_empty() {
        return Err(JsValue::from_str("表格 ID 不能为空"));
    }

    // 提取表格数据
    let table_data = extract_table_data(table_id)?;

    // 根据格式导出
    match format {
        ExportFormat::Csv => export_as_csv(table_data, filename, progress_callback),
        ExportFormat::Xlsx => export_as_xlsx(table_data, filename, progress_callback),
    }
}
