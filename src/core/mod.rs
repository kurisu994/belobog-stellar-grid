/// 核心导出协调模块
///
/// 提供统一的导出接口，协调各个导出模块
mod data_export;
pub(crate) mod export_csv;
pub(crate) mod export_xlsx;
pub(crate) mod style;
pub(crate) mod table_extractor;

// Excel 预览模块
pub mod excel_reader;
pub mod excel_style;
pub mod html_builder;

pub(crate) use data_export::{build_table_data_from_array, build_table_data_from_tree};
use export_csv::{export_as_csv, generate_csv_bytes};
pub(crate) use export_xlsx::create_and_download_xlsx;
use export_xlsx::{export_as_xlsx, export_as_xlsx_multi, generate_xlsx_bytes};
use table_extractor::extract_table_data;
pub(crate) use table_extractor::{
    MergeRange, RowSpanTracker, TableData, extract_table_data_with_merge, get_table_row,
    process_row_cells, resolve_table,
};
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
/// * `with_bom` - 可选，是否添加 UTF-8 BOM（默认为 false，仅对 CSV 有效）
/// * `strict_progress_callback` - 可选，是否启用严格进度回调模式（默认 false）。
///   启用后回调失败会中断导出并返回错误，否则仅 console.warn
///
/// # 返回值
/// * `Ok(())` - 导出成功
/// * `Err(JsValue)` - 导出失败，包含错误信息
///
/// # 示例
/// ```javascript
/// import init, { export_table, ExportFormat } from './pkg/belobog_stellar_grid.js';
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
#[allow(clippy::too_many_arguments)]
pub fn export_table(
    table_id: &str,
    filename: Option<String>,
    format: Option<ExportFormat>,
    exclude_hidden: Option<bool>,
    progress_callback: Option<js_sys::Function>,
    with_bom: Option<bool>,
    strict_progress_callback: Option<bool>,
    header_style: Option<JsValue>,
    cell_style: Option<JsValue>,
) -> Result<(), JsValue> {
    let format = format.unwrap_or_default();
    let exclude_hidden = exclude_hidden.unwrap_or(false);
    let with_bom = with_bom.unwrap_or(false);
    let strict_progress = strict_progress_callback.unwrap_or(false);

    // 输入验证
    if table_id.is_empty() {
        return Err(JsValue::from_str("表格 ID 不能为空"));
    }

    // 解析全局样式
    let hs = header_style.as_ref().and_then(style::parse_cell_style);
    let cs = cell_style.as_ref().and_then(style::parse_cell_style);

    // 根据格式导出
    match format {
        ExportFormat::Csv => {
            // CSV 不支持合并单元格，使用简化提取
            let table_data = extract_table_data(table_id, exclude_hidden)?;
            export_as_csv(
                table_data,
                filename,
                progress_callback,
                with_bom,
                strict_progress,
            )
        }
        ExportFormat::Xlsx => {
            // XLSX 支持合并单元格，提取完整数据
            let mut table_data = extract_table_data_with_merge(table_id, exclude_hidden)?;

            // 注入全局样式
            if hs.is_some() || cs.is_some() {
                table_data.style_sheet = Some(style::StyleSheet {
                    header_style: hs,
                    data_style: cs,
                    ..Default::default()
                });
            }

            export_as_xlsx(
                table_data,
                filename,
                progress_callback,
                strict_progress,
                None,
            )
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
    // 验证输入是否为数组
    if !js_sys::Array::is_array(sheets) {
        return Err(JsValue::from_str("工作表配置必须是数组"));
    }

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
    strict_progress_callback: Option<bool>,
    header_style: Option<JsValue>,
    cell_style: Option<JsValue>,
) -> Result<(), JsValue> {
    let strict_progress = strict_progress_callback.unwrap_or(false);

    // 解析全局样式
    let hs = header_style.as_ref().and_then(style::parse_cell_style);
    let cs = cell_style.as_ref().and_then(style::parse_cell_style);
    let global_ss = if hs.is_some() || cs.is_some() {
        Some(style::StyleSheet {
            header_style: hs,
            data_style: cs,
            ..Default::default()
        })
    } else {
        None
    };

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

        let mut table_data =
            extract_table_data_with_merge(&config.table_id, config.exclude_hidden)?;

        // 注入全局样式
        table_data.style_sheet = global_ss.clone();

        sheets_data.push((sheet_name, table_data));
    }

    // 调用多工作表导出
    export_as_xlsx_multi(
        sheets_data,
        filename,
        progress_callback,
        strict_progress,
        None,
    )
}

/// 从 JS 二维数组解析为 Rust 二维字符串数组
///
/// # 参数
/// * `data` - JS 二维数组 (Array<Array<string>>)
///
/// # 返回值
/// * `Ok(Vec<Vec<String>>)` - 解析成功
/// * `Err(JsValue)` - 解析失败
pub(crate) fn parse_js_array_data(data: &JsValue) -> Result<Vec<Vec<String>>, JsValue> {
    // 验证 data 是否为数组
    if !js_sys::Array::is_array(data) {
        return Err(JsValue::from_str("data 必须是数组"));
    }

    let outer_array = js_sys::Array::from(data);
    let row_count = outer_array.length();

    if row_count == 0 {
        return Err(JsValue::from_str("数据数组不能为空"));
    }

    // 检查第一行数据类型，提供更友好的错误提示
    let first_row = outer_array.get(0);
    if !first_row.is_undefined() && !first_row.is_null() && !js_sys::Array::is_array(&first_row) {
        // 如果第一行不是数组（例如是对象），但用户没有传 columns
        if first_row.is_object() {
            return Err(JsValue::from_str(
                "检测到 data 为对象数组但未提供 columns。如需导出对象数组，请传入 columns 配置；如需导出二维数组，请确保 data 格式为 [[值1, 值2]]",
            ));
        }
    }

    let mut result = Vec::with_capacity(row_count as usize);

    for i in 0..row_count {
        let row_val = outer_array.get(i);

        // 确保每一行都是数组
        if !js_sys::Array::is_array(&row_val) {
            return Err(JsValue::from_str(&format!(
                "第 {} 行数据格式错误：期望是数组，实际不是。未提供 columns 时 data 必须是二维数组",
                i + 1
            )));
        }

        let inner_array = js_sys::Array::from(&row_val);
        let col_count = inner_array.length();

        let mut row_data = Vec::with_capacity(col_count as usize);
        for j in 0..col_count {
            let cell_val = inner_array.get(j);
            row_data.push(data_export::js_value_to_string(&cell_val));
        }

        result.push(row_data);
    }

    Ok(result)
}

/// 从 JavaScript 数组直接导出为文件（不依赖 DOM）
///
/// 接收二维数组数据或对象数组（配合列配置），直接生成 CSV 或 XLSX 文件并触发下载。
/// 当提供 `columns` 时，支持嵌套表头（自动生成多行表头和合并单元格）。
///
/// # 参数
/// * `data` - JS 数组 (二维数组 `Array<Array<any>>` 或对象数组 `Array<Object>`)
/// * `options` - 可选的配置对象，包含以下字段：
///   - `columns`: 表头配置数组，支持嵌套 children 实现多级表头
///   - `filename`: 导出文件名
///   - `format`: 导出格式（ExportFormat.Csv 或 ExportFormat.Xlsx），默认 Csv
///   - `progressCallback`: 进度回调函数，接收 0-100 的进度值
///   - `indentColumn`: 树形模式下需要缩进的列的 key
///   - `childrenKey`: 传入此参数启用树形数据模式，指定子节点字段名
///   - `withBom`: 是否添加 UTF-8 BOM（仅 CSV 有效）
///   - `strictProgressCallback`: 回调失败是否立刻中断导出（默认 false）
///
/// # 返回值
/// * `Ok(())` - 导出成功
/// * `Err(JsValue)` - 导出失败，包含错误信息
///
/// # 示例
/// ```javascript
/// import init, { export_data, ExportFormat } from './pkg/belobog_stellar_grid.js';
/// await init();
///
/// // 1. 二维数组导出（无需 options）
/// const arrayData = [['姓名', '年龄'], ['张三', 28]];
/// export_data(arrayData);
/// export_data(arrayData, { filename: '用户.csv' });
///
/// // 2. 对象数组 + 简单表头
/// const data = [{ name: '张三', age: 28 }];
/// const columns = [
///   { title: '姓名', key: 'name' },
///   { title: '年龄', key: 'age' }
/// ];
/// export_data(data, { columns, filename: '用户.xlsx', format: ExportFormat.Xlsx });
///
/// // 3. 对象数组 + 嵌套表头（多行表头 + 合并单元格）
/// const nestedColumns = [
///   { title: '姓名', key: 'name' },
///   { title: '其他信息', children: [
///     { title: '年龄', key: 'age' },
///     { title: '住址', key: 'address' }
///   ]}
/// ];
/// export_data(data, { columns: nestedColumns, filename: '用户.xlsx', format: ExportFormat.Xlsx });
///
/// // 4. 树形数据导出（递归拍平 children + 层级缩进）
/// const treeData = [
///   { name: 'CEO', children: [
///     { name: 'CTO' },
///     { name: 'CFO', children: [{ name: '会计' }] }
///   ]}
/// ];
/// export_data(treeData, {
///   columns, filename: '组织架构.xlsx', format: ExportFormat.Xlsx,
///   indentColumn: 'name', childrenKey: 'children'
/// });
/// ```
#[wasm_bindgen]
pub fn export_data(data: JsValue, options: Option<JsValue>) -> Result<(), JsValue> {
    // 从 options 对象中解析各个配置项
    let opts = parse_export_data_options(options)?;

    export_data_impl(data, opts)
}

/// 导出数据配置项（从 options 对象解析后的结果）
pub(crate) struct ExportDataOptions {
    pub(crate) columns: Option<JsValue>,
    pub(crate) filename: Option<String>,
    pub(crate) format: ExportFormat,
    pub(crate) progress_callback: Option<js_sys::Function>,
    pub(crate) indent_column: Option<String>,
    pub(crate) children_key: Option<String>,
    pub(crate) with_bom: bool,
    /// 是否启用严格进度回调模式
    pub(crate) strict_progress: bool,
    /// 冻结行数（XLSX 有效，None 表示自动根据表头行数）
    pub(crate) freeze_rows: Option<u32>,
    /// 冻结列数（XLSX 有效，默认 0）
    pub(crate) freeze_cols: Option<u16>,
    /// 表头全局样式（XLSX 有效）
    pub(crate) header_style: Option<style::CellStyle>,
    /// 数据行全局样式（XLSX 有效）
    pub(crate) cell_style: Option<style::CellStyle>,
}

/// 从 options JsValue 对象中解析 export_data 的配置项
pub(crate) fn parse_export_data_options(
    options: Option<JsValue>,
) -> Result<ExportDataOptions, JsValue> {
    let options = match options {
        Some(ref opt) if !opt.is_null() && !opt.is_undefined() => opt,
        _ => {
            return Ok(ExportDataOptions {
                columns: None,
                filename: None,
                format: ExportFormat::default(),
                progress_callback: None,
                indent_column: None,
                children_key: None,
                with_bom: false,
                strict_progress: false,
                freeze_rows: None,
                freeze_cols: None,
                header_style: None,
                cell_style: None,
            });
        }
    };

    // 解析 columns
    let columns = js_sys::Reflect::get(options, &JsValue::from_str("columns"))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null());

    // 解析 filename
    let filename = js_sys::Reflect::get(options, &JsValue::from_str("filename"))
        .ok()
        .and_then(|v| v.as_string());

    // 解析 format（ExportFormat 在 wasm_bindgen 中编码为数字：0 = Csv, 1 = Xlsx）
    // 严格校验：仅接受 0 (Csv) 和 1 (Xlsx)，其他值返回明确错误
    let format_val = js_sys::Reflect::get(options, &JsValue::from_str("format"))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null());

    let format = match format_val {
        Some(v) => {
            let n = v.as_f64().ok_or_else(|| {
                JsValue::from_str(
                    "format 参数类型错误：期望数字（ExportFormat.Csv = 0, ExportFormat.Xlsx = 1）",
                )
            })?;
            match n as u32 {
                0 => ExportFormat::Csv,
                1 => ExportFormat::Xlsx,
                other => {
                    return Err(JsValue::from_str(&format!(
                        "format 参数值非法：{}。仅支持 ExportFormat.Csv (0) 和 ExportFormat.Xlsx (1)",
                        other
                    )));
                }
            }
        }
        None => ExportFormat::default(),
    };

    // 解析 progressCallback
    let progress_callback = js_sys::Reflect::get(options, &JsValue::from_str("progressCallback"))
        .ok()
        .filter(|v| v.is_function())
        .map(js_sys::Function::from);

    // 解析 indentColumn
    let indent_column = js_sys::Reflect::get(options, &JsValue::from_str("indentColumn"))
        .ok()
        .and_then(|v| v.as_string());

    // 解析 childrenKey
    let children_key = js_sys::Reflect::get(options, &JsValue::from_str("childrenKey"))
        .ok()
        .and_then(|v| v.as_string());

    // 解析 withBom
    let with_bom = js_sys::Reflect::get(options, &JsValue::from_str("withBom"))
        .ok()
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    // 解析 strictProgressCallback
    let strict_progress =
        js_sys::Reflect::get(options, &JsValue::from_str("strictProgressCallback"))
            .ok()
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    // 解析 freezeRows
    let freeze_rows = js_sys::Reflect::get(options, &JsValue::from_str("freezeRows"))
        .ok()
        .and_then(|v| v.as_f64())
        .map(|n| n as u32);

    // 解析 freezeCols
    let freeze_cols = js_sys::Reflect::get(options, &JsValue::from_str("freezeCols"))
        .ok()
        .and_then(|v| v.as_f64())
        .map(|n| n as u16);

    // 解析 headerStyle（全局表头样式）
    let header_style = js_sys::Reflect::get(options, &JsValue::from_str("headerStyle"))
        .ok()
        .and_then(|v| style::parse_cell_style(&v));

    // 解析 cellStyle（全局数据行样式）
    let cell_style = js_sys::Reflect::get(options, &JsValue::from_str("cellStyle"))
        .ok()
        .and_then(|v| style::parse_cell_style(&v));

    Ok(ExportDataOptions {
        columns,
        filename,
        format,
        progress_callback,
        indent_column,
        children_key,
        with_bom,
        strict_progress,
        freeze_rows,
        freeze_cols,
        header_style,
        cell_style,
    })
}

