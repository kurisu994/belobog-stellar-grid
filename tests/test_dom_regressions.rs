//! 真实浏览器 DOM 回归：同步与分批导出共享行组、隐藏表头和跨度安全语义。
#![cfg(target_arch = "wasm32")]

use belobog_stellar_grid::{
    ExportFormat, export_table, export_table_to_csv_batch, export_table_to_xlsx_batch,
};
use std::io::{Cursor, Read};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

// 仅本集成测试使用浏览器，不改变 lib 测试的 Node 运行模式。
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = r#"
let root;
let blobs;
let originalCreate;
let originalClick;
export function setupDom(html) {
    root = document.createElement('div');
    root.innerHTML = html;
    document.body.append(root);
    blobs = [];
    originalCreate = URL.createObjectURL;
    originalClick = HTMLAnchorElement.prototype.click;
    URL.createObjectURL = function(blob) {
        blobs.push(blob);
        return originalCreate.call(URL, blob);
    };
    HTMLAnchorElement.prototype.click = function() {};
}
export function cleanupDom() {
    root.remove();
    URL.createObjectURL = originalCreate;
    HTMLAnchorElement.prototype.click = originalClick;
}
export async function takeDownload() {
    const blob = blobs.shift();
    if (!blob) throw new Error('没有捕获到下载文件');
    return new Uint8Array(await blob.arrayBuffer());
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = setupDom)]
    fn setup_dom(html: &str);
    #[wasm_bindgen(js_name = cleanupDom)]
    fn cleanup_dom();
    #[wasm_bindgen(js_name = takeDownload)]
    fn take_download() -> js_sys::Promise;
}

/// 保证每个异步用例结束后恢复下载钩子与 DOM。
struct DomFixture;

impl DomFixture {
    fn new(html: &str) -> Self {
        setup_dom(html);
        Self
    }
}

impl Drop for DomFixture {
    fn drop(&mut self) {
        cleanup_dom();
    }
}

async fn download_bytes() -> Vec<u8> {
    let value = JsFuture::from(take_download()).await.unwrap();
    js_sys::Uint8Array::new(&value).to_vec()
}

async fn export_sync(format: ExportFormat, header_style: Option<JsValue>) -> Vec<u8> {
    export_table(
        "dom-regression-table",
        Some("DOM回归".into()),
        Some(format),
        Some(true),
        None,
        None,
        None,
        header_style,
        None,
    )
    .unwrap();
    download_bytes().await
}

async fn export_batch(
    format: ExportFormat,
    tbody_id: Option<String>,
    header_style: Option<JsValue>,
) -> Vec<u8> {
    match format {
        ExportFormat::Csv => {
            export_table_to_csv_batch(
                "dom-regression-table".into(),
                tbody_id,
                Some("DOM回归.csv".into()),
                Some(1),
                Some(true),
                None,
                None,
                None,
            )
            .await
            .unwrap();
        }
        ExportFormat::Xlsx => {
            export_table_to_xlsx_batch(
                "dom-regression-table".into(),
                tbody_id,
                Some("DOM回归.xlsx".into()),
                Some(1),
                Some(true),
                None,
                None,
                header_style,
                None,
            )
            .await
            .unwrap();
        }
    }
    download_bytes().await
}

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

