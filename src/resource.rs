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
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
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