/// export_data 的内部实现
pub(crate) fn export_data_impl(data: JsValue, opts: ExportDataOptions) -> Result<(), JsValue> {
    let sp = opts.strict_progress;

    // 构建冻结窗格配置：只要用户显式传了任一参数，就使用用户配置
    let freeze_pane = match (opts.freeze_rows, opts.freeze_cols) {
        (Some(r), Some(c)) => Some((r, c)),
        (Some(r), None) => Some((r, 0)),
        (None, Some(c)) => Some((0, c)),
        (None, None) => None, // 自动根据 header_row_count 决定
    };

    // 全局样式配置
    let global_header_style = opts.header_style;
    let global_cell_style = opts.cell_style;

    // 根据是否提供 columns 决定处理方式
    // 注意：parse_export_data_options 已过滤 null/undefined 的 columns，
    // 进入此分支时 cols 一定是有效的 JsValue
    if let Some(cols) = opts.columns {
        // 判断是否为树形数据模式（提供了 children_key）
        if let Some(ck) = opts.children_key {
            let mut table_data =
                build_table_data_from_tree(&cols, &data, opts.indent_column.as_deref(), &ck)?;
            merge_global_styles(
                &mut table_data,
                global_header_style.clone(),
                global_cell_style.clone(),
            );
            return match opts.format {
                ExportFormat::Csv => export_as_csv(
                    table_data.rows,
                    opts.filename,
                    opts.progress_callback,
                    opts.with_bom,
                    sp,
                ),
                ExportFormat::Xlsx => export_as_xlsx(
                    table_data,
                    opts.filename,
                    opts.progress_callback,
                    sp,
                    freeze_pane,
                ),
            };
        }

        // 有 columns 配置：使用 data_export 模块解析嵌套表头
        let mut table_data = build_table_data_from_array(&cols, &data)?;
        merge_global_styles(&mut table_data, global_header_style, global_cell_style);

        return match opts.format {
            ExportFormat::Csv => {
                // CSV 不支持合并单元格，直接用行数据
                export_as_csv(
                    table_data.rows,
                    opts.filename,
                    opts.progress_callback,
                    opts.with_bom,
                    sp,
                )
            }
            ExportFormat::Xlsx => {
                // XLSX 支持合并单元格（多行表头）
                export_as_xlsx(
                    table_data,
                    opts.filename,
                    opts.progress_callback,
                    sp,
                    freeze_pane,
                )
            }
        };
    }

    // 无 columns，按二维数组处理
    let rows = parse_js_array_data(&data)?;
    match opts.format {
        ExportFormat::Csv => export_as_csv(
            rows,
            opts.filename,
            opts.progress_callback,
            opts.with_bom,
            sp,
        ),
        ExportFormat::Xlsx => {
            // 构建全局样式表
            let style_sheet = if global_header_style.is_some() || global_cell_style.is_some() {
                Some(style::StyleSheet {
                    header_style: global_header_style,
                    data_style: global_cell_style,
                    ..Default::default()
                })
            } else {
                None
            };

            let table_data = table_extractor::TableData {
                rows,
                merge_ranges: Vec::new(),
                header_row_count: 0,
                style_sheet,
            };
            export_as_xlsx(
                table_data,
                opts.filename,
                opts.progress_callback,
                sp,
                freeze_pane,
            )
        }
    }
}

