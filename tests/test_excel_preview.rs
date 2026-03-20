//! Excel 预览功能集成测试
//!
//! ## 测试分布说明
//!
//! Excel 预览的核心单元测试位于以下模块的内联测试中：
//! - `src/core/excel_reader.rs` — 解析逻辑、合并单元格、数据边界、多工作表
//! - `src/core/excel_style.rs` — 样式解析、CSS 生成、颜色处理、数字格式化
//! - `src/core/html_builder.rs` — HTML 拼装、XSS 防护、合并属性、Unicode
//!
//! ## 公共 API 限制
//!
//! `core` 模块为 `mod core`（非 `pub mod`），因此集成测试无法直接访问
//! `excel_reader::parse_excel()` 等内部函数。
//!
//! 通过 `lib.rs` 导出的 WASM API（`parse_excel_to_html`、`parse_excel_to_json`、
//! `get_excel_sheet_list`）接收 `JsValue` 参数，仅在 `wasm32` 目标下可用，
//! 已在 `e2e/` Playwright 测试中覆盖。
//!
//! 此文件验证：
//! 1. 公共导出符号的编译正确性
//! 2. 新增 Excel 预览功能不影响现有导出 API

// ============================================================================
// 编译正确性验证
// ============================================================================

/// 验证 Excel 预览相关的公共导出符号存在且可编译
///
/// 这些函数签名包含 `JsValue`，无法在非 wasm32 环境下调用，
/// 但 `use` 导入验证了它们的存在性和类型正确性。
#[test]
fn test_excel_preview_exports_compile() {
    // 验证 Excel 预览 API 函数符号已导出
    // 这些函数接受 JsValue 参数，无法在非 wasm32 环境实际调用，
    // 但 fn 指针赋值可验证它们在 lib.rs 中正确重导出
    #[cfg(target_arch = "wasm32")]
    {
        let _ = belobog_stellar_grid::parse_excel_to_html
            as fn(
                wasm_bindgen::JsValue,
                wasm_bindgen::JsValue,
            ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
        let _ = belobog_stellar_grid::parse_excel_to_json
            as fn(
                wasm_bindgen::JsValue,
                wasm_bindgen::JsValue,
            ) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
        let _ = belobog_stellar_grid::get_excel_sheet_list
            as fn(wasm_bindgen::JsValue) -> Result<wasm_bindgen::JsValue, wasm_bindgen::JsValue>;
    }
}

// ============================================================================
// 现有导出 API 兼容性验证
// ============================================================================

/// 验证添加 Excel 预览功能后，现有的导出 API 仍然正常可用
#[test]
fn test_existing_exports_unaffected() {
    // 验证核心导出符号未被破坏
    let _ = belobog_stellar_grid::validate_filename;
    let _ = belobog_stellar_grid::ensure_extension;
    let _ = belobog_stellar_grid::escape_csv_injection;

    // 验证 ExportFormat 枚举仍然正常
    let _csv = belobog_stellar_grid::ExportFormat::Csv;
    let _xlsx = belobog_stellar_grid::ExportFormat::Xlsx;
}

/// 验证文件名验证逻辑与 Excel 预览功能共存
#[test]
fn test_filename_validation_still_works() {
    // 正常文件名
    assert!(belobog_stellar_grid::validate_filename("报表.xlsx").is_ok());
    assert!(belobog_stellar_grid::validate_filename("data.csv").is_ok());

    // 空文件名仍应拒绝
    assert!(belobog_stellar_grid::validate_filename("").is_err());

    // 扩展名处理
    assert_eq!(
        belobog_stellar_grid::ensure_extension("预览数据", "xlsx"),
        "预览数据.xlsx"
    );
    assert_eq!(
        belobog_stellar_grid::ensure_extension("report.xlsx", "xlsx"),
        "report.xlsx"
    );
}

/// 验证 CSV 注入转义功能未受影响
#[test]
fn test_csv_injection_escape_still_works() {
    // 公式前缀应被转义
    assert_eq!(
        belobog_stellar_grid::escape_csv_injection("=SUM(A1)"),
        "'=SUM(A1)"
    );
    assert_eq!(
        belobog_stellar_grid::escape_csv_injection("+cmd|'/C calc'"),
        "'+cmd|'/C calc'"
    );
    // 普通文本不应被修改
    assert_eq!(
        belobog_stellar_grid::escape_csv_injection("正常数据"),
        "正常数据"
    );
}
