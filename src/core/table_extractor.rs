use crate::utils::{ensure_external_tbody, is_element_hidden};
use std::collections::HashMap;
use wasm_bindgen::JsCast;
/// 表格数据提取模块
///
/// 提供从 DOM 中提取表格数据的功能，支持合并单元格（colspan/rowspan）
use wasm_bindgen::prelude::*;
use web_sys::{
    Element, HtmlCollection, HtmlTableCellElement, HtmlTableElement, HtmlTableRowElement,
    HtmlTableSectionElement,
};

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
    /// 可选的样式表（仅 XLSX 导出时使用）
    pub style_sheet: Option<super::style::StyleSheet>,
}

impl TableData {
    /// 创建新的表格数据
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            merge_ranges: Vec::new(),
            header_row_count: 0,
            style_sheet: None,
        }
    }

    /// 创建指定容量的表格数据
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            merge_ranges: Vec::new(),
            header_row_count: 0,
            style_sheet: None,
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

/// 表格 + 可选外部 tbody 的行数据源（分批导出共用）
pub struct TableRowSources {
    pub table_rows: HtmlCollection,
    pub table_row_count: usize,
    pub tbody_rows: Option<HtmlCollection>,
    pub tbody_row_count: usize,
    pub header_row_count: usize,
}

impl TableRowSources {
    /// 解析 table 与可选外部 tbody
    pub fn open(table_id: &str, tbody_id: Option<&str>) -> Result<Self, JsValue> {
        let table = resolve_table(table_id)?;
        let table_rows = table.rows();
        let table_row_count = table_rows.length() as usize;
        let header_row_count = table
            .t_head()
            .map(|thead| thead.rows().length() as usize)
            .unwrap_or(0);

        let mut tbody_rows = None;
        let mut tbody_row_count = 0;

        if let Some(tid) = tbody_id.filter(|tid| !tid.is_empty()) {
            let window =
                web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
            let document = window
                .document()
                .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

            let tbody_element = document.get_element_by_id(tid).ok_or_else(|| {
                JsValue::from_str(&format!("找不到 ID 为 '{}' 的 tbody 元素", tid))
            })?;

            ensure_external_tbody(&table, table_id, &tbody_element, tid)?;

            let tbody = tbody_element
                .dyn_into::<HtmlTableSectionElement>()
                .map_err(|_| {
                    JsValue::from_str(&format!("元素 '{}' 不是有效的 HTML 表格部分(tbody)", tid))
                })?;

            let rows = tbody.rows();
            tbody_row_count = rows.length() as usize;
            tbody_rows = Some(rows);
        }

        Ok(Self {
            table_rows,
            table_row_count,
            tbody_rows,
            tbody_row_count,
            header_row_count,
        })
    }

    /// 获取表格和外部 tbody 的总行数。
    pub fn total_rows(&self) -> usize {
        self.table_row_count + self.tbody_row_count
    }

    /// 根据跨源索引读取原始表格行。
    pub fn get_row(&self, index: usize) -> Result<HtmlTableRowElement, JsValue> {
        if index < self.table_row_count {
            get_table_row(&self.table_rows, index as u32)
        } else if let Some(ref rows) = self.tbody_rows {
            get_table_row(rows, (index - self.table_row_count) as u32)
        } else {
            Err(JsValue::from_str(&format!(
                "无法获取第 {} 行数据",
                index + 1
            )))
        }
    }
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
    tracker.begin_row(row, row_idx);
    let remaining_rows = tracker.group_end_row.saturating_sub(row_idx).max(1);
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

        let mut span = get_cell_span(&cell);
        // HTML 的 rowspan=0 延伸到当前行组末尾，正跨度也不能越过行组。
        span.rowspan = if span.rowspan == 0 {
            remaining_rows
        } else {
            span.rowspan.min(remaining_rows)
        };
        let next_col = col_idx
            .checked_add(span.colspan as usize)
            .filter(|&col| col <= MAX_DOM_COLUMNS)
            .ok_or_else(|| JsValue::from_str("列数超过表格导出限制 (16384)"))?;

        // 记录单元格位置和跨度信息（供调用方计算合并区域）
        cell_spans.push((col_idx, span.clone()));

        // 仅记录每列的活跃区间，不展开未来行的占位矩阵。
        tracker.add(row_idx, col_idx, &span);