/// 将全局样式（headerStyle / cellStyle）合并到 TableData 的 StyleSheet 中
///
/// 如果 TableData 已有 StyleSheet（来自列配置），将全局样式注入为 header_style / data_style；
/// 否则新建一个仅包含全局样式的 StyleSheet。
fn merge_global_styles(
    table_data: &mut table_extractor::TableData,
    header_style: Option<style::CellStyle>,
    cell_style: Option<style::CellStyle>,
) {
    if header_style.is_none() && cell_style.is_none() {
        return;
    }

    match table_data.style_sheet.as_mut() {
        Some(ss) => {
            // 全局样式注入到已有的 StyleSheet
            if header_style.is_some() {
                ss.header_style = header_style;
            }
            if cell_style.is_some() {
                ss.data_style = cell_style;
            }
        }
        None => {
            // 新建 StyleSheet
            table_data.style_sheet = Some(style::StyleSheet {
                header_style,
                data_style: cell_style,
                ..Default::default()
            });
        }
    }
}

/// 从 JavaScript 数组生成文件字节（不触发下载，供 Web Worker 使用）
///
/// 与 `export_data` 功能相同，但不创建 Blob 和下载链接，
/// 而是直接返回生成的文件字节（CSV 或 XLSX），
/// 适用于 Web Worker 场景：Worker 中生成字节，主线程触发下载。
///
/// # 参数
/// * `data` - JS 数组 (二维数组或对象数组)
/// * `options` - 可选的配置对象（同 export_data，但进度回调在 Worker 中可能不可用）
///
/// # 返回值
/// * `Ok(Uint8Array)` - 生成的文件字节
/// * `Err(JsValue)` - 生成失败
///
/// # 示例
/// ```javascript
/// // 在 Web Worker 中：
/// import init, { generate_data_bytes, ExportFormat } from 'belobog-stellar-grid';
/// await init();
///
/// const bytes = generate_data_bytes(
///   [['姓名', '年龄'], ['张三', 28]],
///   { format: ExportFormat.Xlsx }
/// );
/// // 通过 postMessage 将 bytes 传回主线程
/// self.postMessage({ type: 'result', bytes: bytes.buffer }, [bytes.buffer]);
/// ```
#[wasm_bindgen]
pub fn generate_data_bytes(
    data: JsValue,
    options: Option<JsValue>,
) -> Result<js_sys::Uint8Array, JsValue> {
    let opts = parse_export_data_options(options)?;
    let sp = opts.strict_progress;
    let global_header_style = opts.header_style;
    let global_cell_style = opts.cell_style;

    // 根据是否提供 columns 决定处理方式
    let (mut table_data, format, with_bom) = if let Some(cols) = opts.columns {
        if let Some(ck) = opts.children_key {
            // 树形数据模式
            let td = build_table_data_from_tree(&cols, &data, opts.indent_column.as_deref(), &ck)?;
            (td, opts.format, opts.with_bom)
        } else {
            // 对象数组 + columns 配置
            let td = build_table_data_from_array(&cols, &data)?;
            (td, opts.format, opts.with_bom)
        }
    } else {
        // 二维数组模式
        let rows = parse_js_array_data(&data)?;
        let style_sheet = if global_header_style.is_some() || global_cell_style.is_some() {
            Some(style::StyleSheet {
                header_style: global_header_style.clone(),
                data_style: global_cell_style.clone(),
                ..Default::default()
            })
        } else {
            None
        };
        let td = table_extractor::TableData {
            rows,
            merge_ranges: Vec::new(),
            header_row_count: 0,
            style_sheet,
        };
        (td, opts.format, opts.with_bom)
    };

    // 合并全局样式到 table_data
    merge_global_styles(&mut table_data, global_header_style, global_cell_style);

    // 构建冻结窗格配置
    let freeze_pane = match (opts.freeze_rows, opts.freeze_cols) {
        (Some(r), Some(c)) => Some((r, c)),
        (Some(r), None) => Some((r, 0)),
        (None, Some(c)) => Some((0, c)),
        (None, None) => None,
    };

    // 根据格式生成字节
    let bytes = match format {
        ExportFormat::Csv => generate_csv_bytes(
            table_data.rows,
            opts.progress_callback.as_ref(),
            sp,
            with_bom,
        )?,
        ExportFormat::Xlsx => generate_xlsx_bytes(
            &table_data,
            opts.progress_callback.as_ref(),
            sp,
            freeze_pane,
        )?,
    };

    Ok(js_sys::Uint8Array::from(bytes.as_slice()))
}

