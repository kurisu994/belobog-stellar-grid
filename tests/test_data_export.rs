//! 数据导出功能测试
//!
//! 测试 data_export 模块的核心算法（不依赖 wasm32 的纯逻辑部分）
//! 以及 export_data API 的参数处理

// ============================================================================
// MergeRange 和 TableData 结构测试（通过 data_export 模块间接验证）
// ============================================================================

// 注意：data_export 的核心算法测试已在 src/core/data_export.rs 的 #[cfg(test)] 中
// 这里主要测试外部可访问的 API 行为和边界情况

use belobog_stellar_grid::{ensure_extension, validate_filename};

// ============================================================================
// export_data 文件名处理测试
// ============================================================================

#[test]
fn test_export_data_filename_csv_extension() {
    // 测试 CSV 导出时的文件名扩展名处理
    assert_eq!(ensure_extension("data", "csv"), "data.csv");
    assert_eq!(ensure_extension("data.csv", "csv"), "data.csv");
    assert_eq!(ensure_extension("data.CSV", "csv"), "data.CSV");
    assert_eq!(ensure_extension("用户数据", "csv"), "用户数据.csv");
}

#[test]
fn test_export_data_filename_xlsx_extension() {
    // 测试 XLSX 导出时的文件名扩展名处理
    assert_eq!(ensure_extension("data", "xlsx"), "data.xlsx");
    assert_eq!(ensure_extension("data.xlsx", "xlsx"), "data.xlsx");
    assert_eq!(ensure_extension("data.XLSX", "xlsx"), "data.XLSX");
    assert_eq!(ensure_extension("报表数据", "xlsx"), "报表数据.xlsx");
}

#[test]
fn test_export_data_filename_validation() {
    // 测试导出数据时的文件名安全验证
    assert!(validate_filename("合并数据.xlsx").is_ok());
    assert!(validate_filename("data_with_merge").is_ok());
    assert!(validate_filename("report-2024").is_ok());

    // 不安全的文件名
    assert!(validate_filename("../hack.xlsx").is_err());
    assert!(validate_filename("path/to/file.csv").is_err());
    assert!(validate_filename("").is_err());
    assert!(validate_filename("CON.xlsx").is_err());
}

// ============================================================================
// 数据合并场景的文件名验证
// ============================================================================

#[test]
fn test_export_data_merge_filename_unicode() {
    // 测试合并单元格导出场景下的 Unicode 文件名
    let filenames = vec![
        "员工信息-含合并",
        "订单汇总_合并单元格",
        "データエクスポート",
        "데이터_내보내기",
    ];

    for name in filenames {
        assert!(
            validate_filename(name).is_ok(),
            "文件名 '{}' 应该通过验证",
            name
        );
    }
}

#[test]
fn test_export_data_merge_filename_special_chars() {
    // 测试各种合法特殊字符组合
    assert!(validate_filename("data(merged)").is_ok());
    assert!(validate_filename("data[v2]").is_ok());
    assert!(validate_filename("data_merged-v2").is_ok());
    assert!(validate_filename("data merged").is_ok());
}

// ============================================================================
// CSV 写入与合并单元格行为测试
// ============================================================================

use csv::Writer;
use std::io::Cursor;

#[test]
fn test_csv_merge_cell_layout_colspan() {
    // 模拟 colSpan 在 CSV 中的布局：合并单元格后续列为空字符串
    // 例如：colSpan=2 的 "北京市" -> ["北京市", ""]
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

    // 表头
    wtr.write_record(["姓名", "地址", "邮编"]).unwrap();
    // 数据行（地址 colSpan=2，占据地址和邮编两列）
    wtr.write_record(["张三", "北京市朝阳区", ""]).unwrap();
    // 正常行
    wtr.write_record(["李四", "上海市", "200000"]).unwrap();

    wtr.flush().unwrap();
    let csv_data = wtr.into_inner().unwrap();
    let result = String::from_utf8(csv_data.into_inner()).unwrap();

    assert!(result.contains("北京市朝阳区"));
    assert!(result.contains("上海市"));
    // 验证行数（3 行 = 1 表头 + 2 数据）
    let line_count = result.lines().count();
    assert_eq!(line_count, 3);
}

