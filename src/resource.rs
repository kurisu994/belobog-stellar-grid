/// 资源管理模块
///
/// 提供 RAII 风格的资源管理，确保 Web 资源的正确释放
#[cfg(target_arch = "wasm32")]
use web_sys::Url;

/// RAII 风格的 URL 资源管理器
///
/// 确保在对象销毁时自动释放 Blob URL 资源
pub struct UrlGuard {
    #[allow(dead_code)] // 在非 WASM 环境中，url 字段不会被直接访问
    url: String,
}

impl UrlGuard {
    /// 创建新的 URL 资源管理器
    ///
    /// # 参数
    /// * `url` - 需要管理的 URL 字符串
    ///
    /// # 返回值
    /// 返回 UrlGuard 实例
    pub fn new(url: &str) -> Self {
        Self {
            url: url.to_string(),
        }
    }
}

impl Drop for UrlGuard {
    fn drop(&mut self) {
        // 仅在 WASM 环境中释放 URL 资源
        #[cfg(target_arch = "wasm32")]
        {
            // 确保在对象销毁时释放 URL 资源
            if let Err(e) = Url::revoke_object_url(&self.url) {
                // 记录错误但不阻止程序执行
                web_sys::console::error_1(&wasm_bindgen::JsValue::from_str(&format!(
                    "释放 URL 资源失败: {:?}",
                    e
                )));
            }
        }

        // 在非 WASM 环境（测试环境）中，不执行任何操作
        #[cfg(not(target_arch = "wasm32"))]
        {
            // 测试环境：跳过 URL 释放
            // 这允许在 cargo test 中测试 UrlGuard 的生命周期管理
        }
    }
}

/// 延迟释放 Blob URL
///
/// 通过 `setTimeout(10000)` 延迟调用 `Url::revoke_object_url`，
/// 确保浏览器有足够时间完成下载后再释放 URL 资源。
///
/// # 参数
/// * `window` - 浏览器 window 对象
/// * `url` - 需要释放的 Blob URL
#[cfg(target_arch = "wasm32")]
pub(crate) fn schedule_url_revoke(window: &web_sys::Window, url: String) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::closure::Closure;
    let callback = Closure::once(move || {
        let _ = Url::revoke_object_url(&url);
    });

    // 10 秒后释放 URL，足以让浏览器完成下载初始化
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
        callback.as_ref().unchecked_ref(),
        10_000,
    );

    // 泄漏闭包以保持其存活直到 setTimeout 触发
    callback.forget();
}

/// 非 WASM 环境占位（测试用）
#[cfg(not(target_arch = "wasm32"))]
pub(crate) fn schedule_url_revoke(_window: &web_sys::Window, _url: String) {
    // 测试环境：无需释放 URL
}

/// 从 Blob 片段数组创建文件并触发浏览器下载
///
/// 统一 CSV/XLSX/分片下载路径：先校验文件名，再创建 Object URL，成功后延迟 revoke。
pub(crate) fn trigger_blob_download(
    blob_parts: &js_sys::Array,
    mime_type: &str,
    filename: Option<String>,
    default_name: &str,
    extension: &str,
) -> Result<(), wasm_bindgen::JsValue> {
    use crate::validation::prepare_download_filename;
    use wasm_bindgen::JsCast;
    use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

    let window =
        web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("无法获取 document 对象"))?;

    // 先校验文件名，避免非法路径泄漏已创建的 URL
    let final_filename = prepare_download_filename(filename, default_name, extension)
        .map_err(|e| wasm_bindgen::JsValue::from_str(&format!("文件名验证失败: {}", e)))?;

    let bag = BlobPropertyBag::new();
    bag.set_type(mime_type);
    let blob = Blob::new_with_u8_array_sequence_and_options(blob_parts, &bag)
        .map_err(|e| wasm_bindgen::JsValue::from_str(&format!("创建 Blob 对象失败: {:?}", e)))?;

    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| wasm_bindgen::JsValue::from_str(&format!("创建下载链接失败: {:?}", e)))?;

    let anchor = document
        .create_element("a")
        .map_err(|e| wasm_bindgen::JsValue::from_str(&format!("创建下载链接元素失败: {:?}", e)))?;
    let anchor = anchor
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| wasm_bindgen::JsValue::from_str("创建的元素不是有效的锚点元素"))?;

    anchor.set_href(&url);
    anchor.set_download(&final_filename);
    anchor.click();

    schedule_url_revoke(&window, url);
    Ok(())
}

/// 从完整字节缓冲触发下载（单段 Blob）
pub(crate) fn trigger_bytes_download(
    data: &[u8],
    mime_type: &str,
    filename: Option<String>,
    default_name: &str,
    extension: &str,
) -> Result<(), wasm_bindgen::JsValue> {
    let parts = js_sys::Array::of1(&js_sys::Uint8Array::from(data));
    trigger_blob_download(&parts, mime_type, filename, default_name, extension)
}