#[wasm_bindgen_test]
async fn test_dom_rowspan_row_groups_sync_and_batch() {
    let _fixture = DomFixture::new(
        r#"<table id="dom-regression-table">
        <thead>
            <tr><th rowspan="0">标题</th><th colspan="2">列</th></tr>
            <tr><th>乙</th><th>丙</th></tr>
        </thead>
        <tbody>
            <tr><td rowspan="0">中文🌟</td><td>1</td><td>甲</td></tr>
            <tr style="display:none"><td>隐藏</td><td>隐藏</td></tr>
            <tr><td>2</td><td>乙</td></tr>
        </tbody>
        <tbody>
            <tr><td rowspan="65534" colspan="2">边界</td><td>3</td></tr>
            <tr><td>4</td></tr>
        </tbody>
        <tfoot><tr><td>尾</td><td>5</td><td>终</td></tr></tfoot>
        </table>"#,
    );

    let csv = export_sync(ExportFormat::Csv, None).await;
    assert_eq!(csv, export_batch(ExportFormat::Csv, None, None).await);
    let csv_text = String::from_utf8(csv).unwrap();
    assert_eq!(
        csv_text.lines().collect::<Vec<_>>(),
        vec![
            "标题,列,",
            "标题,乙,丙",
            "中文🌟,1,甲",
            "中文🌟,2,乙",
            "边界,,3",
            "边界,,4",
            "尾,5,终",
        ]
    );

    let sync = export_sync(ExportFormat::Xlsx, None).await;
    let batch = export_batch(ExportFormat::Xlsx, None, None).await;
    for path in ["xl/worksheets/sheet1.xml", "xl/sharedStrings.xml"] {
        assert_eq!(xlsx_xml(&sync, path), xlsx_xml(&batch, path));
    }
    let sheet = xlsx_xml(&sync, "xl/worksheets/sheet1.xml");
    for merge in ["A1:A2", "B1:C1", "A3:A4", "A5:B6"] {
        assert!(sheet.contains(&format!("<mergeCell ref=\"{merge}\"/>")));
    }
    assert!(sheet.contains("ySplit=\"2\""));
}

