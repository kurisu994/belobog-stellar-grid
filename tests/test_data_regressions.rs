//! JS 数据经真实导出入口生成 CSV/XLSX 的 WASM 回归测试。
#![cfg(target_arch = "wasm32")]

use belobog_stellar_grid::generate_data_bytes;
use calamine::{Data, Reader, Xlsx};
use std::io::{Cursor, Read};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

/// 解析测试数据，避免夹具绕过实际 JS 值转换路径。
fn json(source: &str) -> JsValue {
    js_sys::JSON::parse(source).unwrap()
}

/// 构造真实导出选项，树形与普通对象使用相同列配置。
fn options(columns: &str, xlsx: bool, tree: bool) -> JsValue {
    let opts = json(&format!(
        r#"{{"columns":{columns},"format":{}}}"#,
        u8::from(xlsx)
    ));
    if tree {
        js_sys::Reflect::set(&opts, &"childrenKey".into(), &"children".into()).unwrap();
        js_sys::Reflect::set(&opts, &"indentColumn".into(), &"name".into()).unwrap();
    }
    opts
}

/// 调用公开字节导出入口，不模拟 CSV/XLSX 写入过程。
fn export(data: JsValue, columns: &str, xlsx: bool, tree: bool) -> Vec<u8> {
    generate_data_bytes(data, Some(options(columns, xlsx, tree)))
        .unwrap()
        .to_vec()
}

/// 重读 XLSX，验证文件确实可解析且数据未丢失。
fn xlsx_rows(bytes: &[u8]) -> Vec<Vec<String>> {
    let mut workbook: Xlsx<_> = Xlsx::new(Cursor::new(bytes)).unwrap();
    workbook
        .worksheet_range_at(0)
        .unwrap()
        .unwrap()
        .rows()
        .map(|row| {
            row.iter()
                .map(|value| match value {
                    Data::Empty => String::new(),
                    Data::String(text) => text.clone(),
                    other => panic!("导出值应保持文本，实际为 {other:?}"),
                })
                .collect()
        })
        .collect()
}

/// 读取 XLSX 内的 XML，检查合并与样式确实写入文件。
fn xlsx_xml(bytes: &[u8], path: &str) -> String {
    let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
    let mut xml = String::new();
    archive
        .by_name(path)
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    xml
}

/// 解析真实 CSV 输出，覆盖引号、换行和 Unicode。
fn csv_rows(bytes: &[u8]) -> Vec<Vec<String>> {
    csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(bytes)
        .records()
        .map(|record| record.unwrap().iter().map(str::to_string).collect())
        .collect()
}