// ============================================================================
// Excel 预览 API
// ============================================================================

/// 解析 Excel 文件并返回 HTML Table 字符串
///
/// 在 WASM 侧完成全部解析和拼装，返回可直接挂载的 `<table>` HTML。
/// 前端通过 `dangerouslySetInnerHTML` (React) 或 `v-html` (Vue) 直接使用。
///
/// # 参数
/// * `data` - Excel 文件的二进制数据（Uint8Array）
/// * `options` - 可选的预览配置（JsValue 对象）
///
/// # 返回值
/// 包含完整 HTML table 的字符串
#[wasm_bindgen(js_name = parseExcelToHtml)]
pub fn parse_excel_to_html(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> {
    let opts = parse_preview_options(&options)?;
    let workbook = excel_reader::parse_excel(data, &opts).map_err(|e| JsValue::from_str(&e))?;

    if workbook.sheets.is_empty() {
        return Err(JsValue::from_str("Excel 文件中没有可渲染的工作表"));
    }

    let html = html_builder::build_html_table(&workbook.sheets[0]);
    Ok(JsValue::from_str(&html))
}

/// 解析 Excel 文件并返回结构化 JSON 数据
///
/// 返回 ParsedWorkbook 结构化数据，前端可自行渲染。
/// 适合需要自定义渲染逻辑（虚拟滚动、交互等）的场景。
///
/// # 参数
/// * `data` - Excel 文件的二进制数据（Uint8Array）
/// * `options` - 可选的预览配置（JsValue 对象）
///
/// # 返回值
/// ParsedWorkbook 结构的 JSON 对象
#[wasm_bindgen(js_name = parseExcelToJson)]
pub fn parse_excel_to_json(data: &[u8], options: JsValue) -> Result<JsValue, JsValue> {
    let opts = parse_preview_options(&options)?;
    let workbook = excel_reader::parse_excel(data, &opts).map_err(|e| JsValue::from_str(&e))?;

    let json_str = serde_json::to_string(&workbook)
        .map_err(|e| JsValue::from_str(&format!("JSON 序列化失败: {e}")))?;

    js_sys::JSON::parse(&json_str)
}

/// 获取 Excel 文件的工作表列表
///
/// 返回各工作表的名称、索引和行列数信息，用于 Sheet 切换 UI。
///
/// # 参数
/// * `data` - Excel 文件的二进制数据（Uint8Array）
///
/// # 返回值
/// SheetInfo 数组的 JSON 对象
#[wasm_bindgen(js_name = getExcelSheetList)]
pub fn get_excel_sheet_list(data: &[u8]) -> Result<JsValue, JsValue> {
    let sheets = excel_reader::get_sheet_list(data).map_err(|e| JsValue::from_str(&e))?;

    let json_str = serde_json::to_string(&sheets)
        .map_err(|e| JsValue::from_str(&format!("JSON 序列化失败: {e}")))?;

    js_sys::JSON::parse(&json_str)
}

/// 解析预览配置选项
fn parse_preview_options(options: &JsValue) -> Result<excel_reader::PreviewOptions, JsValue> {
    if options.is_undefined() || options.is_null() {
        return Ok(excel_reader::PreviewOptions::default());
    }

    let mut opts = excel_reader::PreviewOptions::default();

    // sheetIndex
    if let Ok(val) = js_sys::Reflect::get(options, &JsValue::from_str("sheetIndex")) {
        if let Some(n) = val.as_f64() {
            opts.sheet_index = Some(n as usize);
        }
    }

    // sheetName
    if let Ok(val) = js_sys::Reflect::get(options, &JsValue::from_str("sheetName")) {
        if let Some(s) = val.as_string() {
            opts.sheet_name = Some(s);
        }
    }

    // maxRows
    if let Ok(val) = js_sys::Reflect::get(options, &JsValue::from_str("maxRows")) {
        if let Some(n) = val.as_f64() {
            opts.max_rows = Some(n as usize);
        }
    }

    // maxCols
    if let Ok(val) = js_sys::Reflect::get(options, &JsValue::from_str("maxCols")) {
        if let Some(n) = val.as_f64() {
            opts.max_cols = Some(n as usize);
        }
    }

    // includeStyles
    if let Ok(val) = js_sys::Reflect::get(options, &JsValue::from_str("includeStyles")) {
        if let Some(b) = val.as_bool() {
            opts.include_styles = b;
        }
    }

    // trimEmpty
    if let Ok(val) = js_sys::Reflect::get(options, &JsValue::from_str("trimEmpty")) {
        if let Some(b) = val.as_bool() {
            opts.trim_empty = b;
        }
    }

    // skipHidden
    if let Ok(val) = js_sys::Reflect::get(options, &JsValue::from_str("skipHidden")) {
        if let Some(b) = val.as_bool() {
            opts.skip_hidden = b;
        }
    }

    Ok(opts)
}
