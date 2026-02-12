/// 数据导出模块
///
/// 提供从 JavaScript 对象数组 + 表头配置直接导出文件的功能，
/// 支持嵌套表头（多行表头 + 合并单元格）
use super::table_extractor::{MergeRange, TableData};
use wasm_bindgen::prelude::*;

/// 解析后的列节点
struct ColumnNode {
    /// 表头标题
    title: String,
    /// 数据字段名（叶子节点才有）
    key: Option<String>,
    /// 子列（分组列才有）
    children: Vec<ColumnNode>,
}

/// 从 JsValue 递归解析列配置
///
/// # 参数
/// * `columns` - JS 数组，每个元素为 { title, key?, children? }
///
/// # 返回值
/// 解析后的 ColumnNode 列表
fn parse_columns(columns: &JsValue) -> Result<Vec<ColumnNode>, JsValue> {
    let array = js_sys::Array::from(columns);
    let length = array.length();

    if length == 0 {
        return Err(JsValue::from_str("表头配置数组不能为空"));
    }

    let mut nodes = Vec::with_capacity(length as usize);

    for i in 0..length {
        let item = array.get(i);
        let node = parse_column_node(&item, i)?;
        nodes.push(node);
    }

    Ok(nodes)
}

/// 解析单个列节点
fn parse_column_node(item: &JsValue, index: u32) -> Result<ColumnNode, JsValue> {
    let title = js_sys::Reflect::get(item, &JsValue::from_str("title"))
        .ok()
        .and_then(|v| v.as_string())
        .ok_or_else(|| {
            JsValue::from_str(&format!("第 {} 个表头配置缺少有效的 title", index + 1))
        })?;

    let key = js_sys::Reflect::get(item, &JsValue::from_str("key"))
        .ok()
        .and_then(|v| v.as_string());

    let children_val = js_sys::Reflect::get(item, &JsValue::from_str("children"))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null());

    let children = if let Some(cv) = children_val {
        let arr = js_sys::Array::from(&cv);
        if arr.length() > 0 {
            parse_columns(&cv)?
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    // 叶子节点必须有 key
    if children.is_empty() && key.is_none() {
        return Err(JsValue::from_str(&format!(
            "叶子列 '{}' 缺少 key 属性",
            title
        )));
    }

    Ok(ColumnNode {
        title,
        key,
        children,
    })
}

/// 计算列树的最大深度
fn calc_depth(nodes: &[ColumnNode]) -> usize {
    let mut max = 0;
    for node in nodes {
        let d = if node.children.is_empty() {
            1
        } else {
            1 + calc_depth(&node.children)
        };
        if d > max {
            max = d;
        }
    }
    max
}

/// 计算节点的叶子数量（即 colspan）
fn calc_leaf_count(node: &ColumnNode) -> usize {
    if node.children.is_empty() {
        1
    } else {
        node.children.iter().map(calc_leaf_count).sum()
    }
}

/// 按顺序收集所有叶子节点的 key
fn collect_leaf_keys(nodes: &[ColumnNode]) -> Vec<String> {
    let mut keys = Vec::new();
    for node in nodes {
        if node.children.is_empty() {
            if let Some(ref key) = node.key {
                keys.push(key.clone());
            }
        } else {
            keys.extend(collect_leaf_keys(&node.children));
        }
    }
    keys
}

/// 构建多行表头和合并区域
///
/// # 参数
/// * `columns` - 列配置节点
/// * `max_depth` - 表头的最大深度
///
/// # 返回值
/// (表头行数据, 合并区域列表)
fn build_header_rows(
    columns: &[ColumnNode],
    max_depth: usize,
) -> (Vec<Vec<String>>, Vec<MergeRange>) {
    let total_cols = columns.iter().map(calc_leaf_count).sum::<usize>();

    // 初始化表头行（全部填空字符串）
    let mut header_rows: Vec<Vec<String>> = vec![vec![String::new(); total_cols]; max_depth];
    let mut merge_ranges: Vec<MergeRange> = Vec::new();

    // 递归填充
    fill_header_cells(
        columns,
        0,
        0,
        max_depth,
        &mut header_rows,
        &mut merge_ranges,
    );

    (header_rows, merge_ranges)
}

/// 递归填充表头单元格
///
/// # 参数
/// * `nodes` - 当前层的列节点
/// * `row` - 当前行索引
/// * `col_start` - 当前列起始索引
/// * `max_depth` - 表头最大深度
/// * `header_rows` - 表头行数据（可变引用）
/// * `merge_ranges` - 合并区域列表（可变引用）
///
/// # 返回值
/// 返回消耗的列数
fn fill_header_cells(
    nodes: &[ColumnNode],
    row: usize,
    col_start: usize,
    max_depth: usize,
    header_rows: &mut [Vec<String>],
    merge_ranges: &mut Vec<MergeRange>,
) -> usize {
    let mut col = col_start;

    for node in nodes {
        if node.children.is_empty() {
            // 叶子节点：可能需要 rowspan
            let rowspan = max_depth - row;
            header_rows[row][col] = node.title.clone();

            if rowspan > 1 {
                merge_ranges.push(MergeRange::new(
                    row as u32,
                    col as u16,
                    (row + rowspan - 1) as u32,
                    col as u16,
                ));
            }

            col += 1;
        } else {
            // 分组节点：需要 colspan
            let leaf_count = calc_leaf_count(node);
            header_rows[row][col] = node.title.clone();

            if leaf_count > 1 {
                merge_ranges.push(MergeRange::new(
                    row as u32,
                    col as u16,
                    row as u32,
                    (col + leaf_count - 1) as u16,
                ));
            }

            // 递归处理子节点
            fill_header_cells(
                &node.children,
                row + 1,
                col,
                max_depth,
                header_rows,
                merge_ranges,
            );

            col += leaf_count;
        }
    }

    col - col_start
}

/// 从 JS 对象数组中按 key 顺序提取数据行
///
/// # 参数
/// * `data` - JS 对象数组
/// * `keys` - 叶子列的 key 列表
///
/// # 返回值
/// 二维字符串数组
fn extract_data_rows(data: &JsValue, keys: &[String]) -> Result<Vec<Vec<String>>, JsValue> {
    let array = js_sys::Array::from(data);
    let length = array.length();

    if length == 0 {
        return Ok(Vec::new());
    }

    let mut rows = Vec::with_capacity(length as usize);

    for i in 0..length {
        let item = array.get(i);
        let mut row = Vec::with_capacity(keys.len());

        for key in keys {
            let val = js_sys::Reflect::get(&item, &JsValue::from_str(key)).unwrap_or(JsValue::NULL);

            let cell_text = js_value_to_string(&val);
            row.push(cell_text);
        }

        rows.push(row);
    }

    Ok(rows)
}

/// 将 JS 值转换为字符串
pub(crate) fn js_value_to_string(val: &JsValue) -> String {
    if val.is_null() || val.is_undefined() {
        String::new()
    } else if let Some(s) = val.as_string() {
        s
    } else if let Some(n) = val.as_f64() {
        // 整数不带小数点
        if n.fract() == 0.0 && n.abs() < i64::MAX as f64 {
            format!("{}", n as i64)
        } else {
            format!("{}", n)
        }
    } else if let Some(b) = val.as_bool() {
        b.to_string()
    } else {
        val.as_string().unwrap_or_default()
    }
}

/// 解析表头配置和数据数组，生成完整的 TableData
///
/// # 参数
/// * `columns` - JS 表头配置数组
/// * `data` - JS 数据对象数组
///
/// # 返回值
/// 包含表头和数据行及合并区域的 TableData
pub fn build_table_data_from_array(
    columns: &JsValue,
    data: &JsValue,
) -> Result<TableData, JsValue> {
    // 1. 解析列配置
    let column_nodes = parse_columns(columns)?;

    // 2. 计算表头深度
    let max_depth = calc_depth(&column_nodes);

    // 3. 构建多行表头和合并区域
    let (header_rows, merge_ranges) = build_header_rows(&column_nodes, max_depth);

    // 4. 收集叶子 key
    let leaf_keys = collect_leaf_keys(&column_nodes);

    // 5. 提取数据行
    let data_rows = extract_data_rows(data, &leaf_keys)?;

    // 6. 合并表头行和数据行
    let mut rows = header_rows;
    rows.extend(data_rows);

    Ok(TableData { rows, merge_ranges })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试列树深度计算
    #[test]
    fn test_calc_depth_single_level() {
        let nodes = vec![
            ColumnNode {
                title: "A".to_string(),
                key: Some("a".to_string()),
                children: vec![],
            },
            ColumnNode {
                title: "B".to_string(),
                key: Some("b".to_string()),
                children: vec![],
            },
        ];
        assert_eq!(calc_depth(&nodes), 1);
    }

    #[test]
    fn test_calc_depth_nested() {
        let nodes = vec![
            ColumnNode {
                title: "A".to_string(),
                key: Some("a".to_string()),
                children: vec![],
            },
            ColumnNode {
                title: "Group".to_string(),
                key: None,
                children: vec![
                    ColumnNode {
                        title: "B".to_string(),
                        key: Some("b".to_string()),
                        children: vec![],
                    },
                    ColumnNode {
                        title: "C".to_string(),
                        key: Some("c".to_string()),
                        children: vec![],
                    },
                ],
            },
        ];
        assert_eq!(calc_depth(&nodes), 2);
    }

    #[test]
    fn test_calc_depth_deeply_nested() {
        let nodes = vec![ColumnNode {
            title: "L1".to_string(),
            key: None,
            children: vec![ColumnNode {
                title: "L2".to_string(),
                key: None,
                children: vec![ColumnNode {
                    title: "L3".to_string(),
                    key: Some("l3".to_string()),
                    children: vec![],
                }],
            }],
        }];
        assert_eq!(calc_depth(&nodes), 3);
    }

    /// 测试叶子数量计算
    #[test]
    fn test_calc_leaf_count() {
        let node = ColumnNode {
            title: "Group".to_string(),
            key: None,
            children: vec![
                ColumnNode {
                    title: "A".to_string(),
                    key: Some("a".to_string()),
                    children: vec![],
                },
                ColumnNode {
                    title: "B".to_string(),
                    key: Some("b".to_string()),
                    children: vec![],
                },
            ],
        };
        assert_eq!(calc_leaf_count(&node), 2);
    }

    /// 测试叶子 key 收集
    #[test]
    fn test_collect_leaf_keys() {
        let nodes = vec![
            ColumnNode {
                title: "姓名".to_string(),
                key: Some("name".to_string()),
                children: vec![],
            },
            ColumnNode {
                title: "其他".to_string(),
                key: None,
                children: vec![
                    ColumnNode {
                        title: "年龄".to_string(),
                        key: Some("age".to_string()),
                        children: vec![],
                    },
                    ColumnNode {
                        title: "住址".to_string(),
                        key: Some("address".to_string()),
                        children: vec![],
                    },
                ],
            },
        ];
        assert_eq!(collect_leaf_keys(&nodes), vec!["name", "age", "address"]);
    }

    /// 测试表头构建 - 单层表头
    #[test]
    fn test_build_header_rows_single_level() {
        let nodes = vec![
            ColumnNode {
                title: "A".to_string(),
                key: Some("a".to_string()),
                children: vec![],
            },
            ColumnNode {
                title: "B".to_string(),
                key: Some("b".to_string()),
                children: vec![],
            },
        ];
        let (rows, merges) = build_header_rows(&nodes, 1);
        assert_eq!(rows, vec![vec!["A".to_string(), "B".to_string()]]);
        assert!(merges.is_empty());
    }

    /// 测试表头构建 - 嵌套表头
    #[test]
    fn test_build_header_rows_nested() {
        let nodes = vec![
            ColumnNode {
                title: "姓名".to_string(),
                key: Some("name".to_string()),
                children: vec![],
            },
            ColumnNode {
                title: "其他".to_string(),
                key: None,
                children: vec![
                    ColumnNode {
                        title: "年龄".to_string(),
                        key: Some("age".to_string()),
                        children: vec![],
                    },
                    ColumnNode {
                        title: "住址".to_string(),
                        key: Some("address".to_string()),
                        children: vec![],
                    },
                ],
            },
        ];

        let (rows, merges) = build_header_rows(&nodes, 2);

        // 第一行: 姓名（占位2行），其他（占位2列）
        assert_eq!(rows[0], vec!["姓名", "其他", ""]);
        // 第二行: 空（被姓名 rowspan 占），年龄，住址
        assert_eq!(rows[1], vec!["", "年龄", "住址"]);

        // 合并区域：姓名 rowspan=2, 其他 colspan=2
        assert_eq!(merges.len(), 2);

        // 姓名: (0,0) -> (1,0)
        assert_eq!(merges[0].first_row, 0);
        assert_eq!(merges[0].first_col, 0);
        assert_eq!(merges[0].last_row, 1);
        assert_eq!(merges[0].last_col, 0);

        // 其他: (0,1) -> (0,2)
        assert_eq!(merges[1].first_row, 0);
        assert_eq!(merges[1].first_col, 1);
        assert_eq!(merges[1].last_row, 0);
        assert_eq!(merges[1].last_col, 2);
    }

    /// 测试 JS 值转字符串（需要 wasm 环境，仅在 wasm32 下运行）
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_js_value_to_string_null() {
        assert_eq!(js_value_to_string(&JsValue::NULL), "");
        assert_eq!(js_value_to_string(&JsValue::UNDEFINED), "");
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_js_value_to_string_number() {
        assert_eq!(js_value_to_string(&JsValue::from_f64(42.0)), "42");
        assert_eq!(js_value_to_string(&JsValue::from_f64(3.14)), "3.14");
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_js_value_to_string_bool() {
        assert_eq!(js_value_to_string(&JsValue::from_bool(true)), "true");
        assert_eq!(js_value_to_string(&JsValue::from_bool(false)), "false");
    }

    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_js_value_to_string_string() {
        assert_eq!(js_value_to_string(&JsValue::from_str("hello")), "hello");
    }
}
