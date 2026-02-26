/// 数据导出模块
///
/// 提供从 JavaScript 对象数组 + 表头配置直接导出文件的功能，
/// 支持嵌套表头（多行表头 + 合并单元格）
use super::table_extractor::{MergeRange, TableData};
use wasm_bindgen::prelude::*;

/// 最大递归深度限制，防止恶意构造的深层嵌套数据导致栈溢出
const MAX_DEPTH: usize = 64;

/// 安全地从 JS 对象中获取属性值
///
/// 与 `Reflect::get(...).unwrap_or(JsValue::NULL)` 不同，本函数会区分
/// "字段不存在（返回 NULL）"和"读取异常（返回 Err）"两种情况，
/// 避免 getter 异常或 Proxy 异常被静默吞掉。
fn get_object_property(obj: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    js_sys::Reflect::get(obj, &JsValue::from_str(key))
        .map_err(|e| JsValue::from_str(&format!("读取对象属性 '{}' 时发生异常: {:?}", key, e)))
}

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
    parse_columns_with_depth(columns, 0)
}

/// 带深度限制的递归解析列配置
fn parse_columns_with_depth(columns: &JsValue, depth: usize) -> Result<Vec<ColumnNode>, JsValue> {
    if depth >= MAX_DEPTH {
        return Err(JsValue::from_str(&format!(
            "表头嵌套层级超过最大限制（{}层），请检查是否存在循环引用",
            MAX_DEPTH
        )));
    }

    let array = js_sys::Array::from(columns);
    let length = array.length();

    if length == 0 {
        return Err(JsValue::from_str("表头配置数组不能为空"));
    }

    let mut nodes = Vec::with_capacity(length as usize);

    for i in 0..length {
        let item = array.get(i);
        let node = parse_column_node_with_depth(&item, i, depth)?;
        nodes.push(node);
    }

    Ok(nodes)
}