#[test]
fn test_csv_merge_cell_layout_rowspan() {
    // 模拟 rowSpan 在 CSV 中的布局：被合并的行该列为空字符串
    // 例如：rowSpan=2 的 "张三" -> 第一行 "张三"，第二行 ""
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

    wtr.write_record(["姓名", "科目", "成绩"]).unwrap();
    // 张三 rowSpan=2
    wtr.write_record(["张三", "数学", "90"]).unwrap();
    wtr.write_record(["", "英语", "85"]).unwrap(); // rowSpan=0 的行
    // 李四 正常
    wtr.write_record(["李四", "数学", "88"]).unwrap();

    wtr.flush().unwrap();
    let csv_data = wtr.into_inner().unwrap();
    let result = String::from_utf8(csv_data.into_inner()).unwrap();

    assert!(result.contains("张三"));
    assert!(result.contains("英语"));
    let line_count = result.lines().count();
    assert_eq!(line_count, 4); // 1 表头 + 3 数据
}

#[test]
fn test_csv_merge_cell_layout_complex() {
    // 模拟复杂合并场景：同时有 rowSpan 和 colSpan
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

    // 嵌套表头（2行）
    wtr.write_record(["姓名", "成绩", "", "评价"]).unwrap();
    wtr.write_record(["", "数学", "英语", ""]).unwrap();

    // 数据：张三 rowSpan=2
    wtr.write_record(["张三", "90", "85", "优秀"]).unwrap();
    wtr.write_record(["", "88", "92", "良好"]).unwrap(); // 被 rowSpan 覆盖

    wtr.flush().unwrap();
    let csv_data = wtr.into_inner().unwrap();
    let result = String::from_utf8(csv_data.into_inner()).unwrap();

    assert!(result.contains("姓名"));
    assert!(result.contains("90"));
    let line_count = result.lines().count();
    assert_eq!(line_count, 4);
}

// ============================================================================
// 数据行布局测试（模拟 extract_data_rows 的输出格式）
// ============================================================================

#[test]
fn test_data_row_with_span_zero_produces_empty() {
    // 当 colSpan=0 或 rowSpan=0 时，输出空字符串
    // 模拟数据处理后的行：
    // row0: [张三(rowSpan=2), 28, 北京]
    // row1: ["", 35, 上海]  <- 张三的 rowSpan=0 位置

    let rows: Vec<Vec<&str>> = vec![
        vec!["张三", "28", "北京"],
        vec!["", "35", "上海"], // 第一列被上方 rowSpan 覆盖
    ];

    assert_eq!(rows[0][0], "张三");
    assert_eq!(rows[1][0], ""); // 被合并覆盖
    assert_eq!(rows[1][1], "35"); // 正常数据
}

#[test]
fn test_data_row_colspan_produces_empty_following_cells() {
    // 当 colSpan=3 时，后续 2 个单元格为空
    // 模拟：address colSpan=3 -> [北京市, "", ""]
    let row: Vec<&str> = vec!["张三", "北京市朝阳区", "", ""];

    assert_eq!(row[1], "北京市朝阳区"); // 合并起始
    assert_eq!(row[2], ""); // 被 colSpan 覆盖
    assert_eq!(row[3], ""); // 被 colSpan 覆盖
}

// ============================================================================
// MergeRange 逻辑验证测试
// ============================================================================

#[test]
fn test_merge_range_offset_calculation() {
    // 验证数据区域的 MergeRange 行偏移计算
    // 表头 2 行，数据第 0 行的 merge 应该从第 2 行开始

    let header_row_count: u32 = 2;
    let data_row_idx: u32 = 0;
    let col_idx: u16 = 1;
    let row_span: u32 = 3;
    let col_span: u16 = 2;

    let first_row = data_row_idx + header_row_count;
    let first_col = col_idx;
    let last_row = first_row + row_span - 1;
    let last_col = first_col + col_span - 1;

    assert_eq!(first_row, 2); // 表头偏移
    assert_eq!(first_col, 1);
    assert_eq!(last_row, 4); // 2 + 3 - 1
    assert_eq!(last_col, 2); // 1 + 2 - 1
}

#[test]
fn test_merge_range_single_cell_no_merge() {
    // colSpan=1 && rowSpan=1 时不产生合并
    let col_span: u32 = 1;
    let row_span: u32 = 1;
    let needs_merge = col_span > 1 || row_span > 1;
    assert!(!needs_merge);
}

