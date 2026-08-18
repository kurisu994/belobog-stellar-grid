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
// 移除 doc(hidden) 因为这是公开 API
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

    // 检查文件名长度（大多数文件系统限制为 255 个字节）
    if filename.len() > 255 {
        return Err("文件名过长（最大 255 个字节）".to_string());
    }

    // 检查 Windows 保留文件名
    let base_name = filename.split('.').next().unwrap_or_default();
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
        // 检查全角点号 (U+FF0E) 和其他可能被视为点号的字符
        || filename.contains('。')
        || filename.contains('．')
        || filename.contains('․')
    {
        return Err("文件名不能以点、空格开头或结尾，且不能包含全角点号".to_string());
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

/// 解析并校验下载文件名（先校验再补扩展名）
///
/// 应在创建 Blob URL 之前调用，避免非法文件名导致已创建 URL 泄漏。
pub fn prepare_download_filename(
    filename: Option<String>,
    default_name: &str,
    extension: &str,
) -> Result<String, String> {
    let name = filename.unwrap_or_else(|| default_name.to_string());
    validate_filename(&name)?;
    Ok(ensure_extension(&name, extension))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_filename_byte_length() {
        // 256 'a's = 256 bytes
        let long_name = "a".repeat(256);
        assert!(validate_filename(&long_name).is_err());

        // 255 'a's = 255 bytes -> OK
        let ok_name = "a".repeat(255);
        assert!(validate_filename(&ok_name).is_ok());

        // Unicodes: '中' takes 3 bytes.
        // 86 '中's = 258 bytes -> should fail
        let long_unicode = "中".repeat(86);
        assert!(long_unicode.chars().count() == 86); // Only 86 chars
        assert!(long_unicode.len() == 258); // 258 bytes
        assert!(validate_filename(&long_unicode).is_err());
    }

    #[test]
    fn test_prepare_download_filename() {
        let ok = prepare_download_filename(Some("report".into()), "default.csv", "csv").unwrap();
        assert_eq!(ok, "report.csv");

        let defaulted = prepare_download_filename(None, "table_export.csv", "csv").unwrap();
        assert_eq!(defaulted, "table_export.csv");

        let bad = prepare_download_filename(Some("../x".into()), "default.csv", "csv");
        assert!(bad.is_err());
    }
}
