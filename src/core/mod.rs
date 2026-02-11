/// 核心导出协调模块
///
/// 提供统一的导出接口，协调各个导出模块
mod export_csv;
pub(crate) mod export_xlsx;
mod table_extractor;

use export_csv::export_as_csv;
use export_xlsx::{export_as_xlsx, export_as_xlsx_multi};
use table_extractor::extract_table_data;
pub(crate) use table_extractor::extract_table_data_with_merge;
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
/// * `exclude_hidden` - 可选，是否排除隐藏的行和列（默认为 false）
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
/// // 导出为 CSV（带进度回调，不排除隐藏行）
/// export_table('my-table', '数据', ExportFormat.Csv, false, (progress) => {
///     console.log(`进度: ${progress.toFixed(1)}%`);
/// });
///
/// // 导出为 Excel（带进度回调，排除隐藏行）
/// export_table('my-table', '报表', ExportFormat.Xlsx, true, (progress) => {
///     document.getElementById('progress').style.width = `${progress}%`;
/// });
/// ```
#[wasm_bindgen]
pub fn export_table(
    table_id: &str,
    filename: Option<String>,
    format: Option<ExportFormat>,
    exclude_hidden: Option<bool>,
    progress_callback: Option<js_sys::Function>,
) -> Result<(), JsValue> {
    let format = format.unwrap_or_default();
    let exclude_hidden = exclude_hidden.unwrap_or(false);

    // 输入验证
    if table_id.is_empty() {
        return Err(JsValue::from_str("表格 ID 不能为空"));
    }

    // 根据格式导出
    match format {
        ExportFormat::Csv => {
            // CSV 不支持合并单元格，使用简化提取
            let table_data = extract_table_data(table_id, exclude_hidden)?;
            export_as_csv(table_data, filename, progress_callback)
        }
        ExportFormat::Xlsx => {
            // XLSX 支持合并单元格，提取完整数据
            let table_data = extract_table_data_with_merge(table_id, exclude_hidden)?;
            export_as_xlsx(table_data, filename, progress_callback)
        }
    }
}

/// 工作表配置项（从 JS 对象解析）
struct SheetConfig {
    /// 表格元素 ID
    table_id: String,
    /// 工作表名称（可选，默认为 Sheet1, Sheet2...）
    sheet_name: Option<String>,
    /// 是否排除隐藏行列
    exclude_hidden: bool,
}

/// 从 JsValue 数组解析工作表配置列表
fn parse_sheet_configs(sheets: &JsValue) -> Result<Vec<SheetConfig>, JsValue> {
    let array = js_sys::Array::from(sheets);
    let length = array.length();

    if length == 0 {
        return Err(JsValue::from_str("工作表配置数组不能为空"));
    }

    let mut configs = Vec::with_capacity(length as usize);

    for i in 0..length {
        let item = array.get(i);

        // 提取 tableId（必填）
        let table_id = js_sys::Reflect::get(&item, &JsValue::from_str("tableId"))
            .ok()
            .and_then(|v| v.as_string())
            .ok_or_else(|| {
                JsValue::from_str(&format!("第 {} 个工作表配置缺少有效的 tableId", i + 1))
            })?;

        if table_id.is_empty() {
            return Err(JsValue::from_str(&format!(
                "第 {} 个工作表配置的 tableId 不能为空",
                i + 1
            )));
        }

        // 提取 sheetName（可选）
        let sheet_name = js_sys::Reflect::get(&item, &JsValue::from_str("sheetName"))
            .ok()
            .and_then(|v| v.as_string());

        // 提取 excludeHidden（可选，默认 false）
        let exclude_hidden = js_sys::Reflect::get(&item, &JsValue::from_str("excludeHidden"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        configs.push(SheetConfig {
            table_id,
            sheet_name,
            exclude_hidden,
        });
    }

    Ok(configs)
}

/// 多工作表导出为 Excel XLSX 文件
///
/// 将多个 HTML 表格导出到同一个 Excel 文件的不同工作表中
///
/// # 参数
/// * `sheets` - JS 数组，每个元素包含 { tableId: string, sheetName?: string, excludeHidden?: boolean }
/// * `filename` - 可选的导出文件名
/// * `progress_callback` - 可选的进度回调函数，接收 0-100 的进度值
///
/// # 示例
/// ```javascript
/// import { export_tables_xlsx } from './pkg/belobog_stellar_grid.js';
///
/// export_tables_xlsx(
///   [
///     { tableId: 'table1', sheetName: '订单列表', excludeHidden: true },
///     { tableId: 'table2', sheetName: '商品列表' },
///   ],
///   'report.xlsx',
///   (progress) => console.log(`进度: ${progress}%`)
/// );
/// ```
#[wasm_bindgen]
pub fn export_tables_xlsx(
    sheets: JsValue,
    filename: Option<String>,
    progress_callback: Option<js_sys::Function>,
) -> Result<(), JsValue> {
    // 解析配置
    let configs = parse_sheet_configs(&sheets)?;

    // 逐个表格提取数据
    let mut sheets_data: Vec<(String, table_extractor::TableData)> =
        Vec::with_capacity(configs.len());

    for (idx, config) in configs.iter().enumerate() {
        let sheet_name = config
            .sheet_name
            .clone()
            .unwrap_or_else(|| format!("Sheet{}", idx + 1));

        let table_data = extract_table_data_with_merge(&config.table_id, config.exclude_hidden)?;

        sheets_data.push((sheet_name, table_data));
    }

    // 调用多工作表导出
    export_as_xlsx_multi(sheets_data, filename, progress_callback)
}