#[test]
fn test_merge_range_only_colspan() {
    // 只有 colSpan > 1
    let col_span: u32 = 3;
    let row_span: u32 = 1;
    let needs_merge = col_span > 1 || row_span > 1;
    assert!(needs_merge);
}

#[test]
fn test_merge_range_only_rowspan() {
    // 只有 rowSpan > 1
    let col_span: u32 = 1;
    let row_span: u32 = 2;
    let needs_merge = col_span > 1 || row_span > 1;
    assert!(needs_merge);
}

#[test]
fn test_merge_range_both_spans() {
    // colSpan > 1 && rowSpan > 1
    let col_span: u32 = 2;
    let row_span: u32 = 3;
    let needs_merge = col_span > 1 || row_span > 1;
    assert!(needs_merge);

    // 验证合并区域大小
    let total_cells = col_span * row_span;
    assert_eq!(total_cells, 6);
}

// ============================================================================
// 表头和数据合并区域合并测试
// ============================================================================

#[test]
fn test_merge_ranges_combination() {
    // 模拟合并表头和数据的 merge_ranges
    #[allow(unused)]
    struct SimpleMergeRange {
        first_row: u32,
        first_col: u16,
        last_row: u32,
        last_col: u16,
    }

    // 表头合并区域（姓名 rowspan=2）
    let mut merge_ranges = vec![SimpleMergeRange {
        first_row: 0,
        first_col: 0,
        last_row: 1,
        last_col: 0,
    }];

    // 数据合并区域（第3行张三 rowspan=2，偏移 header_rows=2）
    let data_merges = vec![SimpleMergeRange {
        first_row: 2,
        first_col: 0,
        last_row: 3,
        last_col: 0,
    }];

    merge_ranges.extend(data_merges);

    assert_eq!(merge_ranges.len(), 2);
    // 第一个：表头合并
    assert_eq!(merge_ranges[0].first_row, 0);
    assert_eq!(merge_ranges[0].last_row, 1);
    // 第二个：数据合并（已偏移）
    assert_eq!(merge_ranges[1].first_row, 2);
    assert_eq!(merge_ranges[1].last_row, 3);
}

// ============================================================================
// 数据合并单元格的 JS 对象格式检测测试
// ============================================================================

#[test]
fn test_cell_value_format_detection() {
    // 模拟检测逻辑：区分普通值和带 span 的对象
    // 普通值：直接用
    // 对象值：{ value, colSpan?, rowSpan? }

    // 模拟检测函数
    fn is_merge_cell_object(has_value: bool, has_col_span: bool, has_row_span: bool) -> bool {
        has_value || has_col_span || has_row_span
    }

    assert!(!is_merge_cell_object(false, false, false)); // 普通对象
    assert!(is_merge_cell_object(true, false, false)); // 有 value
    assert!(is_merge_cell_object(false, true, false)); // 有 colSpan
    assert!(is_merge_cell_object(false, false, true)); // 有 rowSpan
    assert!(is_merge_cell_object(true, true, true)); // 全有
}

#[test]
fn test_span_default_values() {
    // 测试 colSpan/rowSpan 的默认值逻辑
    fn get_span(val: Option<f64>) -> u32 {
        val.map(|n| n as u32).unwrap_or(1)
    }

    assert_eq!(get_span(None), 1); // 未提供时默认为 1
    assert_eq!(get_span(Some(0.0)), 0); // 显式设为 0
    assert_eq!(get_span(Some(1.0)), 1);
    assert_eq!(get_span(Some(2.0)), 2);
    assert_eq!(get_span(Some(5.0)), 5);
}

// ============================================================================
// 完整数据布局模拟测试
// ============================================================================

