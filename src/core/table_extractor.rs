use crate::utils::is_element_hidden;
use std::collections::HashMap;
use wasm_bindgen::JsCast;
/// 表格数据提取模块
///
/// 提供从 DOM 中提取表格数据的功能，支持合并单元格（colspan/rowspan）
use wasm_bindgen::prelude::*;
use web_sys::{Element, HtmlTableCellElement, HtmlTableElement, HtmlTableRowElement};

/// 根据 ID 查找 table 元素
///
/// 支持两种情况：
/// 1. ID 直接指向 `<table>` 元素
/// 2. ID 指向容器元素（如 `<div>`、`<section>`），在其内部查找第一个 `<table>`
///
/// # 参数
/// * `element` - 通过 ID 获取到的 DOM 元素
/// * `element_id` - 元素的 ID（用于错误信息）
///
/// # 返回值
/// * `Ok(HtmlTableElement)` - 找到的表格元素
/// * `Err(JsValue)` - 未找到有效的表格元素
pub fn find_table_element(element: Element, element_id: &str) -> Result<HtmlTableElement, JsValue> {
    // 先尝试直接转换为 HtmlTableElement
    match element.clone().dyn_into::<HtmlTableElement>() {
        Ok(table) => Ok(table),
        Err(_) => {
            // 不是 table 元素，在其内部查找第一个 <table>
            element
                .query_selector("table")
                .map_err(|e| {
                    JsValue::from_str(&format!("在元素 '{}' 内查找表格失败: {:?}", element_id, e))
                })?
                .ok_or_else(|| {
                    JsValue::from_str(&format!(
                        "元素 '{}' 不是表格，且其内部也未找到 <table> 元素",
                        element_id
                    ))
                })?
                .dyn_into::<HtmlTableElement>()
                .map_err(|_| {
                    JsValue::from_str(&format!(
                        "在元素 '{}' 内找到的元素不是有效的 HTML 表格",
                        element_id
                    ))
                })
        }
    }
}

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
    /// 表头行数（用于 XLSX 冻结窗格），0 表示无表头
    pub header_row_count: usize,
}

impl TableData {
    /// 创建新的表格数据
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            merge_ranges: Vec::new(),
            header_row_count: 0,
        }
    }

    /// 创建指定容量的表格数据
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            merge_ranges: Vec::new(),
            header_row_count: 0,
        }
    }

    /// 获取纯文本数据（用于 CSV 导出等场景）
    #[allow(dead_code)]
    pub fn into_rows(self) -> Vec<Vec<String>> {
        self.rows
    }
}

impl Default for TableData {
    fn default() -> Self {
        Self::new()
    }
}

/// 从文档中根据 ID 获取 table 元素
///
/// 封装 window → document → getElementById → find_table_element 的完整流程，
/// 消除各模块重复的 DOM 查找代码。
///
/// # 参数
/// * `table_id` - HTML 表格元素或容器元素的 ID
///
/// # 返回值
/// * `Ok(HtmlTableElement)` - 找到的表格元素
/// * `Err(JsValue)` - 获取失败
pub fn resolve_table(table_id: &str) -> Result<HtmlTableElement, JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

    let element = document
        .get_element_by_id(table_id)
        .ok_or_else(|| JsValue::from_str(&format!("找不到 ID 为 '{}' 的元素", table_id)))?;

    find_table_element(element, table_id)
}

/// 从行集合中获取并转换行元素
///
/// # 参数
/// * `rows` - HTML 表格行集合
/// * `row_idx` - 行索引
///
/// # 返回值
/// * `Ok(HtmlTableRowElement)` - 转换成功的行元素
/// * `Err(JsValue)` - 获取或转换失败
pub fn get_table_row(
    rows: &web_sys::HtmlCollection,
    row_idx: u32,
) -> Result<HtmlTableRowElement, JsValue> {
    let row = rows
        .get_with_index(row_idx)
        .ok_or_else(|| JsValue::from_str(&format!("无法获取第 {} 行数据", row_idx + 1)))?;

    row.dyn_into::<HtmlTableRowElement>()
        .map_err(|_| JsValue::from_str(&format!("第 {} 行不是有效的表格行", row_idx + 1)))
}

