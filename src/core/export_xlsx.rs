/// Excel XLSX 导出模块
///
/// 提供 Excel XLSX 格式的表格导出功能，支持单元格样式
use super::style::StyleSheet;
use super::table_extractor::TableData;
use crate::resource::trigger_bytes_download;
use crate::utils::report_progress;
use rust_xlsxwriter::{Format, Workbook, Worksheet};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;

/// Excel 最大行号（0-based，共 1048576 行）
const EXCEL_MAX_ROW: u32 = 1_048_575;
/// Excel 最大列号（0-based，共 16384 列）
const EXCEL_MAX_COL: u16 = 16_383;

/// XLSX MIME 类型
const XLSX_MIME: &str = "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";

/// 写入阶段进度映射：`progress = start + row_ratio * range`
struct SheetProgress<'a> {
    callback: &'a js_sys::Function,
    strict: bool,
    start: f64,
    range: f64,
    every_n: usize,
}

/// 计算有效冻结窗格；超出数据区时回退为不冻结
fn resolve_freeze_pane(
    freeze_pane: Option<(u32, u16)>,
    header_row_count: usize,
    total_rows: usize,
    total_cols: usize,
) -> Option<(u32, u16)> {
    let (mut row, mut col) = freeze_pane.unwrap_or({
        if header_row_count > 0 {
            (header_row_count as u32, 0)
        } else {
            (0, 0)
        }
    });

    if total_rows == 0 || row as usize >= total_rows {
        row = 0;
    }
    if total_cols == 0 || col as usize >= total_cols {
        col = 0;
    }

    if row > 0 || col > 0 {
        Some((row, col))
    } else {
        None
    }
}

/// 应用列宽配置
fn apply_column_widths(
    worksheet: &mut Worksheet,
    style_sheet: Option<&StyleSheet>,
) -> Result<(), JsValue> {
    if let Some(ss) = style_sheet {
        for (col_idx, width) in ss.column_widths.iter().enumerate() {
            if let Some(w) = width {
                worksheet
                    .set_column_width(col_idx as u16, *w)
                    .map_err(|e| JsValue::from_str(&format!("设置列宽失败: {}", e)))?;
            }
        }
    }
    Ok(())
}