#[wasm_bindgen_test]
async fn test_dom_hidden_header_count_and_data_style() {
    for (all_headers_hidden, hide_thead) in [(true, false), (false, false), (true, true)] {
        let thead_style = if hide_thead { "display:none" } else { "" };
        let second_header = if all_headers_hidden {
            ""
        } else {
            "<tr><th>可见表头</th></tr>"
        };
        let _fixture = DomFixture::new(&format!(
            r#"<table id="dom-regression-table">
            <thead style="{thead_style}"><tr style="display:none"><th>隐藏表头</th></tr>{second_header}</thead>
            <tbody><tr><td>数据一</td></tr><tr><td>数据二</td></tr></tbody>
            </table>"#
        ));
        let header_style = js_sys::JSON::parse(r#"{"bold":true}"#).unwrap();
        let sync = export_sync(ExportFormat::Xlsx, Some(header_style.clone())).await;
        let batch = export_batch(ExportFormat::Xlsx, None, Some(header_style)).await;
        let sheet = xlsx_xml(&sync, "xl/worksheets/sheet1.xml");
        assert_eq!(sheet, xlsx_xml(&batch, "xl/worksheets/sheet1.xml"));
        if all_headers_hidden {
            assert!(!sheet.contains("<pane"));
            assert!(sheet.contains(r#"<c r="A1" t="s">"#));
        } else {
            assert!(sheet.contains("ySplit=\"1\""));
            assert!(sheet.contains(r#"<c r="A2" t="s">"#));
        }
    }
}

#[wasm_bindgen_test]
async fn test_dom_hidden_table_has_no_exportable_data() {
    let _fixture = DomFixture::new(
        r#"<table id="dom-regression-table" style="display:none">
        <thead><tr><th>隐藏表头</th></tr></thead>
        <tbody><tr><td>不能泄漏🌟</td></tr></tbody></table>"#,
    );
    for format in [ExportFormat::Csv, ExportFormat::Xlsx] {
        let error = export_table(
            "dom-regression-table",
            None,
            Some(format),
            Some(true),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_err();
        assert!(error.as_string().unwrap().contains("可见数据"));
    }
    let csv_error = export_table_to_csv_batch(
        "dom-regression-table".into(),
        None,
        None,
        Some(1),
        Some(true),
        None,
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(csv_error.as_string().unwrap().contains("可见数据"));
    let xlsx_error = export_table_to_xlsx_batch(
        "dom-regression-table".into(),
        None,
        None,
        Some(1),
        Some(true),
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(xlsx_error.as_string().unwrap().contains("可见数据"));
}

#[wasm_bindgen_test]
async fn test_dom_large_declared_span_only_expands_real_rows() {
    let _fixture = DomFixture::new(
        r#"<table id="dom-regression-table"><tbody>
        <tr><td rowspan="65534" colspan="1000">恶意🌟</td></tr>
        </tbody></table>"#,
    );
    let sync = export_sync(ExportFormat::Csv, None).await;
    assert_eq!(sync, export_batch(ExportFormat::Csv, None, None).await);
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(sync.as_slice());
    let record = reader.records().next().unwrap().unwrap();
    assert_eq!(record.len(), 1000);
    assert_eq!(&record[0], "恶意🌟");
    assert!(record.iter().skip(1).all(str::is_empty));

    let sync = export_sync(ExportFormat::Xlsx, None).await;
    let batch = export_batch(ExportFormat::Xlsx, None, None).await;
    let sheet = xlsx_xml(&sync, "xl/worksheets/sheet1.xml");
    assert_eq!(sheet, xlsx_xml(&batch, "xl/worksheets/sheet1.xml"));
    assert!(sheet.contains(r#"<mergeCell ref="A1:ALL1"/>"#));
    assert!(!sheet.contains("A65534"));
}

#[wasm_bindgen_test]
async fn test_dom_external_tbody_keeps_its_own_row_group() {
    let _fixture = DomFixture::new(
        r#"<table id="dom-regression-table"><thead>
        <tr><th rowspan="65534">表头</th><th>列</th></tr>
        </thead></table>
        <table><tbody id="dom-regression-external">
        <tr><td rowspan="0">外部🌟</td><td>1</td></tr>
        <tr><td>2</td></tr>
        </tbody></table>"#,
    );
    let tbody_id = Some("dom-regression-external".into());
    let csv = export_batch(ExportFormat::Csv, tbody_id.clone(), None).await;
    assert_eq!(
        String::from_utf8(csv).unwrap(),
        "表头,列\n外部🌟,1\n外部🌟,2\n"
    );
    let xlsx = export_batch(ExportFormat::Xlsx, tbody_id, None).await;
    let sheet = xlsx_xml(&xlsx, "xl/worksheets/sheet1.xml");
    assert!(sheet.contains(r#"<mergeCell ref="A2:A3"/>"#));
    assert!(!sheet.contains(r#"<mergeCell ref="A1:A3"/>"#));
}

#[wasm_bindgen_test]
async fn test_dom_csv_ragged_rows_rejected_across_batches() {
    let _fixture = DomFixture::new(
        r#"<table id="dom-regression-table"><tbody>
        <tr><td>甲</td><td>乙</td></tr><tr><td>不等宽🌟</td></tr>
        </tbody></table>"#,
    );
    assert!(
        export_table(
            "dom-regression-table",
            None,
            Some(ExportFormat::Csv),
            Some(true),
            None,
            None,
            None,
            None,
            None,
        )
        .is_err()
    );
    for batch_size in [1, 2] {
        let error = export_table_to_csv_batch(
            "dom-regression-table".into(),
            None,
            None,
            Some(batch_size),
            Some(true),
            None,
            None,
            None,
        )
        .await
        .unwrap_err();
        assert!(error.as_string().unwrap().contains("列数不一致"));
    }
}

#[wasm_bindgen_test]
async fn test_dom_column_amplification_rejected_sync_and_batch() {
    let cells = r#"<td rowspan="65534" colspan="1000">放大</td>"#.repeat(17);
    let _fixture = DomFixture::new(&format!(
        r#"<table id="dom-regression-table"><tbody><tr>{cells}</tr></tbody></table>"#
    ));
    for format in [ExportFormat::Csv, ExportFormat::Xlsx] {
        let error = export_table(
            "dom-regression-table",
            None,
            Some(format),
            Some(true),
            None,
            None,
            None,
            None,
            None,
        )
        .unwrap_err();
        assert!(error.as_string().unwrap().contains("16384"));
    }
    let csv_error = export_table_to_csv_batch(
        "dom-regression-table".into(),
        None,
        None,
        Some(1),
        Some(true),
        None,
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(csv_error.as_string().unwrap().contains("16384"));
    let xlsx_error = export_table_to_xlsx_batch(
        "dom-regression-table".into(),
        None,
        None,
        Some(1),
        Some(true),
        None,
        None,
        None,
        None,
    )
    .await
    .unwrap_err();
    assert!(xlsx_error.as_string().unwrap().contains("16384"));
}
