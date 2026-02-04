use crate::utils::is_element_hidden;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
/// 表格数据提取模块
///
/// 提供从 DOM 中提取表格数据的功能，支持合并单元格（colspan/rowspan）
use wasm_bindgen::prelude::*;
use web_sys::{HtmlTableCellElement, HtmlTableElement, HtmlTableRowElement};

/// 合并单元格区域信息
#[derive(Debug, Clone)]
pub struct MergeRange {
    /// 起始行索引（0-based）
    pub first_row: u32,
    /// 起始列索引（0-based）
    pub first_col: u16,
    /// 结束行索引（0-based，inclusive）
    pub last_row: u32,
    /// 结束列索引（0-based，inclusive）
    pub last_col: u16,
}

impl MergeRange {
    /// 创建新的合并区域
    pub fn new(first_row: u32, first_col: u16, last_row: u32, last_col: u16) -> Self {
        Self {
            first_row,
            first_col,
            last_row,
            last_col,
        }
    }
}

/// 表格数据结构，包含单元格数据和合并信息
#[derive(Debug, Clone)]
pub struct TableData {
    /// 二维字符串数组，表示表格数据
    pub rows: Vec<Vec<String>>,
    /// 合并单元格区域列表
    pub merge_ranges: Vec<MergeRange>,
}

impl TableData {
    /// 创建新的表格数据
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            merge_ranges: Vec::new(),
        }
    }

    /// 获取纯文本数据（用于 CSV 导出等场景）
    pub fn into_rows(self) -> Vec<Vec<String>> {
        self.rows
    }
}

impl Default for TableData {
    fn default() -> Self {
        Self::new()
    }
}

/// 单元格跨度信息
struct CellSpan {
    /// 单元格文本内容
    text: String,
    /// 列跨度（colspan 属性值）
    colspan: u32,
    /// 行跨度（rowspan 属性值）
    rowspan: u32,
}

/// 获取单元格的跨度信息
///
/// # 参数
/// * `cell` - HTML 表格单元格元素
///
/// # 返回值
/// 包含文本内容和跨度信息的 CellSpan 结构
fn get_cell_span(cell: &HtmlTableCellElement) -> CellSpan {
    let text = cell.inner_text();
    // colspan/rowspan 最小为 1
    let colspan = cell.col_span().max(1);
    let rowspan = cell.row_span().max(1);

    CellSpan {
        text,
        colspan,
        rowspan,
    }
}

/// 从 HTML 表格中提取数据（简化版，仅返回文本数据）
///
/// 使用占位矩阵算法处理 colspan 和 rowspan：
/// - colspan > 1: 填充空字符串到后续列
/// - rowspan > 1: 将当前值预填到下方行的对应位置
///
/// # 参数
/// * `table_id` - HTML 表格元素的 ID
/// * `exclude_hidden` - 是否排除隐藏的行和列
///
/// # 返回值
/// * `Ok(Vec<Vec<String>>)` - 二维字符串数组，表示表格数据
/// * `Err(JsValue)` - 提取失败，包含错误信息
pub fn extract_table_data(
    table_id: &str,
    exclude_hidden: bool,
) -> Result<Vec<Vec<String>>, JsValue> {
    let table_data = extract_table_data_with_merge(table_id, exclude_hidden)?;
    Ok(table_data.into_rows())
}

