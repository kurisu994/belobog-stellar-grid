use belobog_stellar_grid::escape_csv_injection;
use std::borrow::Cow;

#[test]
fn test_escape_csv_injection() {
    // 正常文本不应被转义
    assert_eq!(escape_csv_injection("abc"), Cow::Borrowed("abc"));
    assert_eq!(escape_csv_injection("123"), Cow::Borrowed("123"));
    assert_eq!(escape_csv_injection(""), Cow::Borrowed(""));

    // 注入字符应被转义
    assert_eq!(escape_csv_injection("=1+1"), Cow::Borrowed("'=1+1"));
    assert_eq!(escape_csv_injection("+1+1"), Cow::Borrowed("'+1+1"));
    assert_eq!(escape_csv_injection("-1+1"), Cow::Borrowed("'-1+1"));
    assert_eq!(
        escape_csv_injection("@SUM(1,2)"),
        Cow::Borrowed("'@SUM(1,2)")
    );

    // 已转义的文本（以 ' 开头）不应被再次转义
    // 规则是：如果以 =, +, -, @ 开头则转义。如果以 ' 开头，则不需要转义。
    // Excel 将开头的 ' 视为文本指示符。所以 "'=1+1" 变为 "=1+1" (文本)。
    // 如果输入是 "'=1+1"，escape_csv_injection 返回 "'=1+1"。
    assert_eq!(escape_csv_injection("'=1+1"), Cow::Borrowed("'=1+1"));
}

#[test]
fn test_escape_does_not_allocate_for_safe_strings() {
    // 安全字符串不应分配新内存
    let safe = "safe string";
    let escaped = escape_csv_injection(safe);
    assert!(matches!(escaped, Cow::Borrowed(_)));
}

#[test]
fn test_escape_allocates_for_unsafe_strings() {
    // 不安全字符串应分配新内存进行转义
    let unsafe_str = "=unsafe";
    let escaped = escape_csv_injection(unsafe_str);
    assert!(matches!(escaped, Cow::Owned(_)));
    assert_eq!(escaped, "'=unsafe");
}
