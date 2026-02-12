/// CSV 导出模块
///
/// 提供 CSV 格式的表格导出功能
use crate::resource::UrlGuard;
use crate::validation::{ensure_extension, validate_filename};
use csv::Writer;
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use web_sys::{Blob, HtmlAnchorElement, Url};

/// 导出为 CSV 格式
///
/// # 参数
/// * `table_data` - 表格数据（二维字符串数组）
/// * `filename` - 可选的导出文件名
/// * `progress_callback` - 可选的进度回调函数
///
/// # 返回值
/// * `Ok(())` - 导出成功
/// * `Err(JsValue)` - 导出失败，包含错误信息
pub fn export_as_csv(
    table_data: Vec<Vec<String>>,
    filename: Option<String>,
    progress_callback: Option<js_sys::Function>,
    with_bom: bool,
) -> Result<(), JsValue> {
    let total_rows = table_data.len();

    // 报告初始进度
    if let Some(ref callback) = progress_callback {
        if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(0.0)) {
            web_sys::console::warn_1(&e);
        }
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
        if let Some(ref callback) = progress_callback
            && (index % 10 == 0 || index == total_rows - 1)
        {
            let progress = ((index + 1) as f64 / total_rows as f64) * 100.0;
            if let Err(e) = callback.call1(&JsValue::NULL, &JsValue::from_f64(progress)) {
                web_sys::console::warn_1(&e);
            }
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

    // 创建并下载文件
    create_and_download_csv(csv_data.get_ref(), filename, with_bom)
}

/// 创建 CSV Blob 并触发下载
///
/// # 参数
/// * `data` - CSV 数据字节
/// * `filename` - 可选的导出文件名
/// * `with_bom` - 是否添加 UTF-8 BOM
pub(crate) fn create_and_download_csv(
    data: &[u8],
    filename: Option<String>,
    with_bom: bool,
) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

    // 创建 CSV Blob 对象
    let blob_property_bag = web_sys::BlobPropertyBag::new();
    blob_property_bag.set_type("text/csv;charset=utf-8");

    let array = if with_bom {
        // 添加 UTF-8 BOM: 0xEF, 0xBB, 0xBF
        let bom = js_sys::Uint8Array::from(&[0xEF, 0xBB, 0xBF][..]);
        let data_array = js_sys::Uint8Array::from(data);
        js_sys::Array::of2(&bom, &data_array)
    } else {
        js_sys::Array::of1(&js_sys::Uint8Array::from(data))
    };

    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_property_bag)
        .map_err(|e| JsValue::from_str(&format!("创建 Blob 对象失败: {:?}", e)))?;

    // 创建下载链接
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| JsValue::from_str(&format!("创建下载链接失败: {:?}", e)))?;

    // 使用 RAII 模式确保 URL 资源释放
    let _url_guard = UrlGuard::new(&url);

    // 设置文件名
    let final_filename = filename.unwrap_or_else(|| "table_export.csv".to_string());

    // 验证文件名安全性
    if let Err(e) = validate_filename(&final_filename) {
        return Err(JsValue::from_str(&format!("文件名验证失败: {}", e)));
    }

    let final_filename = ensure_extension(&final_filename, "csv");

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
