use belobog_stellar_grid::escape_csv_injection;
use std::borrow::Cow;

#[test]
fn test_escape_csv_injection() {
    // Normal text should not be escaped
    assert_eq!(escape_csv_injection("abc"), Cow::Borrowed("abc"));
    assert_eq!(escape_csv_injection("123"), Cow::Borrowed("123"));
    assert_eq!(escape_csv_injection(""), Cow::Borrowed(""));

    // Injection characters should be escaped
    assert_eq!(escape_csv_injection("=1+1"), Cow::Borrowed("'=1+1"));
    assert_eq!(escape_csv_injection("+1+1"), Cow::Borrowed("'+1+1"));
    assert_eq!(escape_csv_injection("-1+1"), Cow::Borrowed("'-1+1"));
    assert_eq!(
        escape_csv_injection("@SUM(1,2)"),
        Cow::Borrowed("'@SUM(1,2)")
    );

    // Already escaped text (starts with ') should not be re-escaped?
    // The rule is: if it starts with =, +, -, @. If it starts with ', it doesn't need escaping?
    // Excel treats ' at start as text indicator. So "'=1+1" becomes "=1+1" (text).
    // If input is "'=1+1", escape_csv_injection returns "'=1+1".
    assert_eq!(escape_csv_injection("'=1+1"), Cow::Borrowed("'=1+1"));
}

#[test]
fn test_escape_does_not_allocate_for_safe_strings() {
    let safe = "safe string";
    let escaped = escape_csv_injection(safe);
    assert!(matches!(escaped, Cow::Borrowed(_)));
}

#[test]
fn test_escape_allocates_for_unsafe_strings() {
    let unsafe_str = "=unsafe";
    let escaped = escape_csv_injection(unsafe_str);
    assert!(matches!(escaped, Cow::Owned(_)));
    assert_eq!(escaped, "'=unsafe");
}