/// 单行处理结果
pub struct RowProcessResult {
    /// 行数据（单元格文本列表）
    pub row_data: Vec<String>,
    /// 每个非隐藏单元格的位置和跨度信息，用于计算合并区域
    /// 格式：(起始列索引, CellSpan)
    pub cell_spans: Vec<(usize, CellSpan)>,
}

/// 处理一行表格的所有单元格
///
/// 封装单元格遍历、rowspan/colspan 处理、隐藏列检测等核心逻辑，
/// 消除 4 个 DOM 遍历函数中的重复代码。
///
/// # 参数
/// * `row` - 表格行元素
/// * `row_idx` - 当前行在原始表格中的索引（用于 tracker 和错误信息）
/// * `tracker` - rowspan 追踪器
/// * `exclude_hidden` - 是否排除隐藏的列
///
/// # 返回值
/// * `Ok(RowProcessResult)` - 包含行数据和单元格跨度信息
/// * `Err(JsValue)` - 处理失败
pub fn process_row_cells(
    row: &HtmlTableRowElement,
    row_idx: u32,
    tracker: &mut RowSpanTracker,
    exclude_hidden: bool,
) -> Result<RowProcessResult, JsValue> {
    let mut row_data = Vec::new();
    let mut cell_spans = Vec::new();
    let cells = row.cells();
    let cell_count = cells.length();
    let mut col_idx: usize = 0;

    for cell_idx in 0..cell_count {
        // 处理被上方 rowspan 占用的列
        while let Some(text) = tracker.pop(row_idx, col_idx) {
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

        if exclude_hidden && is_element_hidden(&cell) {
            continue;
        }

        let span = get_cell_span(&cell);

        // 记录单元格位置和跨度信息（供调用方计算合并区域）
        cell_spans.push((col_idx, span.clone()));

        // 处理 rowspan: 将当前单元格内容预填到后续行的对应位置
        tracker.add(row_idx, col_idx, &span);

        // 处理 colspan: 当前单元格内容 + 空白填充
        row_data.push(span.text);
        for _ in 1..span.colspan {
            row_data.push(String::new());
        }

        col_idx += span.colspan as usize;
    }

    // 处理行尾残留的 rowspan 占位
    while let Some(text) = tracker.pop(row_idx, col_idx) {
        row_data.push(text);
        col_idx += 1;
    }

    Ok(RowProcessResult {
        row_data,
        cell_spans,
    })
}

/// 单元格跨度信息
#[derive(Clone)]
pub(crate) struct CellSpan {
    /// 单元格文本内容
    pub text: String,
    /// 列跨度（colspan 属性值）
    pub colspan: u32,
    /// 行跨度（rowspan 属性值）
    pub rowspan: u32,
}

/// 获取单元格的跨度信息
///
/// # 参数
/// * `cell` - HTML 表格单元格元素
///
/// # 返回值
/// 包含文本内容和跨度信息的 CellSpan 结构
pub(crate) fn get_cell_span(cell: &HtmlTableCellElement) -> CellSpan {
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

/// 用于追踪被 rowspan 占用的单元格
pub struct RowSpanTracker {
    tracker: HashMap<(u32, usize), String>,
}

impl RowSpanTracker {
    pub fn new() -> Self {
        Self {
            tracker: HashMap::new(),
        }
    }

    /// 获取并移除指定位置的预填内容
    pub fn pop(&mut self, row_idx: u32, col_idx: usize) -> Option<String> {
        self.tracker.remove(&(row_idx, col_idx))
    }

    /// 记录跨行单元格的占位
    pub fn add(&mut self, row_idx: u32, col_idx: usize, span: &CellSpan) {
        if span.rowspan > 1 {
            for r in 1..span.rowspan {
                for c in 0..span.colspan as usize {
                    let fill_text = if c == 0 {
                        span.text.clone()
                    } else {
                        String::new()
                    };
                    self.tracker.insert((row_idx + r, col_idx + c), fill_text);
                }
            }
        }
    }
}

impl Default for RowSpanTracker {
    fn default() -> Self {
        Self::new()
    }
}

/// 从 HTML 表格中提取数据（简化版，仅返回文本数据）
///
/// CSV 等不需要合并单元格信息的场景使用此函数，
/// 跳过 merge_ranges 的计算和内存分配，提升性能。
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
    let table = resolve_table(table_id)?;
    let rows = table.rows();
    let row_count = rows.length();

    if row_count == 0 {
        return Err(JsValue::from_str("表格为空，没有数据可导出"));
    }

    let mut result: Vec<Vec<String>> = Vec::new();
    let mut tracker = RowSpanTracker::new();

    for row_idx in 0..row_count {
        let row = get_table_row(&rows, row_idx)?;

        if exclude_hidden && is_element_hidden(&row) {
            continue;
        }

        let proc_result = process_row_cells(&row, row_idx, &mut tracker, exclude_hidden)?;
        result.push(proc_result.row_data);
    }

    Ok(result)
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
    let table = resolve_table(table_id)?;
    let rows = table.rows();
    let row_count = rows.length();

    if row_count == 0 {
        return Err(JsValue::from_str("表格为空，没有数据可导出"));
    }

    // 自动检测 thead 行数，用于 XLSX 冻结窗格
    let header_row_count = table
        .t_head()
        .map(|thead| thead.rows().length() as usize)
        .unwrap_or(0);

    let mut result = TableData::new();
    result.header_row_count = header_row_count;
    let mut tracker = RowSpanTracker::new();
    let mut output_row_idx: u32 = 0;

    for row_idx in 0..row_count {
        let row = get_table_row(&rows, row_idx)?;

        if exclude_hidden && is_element_hidden(&row) {
            continue;
        }

        let proc_result = process_row_cells(&row, row_idx, &mut tracker, exclude_hidden)?;

        // 根据 cell_spans 计算合并区域
        compute_merge_ranges(
            &proc_result.cell_spans,
            row_idx,
            output_row_idx,
            exclude_hidden,
            &rows,
            &mut result.merge_ranges,
        );

        result.rows.push(proc_result.row_data);
        output_row_idx += 1;
    }

    Ok(result)
}

/// 根据单元格跨度信息计算合并区域
///
/// # 参数
/// * `cell_spans` - 单元格位置和跨度信息
/// * `row_idx` - 原始表格中的行索引
/// * `output_row_idx` - 输出行索引（排除隐藏行后的索引）
/// * `exclude_hidden` - 是否排除隐藏行
/// * `rows` - 表格行集合（用于检测后续行的可见性）
/// * `merge_ranges` - 合并区域列表（输出）
fn compute_merge_ranges(
    cell_spans: &[(usize, CellSpan)],
    row_idx: u32,
    output_row_idx: u32,
    exclude_hidden: bool,
    rows: &web_sys::HtmlCollection,
    merge_ranges: &mut Vec<MergeRange>,
) {
    for (col_idx, span) in cell_spans {
        // 计算实际覆盖的可见行数
        let visible_rows_covered = count_visible_rows(span.rowspan, row_idx, exclude_hidden, rows);

        let last_row = output_row_idx + visible_rows_covered;
        let last_col = (*col_idx + span.colspan as usize - 1) as u16;

        // 记录合并区域（仅当范围覆盖多个单元格时）
        if last_row > output_row_idx || last_col as usize > *col_idx {
            merge_ranges.push(MergeRange::new(
                output_row_idx,
                *col_idx as u16,
                last_row,
                last_col,
            ));
        }
    }
}

/// 计算 rowspan 覆盖的可见行数
///
/// # 参数
/// * `rowspan` - 原始 rowspan 值
/// * `row_idx` - 当前行索引
/// * `exclude_hidden` - 是否排除隐藏行
/// * `rows` - 表格行集合
///
/// # 返回值
/// 可见行数（不含当前行）
pub(crate) fn count_visible_rows(
    rowspan: u32,
    row_idx: u32,
    exclude_hidden: bool,
    rows: &web_sys::HtmlCollection,
) -> u32 {
    if rowspan <= 1 {
        return 0;
    }

    let mut visible_rows_covered = 0;
    for r in 1..rowspan {
        let next_row_idx = row_idx + r;
        if let Some(next_row) = rows.get_with_index(next_row_idx) {
            #[allow(clippy::collapsible_if)]
            if let Ok(next_row_el) = next_row.dyn_into::<HtmlTableRowElement>() {
                if !exclude_hidden || !is_element_hidden(&next_row_el) {
                    visible_rows_covered += 1;
                }
            }
        }
    }
    visible_rows_covered
}
