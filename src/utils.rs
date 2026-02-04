#[allow(dead_code)] // 这个函数在测试中可能不直接使用，但是重要的开发工具
pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
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
