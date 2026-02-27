/// Excel XLSX 导出模块
///
/// 提供 Excel XLSX 格式的表格导出功能
use super::table_extractor::TableData;
use crate::utils::report_progress;
use crate::validation::{ensure_extension, validate_filename};
use rust_xlsxwriter::{Format, Workbook};
use wasm_bindgen::prelude::*;
use web_sys::{Blob, HtmlAnchorElement, Url};

/// 生成 XLSX 文件字节（不触发下载）
///
/// 仅生成 Excel 格式的字节数据，供 Worker 等场景使用。
///
/// # 参数
/// * `table_data` - 表格数据（包含单元格数据和合并区域信息）
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
    let total_rows = table_data.rows.len();

    // 报告初始进度
    if let Some(callback) = progress_callback {
        report_progress(callback, 0.0, strict_progress)?;
    }

    // 创建工作簿与工作表
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 写入所有数据，并报告进度
    // 安全策略：所有单元格统一使用 write_string，禁用公式自动执行，防止公式注入攻击
    for (i, row_data) in table_data.rows.iter().enumerate() {
        for (j, cell_text) in row_data.iter().enumerate() {
            if j > 16383 {
                return Err(JsValue::from_str("列数超过 Excel 限制 (16384)"));
            }
            worksheet
                .write_string(i as u32, j as u16, cell_text)
                .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
        }

        // 定期报告进度（每10行或最后一行）（数据写入阶段占 0% - 80%）
        if let Some(callback) = progress_callback
            && (i % 10 == 0 || i == total_rows - 1)
        {
            let progress = ((i + 1) as f64 / total_rows as f64) * 80.0;
            report_progress(callback, progress, strict_progress)?;
        }
    }

    // 应用合并单元格（merge_range 会覆盖首单元格内容，需传入实际文本）
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

    // 应用冻结窗格：用户配置优先，否则自动根据表头行数冻结
    let effective_freeze = freeze_pane.unwrap_or({
        if table_data.header_row_count > 0 {
            (table_data.header_row_count as u32, 0)
        } else {
            (0, 0)
        }
    });
    if effective_freeze.0 > 0 || effective_freeze.1 > 0 {
        worksheet
            .set_freeze_panes(effective_freeze.0, effective_freeze.1)
            .map_err(|e| JsValue::from_str(&format!("设置冻结窗格失败: {}", e)))?;
    }

    // 报告合并单元格完成进度
    if let Some(callback) = progress_callback {
        report_progress(callback, 90.0, strict_progress)?;
    }

    // 将工作簿写入内存缓冲区
    let xlsx_bytes = workbook
        .save_to_buffer()
        .map_err(|e| JsValue::from_str(&format!("生成 Excel 文件失败: {}", e)))?;

    if xlsx_bytes.is_empty() {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    Ok(xlsx_bytes)
}

/// 导出为 Excel XLSX 格式（生成文件并触发下载）
///
/// # 参数
/// * `table_data` - 表格数据（包含单元格数据和合并区域信息）
/// * `filename` - 可选的导出文件名
/// * `progress_callback` - 可选的进度回调函数
/// * `strict_progress` - 是否启用严格进度回调模式
///
/// # 返回值
/// * `Ok(())` - 导出成功
/// * `Err(JsValue)` - 导出失败，包含错误信息
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

    // 创建并下载文件
    create_and_download_xlsx(&xlsx_bytes, filename)
}

/// 生成多工作表 XLSX 文件字节（不触发下载）
///
/// # 参数
/// * `sheets_data` - 工作表数据列表，每个元素为 (工作表名称, 表格数据)
/// * `progress_callback` - 可选的进度回调函数
/// * `strict_progress` - 是否启用严格进度回调模式
/// * `freeze_pane` - 可选的冻结窗格位置，应用到所有工作表
///
/// # 返回值
/// * `Ok(Vec<u8>)` - 生成的 XLSX 字节
/// * `Err(JsValue)` - 生成失败
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

    // 报告初始进度
    if let Some(callback) = progress_callback {
        report_progress(callback, 0.0, strict_progress)?;
    }

    // 创建工作簿
    let mut workbook = Workbook::new();

    // 逐个工作表写入数据
    let merge_format = Format::new();
    for (sheet_idx, (sheet_name, table_data)) in sheets_data.iter().enumerate() {
        let worksheet = workbook.add_worksheet();

        // 设置工作表名称
        worksheet
            .set_name(sheet_name)
            .map_err(|e| JsValue::from_str(&format!("设置工作表名称失败: {}", e)))?;

        let total_rows = table_data.rows.len();

        // 写入数据
        for (i, row_data) in table_data.rows.iter().enumerate() {
            for (j, cell_text) in row_data.iter().enumerate() {
                if j > 16383 {
                    return Err(JsValue::from_str("列数超过 Excel 限制 (16384)"));
                }
                worksheet
                    .write_string(i as u32, j as u16, cell_text)
                    .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
            }

            // 报告进度（数据写入阶段占 0% - 80%，按 sheet 均分）
            if let Some(callback) = progress_callback
                && total_rows > 0
                && (i % 10 == 0 || i == total_rows - 1)
            {
                let sheet_progress_start = (sheet_idx as f64 / total_sheets as f64) * 80.0;
                let sheet_progress_range = 80.0 / total_sheets as f64;
                let row_progress = (i + 1) as f64 / total_rows as f64;
                let progress = sheet_progress_start + row_progress * sheet_progress_range;
                report_progress(callback, progress, strict_progress)?;
            }
        }

        // 应用合并单元格（merge_range 会覆盖首单元格内容，需传入实际文本）
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

        // 应用冻结窗格：用户配置优先，否则自动根据各 sheet 的表头行数冻结
        let effective_freeze = freeze_pane.unwrap_or({
            if table_data.header_row_count > 0 {
                (table_data.header_row_count as u32, 0)
            } else {
                (0, 0)
            }
        });
        if effective_freeze.0 > 0 || effective_freeze.1 > 0 {
            worksheet
                .set_freeze_panes(effective_freeze.0, effective_freeze.1)
                .map_err(|e| JsValue::from_str(&format!("设置冻结窗格失败: {}", e)))?;
        }
    }

    // 报告合并单元格完成进度
    if let Some(callback) = progress_callback {
        report_progress(callback, 90.0, strict_progress)?;
    }

    // 将工作簿写入内存缓冲区
    let xlsx_bytes = workbook
        .save_to_buffer()
        .map_err(|e| JsValue::from_str(&format!("生成 Excel 文件失败: {}", e)))?;

    if xlsx_bytes.is_empty() {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    Ok(xlsx_bytes)
}

/// 多工作表导出为 Excel XLSX 格式（生成文件并触发下载）
///
/// # 参数
/// * `sheets_data` - 工作表数据列表，每个元素为 (工作表名称, 表格数据)
/// * `filename` - 可选的导出文件名
/// * `progress_callback` - 可选的进度回调函数
/// * `strict_progress` - 是否启用严格进度回调模式
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

    // 创建并下载文件
    create_and_download_xlsx(&xlsx_bytes, filename)
}

/// 创建 Excel Blob 并触发下载
///
/// # 参数
/// * `data` - Excel 文件数据字节
/// * `filename` - 可选的导出文件名
pub(crate) fn create_and_download_xlsx(
    data: &[u8],
    filename: Option<String>,
) -> Result<(), JsValue> {
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

    // 延迟 10 秒后释放 Blob URL，避免 click 后立即 revoke 导致下载竞态
    crate::resource::schedule_url_revoke(&window, url);

    Ok(())
}