#[wasm_bindgen_test]
fn test_data_export_clamped_single_cell_xlsx_roundtrip() {
    let columns = r#"[{"title":"姓名🌸","key":"name"}]"#;
    for tree in [false, true] {
        for span in [
            r#""colSpan":999"#,
            r#""rowSpan":999"#,
            r#""colSpan":999,"rowSpan":999"#,
        ] {
            let data = json(&format!(r#"[{{"name":{{"value":"风堇🌸",{span}}}}}]"#));
            let bytes = export(data, columns, true, tree);
            assert_eq!(xlsx_rows(&bytes), vec![vec!["姓名🌸"], vec!["风堇🌸"]]);
            assert!(!xlsx_xml(&bytes, "xl/worksheets/sheet1.xml").contains("<mergeCell "));
        }
    }
}

#[wasm_bindgen_test]
fn test_data_export_boundary_clamp_keeps_valid_merge() {
    let columns = r#"[{"title":"名称","key":"name"},{"title":"备注","key":"note"}]"#;
    let data = json(
        r#"[
        {"name":{"value":"跨两列","colSpan":99},"note":{"value":"覆盖","colSpan":0}},
        {"name":"末行","note":{"value":"最后🌸","colSpan":99,"rowSpan":99}}
    ]"#,
    );
    let bytes = export(data, columns, true, false);
    assert_eq!(
        xlsx_rows(&bytes),
        vec![
            vec!["名称", "备注"],
            vec!["跨两列", ""],
            vec!["末行", "最后🌸"]
        ]
    );
    let sheet = xlsx_xml(&bytes, "xl/worksheets/sheet1.xml");
    assert!(sheet.contains("ref=\"A2:B2\""));
    assert!(!sheet.contains("ref=\"B3:B3\""));
}

#[wasm_bindgen_test]
fn test_data_export_number_precision_csv_and_xlsx_roundtrip() {
    let columns = r#"[{"title":"数值","key":"value"}]"#;
    let data = json(
        r#"[
        {"value":0.30000000000000004},
        {"value":1.2345678901234567},
        {"value":-1.2345678901234567},
        {"value":0.0000012345678901234567},
        {"value":1.0000000000000002},
        {"value":0.000000000000001}
    ]"#,
    );
    let expected = vec![
        vec!["数值"],
        vec!["0.30000000000000004"],
        vec!["1.2345678901234567"],
        vec!["-1.2345678901234567"],
        vec!["0.0000012345678901234567"],
        vec!["1.0000000000000002"],
        vec!["0.000000000000001"],
    ];
    assert_eq!(
        xlsx_rows(&export(data.clone(), columns, true, false)),
        expected
    );
    let mut csv_expected = expected;
    // CSV 保留既有公式注入防护，对负号开头文本加前缀。
    csv_expected[3][0] = "'-1.2345678901234567";
    assert_eq!(csv_rows(&export(data, columns, false, false)), csv_expected);
}

#[wasm_bindgen_test]
fn test_data_export_tree_wrapped_values_merge_style_and_indent() {
    let columns = r#"[{"title":"名称","key":"name"},{"title":"数量","key":"count"},{"title":"备注","key":"note"}]"#;
    let data = json(
        r##"[{
        "name":"根节点🌳","count":1,"note":"日本語",
        "children":[{
            "name":{"value":"子节点🌸","colSpan":2,"rowSpan":2,"style":{"bold":true,"fontColor":"#FF0000"}},
            "count":{"colSpan":0},"note":"한국어",
            "children":[{
                "name":{"rowSpan":0},"count":{"rowSpan":0},
                "note":{"value":"末尾🌟","colSpan":99,"rowSpan":99}
            }]
        }]
    }]"##,
    );
    let expected = vec![
        vec!["名称", "数量", "备注"],
        vec!["根节点🌳", "1", "日本語"],
        vec!["    子节点🌸", "", "한국어"],
        vec!["", "", "末尾🌟"],
    ];
    assert_eq!(
        csv_rows(&export(data.clone(), columns, false, true)),
        expected
    );
    let bytes = export(data, columns, true, true);
    assert_eq!(xlsx_rows(&bytes), expected);
    let sheet = xlsx_xml(&bytes, "xl/worksheets/sheet1.xml");
    assert!(sheet.contains("ref=\"A3:B4\""));
    assert!(sheet.contains("r=\"A3\" s=\""));
    let styles = xlsx_xml(&bytes, "xl/styles.xml");
    assert!(styles.contains("<b/>"));
    assert!(styles.contains("FFFF0000"));
}