        // 处理 colspan: 当前单元格内容 + 空白填充
        row_data.push(span.text);
        for _ in 1..span.colspan {
            row_data.push(String::new());
        }

        col_idx = next_col;
    }

    // 行尾可能存在不连续的跨行占位，保留中间的空列。
    if let Some(last_col) = tracker.last_active_column(row_idx) {
        while col_idx <= last_col {
            row_data.push(tracker.pop(row_idx, col_idx).unwrap_or_default());
            col_idx += 1;
        }
    }
    tracker.record_output_cells(row_data.len())?;

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
    // rowspan=0 有特殊的行组语义，在处理行时结合剩余行数规范化。
    let colspan = cell.col_span().max(1);
    let rowspan = cell.row_span();

    CellSpan {
        text,
        colspan,
        rowspan,
    }
}

/// DOM 导出与数据导出使用相同量级的总单元格预算，避免跨度放大内存。
const MAX_DOM_CELLS: usize = 5_000_000;
const MAX_DOM_COLUMNS: usize = 16_384;

/// 计算当前行组内剩余的真实行数（含当前行）。
fn remaining_group_rows(row: &HtmlTableRowElement) -> u32 {
    let row_index = row.section_row_index();
    if row_index < 0 {
        return 1;
    }
    if let Some(parent) = row.parent_element() {
        if let Some(section) = parent.dyn_ref::<HtmlTableSectionElement>() {
            return section
                .rows()
                .length()
                .saturating_sub(row_index as u32)
                .max(1);
        }
        // 通过 DOM API 直接插入 table 的 tr 不一定有 tbody，只覆盖连续的兄弟行。
        let mut remaining = 1_u32;
        let mut next = row.next_element_sibling();
        while let Some(element) = next {
            if !element.is_instance_of::<HtmlTableRowElement>() {
                break;
            }
            remaining = remaining.saturating_add(1);
            next = element.next_element_sibling();
        }
        return remaining;
    }
    1
}

/// 一列的跨行占位区间，结束行不包含在区间内。
struct ActiveRowSpan {
    first_row: u32,
    end_row: u32,
    text: String,
}

/// 用于追踪被 rowspan 占用的列，内存仅与活跃列数有关。
pub struct RowSpanTracker {
    tracker: HashMap<usize, ActiveRowSpan>,
    row_group: Option<Element>,
    group_end_row: u32,
    output_cells: usize,
}

impl RowSpanTracker {
    /// 创建空的跨行占位追踪器。
    pub fn new() -> Self {
        Self {
            tracker: HashMap::new(),
            row_group: None,
            group_end_row: 0,
            output_cells: 0,
        }
    }

    /// 清理过期占位，并确保任何跨度都不会进入下一个行组或外部 tbody。
    fn begin_row(&mut self, row: &HtmlTableRowElement, row_idx: u32) {
        let group = row.parent_element();
        if self.row_group != group || row_idx >= self.group_end_row {
            self.tracker.clear();
            self.row_group = group;
            // sectionRowIndex 可能遍历前面的兄弟行，只在进入行组时读取一次。
            self.group_end_row = row_idx.saturating_add(remaining_group_rows(row));
        }
        self.tracker.retain(|_, span| span.end_row > row_idx);
    }

    /// 读取指定位置的占位；区间保留至末行，不为未来每行分配字符串。
    pub fn pop(&mut self, row_idx: u32, col_idx: usize) -> Option<String> {
        self.tracker.get(&col_idx).and_then(|span| {
            (span.first_row <= row_idx && row_idx < span.end_row).then(|| span.text.clone())
        })
    }

    /// 获取当前行最后一个活跃占位列。
    fn last_active_column(&self, row_idx: u32) -> Option<usize> {
        self.tracker
            .iter()
            .filter(|(_, span)| span.first_row <= row_idx && row_idx < span.end_row)
            .map(|(&col, _)| col)
            .max()
    }

    /// 记录跨行单元格的列区间，不按 rowspan × colspan 展开。
    pub fn add(&mut self, row_idx: u32, col_idx: usize, span: &CellSpan) {
        if span.rowspan > 1 {
            for c in 0..span.colspan as usize {
                self.tracker.insert(
                    col_idx + c,
                    ActiveRowSpan {
                        first_row: row_idx.saturating_add(1),
                        end_row: row_idx.saturating_add(span.rowspan),
                        text: if c == 0 {
                            span.text.clone()
                        } else {
                            String::new()
                        },
                    },
                );
            }
        }
    }