/// 从 HTML 表格中提取数据（完整版，包含合并单元格信息）
///
/// 使用占位矩阵算法处理 colspan 和 rowspan，同时记录合并区域用于 Excel 导出
///
/// # 参数
/// * `table_id` - HTML 表格元素的 ID
/// * `exclude_hidden` - 是否排除隐藏的行和列
///
/// # 返回值
/// * `Ok(TableData)` - 包含表格数据和合并区域信息
/// * `Err(JsValue)` - 提取失败，包含错误信息
pub fn extract_table_data_with_merge(
    table_id: &str,
    exclude_hidden: bool,
) -> Result<TableData, JsValue> {
    // 安全地获取全局的 window 和 document 对象
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

    // 根据 table_id 获取 table 元素，并进行类型检查
    let table_element = document
        .get_element_by_id(table_id)
        .ok_or_else(|| JsValue::from_str(&format!("找不到 ID 为 '{}' 的表格元素", table_id)))?;

    let table = table_element
        .dyn_into::<HtmlTableElement>()
        .map_err(|_| JsValue::from_str(&format!("元素 '{}' 不是有效的 HTML 表格", table_id)))?;

    // 遍历 table 中的每一行
    let rows = table.rows();
    let row_count = rows.length();

    if row_count == 0 {
        return Err(JsValue::from_str("表格为空，没有数据可导出"));
    }

    let mut result = TableData::new();

    // 用于追踪被 rowspan 占用的位置: (row, col) -> cell_text
    // 当某个单元格有 rowspan > 1 时，预先将其内容填入后续行的对应列位置
    let mut rowspan_tracker: HashMap<(u32, usize), String> = HashMap::new();

    // 跟踪实际输出的行索引（因为隐藏行可能被跳过）
    let mut output_row_idx: u32 = 0;

    for row_idx in 0..row_count {
        let row = rows
            .get_with_index(row_idx)
            .ok_or_else(|| JsValue::from_str(&format!("无法获取第 {} 行数据", row_idx + 1)))?;

        let row = row
            .dyn_into::<HtmlTableRowElement>()
            .map_err(|_| JsValue::from_str(&format!("第 {} 行不是有效的表格行", row_idx + 1)))?;

        // 如果需要排除隐藏行，检查 display 属性
        if exclude_hidden && is_element_hidden(&row) {
            continue;
        }

        let mut row_data = Vec::new();
        let cells = row.cells();
        let cell_count = cells.length();

        // col_idx: 实际输出列位置（考虑 colspan/rowspan 后的逻辑位置）
        let mut col_idx: usize = 0;

        for cell_idx in 0..cell_count {
            // 处理被上方 rowspan 占用的列：从 tracker 中取出预填的值
            while let Some(text) = rowspan_tracker.remove(&(row_idx, col_idx)) {
                row_data.push(text);
                col_idx += 1;
            }

            let cell = cells.get_with_index(cell_idx).ok_or_else(|| {
                JsValue::from_str(&format!(
                    "无法获取第 {} 行第 {} 列单元格",
                    row_idx + 1,
                    cell_idx + 1
                ))
            })?;

            let cell = cell.dyn_into::<HtmlTableCellElement>().map_err(|_| {
                JsValue::from_str(&format!(
                    "第 {} 行第 {} 列不是有效的表格单元格",
                    row_idx + 1,
                    cell_idx + 1
                ))
            })?;

            // 如果需要排除隐藏列，检查 display 属性
            if exclude_hidden && is_element_hidden(&cell) {
                continue;
            }

            let span = get_cell_span(&cell);

            // 记录合并区域（仅当 colspan > 1 或 rowspan > 1 时）
            if span.colspan > 1 || span.rowspan > 1 {
                result.merge_ranges.push(MergeRange::new(
                    output_row_idx,
                    col_idx as u16,
                    output_row_idx + span.rowspan - 1,
                    (col_idx + span.colspan as usize - 1) as u16,
                ));
            }

            // 处理 rowspan: 将当前单元格内容预填到后续行的对应位置
            if span.rowspan > 1 {
                for r in 1..span.rowspan {
                    // 对于 rowspan 覆盖的每一行，需要处理 colspan
                    for c in 0..span.colspan as usize {
                        let fill_text = if c == 0 {
                            span.text.clone()
                        } else {
                            String::new()
                        };
                        rowspan_tracker.insert((row_idx + r, col_idx + c), fill_text);
                    }
                }
            }

            // 处理 colspan: 当前单元格内容 + 空白填充
            row_data.push(span.text);
            for _ in 1..span.colspan {
                row_data.push(String::new());
            }

            col_idx += span.colspan as usize;
        }

        // 处理行尾残留的 rowspan 占位（当最右边的列有 rowspan 时）
        while let Some(text) = rowspan_tracker.remove(&(row_idx, col_idx)) {
            row_data.push(text);
            col_idx += 1;
        }

        result.rows.push(row_data);
        output_row_idx += 1;
    }

    Ok(result)
}
