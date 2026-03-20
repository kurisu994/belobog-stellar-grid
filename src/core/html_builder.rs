/// HTML Table 拼装模块
///
/// 将 ParsedSheet 数据拼装为 HTML `<table>` 字符串，
/// 处理合并单元格、列宽、行高，并进行 XSS 防护。
use super::excel_reader::ParsedSheet;

/// 将 ParsedSheet 转换为 HTML table 字符串
pub fn build_html_table(sheet: &ParsedSheet) -> String {
    if sheet.rows.is_empty() {
        return String::from("<table></table>");
    }

    // 预估容量
    let estimated_capacity = sheet.rows.len() * sheet.col_widths.len() * 60 + 512;
    let mut html = String::with_capacity(estimated_capacity);

    // 嵌入默认样式（通过 class 作用域隔离，内联样式可覆盖）
    html.push_str(concat!(
        "<style>",
        ".bsg-preview-table{border-collapse:collapse;table-layout:auto;",
        "font-family:'Calibri','Microsoft YaHei',sans-serif}",
        ".bsg-preview-table td,.bsg-preview-table th{",
        "padding:2px 4px;",
        "border:0.5px solid #d9d9d9;",
        "vertical-align:middle;",
        "white-space:nowrap;",
        "font-size:11px;",
        "line-height:1.4;",
        "box-sizing:border-box}",
        "</style>",
    ));

    // 表格开始
    html.push_str("<table class=\"bsg-preview-table\">");

    // 列宽定义（使用 min-width 允许列按内容扩展）
    if !sheet.col_widths.is_empty() {
        html.push_str("<colgroup>");
        for w in &sheet.col_widths {
            html.push_str(&format!("<col style=\"min-width:{w:.0}px\">"));
        }
        html.push_str("</colgroup>");
    }

    // 表格主体
    html.push_str("<tbody>");
    for row in &sheet.rows {
        html.push_str("<tr");
        if let Some(h) = row.height {
            html.push_str(&format!(" style=\"height:{h:.0}px\""));
        }
        html.push('>');

        for cell in &row.cells {
            match cell {
                None => {
                    // 被合并占用的单元格不输出
                    continue;
                }
                Some(cell_data) => {
                    html.push_str("<td");

                    // 合并属性
                    if let Some(cs) = cell_data.col_span {
                        if cs > 1 {
                            html.push_str(&format!(" colspan=\"{cs}\""));
                        }
                    }
                    if let Some(rs) = cell_data.row_span {
                        if rs > 1 {
                            html.push_str(&format!(" rowspan=\"{rs}\""));
                        }
                    }

                    // 样式
                    if let Some(ref style) = cell_data.style {
                        html.push_str(" style=\"");
                        html.push_str(style);
                        html.push('"');
                    }

                    html.push('>');

                    // 内容（HTML 实体转义）
                    html.push_str(&escape_html(&cell_data.value));

                    html.push_str("</td>");
                }
            }
        }

        html.push_str("</tr>");
    }
    html.push_str("</tbody>");
    html.push_str("</table>");

    html
}