/// 应用合并单元格，并保留样式
fn apply_merge_ranges(
    worksheet: &mut Worksheet,
    table_data: &TableData,
    style_sheet: Option<&StyleSheet>,
) -> Result<(), JsValue> {
    for merge in &table_data.merge_ranges {
        let text = table_data
            .rows
            .get(merge.first_row as usize)
            .and_then(|row| row.get(merge.first_col as usize))
            .map(|s| s.as_str())
            .unwrap_or("");

        // 获取合并区域首单元格的样式
        let merge_format = if let Some(ss) = style_sheet {
            ss.resolve(
                merge.first_row,
                merge.first_col,
                table_data.header_row_count,
            )
            .unwrap_or_else(Format::new)
        } else {
            Format::new()
        };

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
    Ok(())
}

/// 将单个工作表的数据/合并/冻结写入 worksheet（供单表与多表复用）
fn write_sheet(
    worksheet: &mut Worksheet,
    table_data: &TableData,
    freeze_pane: Option<(u32, u16)>,
    progress: Option<SheetProgress<'_>>,
) -> Result<(), JsValue> {
    let style_sheet = table_data.style_sheet.as_ref();
    apply_column_widths(worksheet, style_sheet)?;

    let total_rows = table_data.rows.len();
    let mut max_cols = 0usize;

    // 无单元格覆盖时缓存「全局+列级」Format，避免逐格 clone/merge/to_format
    let mut header_fmt_cache: HashMap<u16, Option<Format>> = HashMap::new();
    let mut data_fmt_cache: HashMap<u16, Option<Format>> = HashMap::new();

    for (i, row_data) in table_data.rows.iter().enumerate() {
        // 在 usize 下比较，避免先 as u32 造成截断后漏检
        if i > EXCEL_MAX_ROW as usize {
            return Err(JsValue::from_str("行数超过 Excel 限制 (1048576)"));
        }
        max_cols = max_cols.max(row_data.len());
        let is_header = i < table_data.header_row_count;
        let row = i as u32;

        for (j, cell_text) in row_data.iter().enumerate() {
            // 同上：在 usize 下比较，避免 as u16 截断
            if j > EXCEL_MAX_COL as usize {
                return Err(JsValue::from_str("列数超过 Excel 限制 (16384)"));
            }
            let col = j as u16;

            let format = if let Some(ss) = style_sheet {
                if ss.cell_overrides.contains_key(&(row, col)) {
                    ss.resolve(row, col, table_data.header_row_count)
                } else {
                    let cache = if is_header {
                        &mut header_fmt_cache
                    } else {
                        &mut data_fmt_cache
                    };
                    cache
                        .entry(col)
                        .or_insert_with(|| ss.resolve_column(is_header, col))
                        .clone()
                }
            } else {
                None
            };

            if let Some(ref fmt) = format {
                worksheet
                    .write_string_with_format(row, col, cell_text, fmt)
                    .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
            } else {
                worksheet
                    .write_string(row, col, cell_text)
                    .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
            }
        }

        if let Some(ref p) = progress
            && total_rows > 0
            && (i % p.every_n == 0 || i == total_rows - 1)
        {
            let ratio = (i + 1) as f64 / total_rows as f64;
            report_progress(p.callback, p.start + ratio * p.range, p.strict)?;
        }
    }

    apply_merge_ranges(worksheet, table_data, style_sheet)?;

    if let Some((fr, fc)) = resolve_freeze_pane(
        freeze_pane,
        table_data.header_row_count,
        total_rows,
        max_cols,
    ) {
        worksheet
            .set_freeze_panes(fr, fc)
            .map_err(|e| JsValue::from_str(&format!("设置冻结窗格失败: {}", e)))?;
    }

    Ok(())
}

/// 生成 XLSX 文件字节（不触发下载）
///
/// 仅生成 Excel 格式的字节数据，供 Worker 等场景使用。
///
/// # 参数
/// * `table_data` - 表格数据（包含单元格数据、合并区域和可选样式表）
/// * `progress_callback` - 可选的进度回调函数
/// * `strict_progress` - 是否启用严格进度回调模式
/// * `freeze_pane` - 可选的冻结窗格位置 (freeze_row, freeze_col)，为 None 时自动根据 header_row_count 冻结
///
/// # 返回值
/// * `Ok(Vec<u8>)` - 生成的 XLSX 字节
/// * `Err(JsValue)` - 生成失败
pub fn generate_xlsx_bytes(
    table_data: &TableData,
    progress_callback: Option<&js_sys::Function>,
    strict_progress: bool,
    freeze_pane: Option<(u32, u16)>,
) -> Result<Vec<u8>, JsValue> {
    if let Some(callback) = progress_callback {
        report_progress(callback, 0.0, strict_progress)?;
    }

    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    let progress = progress_callback.map(|callback| SheetProgress {
        callback,
        strict: strict_progress,
        start: 0.0,
        range: 80.0,
        every_n: 10,
    });
    write_sheet(worksheet, table_data, freeze_pane, progress)?;

    if let Some(callback) = progress_callback {
        report_progress(callback, 90.0, strict_progress)?;
    }

    let xlsx_bytes = workbook
        .save_to_buffer()
        .map_err(|e| JsValue::from_str(&format!("生成 Excel 文件失败: {}", e)))?;

    if xlsx_bytes.is_empty() {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    Ok(xlsx_bytes)
}

/// 导出为 Excel XLSX 格式（生成文件并触发下载）
pub fn export_as_xlsx(
    table_data: TableData,
    filename: Option<String>,
    progress_callback: Option<js_sys::Function>,
    strict_progress: bool,
    freeze_pane: Option<(u32, u16)>,
) -> Result<(), JsValue> {
    let xlsx_bytes = generate_xlsx_bytes(
        &table_data,
        progress_callback.as_ref(),
        strict_progress,
        freeze_pane,
    )?;
    create_and_download_xlsx(&xlsx_bytes, filename)
}

/// 生成多工作表 XLSX 文件字节（不触发下载）
pub fn generate_xlsx_multi_bytes(
    sheets_data: &[(String, TableData)],
    progress_callback: Option<&js_sys::Function>,
    strict_progress: bool,
    freeze_pane: Option<(u32, u16)>,
) -> Result<Vec<u8>, JsValue> {
    if sheets_data.is_empty() {
        return Err(JsValue::from_str("没有可导出的工作表数据"));
    }

    let total_sheets = sheets_data.len();

    if let Some(callback) = progress_callback {
        report_progress(callback, 0.0, strict_progress)?;
    }

    let mut workbook = Workbook::new();

    for (sheet_idx, (sheet_name, table_data)) in sheets_data.iter().enumerate() {
        let worksheet = workbook.add_worksheet();
        worksheet
            .set_name(sheet_name)
            .map_err(|e| JsValue::from_str(&format!("设置工作表名称失败: {}", e)))?;

        let progress = progress_callback.map(|callback| {
            let start = (sheet_idx as f64 / total_sheets as f64) * 80.0;
            let range = 80.0 / total_sheets as f64;
            SheetProgress {
                callback,
                strict: strict_progress,
                start,
                range,
                every_n: 10,
            }
        });
        write_sheet(worksheet, table_data, freeze_pane, progress)?;
    }

    if let Some(callback) = progress_callback {
        report_progress(callback, 90.0, strict_progress)?;
    }

    let xlsx_bytes = workbook
        .save_to_buffer()
        .map_err(|e| JsValue::from_str(&format!("生成 Excel 文件失败: {}", e)))?;

    if xlsx_bytes.is_empty() {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    Ok(xlsx_bytes)
}

/// 多工作表导出为 Excel XLSX 格式（生成文件并触发下载）
pub fn export_as_xlsx_multi(
    sheets_data: Vec<(String, TableData)>,
    filename: Option<String>,
    progress_callback: Option<js_sys::Function>,
    strict_progress: bool,
    freeze_pane: Option<(u32, u16)>,
) -> Result<(), JsValue> {
    let xlsx_bytes = generate_xlsx_multi_bytes(
        &sheets_data,
        progress_callback.as_ref(),
        strict_progress,
        freeze_pane,
    )?;
    create_and_download_xlsx(&xlsx_bytes, filename)
}

/// 创建 Excel Blob 并触发下载
pub(crate) fn create_and_download_xlsx(
    data: &[u8],
    filename: Option<String>,
) -> Result<(), JsValue> {
    trigger_bytes_download(data, XLSX_MIME, filename, "table_export.xlsx", "xlsx")
}

/// 供分批导出使用：在已有 workbook 上写入单表（进度可映射到自定义区间）
#[allow(clippy::too_many_arguments)]
pub(crate) fn write_sheet_with_progress(
    worksheet: &mut Worksheet,
    table_data: &TableData,
    freeze_pane: Option<(u32, u16)>,
    progress_callback: Option<&js_sys::Function>,
    strict: bool,
    progress_start: f64,
    progress_range: f64,
    every_n: usize,
) -> Result<(), JsValue> {
    let progress = progress_callback.map(|callback| SheetProgress {
        callback,
        strict,
        start: progress_start,
        range: progress_range,
        every_n,
    });
    write_sheet(worksheet, table_data, freeze_pane, progress)
}
