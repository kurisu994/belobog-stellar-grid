/// Excel XLSX 导出模块
///
/// 提供 Excel XLSX 格式的表格导出功能
use super::table_extractor::TableData;
use crate::resource::UrlGuard;
use crate::validation::{ensure_extension, validate_filename};
use rust_xlsxwriter::{Format, Workbook};
use wasm_bindgen::prelude::*;
use web_sys::{Blob, HtmlAnchorElement, Url};

/// 导出为 Excel XLSX 格式
///
/// # 参数
/// * `table_data` - 表格数据（包含单元格数据和合并区域信息）
/// * `filename` - 可选的导出文件名
/// * `progress_callback` - 可选的进度回调函数
///
/// # 返回值
/// * `Ok(())` - 导出成功
/// * `Err(JsValue)` - 导出失败，包含错误信息
pub fn export_as_xlsx(
    table_data: TableData,
    filename: Option<String>,
    progress_callback: Option<js_sys::Function>,
) -> Result<(), JsValue> {
    let total_rows = table_data.rows.len();

    // 报告初始进度
    if let Some(ref callback) = progress_callback {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0));
    }

    // 创建工作簿与工作表
    let mut workbook = Workbook::new();
    let worksheet = workbook.add_worksheet();

    // 写入所有数据，并报告进度
    for (i, row_data) in table_data.rows.iter().enumerate() {
        for (j, cell_text) in row_data.iter().enumerate() {
            // 检测公式：以 = 开头且长度大于 1 的内容视为公式
            if cell_text.starts_with('=') && cell_text.len() > 1 {
                worksheet
                    .write_formula(i as u32, j as u16, cell_text.as_str())
                    .map_err(|e| JsValue::from_str(&format!("写入 Excel 公式失败: {}", e)))?;
            } else {
                worksheet
                    .write_string(i as u32, j as u16, cell_text)
                    .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
            }
        }

        // 定期报告进度（每10行或最后一行）（数据写入阶段占 0% - 80%）
        if let Some(ref callback) = progress_callback
            && (i % 10 == 0 || i == total_rows - 1)
        {
            let progress = ((i + 1) as f64 / total_rows as f64) * 80.0;
            let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress));
        }
    }

    // 应用合并单元格
    let merge_format = Format::new();
    for merge in &table_data.merge_ranges {
        worksheet
            .merge_range(
                merge.first_row,
                merge.first_col,
                merge.last_row,
                merge.last_col,
                "", // 使用空字符串，因为内容已经写入了
                &merge_format,
            )
            .map_err(|e| JsValue::from_str(&format!("合并单元格失败: {}", e)))?;
    }

    // 报告合并单元格完成进度
    if let Some(ref callback) = progress_callback {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(90.0));
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

/// 多工作表导出为 Excel XLSX 格式
///
/// 将多个表格数据写入同一个 Excel 文件的不同工作表中
///
/// # 参数
/// * `sheets_data` - 工作表数据列表，每个元素为 (工作表名称, 表格数据)
/// * `filename` - 可选的导出文件名
/// * `progress_callback` - 可选的进度回调函数
pub fn export_as_xlsx_multi(
    sheets_data: Vec<(String, TableData)>,
    filename: Option<String>,
    progress_callback: Option<js_sys::Function>,
) -> Result<(), JsValue> {
    if sheets_data.is_empty() {
        return Err(JsValue::from_str("没有可导出的工作表数据"));
    }

    let total_sheets = sheets_data.len();

    // 报告初始进度
    if let Some(ref callback) = progress_callback {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0));
    }

    // 创建工作簿
    let mut workbook = Workbook::new();

    // 逐个工作表写入数据
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
                // 检测公式：以 = 开头且长度大于 1 的内容视为公式
                if cell_text.starts_with('=') && cell_text.len() > 1 {
                    worksheet
                        .write_formula(i as u32, j as u16, cell_text.as_str())
                        .map_err(|e| JsValue::from_str(&format!("写入 Excel 公式失败: {}", e)))?;
                } else {
                    worksheet
                        .write_string(i as u32, j as u16, cell_text)
                        .map_err(|e| JsValue::from_str(&format!("写入 Excel 单元格失败: {}", e)))?;
                }
            }

            // 报告进度（数据写入阶段占 0% - 80%，按 sheet 均分）
            if let Some(ref callback) = progress_callback
                && total_rows > 0
                && (i % 10 == 0 || i == total_rows - 1)
            {
                let sheet_progress_start = (sheet_idx as f64 / total_sheets as f64) * 80.0;
                let sheet_progress_range = 80.0 / total_sheets as f64;
                let row_progress = (i + 1) as f64 / total_rows as f64;
                let progress = sheet_progress_start + row_progress * sheet_progress_range;
                let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress));
            }
        }

        // 应用合并单元格
        let merge_format = Format::new();
        for merge in &table_data.merge_ranges {
            worksheet
                .merge_range(
                    merge.first_row,
                    merge.first_col,
                    merge.last_row,
                    merge.last_col,
                    "",
                    &merge_format,
                )
                .map_err(|e| JsValue::from_str(&format!("合并单元格失败: {}", e)))?;
        }
    }

    // 报告合并单元格完成进度
    if let Some(ref callback) = progress_callback {
        let _ = callback.call1(&JsValue::NULL, &JsValue::from_f64(90.0));
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

    // 使用 RAII 模式确保 URL 资源释放
    let _url_guard = UrlGuard::new(&url);

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

    Ok(())
}