#[wasm_bindgen_test]
fn test_data_export_unicode_csv_escaping_and_injection() {
    let columns = r#"[{"title":"说明🌸","key":"name"}]"#;
    let data = json(r#"[{"name":{"value":"中文,\"引号\"\n🌟"}},{"name":{"value":"=恶意公式()"}}]"#);
    assert_eq!(
        csv_rows(&export(data.clone(), columns, false, false)),
        vec![
            vec!["说明🌸"],
            vec!["中文,\"引号\"\n🌟"],
            vec!["'=恶意公式()"]
        ]
    );
    assert_eq!(
        xlsx_rows(&export(data, columns, true, false)),
        vec![
            vec!["说明🌸"],
            vec!["中文,\"引号\"\n🌟"],
            vec!["=恶意公式()"]
        ]
    );
}

#[wasm_bindgen_test]
fn test_data_export_wrapped_getter_errors_propagate() {
    let columns = r#"[{"title":"名称","key":"name"}]"#;
    for tree in [false, true] {
        for property in ["value", "colSpan", "rowSpan", "style"] {
            let data = js_sys::eval(&format!(
                "(() => {{ const cell = {{value: '值🌸'}}; Object.defineProperty(cell, '{property}', {{get() {{ throw new Error('拒绝读取'); }} }}); return [{{name: cell}}]; }})()"
            )).unwrap();
            let error = generate_data_bytes(data, Some(options(columns, true, tree))).unwrap_err();
            assert!(
                error
                    .as_string()
                    .unwrap()
                    .contains(&format!("读取对象属性 '{property}' 时发生异常"))
            );
        }
    }
}

#[wasm_bindgen_test]
fn test_data_export_nested_cell_style_getter_errors_propagate() {
    let columns = r#"[{"title":"名称","key":"name"}]"#;
    for tree in [false, true] {
        for (property, style) in [
            (
                "bold",
                "({get bold() { throw new Error('字体读取失败'); }})",
            ),
            (
                "border",
                "({get border() { throw new Error('边框读取失败'); }})",
            ),
            (
                "top",
                "({border: {get top() { throw new Error('上边框读取失败'); }}})",
            ),
        ] {
            let data =
                js_sys::eval(&format!("[{{name: {{value: '样式🌸', style: {style}}}}}]")).unwrap();
            let error = generate_data_bytes(data, Some(options(columns, true, tree))).unwrap_err();
            let message = error.as_string().unwrap();
            assert!(
                message.contains(property),
                "应保留异常属性上下文：{message}"
            );
            assert!(
                message.contains("读取") && message.contains("失败"),
                "不应静默忽略样式异常：{message}"
            );
        }
    }
}

#[wasm_bindgen_test]
fn test_data_export_nested_column_style_getter_errors_propagate() {
    for tree in [false, true] {
        for style_key in ["style", "headerStyle"] {
            for (property, style) in [
                (
                    "bold",
                    "({get bold() { throw new Error('字体读取失败'); }})",
                ),
                (
                    "top",
                    "({border: {get top() { throw new Error('上边框读取失败'); }}})",
                ),
            ] {
                let columns = js_sys::eval(&format!(
                    "[{{title: '名称', key: 'name', {style_key}: {style}}}]"
                ))
                .unwrap();
                let opts = options("[]", true, tree);
                js_sys::Reflect::set(&opts, &"columns".into(), &columns).unwrap();
                let error =
                    generate_data_bytes(json(r#"[{"name":"值🌸"}]"#), Some(opts)).unwrap_err();
                let message = error.as_string().unwrap();
                assert!(
                    message.contains(property),
                    "应保留异常属性上下文：{message}"
                );
                assert!(
                    message.contains("读取") && message.contains("失败"),
                    "不应静默忽略列样式异常：{message}"
                );
            }
        }
    }
}

#[wasm_bindgen_test]
fn test_data_export_tree_cycle_and_children_getter_errors() {
    let columns = r#"[{"title":"名称","key":"name"}]"#;
    let cyclic = js_sys::eval(
        "(() => { const node = {name: '循环🌸'}; node.children = [node]; return [node]; })()",
    )
    .unwrap();
    let error = generate_data_bytes(cyclic, Some(options(columns, false, true))).unwrap_err();
    assert!(
        error
            .as_string()
            .unwrap()
            .contains("树形数据嵌套层级超过最大限制")
    );

    let throwing =
        js_sys::eval("[{name: '根', get children() { throw new Error('子节点读取失败'); }}]")
            .unwrap();
    let error = generate_data_bytes(throwing, Some(options(columns, true, true))).unwrap_err();
    assert!(
        error
            .as_string()
            .unwrap()
            .contains("读取对象属性 'children' 时发生异常")
    );
}
