//! 公开 WASM 入口的参数校验回归，实际调用生产解析路径。
#![cfg(target_arch = "wasm32")]

use belobog_stellar_grid::{export_tables_xlsx, generate_data_bytes, parse_excel_to_json};
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

/// 构造单格数据，避免验证被空数据错误掩盖。
fn data() -> JsValue {
    Array::of1(&Array::of1(&JsValue::from_str("中文值"))).into()
}

/// 构造只含一个选项的对象。
fn option(name: &str, value: f64) -> JsValue {
    let options = Object::new();
    Reflect::set(
        &options,
        &JsValue::from_str(name),
        &JsValue::from_f64(value),
    )
    .unwrap();
    options.into()
}

#[wasm_bindgen_test]
fn test_options_format_rejects_fraction_nonfinite_and_negative() {
    for value in [0.5, 1.5, -1.0, f64::NAN, f64::INFINITY, 2.0] {
        let error = generate_data_bytes(data(), Some(option("format", value))).unwrap_err();
        assert!(error.as_string().unwrap().contains("format"));
    }
    for value in [0.0, 1.0] {
        assert!(generate_data_bytes(data(), Some(option("format", value))).is_ok());
    }
}

#[wasm_bindgen_test]
fn test_options_freeze_rejects_fraction_and_narrowing_overflow() {
    for (name, invalid) in [
        ("freezeRows", 1.5),
        ("freezeRows", 4_294_967_296.0),
        ("freezeCols", 65_536.0),
        ("freezeCols", -1.0),
    ] {
        let error = generate_data_bytes(data(), Some(option(name, invalid))).unwrap_err();
        assert!(error.as_string().unwrap().contains(name));
    }
    assert!(generate_data_bytes(data(), Some(option("freezeCols", 0.0))).is_ok());
}

#[wasm_bindgen_test]
fn test_options_preview_rejects_invalid_number_before_parsing_file() {
    for (name, invalid) in [
        ("sheetIndex", 0.5),
        ("maxRows", 0.0),
        ("maxCols", f64::INFINITY),
    ] {
        let error = parse_excel_to_json(&[], option(name, invalid)).unwrap_err();
        assert!(error.as_string().unwrap().contains(name));
    }
}

#[wasm_bindgen_test]
fn test_options_getters_propagate_for_export_preview_and_sheet_config() {
    for name in ["columns", "filename", "format", "freezeRows", "headerStyle"] {
        let object = Function::new_no_args(&format!(
            "return {{ get {name}() {{ throw new Error('读取失败'); }} }}"
        ))
        .call0(&JsValue::NULL)
        .unwrap();
        let error = generate_data_bytes(data(), Some(object)).unwrap_err();
        assert!(error.as_string().unwrap().contains(name));
    }
    let preview =
        Function::new_no_args("return {get includeStyles(){throw new Error('读取失败')}}")
            .call0(&JsValue::NULL)
            .unwrap();
    assert!(
        parse_excel_to_json(&[], preview)
            .unwrap_err()
            .as_string()
            .unwrap()
            .contains("includeStyles")
    );
    let sheets = Function::new_no_args(
        "return [{tableId:'t',get sheetName(){throw new Error('读取失败')}}]",
    )
    .call0(&JsValue::NULL)
    .unwrap();
    assert!(
        export_tables_xlsx(sheets, None, None, None, None, None)
            .unwrap_err()
            .as_string()
            .unwrap()
            .contains("sheetName")
    );
}