/// 解析单个列节点
fn parse_column_node_with_depth(
    item: &JsValue,
    index: u32,
    depth: usize,
) -> Result<ColumnNode, JsValue> {
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
            parse_columns_with_depth(&cv, depth + 1)?
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
) -> Result<(Vec<Vec<String>>, Vec<MergeRange>), JsValue> {
    let total_cols = columns.iter().map(calc_leaf_count).sum::<usize>();
    // 安全检查：防止分配过大的内存 (防止 OOM)
    const MAX_HEADER_CELLS: usize = 100_000;
    if total_cols * max_depth > MAX_HEADER_CELLS {
        return Err(JsValue::from_str(&format!(
            "表头过大（{} x {}），超过最大单元格数限制 ({})",
            total_cols, max_depth, MAX_HEADER_CELLS
        )));
    }

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

    Ok((header_rows, merge_ranges))
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

/// 单元格解析结果，包含值和合并信息
struct CellInfo {
    /// 单元格文本内容
    value: String,
    /// 列跨度（默认 1，0 表示被左侧单元格覆盖）
    col_span: u32,
    /// 行跨度（默认 1，0 表示被上方单元格覆盖）
    row_span: u32,
}

/// 解析单元格值，支持普通值和带 colSpan/rowSpan 的对象
///
/// 支持两种格式：
/// 1. 普通值: `"张三"`, `28`, `true` 等
/// 2. 对象值: `{ value: "张三", colSpan: 2, rowSpan: 3 }`
///
/// 当 colSpan 或 rowSpan 为 0 时，表示该单元格被其他单元格的合并覆盖
fn parse_cell_value(val: &JsValue) -> CellInfo {
    // 如果是对象（非 null/undefined/string/number/bool），尝试解析合并属性
    if val.is_object() && !val.is_null() && val.as_string().is_none() {
        // 检查是否有 value 属性（区分普通对象和合并单元格对象）
        let has_value = js_sys::Reflect::get(val, &JsValue::from_str("value"))
            .ok()
            .filter(|v| !v.is_undefined())
            .is_some();

        let has_col_span = js_sys::Reflect::get(val, &JsValue::from_str("colSpan"))
            .ok()
            .filter(|v| !v.is_undefined())
            .is_some();

        let has_row_span = js_sys::Reflect::get(val, &JsValue::from_str("rowSpan"))
            .ok()
            .filter(|v| !v.is_undefined())
            .is_some();

        // 只有当对象包含 value、colSpan 或 rowSpan 属性时，才按合并单元格处理
        if has_value || has_col_span || has_row_span {
            let value = js_sys::Reflect::get(val, &JsValue::from_str("value"))
                .ok()
                .map(|v| js_value_to_string(&v))
                .unwrap_or_default();

            let col_span = js_sys::Reflect::get(val, &JsValue::from_str("colSpan"))
                .ok()
                .and_then(|v| v.as_f64())
                .map(|n| n as u32)
                .unwrap_or(1);

            let row_span = js_sys::Reflect::get(val, &JsValue::from_str("rowSpan"))
                .ok()
                .and_then(|v| v.as_f64())
                .map(|n| n as u32)
                .unwrap_or(1);

            return CellInfo {
                value,
                col_span,
                row_span,
            };
        }
    }

    // 普通值
    CellInfo {
        value: js_value_to_string(val),
        col_span: 1,
        row_span: 1,
    }
}

/// 从 JS 对象数组中按 key 顺序提取数据行，支持 colSpan/rowSpan
///
/// # 参数
/// * `data` - JS 对象数组
/// * `keys` - 叶子列的 key 列表
/// * `header_row_count` - 表头行数（用于 MergeRange 的行偏移）
///
/// # 返回值
/// (二维字符串数组, 数据区域的合并区域列表)
fn extract_data_rows(
    data: &JsValue,
    keys: &[String],
    header_row_count: usize,
) -> Result<(Vec<Vec<String>>, Vec<MergeRange>), JsValue> {
    let array = js_sys::Array::from(data);
    let length = array.length();

    if length == 0 {
        return Ok((Vec::new(), Vec::new()));
    }

    let mut rows = Vec::with_capacity(length as usize);
    let mut merge_ranges = Vec::new();

    for i in 0..length {
        let item = array.get(i);
        let mut row = Vec::with_capacity(keys.len());

        for (col_idx, key) in keys.iter().enumerate() {
            let val = get_object_property(&item, key)?;
            let cell_info = parse_cell_value(&val);

            if cell_info.col_span == 0 || cell_info.row_span == 0 {
                // 被其他单元格的合并覆盖，输出空字符串
                row.push(String::new());
            } else {
                row.push(cell_info.value);

                // 生成合并区域（colSpan>1 或 rowSpan>1）
                // 增加防护：确保 row_span 和 col_span 至少为 1，防止计算下溢
                if cell_info.col_span > 1 || cell_info.row_span > 1 {
                    let first_row = (i as usize + header_row_count) as u32;
                    let first_col = col_idx as u16;

                    // parse_cell_value 已确保 row_span 和 col_span 至少为 1
                    let last_row = first_row + cell_info.row_span - 1;
                    let last_col = first_col + (cell_info.col_span as u16) - 1;

                    merge_ranges.push(MergeRange::new(first_row, first_col, last_row, last_col));
                }
            }
        }

        rows.push(row);
    }

    Ok((rows, merge_ranges))
}

/// 将 JS 值转换为字符串
pub(crate) fn js_value_to_string(val: &JsValue) -> String {
    if val.is_null() || val.is_undefined() {
        String::new()
    } else if let Some(s) = val.as_string() {
        s
    } else if let Some(n) = val.as_f64() {
        n.to_string()
    } else if let Some(b) = val.as_bool() {
        b.to_string()
    } else {
        // Symbol、BigInt 等其他类型，使用 Debug 格式输出
        format!("{:?}", val)
    }
}

/// 递归遍历树形数据，将嵌套的 children 拍平为行数据
///
/// # 参数
/// * `data` - JS 对象数组（可能包含 children）
/// * `keys` - 叶子列的 key 列表
/// * `indent_key` - 需要缩进的列的 key（可选）
/// * `children_key` - 子节点字段名
/// * `depth` - 当前递归深度
/// * `rows` - 输出行数据（可变引用，递归中累积）
fn flatten_tree_data(
    data: &JsValue,
    keys: &[String],
    indent_key: Option<&str>,
    children_key: &str,
    depth: usize,
    rows: &mut Vec<Vec<String>>,
) -> Result<(), JsValue> {
    if depth >= MAX_DEPTH {
        return Err(JsValue::from_str(&format!(
            "树形数据嵌套层级超过最大限制（{}层），请检查是否存在循环引用",
            MAX_DEPTH
        )));
    }

    let array = js_sys::Array::from(data);
    let length = array.length();

    for i in 0..length {
        let item = array.get(i);
        let mut row = Vec::with_capacity(keys.len());

        for key in keys {
            let val = get_object_property(&item, key)?;
            let mut cell_text = js_value_to_string(&val);

            // 对指定的缩进列添加层级缩进（每层 4 个空格）
            if let Some(ik) = indent_key
                && key == ik
                && depth > 0
            {
                let indent = "    ".repeat(depth);
                cell_text = format!("{}{}", indent, cell_text);
            }

            row.push(cell_text);
        }

        rows.push(row);

        // 递归处理子节点
        let children_val = js_sys::Reflect::get(&item, &JsValue::from_str(children_key))
            .ok()
            .filter(|v| !v.is_undefined() && !v.is_null());

        if let Some(cv) = children_val {
            let child_arr = js_sys::Array::from(&cv);
            if child_arr.length() > 0 {
                flatten_tree_data(&cv, keys, indent_key, children_key, depth + 1, rows)?;
            }
        }
    }

    Ok(())
}

/// 解析表头配置和树形数据，递归拍平后生成 TableData
///
/// 将嵌套的 children 结构递归遍历为扁平的行数据，
/// 可选在指定列添加层级缩进以体现树形层次关系。
///
/// # 参数
/// * `columns` - JS 表头配置数组
/// * `data` - JS 树形数据数组（可包含 children）
/// * `indent_column` - 需要缩进的列的 key（可选，如 "name"）
/// * `children_key` - 子节点字段名（默认 "children"）
///
/// # 返回值
/// 包含表头和拍平后的数据行的 TableData
pub fn build_table_data_from_tree(
    columns: &JsValue,
    data: &JsValue,
    indent_column: Option<&str>,
    children_key: &str,
) -> Result<TableData, JsValue> {
    // 1. 解析列配置
    let column_nodes = parse_columns(columns)?;

    // 2. 计算表头深度
    let max_depth = calc_depth(&column_nodes);

    // 3. 构建多行表头和合并区域
    let (header_rows, merge_ranges) = build_header_rows(&column_nodes, max_depth)?;

    // 4. 收集叶子 key
    let leaf_keys = collect_leaf_keys(&column_nodes);

    // 5. 递归拍平树形数据
    let mut data_rows = Vec::new();
    flatten_tree_data(
        data,
        &leaf_keys,
        indent_column,
        children_key,
        0,
        &mut data_rows,
    )?;

    // 6. 合并表头行和数据行
    let mut rows = header_rows;
    rows.extend(data_rows);

    Ok(TableData { rows, merge_ranges })
}

/// 解析表头配置和数据数组，生成完整的 TableData
///
/// 支持数据中的合并单元格：单元格值可以是 `{ value, colSpan?, rowSpan? }` 格式的对象。
/// - `colSpan: 0` 或 `rowSpan: 0` 表示该单元格被其他合并覆盖
/// - `colSpan: n` (n>1) 表示横跨 n 列
/// - `rowSpan: n` (n>1) 表示纵跨 n 行
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
    let (header_rows, mut merge_ranges) = build_header_rows(&column_nodes, max_depth)?;
    let header_row_count = header_rows.len();

    // 4. 收集叶子 key
    let leaf_keys = collect_leaf_keys(&column_nodes);

    // 5. 提取数据行（含数据区域合并信息）
    let (data_rows, data_merge_ranges) = extract_data_rows(data, &leaf_keys, header_row_count)?;

    // 6. 合并表头行和数据行
    let mut rows = header_rows;
    rows.extend(data_rows);

    // 7. 合并表头合并区域和数据合并区域
    merge_ranges.extend(data_merge_ranges);

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
        let (rows, merges) = build_header_rows(&nodes, 1).unwrap();
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

        let (rows, merges) = build_header_rows(&nodes, 2).unwrap();

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

    /// 测试三级嵌套表头构建
    #[test]
    fn test_build_header_rows_three_level() {
        // ID | 基本信息(colspan=3) | 部门
        //    | 姓名 | 联系方式(colspan=2) |
        //    |      | 电话 | 邮箱 |
        let nodes = vec![
            ColumnNode {
                title: "ID".to_string(),
                key: Some("id".to_string()),
                children: vec![],
            },
            ColumnNode {
                title: "基本信息".to_string(),
                key: None,
                children: vec![
                    ColumnNode {
                        title: "姓名".to_string(),
                        key: Some("name".to_string()),
                        children: vec![],
                    },
                    ColumnNode {
                        title: "联系方式".to_string(),
                        key: None,
                        children: vec![
                            ColumnNode {
                                title: "电话".to_string(),
                                key: Some("phone".to_string()),
                                children: vec![],
                            },
                            ColumnNode {
                                title: "邮箱".to_string(),
                                key: Some("email".to_string()),
                                children: vec![],
                            },
                        ],
                    },
                ],
            },
            ColumnNode {
                title: "部门".to_string(),
                key: Some("dept".to_string()),
                children: vec![],
            },
        ];

        let (rows, merges) = build_header_rows(&nodes, 3).unwrap();

        assert_eq!(rows.len(), 3);
        // 第一行: ID(rowspan=3), 基本信息(colspan=3), 部门(rowspan=3)
        assert_eq!(rows[0], vec!["ID", "基本信息", "", "", "部门"]);
        // 第二行: "", 姓名(rowspan=2), 联系方式(colspan=2), ""
        assert_eq!(rows[1], vec!["", "姓名", "联系方式", "", ""]);
        // 第三行: "", "", 电话, 邮箱, ""
        assert_eq!(rows[2], vec!["", "", "电话", "邮箱", ""]);

        // 合并区域验证：ID(rowspan=3), 基本信息(colspan=3), 部门(rowspan=3), 姓名(rowspan=2), 联系方式(colspan=2)
        assert_eq!(merges.len(), 5);
    }

    /// 测试填充表头单元格的返回值（消耗的列数）
    #[test]
    fn test_fill_header_cells_returns_consumed_cols() {
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

        let mut header_rows = vec![vec![String::new(); 3]; 2];
        let mut merge_ranges = Vec::new();
        let consumed = fill_header_cells(&nodes, 0, 0, 2, &mut header_rows, &mut merge_ranges);

        assert_eq!(consumed, 3); // A + B + C = 3 列
    }

    /// 测试空子节点列表的叶子 key 收集
    #[test]
    fn test_collect_leaf_keys_empty() {
        let nodes: Vec<ColumnNode> = vec![];
        let keys = collect_leaf_keys(&nodes);
        assert!(keys.is_empty());
    }

    /// 测试叶子数量计算 - 嵌套组
    #[test]
    fn test_calc_leaf_count_nested() {
        let node = ColumnNode {
            title: "Top".to_string(),
            key: None,
            children: vec![
                ColumnNode {
                    title: "A".to_string(),
                    key: Some("a".to_string()),
                    children: vec![],
                },
                ColumnNode {
                    title: "Sub".to_string(),
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
            ],
        };
        assert_eq!(calc_leaf_count(&node), 3);
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

    /// 测试 parse_cell_value - 普通字符串值
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parse_cell_value_plain_string() {
        let val = JsValue::from_str("张三");
        let info = parse_cell_value(&val);
        assert_eq!(info.value, "张三");
        assert_eq!(info.col_span, 1);
        assert_eq!(info.row_span, 1);
    }

    /// 测试 parse_cell_value - 普通数字值
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parse_cell_value_plain_number() {
        let val = JsValue::from_f64(42.0);
        let info = parse_cell_value(&val);
        assert_eq!(info.value, "42");
        assert_eq!(info.col_span, 1);
        assert_eq!(info.row_span, 1);
    }

    /// 测试 parse_cell_value - null/undefined
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parse_cell_value_null() {
        let info_null = parse_cell_value(&JsValue::NULL);
        assert_eq!(info_null.value, "");
        assert_eq!(info_null.col_span, 1);
        assert_eq!(info_null.row_span, 1);

        let info_undef = parse_cell_value(&JsValue::UNDEFINED);
        assert_eq!(info_undef.value, "");
        assert_eq!(info_undef.col_span, 1);
        assert_eq!(info_undef.row_span, 1);
    }

    /// 测试 parse_cell_value - 带 rowSpan 的对象
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parse_cell_value_with_row_span() {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("value"),
            &JsValue::from_str("张三"),
        )
        .unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("rowSpan"), &JsValue::from_f64(2.0)).unwrap();
        let val = JsValue::from(obj);

        let info = parse_cell_value(&val);
        assert_eq!(info.value, "张三");
        assert_eq!(info.col_span, 1);
        assert_eq!(info.row_span, 2);
    }

    /// 测试 parse_cell_value - 带 colSpan 的对象
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parse_cell_value_with_col_span() {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("value"),
            &JsValue::from_str("北京"),
        )
        .unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("colSpan"), &JsValue::from_f64(3.0)).unwrap();
        let val = JsValue::from(obj);

        let info = parse_cell_value(&val);
        assert_eq!(info.value, "北京");
        assert_eq!(info.col_span, 3);
        assert_eq!(info.row_span, 1);
    }

    /// 测试 parse_cell_value - span 为 0（被合并覆盖）
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parse_cell_value_span_zero() {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("value"), &JsValue::from_str("")).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("rowSpan"), &JsValue::from_f64(0.0)).unwrap();
        let val = JsValue::from(obj);

        let info = parse_cell_value(&val);
        assert_eq!(info.value, "");
        assert_eq!(info.col_span, 1);
        assert_eq!(info.row_span, 0);
    }

    /// 测试 parse_cell_value - 同时有 colSpan 和 rowSpan
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parse_cell_value_with_both_spans() {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("value"),
            &JsValue::from_str("合并"),
        )
        .unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("colSpan"), &JsValue::from_f64(2.0)).unwrap();
        js_sys::Reflect::set(&obj, &JsValue::from_str("rowSpan"), &JsValue::from_f64(3.0)).unwrap();
        let val = JsValue::from(obj);

        let info = parse_cell_value(&val);
        assert_eq!(info.value, "合并");
        assert_eq!(info.col_span, 2);
        assert_eq!(info.row_span, 3);
    }

    /// 测试 parse_cell_value - 普通对象（没有 value/colSpan/rowSpan，不当合并处理）
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_parse_cell_value_plain_object() {
        let obj = js_sys::Object::new();
        js_sys::Reflect::set(&obj, &JsValue::from_str("name"), &JsValue::from_str("test")).unwrap();
        let val = JsValue::from(obj);

        let info = parse_cell_value(&val);
        // 没有 value/colSpan/rowSpan，按普通值处理
        assert_eq!(info.col_span, 1);
        assert_eq!(info.row_span, 1);
    }

    // ========================================================================
    // 树形数据拍平测试
    // ========================================================================

    /// 测试 flatten_tree_data - 基本拍平（无缩进）
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_flatten_tree_data_basic() {
        let root = js_sys::Array::new();
        let item1 = js_sys::Object::new();
        js_sys::Reflect::set(&item1, &JsValue::from_str("name"), &JsValue::from_str("A")).unwrap();

        let child = js_sys::Object::new();
        js_sys::Reflect::set(&child, &JsValue::from_str("name"), &JsValue::from_str("B")).unwrap();

        let children = js_sys::Array::new();
        children.push(&child.into());
        js_sys::Reflect::set(&item1, &JsValue::from_str("children"), &children.into()).unwrap();

        root.push(&item1.into());

        let keys = vec!["name".to_string()];
        let mut rows = Vec::new();
        flatten_tree_data(&root.into(), &keys, None, "children", 0, &mut rows).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "A");
        assert_eq!(rows[1][0], "B");
    }

    /// 测试 flatten_tree_data - 带缩进
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_flatten_tree_data_with_indent() {
        let root = js_sys::Array::new();
        let item1 = js_sys::Object::new();
        js_sys::Reflect::set(&item1, &JsValue::from_str("name"), &JsValue::from_str("根")).unwrap();

        let child = js_sys::Object::new();
        js_sys::Reflect::set(&child, &JsValue::from_str("name"), &JsValue::from_str("子")).unwrap();

        let grandchild = js_sys::Object::new();
        js_sys::Reflect::set(
            &grandchild,
            &JsValue::from_str("name"),
            &JsValue::from_str("孙"),
        )
        .unwrap();

        let gc_arr = js_sys::Array::new();
        gc_arr.push(&grandchild.into());
        js_sys::Reflect::set(&child, &JsValue::from_str("children"), &gc_arr.into()).unwrap();

        let c_arr = js_sys::Array::new();
        c_arr.push(&child.into());
        js_sys::Reflect::set(&item1, &JsValue::from_str("children"), &c_arr.into()).unwrap();

        root.push(&item1.into());

        let keys = vec!["name".to_string()];
        let mut rows = Vec::new();
        flatten_tree_data(&root.into(), &keys, Some("name"), "children", 0, &mut rows).unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0], "根"); // 根节点，depth=0，无缩进
        assert_eq!(rows[1][0], "    子"); // depth=1，4 个空格
        assert_eq!(rows[2][0], "        孙"); // depth=2，8 个空格
    }

    /// 测试 flatten_tree_data - 自定义 children_key
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_flatten_tree_data_custom_children_key() {
        let root = js_sys::Array::new();
        let item1 = js_sys::Object::new();
        js_sys::Reflect::set(&item1, &JsValue::from_str("name"), &JsValue::from_str("A")).unwrap();

        let child = js_sys::Object::new();
        js_sys::Reflect::set(&child, &JsValue::from_str("name"), &JsValue::from_str("B")).unwrap();

        let subs = js_sys::Array::new();
        subs.push(&child.into());
        js_sys::Reflect::set(&item1, &JsValue::from_str("subs"), &subs.into()).unwrap();

        root.push(&item1.into());

        let keys = vec!["name".to_string()];
        let mut rows = Vec::new();
        // 使用 "subs" 而非默认 "children"
        flatten_tree_data(&root.into(), &keys, None, "subs", 0, &mut rows).unwrap();

        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0][0], "A");
        assert_eq!(rows[1][0], "B");
    }

    /// 测试 flatten_tree_data - 空数组
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_flatten_tree_data_empty() {
        let root = js_sys::Array::new();
        let keys = vec!["name".to_string()];
        let mut rows = Vec::new();
        flatten_tree_data(&root.into(), &keys, None, "children", 0, &mut rows).unwrap();
        assert!(rows.is_empty());
    }

    /// 测试 flatten_tree_data - 多个根节点
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_flatten_tree_data_multiple_roots() {
        let root = js_sys::Array::new();
        for name in &["A", "B", "C"] {
            let item = js_sys::Object::new();
            js_sys::Reflect::set(&item, &JsValue::from_str("name"), &JsValue::from_str(name))
                .unwrap();
            root.push(&item.into());
        }

        let keys = vec!["name".to_string()];
        let mut rows = Vec::new();
        flatten_tree_data(&root.into(), &keys, None, "children", 0, &mut rows).unwrap();

        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0][0], "A");
        assert_eq!(rows[1][0], "B");
        assert_eq!(rows[2][0], "C");
    }

    /// 测试 flatten_tree_data - 缩进不影响非目标列
    #[cfg(target_arch = "wasm32")]
    #[test]
    fn test_flatten_tree_indent_only_affects_target_column() {
        let root = js_sys::Array::new();
        let item = js_sys::Object::new();
        js_sys::Reflect::set(&item, &JsValue::from_str("name"), &JsValue::from_str("A")).unwrap();
        js_sys::Reflect::set(&item, &JsValue::from_str("age"), &JsValue::from_str("10")).unwrap();

        let child = js_sys::Object::new();
        js_sys::Reflect::set(&child, &JsValue::from_str("name"), &JsValue::from_str("B")).unwrap();
        js_sys::Reflect::set(&child, &JsValue::from_str("age"), &JsValue::from_str("5")).unwrap();

        let c_arr = js_sys::Array::new();
        c_arr.push(&child.into());
        js_sys::Reflect::set(&item, &JsValue::from_str("children"), &c_arr.into()).unwrap();

        root.push(&item.into());

        let keys = vec!["name".to_string(), "age".to_string()];
        let mut rows = Vec::new();
        flatten_tree_data(&root.into(), &keys, Some("name"), "children", 0, &mut rows).unwrap();

        assert_eq!(rows.len(), 2);
        // name 列：子节点有缩进
        assert_eq!(rows[1][0], "    B");
        // age 列：子节点没有缩进
        assert_eq!(rows[1][1], "5");
    }
}
