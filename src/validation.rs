/// 文件名验证模块
///
/// 提供安全的文件名验证功能，防止路径遍历和非法文件名攻击
///
/// 验证文件名是否安全合法
///
/// # 参数
/// * `filename` - 要验证的文件名
///
/// # 返回值
/// * `Ok(())` - 文件名合法
/// * `Err(String)` - 文件名不合法，包含错误信息
///
/// # 注意
/// 这个函数主要供内部使用，但也导出以便测试
#[doc(hidden)]
pub fn validate_filename(filename: &str) -> Result<(), String> {
    // 检查文件名是否为空
    if filename.is_empty() {
        return Err("文件名不能为空".to_string());
    }

    // 检查文件名中的危险字符（路径分隔符）
    if filename.contains('/') || filename.contains('\\') {
        return Err("文件名不能包含路径分隔符".to_string());
    }

    // 检查 ASCII 控制字符（0x00-0x1F）
    if filename.chars().any(|c| c.is_ascii_control()) {
        return Err("文件名不能包含控制字符".to_string());
    }

    // 检查其他危险字符
    let dangerous_chars = ['<', '>', ':', '"', '|', '?', '*'];
    if let Some(&ch) = dangerous_chars.iter().find(|&&c| filename.contains(c)) {
        return Err(format!("文件名不能包含非法字符: {}", ch));
    }

    // 检查文件名长度（大多数文件系统限制为 255 个字符）
    if filename.chars().count() > 255 {
        return Err("文件名过长（最大 255 个字符）".to_string());
    }

    // 检查 Windows 保留文件名
    let base_name = filename.split('.').next().unwrap_or("");
    let reserved_names = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    if reserved_names.contains(&base_name.to_uppercase().as_str()) {
        return Err(format!("文件名 '{}' 是系统保留名称", base_name));
    }

    // 检查文件名是否以点或空格开头/结尾（Windows 不支持）
    if filename.starts_with('.')
        || filename.starts_with(' ')
        || filename.ends_with('.')
        || filename.ends_with(' ')
    {
        return Err("文件名不能以点或空格开头或结尾".to_string());
    }

    Ok(())
}

/// 确保文件名有正确的扩展名
///
/// # 参数
/// * `filename` - 原始文件名
/// * `extension` - 期望的扩展名（如 "csv"）
///
/// # 返回值
/// 返回带有正确扩展名的文件名
pub fn ensure_extension(filename: &str, extension: &str) -> String {
    if filename
        .to_lowercase()
        .ends_with(&format!(".{}", extension.to_lowercase()))
    {
        filename.to_string()
    } else {
        format!("{}.{}", filename, extension)
    }
}