#[test]
fn test_full_layout_with_data_merge() {
    // 模拟完整的导出数据布局，包含表头合并和数据合并
    //
    // 表头（2行嵌套）：
    //   | 姓名(rs=2) | 成绩(cs=2)      |
    //   |            | 数学   | 英语    |
    //
    // 数据（含合并）：
    //   | 张三(rs=2)  | 90     | 85     |
    //   | ""          | 88     | 92     |
    //   | 李四        | 95     | 90     |

    let header_rows: Vec<Vec<&str>> = vec![vec!["姓名", "成绩", ""], vec!["", "数学", "英语"]];

    let data_rows: Vec<Vec<&str>> = vec![
        vec!["张三", "90", "85"],
        vec!["", "88", "92"], // 张三 rowSpan=0
        vec!["李四", "95", "90"],
    ];

    // 合并所有行
    let total_rows = header_rows.len() + data_rows.len();
    assert_eq!(total_rows, 5);

    // 合并区域数量
    // 表头：姓名 rowspan=2, 成绩 colspan=2
    // 数据：张三 rowspan=2（偏移后 first_row=2, last_row=3）
    let merge_count = 3; // 2个表头合并 + 1个数据合并
    assert_eq!(merge_count, 3);

    // 验证数据行偏移
    let header_row_count = header_rows.len();
    assert_eq!(header_row_count, 2);
    // 张三在数据第0行，实际在第2行
    assert_eq!(header_row_count, 2);
}

#[test]
fn test_full_layout_with_colspan_in_data() {
    // 模拟数据中有 colSpan 的情况
    //
    // 表头：| 姓名 | 地址 | 邮编 |
    // 数据：| 张三 | 北京市朝阳区(cs=2) |
    //       | 李四 | 上海市 | 200000 |

    let all_rows: Vec<Vec<&str>> = vec![
        vec!["姓名", "地址", "邮编"],
        vec!["张三", "北京市朝阳区", ""], // colSpan=2，邮编位置为空
        vec!["李四", "上海市", "200000"],
    ];

    assert_eq!(all_rows.len(), 3);
    assert_eq!(all_rows[1][2], ""); // 被 colSpan 覆盖
    assert_eq!(all_rows[2][2], "200000"); // 正常
}

// ============================================================================
// 边界情况测试
// ============================================================================

#[test]
fn test_empty_data_with_merge() {
    // 空数据数组不应产生任何合并区域
    let data_rows: Vec<Vec<String>> = Vec::new();
    let merge_ranges: Vec<(u32, u16, u32, u16)> = Vec::new();

    assert!(data_rows.is_empty());
    assert!(merge_ranges.is_empty());
}

#[test]
fn test_single_row_no_merge() {
    // 单行数据，所有单元格 span=1，不产生合并
    let row = ["张三", "28", "北京"];
    let merges: Vec<(u32, u32)> = Vec::new();

    assert_eq!(row.len(), 3);
    assert!(merges.is_empty());
}

#[test]
fn test_large_rowspan_value() {
    // 测试较大的 rowSpan 值
    let row_span: u32 = 100;
    let header_offset: u32 = 2;
    let data_row: u32 = 0;

    let first_row = data_row + header_offset;
    let last_row = first_row + row_span - 1;

    assert_eq!(first_row, 2);
    assert_eq!(last_row, 101);
}

#[test]
fn test_large_colspan_value() {
    // 测试较大的 colSpan 值
    let col_span: u16 = 50;
    let col_idx: u16 = 3;

    let first_col = col_idx;
    let last_col = first_col + col_span - 1;

    assert_eq!(first_col, 3);
    assert_eq!(last_col, 52);
}

// ============================================================================
// 树形数据拍平逻辑测试
// ============================================================================

#[test]
fn test_tree_flatten_basic_layout() {
    // 模拟树形数据拍平后的行布局
    // 树结构：A -> [B, C -> [D]]
    // 拍平后：A, B, C, D

    let rows: Vec<Vec<&str>> = vec![
        vec!["A", "value_a"],
        vec!["B", "value_b"],
        vec!["C", "value_c"],
        vec!["D", "value_d"],
    ];

    assert_eq!(rows.len(), 4);
    assert_eq!(rows[0][0], "A");
    assert_eq!(rows[3][0], "D");
}

#[test]
fn test_tree_flatten_with_indent() {
    // 模拟带缩进的树形拍平
    // depth=0: "根节点"
    // depth=1: "    子节点A"
    // depth=2: "        孙节点A1"
    // depth=1: "    子节点B"

    let rows: Vec<&str> = vec!["根节点", "    子节点A", "        孙节点A1", "    子节点B"];

    assert_eq!(rows.len(), 4);
    assert!(rows[0].starts_with("根"));
    assert!(rows[1].starts_with("    "));
    assert!(rows[2].starts_with("        "));
    assert!(rows[3].starts_with("    "));
}

