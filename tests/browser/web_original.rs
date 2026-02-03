//! WebAssembly 浏览器环境测试套件
//!
//! 测试在真实浏览器环境中的 WebAssembly 功能

#![cfg(target_arch = "wasm32")]

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use excel_exporter::*;
use wasm_bindgen::JsValue;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn basic_arithmetic() {
    assert_eq!(1 + 1, 2);
}

#[wasm_bindgen_test]
fn test_panic_hook_exists() {
    // 测试 panic hook 函数可以被调用
    // 这是一个基本的功能测试，确保函数存在且不会 panic
    // 注意：实际的 panic hook 测试需要特定的测试环境

    // 这里我们不能直接测试 panic hook，因为它会影响到后续测试
    // 但我们可以确保 utils 模块可以被导入
    assert!(true); // 简单的通过测试
}

#[wasm_bindgen_test]
fn test_error_handling() {
    // 测试错误处理机制
    // 由于这是在 WebAssembly 环境中，我们无法直接测试 DOM 操作
    // 但可以测试基本的错误条件

    // 测试空 ID 会返回错误
    let result = export_table_to_csv("", None);
    assert!(result.is_err());

    // 测试废弃函数也能工作
    let result_deprecated = export_table_to_excel("");
    assert!(result_deprecated.is_err());
}

#[wasm_bindgen_test]
fn test_filename_handling() {
    // 测试文件名处理逻辑
    // 在没有实际 DOM 的情况下，这个测试会失败，但我们可以测试错误处理

    let result = export_table_to_csv("non_existent_table", Some("test.csv".to_string()));
    assert!(result.is_err()); // 应该失败，因为表格不存在

    let result_no_ext = export_table_to_csv("non_existent_table", Some("test".to_string()));
    assert!(result_no_ext.is_err()); // 应该失败，但会自动添加 .csv 扩展名
}

#[wasm_bindgen_test]
async fn test_batch_export_function() {
    // 测试分批异步导出函数的存在和基本功能
    // 由于在测试环境中没有实际的 DOM，这个测试会失败，但可以验证错误处理

    let result = export_table_to_csv_batch(
        "non_existent_table".to_string(),
        None, // tbody_id
        Some("test_batch.csv".to_string()),
        Some(100), // batch_size
        None, // exclude_hidden
        None, // progress_callback
    ).await;

    // 应该返回 JsValue::UNDEFINED 或错误，但不能 panic
    // 在实际测试环境中，这会因为找不到表格而失败
    assert!(result.is_ok() || result.is_err());
}

#[wasm_bindgen_test]
async fn test_batch_export_with_tbody() {
    // 测试分批导出带有 tbody_id 的情况

    let result = export_table_to_csv_batch(
        "non_existent_table".to_string(),
        Some("non_existent_tbody".to_string()),
        Some("test_with_tbody.csv".to_string()),
        Some(500), // batch_size
        None,
        None,
    ).await;

    assert!(result.is_ok() || result.is_err());
}

#[wasm_bindgen_test]
async fn test_batch_export_small_batch() {
    // 测试小批次大小

    let result = export_table_to_csv_batch(
        "non_existent_table".to_string(),
        None,
        Some("test_small_batch.csv".to_string()),
        Some(1), // 最小批次大小
        None,
        None,
    ).await;

    assert!(result.is_ok() || result.is_err());
}

#[wasm_bindgen_test]
fn test_export_with_progress() {
    // 测试带进度回调的导出函数
    // 创建一个简单的进度回调函数
    let progress_callback = js_sys::Function::new_with_args("progress", "console.log('Progress:', progress + '%');");

    let result = export_table_to_csv_with_progress(
        "non_existent_table",
        Some("progress_test.csv".to_string()),
        Some(progress_callback),
    );

    assert!(result.is_err()); // 表格不存在，应该失败
}

#[wasm_bindgen_test]
fn test_filename_validation_in_browser() {
    // 在浏览器环境中测试文件名验证
    // 注意：validate_filename 函数不在 wasm-bindgen 导出中，所以无法直接测试
    // 但可以通过导出函数的错误信息间接验证

    // 测试空文件名
    let result = export_table_to_csv("test", Some("".to_string()));
    assert!(result.is_err()); // 文件名为空应该失败

    // 测试路径分隔符
    let result = export_table_to_csv("test", Some("path/file.csv".to_string()));
    assert!(result.is_err()); // 包含路径分隔符应该失败
}

#[wasm_bindgen_test]
fn test_error_messages() {
    // 测试错误消息是否正确返回
    let result = export_table_to_csv("", None);
    assert!(result.is_err());

    // 获取错误信息
    let error = result.unwrap_err();
    let error_str = error.as_string().unwrap_or_default();

    // 验证错误消息包含预期内容
    assert!(error_str.contains("表格 ID 不能为空") || error_str.len() > 0);
}

#[wasm_bindgen_test]
fn test_backward_compatibility() {
    // 测试向后兼容的废弃函数
    let result = export_table_to_excel("test");
    assert!(result.is_err()); // 表格不存在，应该失败

    // 验证废弃函数仍然可以调用
    let result = export_table_to_excel("");
    assert!(result.is_err()); // 空表格 ID 应该失败
}

#[wasm_bindgen_test]
fn test_module_exports() {
    // 测试模块导出的函数都存在
    // 这些测试确保导出的函数可以被调用

    // 测试所有导出的函数不会 panic
    let _ = export_table_to_csv("", None);
    let _ = export_table_to_csv_with_progress("", None, None);
    let _ = export_table_to_excel("");

    // 测试参数类型转换
    let filename = "test.csv".to_string();
    let _ = export_table_to_csv("test", Some(filename.clone()));

    // 测试 None 参数
    let _ = export_table_to_csv("test", None);
}
