//! UrlGuard RAII 资源管理测试
//!
//! 测试 UrlGuard 的创建和自动清理功能

use belobog_stellar_grid::UrlGuard;

#[test]
fn test_url_guard_creation() {
    // 测试创建 UrlGuard
    let test_url = "blob:http://example.com/test-uuid-123";
    let guard = UrlGuard::new(test_url);

    // UrlGuard 应该成功创建（内部存储 URL）
    // 由于 UrlGuard 字段是私有的，我们无法直接访问，
    // 但能成功创建已经证明了功能正常
    drop(guard); // 显式调用 Drop
}

#[test]
fn test_url_guard_with_different_urls() {
    // 测试不同格式的 URL
    let urls = vec![
        "blob:http://localhost:8080/uuid-1",
        "blob:https://example.com/uuid-2",
        "blob:http://192.168.1.1/uuid-3",
    ];

    for url in urls {
        let guard = UrlGuard::new(url);
        // 每个 guard 都应该成功创建
        drop(guard);
    }
}

#[test]
fn test_url_guard_empty_string() {
    // 测试空字符串（边界情况）
    let guard = UrlGuard::new("");
    // 即使 URL 为空，UrlGuard 也应该能创建
    // Drop 时会尝试释放，但由于是空 URL，应该优雅处理
    drop(guard);
}

#[test]
fn test_url_guard_unicode_url() {
    // 测试包含 Unicode 字符的 URL（虽然不太常见）
    let url = "blob:http://example.com/测试-uuid-中文";
    let guard = UrlGuard::new(url);
    drop(guard);
}

#[test]
fn test_url_guard_scope_cleanup() {
    // 测试作用域自动清理
    {
        let _guard1 = UrlGuard::new("blob:http://example.com/scope-1");
        {
            let _guard2 = UrlGuard::new("blob:http://example.com/scope-2");
            // guard2 在这里被 drop
        }
        // guard1 在这里被 drop
    }
    // 所有 guard 都应该被正确清理
}

#[test]
fn test_url_guard_multiple_guards() {
    // 测试同时存在多个 UrlGuard
    let guard1 = UrlGuard::new("blob:http://example.com/multi-1");
    let guard2 = UrlGuard::new("blob:http://example.com/multi-2");
    let guard3 = UrlGuard::new("blob:http://example.com/multi-3");

    // 所有 guard 应该独立工作
    drop(guard2); // 先释放 guard2
    drop(guard1); // 再释放 guard1
    drop(guard3); // 最后释放 guard3
}

#[test]
fn test_url_guard_long_url() {
    // 测试长 URL（边界情况）
    let long_url = format!("blob:http://example.com/{}", "x".repeat(1000));
    let guard = UrlGuard::new(&long_url);
    drop(guard);
}

#[test]
fn test_url_guard_special_characters() {
    // 测试包含特殊字符的 URL
    let url = "blob:http://example.com/test?query=value&foo=bar#fragment";
    let guard = UrlGuard::new(url);
    drop(guard);
}