/// HTML 实体转义（防止 XSS 注入）
pub fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#x27;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::excel_reader::{ParsedCell, ParsedRow};

    #[test]
    fn test_escape_html_basic() {
        assert_eq!(escape_html("hello"), "hello");
        assert_eq!(escape_html("<script>"), "&lt;script&gt;");
        assert_eq!(escape_html("a & b"), "a &amp; b");
        assert_eq!(escape_html("\"test\""), "&quot;test&quot;");
        assert_eq!(escape_html("'test'"), "&#x27;test&#x27;");
    }

    #[test]
    fn test_escape_html_xss_vectors() {
        assert_eq!(
            escape_html("<img src=x onerror=alert(1)>"),
            "&lt;img src=x onerror=alert(1)&gt;"
        );
        assert_eq!(
            escape_html("<script>alert('xss')</script>"),
            "&lt;script&gt;alert(&#x27;xss&#x27;)&lt;/script&gt;"
        );
    }

    #[test]
    fn test_build_html_table_empty() {
        let sheet = ParsedSheet {
            name: "Test".to_string(),
            rows: Vec::new(),
            col_widths: Vec::new(),
            merged_cells: Vec::new(),
            truncated: None,
        };
        assert_eq!(build_html_table(&sheet), "<table></table>");
    }

    #[test]
    fn test_build_html_table_basic() {
        let sheet = ParsedSheet {
            name: "Test".to_string(),
            rows: vec![
                ParsedRow {
                    height: None,
                    cells: vec![
                        Some(ParsedCell {
                            value: "A1".to_string(),
                            style: None,
                            col_span: None,
                            row_span: None,
                        }),
                        Some(ParsedCell {
                            value: "B1".to_string(),
                            style: None,
                            col_span: None,
                            row_span: None,
                        }),
                    ],
                },
                ParsedRow {
                    height: None,
                    cells: vec![
                        Some(ParsedCell {
                            value: "A2".to_string(),
                            style: None,
                            col_span: None,
                            row_span: None,
                        }),
                        Some(ParsedCell {
                            value: "B2".to_string(),
                            style: None,
                            col_span: None,
                            row_span: None,
                        }),
                    ],
                },
            ],
            col_widths: vec![100.0, 150.0],
            merged_cells: Vec::new(),
            truncated: None,
        };

        let html = build_html_table(&sheet);
        assert!(html.contains("<table class=\"bsg-preview-table\">"));
        assert!(html.contains("<colgroup>"));
        assert!(html.contains("min-width:100px"));
        assert!(html.contains("min-width:150px"));
        assert!(html.contains("<td>A1</td>"));
        assert!(html.contains("<td>B2</td>"));
    }

    #[test]
    fn test_build_html_table_with_merge() {
        let sheet = ParsedSheet {
            name: "Test".to_string(),
            rows: vec![ParsedRow {
                height: None,
                cells: vec![
                    Some(ParsedCell {
                        value: "合并".to_string(),
                        style: None,
                        col_span: Some(2),
                        row_span: Some(2),
                    }),
                    None, // 被合并
                ],
            }],
            col_widths: vec![100.0, 100.0],
            merged_cells: Vec::new(),
            truncated: None,
        };

        let html = build_html_table(&sheet);
        assert!(html.contains("colspan=\"2\""));
        assert!(html.contains("rowspan=\"2\""));
        assert!(html.contains("合并"));
    }

    #[test]
    fn test_build_html_table_with_style() {
        let sheet = ParsedSheet {
            name: "Test".to_string(),
            rows: vec![ParsedRow {
                height: Some(30.0),
                cells: vec![Some(ParsedCell {
                    value: "styled".to_string(),
                    style: Some("font-weight:bold;color:#FF0000".to_string()),
                    col_span: None,
                    row_span: None,
                })],
            }],
            col_widths: vec![100.0],
            merged_cells: Vec::new(),
            truncated: None,
        };

        let html = build_html_table(&sheet);
        assert!(html.contains("height:30px"));
        assert!(html.contains("font-weight:bold;color:#FF0000"));
    }

    #[test]
    fn test_build_html_table_xss_protection() {
        let sheet = ParsedSheet {
            name: "Test".to_string(),
            rows: vec![ParsedRow {
                height: None,
                cells: vec![Some(ParsedCell {
                    value: "<script>alert('xss')</script>".to_string(),
                    style: None,
                    col_span: None,
                    row_span: None,
                })],
            }],
            col_widths: vec![100.0],
            merged_cells: Vec::new(),
            truncated: None,
        };

        let html = build_html_table(&sheet);
        // 确保 script 标签被转义
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }

    #[test]
    fn test_build_html_table_unicode() {
        let sheet = ParsedSheet {
            name: "Test".to_string(),
            rows: vec![ParsedRow {
                height: None,
                cells: vec![Some(ParsedCell {
                    value: "中文测试🎉".to_string(),
                    style: None,
                    col_span: None,
                    row_span: None,
                })],
            }],
            col_widths: vec![100.0],
            merged_cells: Vec::new(),
            truncated: None,
        };

        let html = build_html_table(&sheet);
        assert!(html.contains("中文测试🎉"));
    }
}
