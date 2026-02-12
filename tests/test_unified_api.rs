//! 统一API的枚举测试
//!
//! 主要测试 `ExportFormat` 枚举的默认值和比较逻辑。
//! 注意：`export_table` 函数依赖 WASM 环境，无法在标准 `cargo test` 中直接测试。

use belobog_stellar_grid::ExportFormat;

#[test]
fn test_export_format_default() {
    let format = ExportFormat::default();
    assert_eq!(format, ExportFormat::Csv);
}

#[test]
fn test_export_format_csv() {
    let format = ExportFormat::Csv;
    assert_eq!(format, ExportFormat::Csv);
}

#[test]
fn test_export_format_xlsx() {
    let format = ExportFormat::Xlsx;
    assert_eq!(format, ExportFormat::Xlsx);
}

#[test]
fn test_export_format_equality() {
    assert_eq!(ExportFormat::Csv, ExportFormat::default());
    assert_ne!(ExportFormat::Xlsx, ExportFormat::Csv);
}