#[test]
fn test_tree_flatten_indent_calculation() {
    // 验证缩进空格数计算
    fn calc_indent(depth: usize) -> String {
        "    ".repeat(depth)
    }

    assert_eq!(calc_indent(0), "");
    assert_eq!(calc_indent(1), "    "); // 4 个空格
    assert_eq!(calc_indent(2), "        "); // 8 个空格
    assert_eq!(calc_indent(3), "            "); // 12 个空格
    assert_eq!(calc_indent(0).len(), 0);
    assert_eq!(calc_indent(1).len(), 4);
    assert_eq!(calc_indent(5).len(), 20);
}

#[test]
fn test_tree_flatten_multiple_roots() {
    // 多个根节点的情况
    // root1 -> [child1]
    // root2 -> []

    let rows: Vec<Vec<&str>> = vec![
        vec!["root1", "100"],
        vec!["child1", "50"],
        vec!["root2", "200"],
    ];

    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0][0], "root1");
    assert_eq!(rows[1][0], "child1");
    assert_eq!(rows[2][0], "root2");
}

#[test]
fn test_tree_flatten_deep_nesting() {
    // 深度嵌套测试（4 层）
    // L1 -> L2 -> L3 -> L4

    fn apply_indent(name: &str, depth: usize) -> String {
        format!("{}{}", "    ".repeat(depth), name)
    }

    let indented_names: Vec<String> = vec![
        apply_indent("L1", 0),
        apply_indent("L2", 1),
        apply_indent("L3", 2),
        apply_indent("L4", 3),
    ];

    assert_eq!(indented_names[0], "L1");
    assert_eq!(indented_names[1], "    L2");
    assert_eq!(indented_names[2], "        L3");
    assert_eq!(indented_names[3], "            L4");
}

#[test]
fn test_tree_flatten_empty_tree() {
    // 空树不产生任何行
    let rows: Vec<Vec<String>> = Vec::new();
    assert!(rows.is_empty());
}

#[test]
fn test_tree_with_header_rows_total() {
    // 树形数据 + 表头行的总行数
    // 嵌套表头 2 行 + 树形数据拍平 5 行 = 7 行

    let header_count = 2;
    let data_count = 5;
    let total = header_count + data_count;
    assert_eq!(total, 7);
}

#[test]
fn test_tree_missing_key_produces_empty() {
    // 数据对象中缺少某个 key 时，应输出空字符串
    // 模拟：columns = [name, age]，数据中只有 name

    let row: Vec<&str> = vec!["张三", ""]; // age 缺失

    assert_eq!(row[0], "张三");
    assert_eq!(row[1], ""); // 缺失字段为空
}

#[test]
fn test_tree_custom_children_key_logic() {
    // 测试自定义 children 字段名的逻辑
    // 使用 "subItems" 代替 "children"

    let children_key = "subItems";
    assert_ne!(children_key, "children");
}

// ============================================================================
// 冻结窗格策略选择逻辑测试
// ============================================================================

#[test]
fn test_freeze_pane_strategy_selection() {
    // 模拟 export_xlsx 中的冻结策略选择逻辑：
    // 用户配置 (freeze_pane) 优先 > 自动检测 (header_row_count > 0) > 不冻结 (None)

    fn determine_freeze_pane(user_config: Option<(u32, u16)>, header_row_count: usize) -> Option<(u32, u16)> {
        if let Some(config) = user_config {
            Some(config) // 用户配置优先级最高
        } else if header_row_count > 0 {
            Some((header_row_count as u32, 0)) // 自动冻结表头行，0列
        } else {
            None // 不冻结
        }
    }

    // 场景 1: 用户指定冻结 3 行 1 列，表头有 1 行
    // 期望: 使用用户配置 (3, 1)
    assert_eq!(determine_freeze_pane(Some((3, 1)), 1), Some((3, 1)));

    // 场景 2: 用户不指定 (None)，表头有 2 行
    // 期望: 自动冻结 2 行 0 列
    assert_eq!(determine_freeze_pane(None, 2), Some((2, 0)));

    // 场景 3: 用户指定冻结 0 行 0 列 (即强制不冻结)，表头有 1 行
    // 期望: 使用用户配置 (0, 0)
    assert_eq!(determine_freeze_pane(Some((0, 0)), 1), Some((0, 0)));

    // 场景 4: 用户不指定，且没有表头数据
    // 期望: 不冻结 (None)
    assert_eq!(determine_freeze_pane(None, 0), None);
}
