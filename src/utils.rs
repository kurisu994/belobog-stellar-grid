/// 设置 panic 钩子
///
/// 启用 `console_error_panic_hook` 后，当 WASM 代码 panic 时，
/// 可以在浏览器控制台看到更友好的错误信息。
///
/// 建议在初始化时调用一次。
#[allow(dead_code)]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// 防止 CSV 注入攻击
///
/// 如果字段以 `=`, `+`, `-`, `@` 开头，则在前面添加单引号 `'`
pub fn escape_csv_injection(text: &str) -> std::borrow::Cow<'_, str> {
    if text.starts_with(['=', '+', '-', '@', '\t']) {
        format!("'{}", text).into()
    } else {
        text.into()
    }
}

/// 检查元素是否隐藏 (display: none)
pub fn is_element_hidden(element: &web_sys::Element) -> bool {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return false,
    };

    if let Ok(Some(style)) = window.get_computed_style(element)
        && let Ok(display) = style.get_property_value("display")
    {
        return display == "none";
    }
    false
}

/// 让出控制权给浏览器事件循环
///
/// 使用 setTimeout(0) 创建一个宏任务，允许浏览器处理其他事件，
/// 防止长时间同步操作阻塞 UI 线程。
pub(crate) async fn yield_to_browser() -> Result<(), wasm_bindgen::JsValue> {
    // 先获取 window 对象，避免在 Promise 闭包内 panic
    let window =
        web_sys::window().ok_or_else(|| wasm_bindgen::JsValue::from_str("无法获取 window 对象"))?;

    let promise = js_sys::Promise::new(&mut |resolve, _reject| {
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, 0);
    });

    wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_injection_tab() {
        let tab_injection = "\tCOMMAND";
        let escaped = escape_csv_injection(tab_injection);
        assert_eq!(escaped, "'\tCOMMAND");
    }

    #[test]
    fn test_escape_csv_injection_basic() {
        assert_eq!(escape_csv_injection("=cmd"), "'=cmd");
        assert_eq!(escape_csv_injection("+cmd"), "'+cmd");
        assert_eq!(escape_csv_injection("-cmd"), "'-cmd");
        assert_eq!(escape_csv_injection("@cmd"), "'@cmd");
        assert_eq!(escape_csv_injection("safe"), "safe");
    }
}