    /// 累计实际输出规模，空行也计入预算以限制行容器分配。
    fn record_output_cells(&mut self, count: usize) -> Result<(), JsValue> {
        self.output_cells = self
            .output_cells
            .checked_add(count.max(1))
            .filter(|&total| total <= MAX_DOM_CELLS)
            .ok_or_else(|| JsValue::from_str("表格数据总量超过单元格限制 (5000000)"))?;
        Ok(())
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

    if result.is_empty() {
        return Err(JsValue::from_str("没有可导出的可见数据"));
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

    // 按原始 thead 范围识别表头，再累计实际导出的可见表头行。
    let header_row_count = table
        .t_head()
        .map(|thead| thead.rows().length() as usize)
        .unwrap_or(0);

    let mut result = TableData::new();
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

        if (row_idx as usize) < header_row_count {
            result.header_row_count += 1;
        }
        result.rows.push(proc_result.row_data);
        output_row_idx += 1;
    }

    if result.rows.is_empty() {
        return Err(JsValue::from_str("没有可导出的可见数据"));
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
        let last_col_usize = col_idx
            .saturating_add(span.colspan as usize)
            .saturating_sub(1);
        // 超出 Excel 列上限则钳制，避免 u16 截断
        if last_col_usize > 16383 {
            continue;
        }
        let last_col = last_col_usize as u16;

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
    let end = row_idx.saturating_add(rowspan).min(rows.length());
    for next_row_idx in row_idx.saturating_add(1)..end {
        if let Some(next_row) = rows.get_with_index(next_row_idx) {
            if let Ok(next_row_el) = next_row.dyn_into::<HtmlTableRowElement>() {
                if !exclude_hidden || !is_element_hidden(&next_row_el) {
                    visible_rows_covered += 1;
                }
            }
        }
    }
    visible_rows_covered
}

#[cfg(test)]
mod tests {
    use super::{CellSpan, RowSpanTracker};

    #[test]
    fn test_row_span_tracker_storage_scales_with_columns() {
        let mut tracker = RowSpanTracker::new();
        tracker.add(
            0,
            0,
            &CellSpan {
                text: "边界🌟".to_string(),
                colspan: 1000,
                rowspan: 65_534,
            },
        );

        // 即使声明六万多行，也只记录一千个活跃列，而非六千多万个坐标。
        assert_eq!(tracker.tracker.len(), 1000);
        assert_eq!(tracker.pop(0, 0), None);
        assert_eq!(tracker.pop(1, 0).as_deref(), Some("边界🌟"));
        assert_eq!(tracker.pop(65_533, 0).as_deref(), Some("边界🌟"));
        assert_eq!(tracker.pop(65_533, 999).as_deref(), Some(""));
        assert_eq!(tracker.pop(65_534, 0), None);
        assert_eq!(tracker.pop(1, 1000), None);
    }

    #[cfg(target_arch = "wasm32")]
    #[wasm_bindgen_test::wasm_bindgen_test]
    fn test_row_span_tracker_output_budget_and_overflow() {
        let mut tracker = RowSpanTracker::new();
        tracker.output_cells = super::MAX_DOM_CELLS - 1;
        tracker.record_output_cells(0).unwrap();
        assert_eq!(tracker.output_cells, super::MAX_DOM_CELLS);
        assert!(
            tracker
                .record_output_cells(1)
                .unwrap_err()
                .as_string()
                .unwrap()
                .contains("5000000")
        );
        tracker.output_cells = usize::MAX;
        assert!(tracker.record_output_cells(1).is_err());
    }

    #[test]
    fn test_row_span_tracker_sparse_tail_column() {
        let mut tracker = RowSpanTracker::new();
        tracker.add(
            2,
            4,
            &CellSpan {
                text: "尾列".to_string(),
                colspan: 2,
                rowspan: 3,
            },
        );
        assert_eq!(tracker.last_active_column(2), None);
        assert_eq!(tracker.last_active_column(3), Some(5));
        assert_eq!(tracker.last_active_column(5), None);
    }
}
