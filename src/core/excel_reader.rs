/// Excel 文件解析核心模块
///
/// 使用 calamine 提取单元格数据和合并区域，
/// 使用 zip + quick-xml 提取样式索引、行高和列宽信息。
use calamine::{Data, Reader, SheetVisible, Sheets, Xlsx, open_workbook_auto_from_rs};
use serde::Serialize;
use std::io::{Cursor, Read, Seek};

use super::excel_style::{ExcelStyleSheet, cell_style_to_css, format_number};

/// 最大渲染行数上限（防止 DOM 假死）
const MAX_ROWS_LIMIT: usize = 100_000;
/// 最大渲染列数上限
const MAX_COLS_LIMIT: usize = 16_384;

/// 判断数据是否为 XLSX 格式（ZIP 文件以 PK 开头）
fn is_xlsx_format(data: &[u8]) -> bool {
    data.len() >= 4 && data[0] == 0x50 && data[1] == 0x4B
}

/// 预览配置选项
#[derive(Debug, Clone)]
pub struct PreviewOptions {
    /// 指定渲染的 Sheet 索引（默认 0）
    pub sheet_index: Option<usize>,
    /// 按名称指定 Sheet
    pub sheet_name: Option<String>,
    /// 最大渲染行数
    pub max_rows: Option<usize>,
    /// 最大渲染列数
    pub max_cols: Option<usize>,
    /// 是否保留样式（默认 true）
    pub include_styles: bool,
    /// 是否裁剪空白区域（默认 true）
    pub trim_empty: bool,
    /// 是否跳过隐藏工作表（默认 true）
    pub skip_hidden: bool,
}

impl Default for PreviewOptions {
    fn default() -> Self {
        Self {
            sheet_index: None,
            sheet_name: None,
            max_rows: None,
            max_cols: None,
            include_styles: true,
            trim_empty: true,
            skip_hidden: true,
        }
    }
}

/// 工作表信息
#[derive(Debug, Clone, Serialize)]
pub struct SheetInfo {
    /// 工作表名称
    pub name: String,
    /// 索引（在原始工作簿中的位置）
    pub index: usize,
    /// 行数
    pub rows: usize,
    /// 列数
    pub cols: usize,
    /// 是否隐藏
    pub hidden: bool,
}

/// 解析后的工作簿
#[derive(Debug, Clone, Serialize)]
pub struct ParsedWorkbook {
    /// 所有工作表
    pub sheets: Vec<ParsedSheet>,
    /// 活动工作表索引
    #[serde(rename = "activeSheet")]
    pub active_sheet: usize,
}

/// 解析后的工作表
#[derive(Debug, Clone, Serialize)]
pub struct ParsedSheet {
    /// 工作表名称
    pub name: String,
    /// 数据行
    pub rows: Vec<ParsedRow>,
    /// 列宽数组（像素）
    #[serde(rename = "colWidths")]
    pub col_widths: Vec<f64>,
    /// 合并区域
    #[serde(rename = "mergedCells")]
    pub merged_cells: Vec<MergeRegion>,
    /// 是否因 maxRows 被截断
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncated: Option<bool>,
}

/// 解析后的行
#[derive(Debug, Clone, Serialize)]
pub struct ParsedRow {
    /// 行高（像素）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
    /// 单元格
    pub cells: Vec<Option<ParsedCell>>,
}

/// 解析后的单元格
#[derive(Debug, Clone, Serialize)]
pub struct ParsedCell {
    /// 显示值
    pub value: String,
    /// 内联 CSS 样式
    #[serde(skip_serializing_if = "Option::is_none")]
    pub style: Option<String>,
    /// 列合并数
    #[serde(skip_serializing_if = "Option::is_none", rename = "colSpan")]
    pub col_span: Option<u32>,
    /// 行合并数
    #[serde(skip_serializing_if = "Option::is_none", rename = "rowSpan")]
    pub row_span: Option<u32>,
}

/// 合并区域
#[derive(Debug, Clone, Serialize)]
pub struct MergeRegion {
    /// 起始行
    #[serde(rename = "startRow")]
    pub start_row: u32,
    /// 起始列
    #[serde(rename = "startCol")]
    pub start_col: u32,
    /// 结束行
    #[serde(rename = "endRow")]
    pub end_row: u32,
    /// 结束列
    #[serde(rename = "endCol")]
    pub end_col: u32,
}

/// 行高/列宽信息（从 sheet xml 中解析）
#[derive(Debug, Default)]
struct SheetDimensions {
    /// 行号 → 行高（pt）
    row_heights: std::collections::HashMap<u32, f64>,
    /// 列号 → 列宽（字符数）
    col_widths: std::collections::HashMap<u32, f64>,
    /// 单元格样式索引：(行, 列) → 样式索引
    cell_styles: std::collections::HashMap<(u32, u32), usize>,
    /// 默认行高
    default_row_height: Option<f64>,
    /// 默认列宽
    default_col_width: Option<f64>,
}

/// 获取 Excel 文件的工作表列表（含隐藏状态）
pub fn get_sheet_list(data: &[u8]) -> Result<Vec<SheetInfo>, String> {
    let cursor = Cursor::new(data);
    let mut workbook =
        open_workbook_auto_from_rs(cursor).map_err(|e| format!("无法解析 Excel 文件: {e}"))?;

    let sheet_names = workbook.sheet_names().to_vec();

    // 从 calamine 的 metadata 获取隐藏状态（适用于所有格式）
    let hidden_set = build_hidden_set(&workbook);

    let mut sheets = Vec::new();
    for (index, name) in sheet_names.iter().enumerate() {
        let (rows, cols) = match workbook.worksheet_range(name) {
            Ok(range) => range.get_size(),
            Err(_) => (0, 0),
        };
        sheets.push(SheetInfo {
            name: name.clone(),
            index,
            rows,
            cols,
            hidden: hidden_set.contains(name),
        });
    }

    Ok(sheets)
}

/// 解析 Excel 文件为结构化数据
pub fn parse_excel(data: &[u8], options: &PreviewOptions) -> Result<ParsedWorkbook, String> {
    let xlsx = is_xlsx_format(data);

    let cursor = Cursor::new(data);
    let mut workbook =
        open_workbook_auto_from_rs(cursor).map_err(|e| format!("无法解析 Excel 文件: {e}"))?;

    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Err("Excel 文件中没有工作表".to_string());
    }

    // 从 calamine metadata 获取隐藏 sheet（适用于所有格式）
    let hidden_set = if options.skip_hidden {
        build_hidden_set(&workbook)
    } else {
        std::collections::HashSet::new()
    };

    // 解析 OOXML 样式表（仅 XLSX 支持）
    let style_sheet = if options.include_styles && xlsx {
        let style_cursor = Cursor::new(data);
        ExcelStyleSheet::from_xlsx_zip(style_cursor).ok()
    } else {
        None
    };

    // 确定要渲染的 Sheet
    let target_index = resolve_sheet_index(&sheet_names, &hidden_set, options)?;
    let target_name = &sheet_names[target_index];

    // 解析 sheet 维度信息（仅 XLSX 支持行高/列宽/样式索引）
    let dimensions = if options.include_styles && xlsx {
        let dim_cursor = Cursor::new(data);
        parse_sheet_dimensions(dim_cursor, target_index).unwrap_or_default()
    } else {
        SheetDimensions::default()
    };

    // 读取数据
    let range = workbook
        .worksheet_range(target_name)
        .map_err(|e| format!("读取工作表 '{target_name}' 失败: {e}"))?;

    // 获取合并区域（仅 XLSX 支持）
    let merge_regions = if xlsx {
        get_merge_regions_xlsx(data, target_name)
    } else {
        Vec::new()
    };

    // 构建 ParsedSheet
    let sheet = build_parsed_sheet(
        target_name,
        &range,
        &merge_regions,
        &dimensions,
        style_sheet.as_ref(),
        options,
    )?;

    Ok(ParsedWorkbook {
        sheets: vec![sheet],
        active_sheet: 0,
    })
}

/// 确定目标 Sheet 索引（考虑隐藏状态）
fn resolve_sheet_index(
    sheet_names: &[String],
    hidden_set: &std::collections::HashSet<String>,
    options: &PreviewOptions,
) -> Result<usize, String> {
    if let Some(ref name) = options.sheet_name {
        sheet_names
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| format!("未找到工作表: {name}"))
    } else if let Some(idx) = options.sheet_index {
        // 显式指定索引时直接使用
        if idx >= sheet_names.len() {
            Err(format!(
                "工作表索引 {idx} 超出范围（共 {} 个工作表）",
                sheet_names.len()
            ))
        } else {
            Ok(idx)
        }
    } else {
        // 未指定时自动选择第一个可见 sheet
        if options.skip_hidden && !hidden_set.is_empty() {
            for (i, name) in sheet_names.iter().enumerate() {
                if !hidden_set.contains(name) {
                    return Ok(i);
                }
            }
        }
        Ok(0)
    }
}

/// 从 calamine 的 metadata 构建隐藏 Sheet 名称集合（适用于所有格式）
fn build_hidden_set<RS: Read + Seek>(workbook: &Sheets<RS>) -> std::collections::HashSet<String> {
    workbook
        .sheets_metadata()
        .iter()
        .filter(|s| s.visible != SheetVisible::Visible)
        .map(|s| s.name.clone())
        .collect()
}

/// 获取 XLSX 文件中指定 Sheet 的合并区域
fn get_merge_regions_xlsx(data: &[u8], sheet_name: &str) -> Vec<MergeRegion> {
    let cursor = Cursor::new(data);
    let mut xlsx: Xlsx<_> = match Xlsx::new(cursor) {
        Ok(wb) => wb,
        Err(_) => return Vec::new(),
    };
    if xlsx.load_merged_regions().is_err() {
        return Vec::new();
    }
    xlsx.merged_regions_by_sheet(sheet_name)
        .iter()
        .map(|(_, _, dim)| MergeRegion {
            start_row: dim.start.0,
            start_col: dim.start.1,
            end_row: dim.end.0,
            end_col: dim.end.1,
        })
        .collect()
}

/// 构建 ParsedSheet
fn build_parsed_sheet(
    name: &str,
    range: &calamine::Range<Data>,
    merge_regions: &[MergeRegion],
    dimensions: &SheetDimensions,
    style_sheet: Option<&ExcelStyleSheet>,
    options: &PreviewOptions,
) -> Result<ParsedSheet, String> {
    let (total_rows, total_cols) = range.get_size();
    if total_rows == 0 || total_cols == 0 {
        return Ok(ParsedSheet {
            name: name.to_string(),
            rows: Vec::new(),
            col_widths: Vec::new(),
            merged_cells: Vec::new(),
            truncated: None,
        });
    }

    // 确定实际数据范围
    let (data_rows, data_cols, truncated) = if options.trim_empty {
        let (rows, cols) = find_data_bounds(range);
        let max_rows = options
            .max_rows
            .map(|m| m.min(MAX_ROWS_LIMIT))
            .unwrap_or(MAX_ROWS_LIMIT);
        let max_cols = options
            .max_cols
            .map(|m| m.min(MAX_COLS_LIMIT))
            .unwrap_or(MAX_COLS_LIMIT);
        let actual_rows = rows.min(max_rows);
        let actual_cols = cols.min(max_cols);
        let truncated = actual_rows < rows;
        (actual_rows, actual_cols, truncated)
    } else {
        let max_rows = options
            .max_rows
            .map(|m| m.min(MAX_ROWS_LIMIT))
            .unwrap_or(total_rows.min(MAX_ROWS_LIMIT));
        let max_cols = options
            .max_cols
            .map(|m| m.min(MAX_COLS_LIMIT))
            .unwrap_or(total_cols.min(MAX_COLS_LIMIT));
        let truncated = max_rows < total_rows;
        (max_rows, max_cols, truncated)
    };

    // 构建合并单元格查找表：(row, col) → (rowSpan, colSpan)
    let merge_map = build_merge_map(merge_regions, data_rows as u32, data_cols as u32);
    // 被合并占用的单元格集合
    let skip_set = build_skip_set(merge_regions, data_rows as u32, data_cols as u32);

    let start = range.start().unwrap_or((0, 0));

    // 构建行数据
    let mut rows = Vec::with_capacity(data_rows);
    for r in 0..data_rows {
        let abs_row = start.0 + r as u32;
        let height = dimensions
            .row_heights
            .get(&abs_row)
            .copied()
            .or(dimensions.default_row_height)
            .map(|h| h * 1.333); // pt → px 近似转换

        let mut cells = Vec::with_capacity(data_cols);
        for c in 0..data_cols {
            let abs_col = start.1 + c as u32;

            // 被合并占用的单元格输出 null
            if skip_set.contains(&(abs_row, abs_col)) {
                cells.push(None);
                continue;
            }

            let cell_data = range.get((r, c));
            let value = cell_value_to_string(cell_data, dimensions, abs_row, abs_col, style_sheet);

            // 获取样式
            let style_css = if let Some(ss) = style_sheet {
                let style_idx = dimensions
                    .cell_styles
                    .get(&(abs_row, abs_col))
                    .copied()
                    .unwrap_or(0);
                if style_idx > 0 && style_idx < ss.xf_count() {
                    let cell_style = ss.get_cell_style(style_idx);
                    let css = cell_style_to_css(&cell_style);
                    if css.is_empty() { None } else { Some(css) }
                } else {
                    None
                }
            } else {
                None
            };

            // 合并信息
            let (row_span, col_span) = merge_map
                .get(&(abs_row, abs_col))
                .copied()
                .unwrap_or((1, 1));

            cells.push(Some(ParsedCell {
                value,
                style: style_css,
                col_span: if col_span > 1 { Some(col_span) } else { None },
                row_span: if row_span > 1 { Some(row_span) } else { None },
            }));
        }

        rows.push(ParsedRow { height, cells });
    }

    // 构建列宽
    let col_widths = (0..data_cols as u32)
        .map(|c| {
            let abs_col = start.1 + c;
            dimensions
                .col_widths
                .get(&abs_col)
                .copied()
                .or(dimensions.default_col_width)
                .map(|w| (w * 7.0 + 12.0).clamp(55.0, 500.0)) // 字符宽 → 像素近似
                .unwrap_or(60.0) // 默认宽度
        })
        .collect();

    // 过滤合并区域到数据范围内
    let filtered_merges: Vec<MergeRegion> = merge_regions
        .iter()
        .filter(|m| {
            (m.start_row as usize) < data_rows + start.0 as usize
                && (m.start_col as usize) < data_cols + start.1 as usize
        })
        .cloned()
        .collect();

    Ok(ParsedSheet {
        name: name.to_string(),
        rows,
        col_widths,
        merged_cells: filtered_merges,
        truncated: if truncated { Some(true) } else { None },
    })
}

/// 构建合并映射表：主单元格 → (rowSpan, colSpan)
fn build_merge_map(
    regions: &[MergeRegion],
    max_row: u32,
    max_col: u32,
) -> std::collections::HashMap<(u32, u32), (u32, u32)> {
    let mut map = std::collections::HashMap::new();
    for m in regions {
        if m.start_row < max_row && m.start_col < max_col {
            let row_span = (m.end_row - m.start_row + 1).min(max_row - m.start_row);
            let col_span = (m.end_col - m.start_col + 1).min(max_col - m.start_col);
            map.insert((m.start_row, m.start_col), (row_span, col_span));
        }
    }
    map
}

/// 构建被合并占用的单元格集合（不含主单元格）
fn build_skip_set(
    regions: &[MergeRegion],
    max_row: u32,
    max_col: u32,
) -> std::collections::HashSet<(u32, u32)> {
    let mut set = std::collections::HashSet::new();
    for m in regions {
        for r in m.start_row..=m.end_row.min(max_row.saturating_sub(1)) {
            for c in m.start_col..=m.end_col.min(max_col.saturating_sub(1)) {
                if r != m.start_row || c != m.start_col {
                    set.insert((r, c));
                }
            }
        }
    }
    set
}

/// 查找数据实际边界（排除尾部空行空列）
fn find_data_bounds(range: &calamine::Range<Data>) -> (usize, usize) {
    let (total_rows, total_cols) = range.get_size();
    if total_rows == 0 || total_cols == 0 {
        return (0, 0);
    }

    let mut max_row = 0;
    let mut max_col = 0;

    for (r, c, _) in range.used_cells() {
        if r >= max_row {
            max_row = r + 1;
        }
        if c >= max_col {
            max_col = c + 1;
        }
    }

    (max_row.min(total_rows), max_col.min(total_cols))
}

/// 将单元格数据转换为显示字符串
fn cell_value_to_string(
    data: Option<&Data>,
    dimensions: &SheetDimensions,
    row: u32,
    col: u32,
    style_sheet: Option<&ExcelStyleSheet>,
) -> String {
    match data {
        None | Some(Data::Empty) => String::new(),
        Some(Data::String(s)) => s.clone(),
        Some(Data::Float(f)) => {
            // 尝试使用数字格式
            if let Some(ss) = style_sheet {
                let style_idx = dimensions
                    .cell_styles
                    .get(&(row, col))
                    .copied()
                    .unwrap_or(0);
                let cell_style = ss.get_cell_style(style_idx);
                if let Some(ref fmt) = cell_style.number_format {
                    return format_number(*f, fmt);
                }
            }
            // 无样式表时使用 General 格式
            format_number(*f, "General")
        }
        Some(Data::Int(i)) => format!("{i}"),
        Some(Data::Bool(b)) => {
            if *b {
                "TRUE".to_string()
            } else {
                "FALSE".to_string()
            }
        }
        Some(Data::Error(e)) => format!("{e:?}"),
        Some(Data::DateTime(dt)) => format!("{dt}"),
        Some(Data::DateTimeIso(s)) => s.clone(),
        Some(Data::DurationIso(s)) => s.clone(),
    }
}

/// 从 xlsx zip 中解析 sheet 的行高/列宽/样式索引
fn parse_sheet_dimensions<R: Read + Seek>(
    reader: R,
    sheet_index: usize,
) -> Result<SheetDimensions, String> {
    let mut archive = zip::ZipArchive::new(reader).map_err(|e| format!("无法读取 zip: {e}"))?;

    // 确定 sheet xml 路径
    let sheet_path = find_sheet_path(&mut archive, sheet_index)?;

    let mut file = archive
        .by_name(&sheet_path)
        .map_err(|e| format!("无法找到 {sheet_path}: {e}"))?;
    let mut xml = String::new();
    file.read_to_string(&mut xml)
        .map_err(|e| format!("读取 {sheet_path} 失败: {e}"))?;

    parse_sheet_xml(&xml)
}

/// 查找 sheet xml 的路径
fn find_sheet_path<R: Read + Seek>(
    archive: &mut zip::ZipArchive<R>,
    sheet_index: usize,
) -> Result<String, String> {
    // 先尝试直接路径
    let direct = format!("xl/worksheets/sheet{}.xml", sheet_index + 1);
    if archive.by_name(&direct).is_ok() {
        return Ok(direct);
    }

    // 解析 workbook.xml.rels 获取正确路径
    if let Ok(mut rels_file) = archive.by_name("xl/_rels/workbook.xml.rels") {
        let mut rels_xml = String::new();
        if rels_file.read_to_string(&mut rels_xml).is_ok() {
            let mut reader = quick_xml::Reader::from_str(&rels_xml);
            let mut buf = Vec::new();
            let mut sheet_paths = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Ok(quick_xml::events::Event::Empty(ref e))
                    | Ok(quick_xml::events::Event::Start(ref e)) => {
                        let name_bytes = e.name().as_ref().to_vec();
                        let local = std::str::from_utf8(&name_bytes).unwrap_or("");
                        if local.ends_with("Relationship") {
                            let typ = get_rel_attr(e, "Type");
                            let target = get_rel_attr(e, "Target");
                            if let (Some(t), Some(tgt)) = (typ, target) {
                                if t.contains("worksheet") {
                                    // 归一化路径
                                    let path = if let Some(stripped) = tgt.strip_prefix('/') {
                                        stripped.to_string()
                                    } else {
                                        format!("xl/{tgt}")
                                    };
                                    sheet_paths.push(path);
                                }
                            }
                        }
                    }
                    Ok(quick_xml::events::Event::Eof) => break,
                    Err(_) => break,
                    _ => {}
                }
                buf.clear();
            }

            if let Some(path) = sheet_paths.get(sheet_index) {
                return Ok(path.clone());
            }
        }
    }

    // 回退：使用默认路径
    Ok(direct)
}

/// 从 rels XML 属性获取值
fn get_rel_attr(event: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    for attr in event.attributes().flatten() {
        if attr.key.as_ref() == name.as_bytes() {
            return String::from_utf8(attr.value.to_vec()).ok();
        }
    }
    None
}

/// 解析 sheet xml 获取行高/列宽/样式索引
fn parse_sheet_xml(xml: &str) -> Result<SheetDimensions, String> {
    let mut reader = quick_xml::Reader::from_str(xml);
    let mut buf = Vec::new();
    let mut dims = SheetDimensions::default();
    let mut current_row: Option<u32> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(ref e))
            | Ok(quick_xml::events::Event::Empty(ref e)) => {
                let name_bytes = e.name().as_ref().to_vec();
                let local = local_name_str(&name_bytes);

                match local {
                    "sheetFormatPr" => {
                        dims.default_row_height =
                            get_rel_attr(e, "defaultRowHeight").and_then(|v| v.parse::<f64>().ok());
                        dims.default_col_width =
                            get_rel_attr(e, "defaultColWidth").and_then(|v| v.parse::<f64>().ok());
                    }
                    "col" => {
                        if let (Some(min_s), Some(max_s)) =
                            (get_rel_attr(e, "min"), get_rel_attr(e, "max"))
                        {
                            if let (Ok(min), Ok(max)) = (min_s.parse::<u32>(), max_s.parse::<u32>())
                            {
                                let width =
                                    get_rel_attr(e, "width").and_then(|v| v.parse::<f64>().ok());
                                if let Some(w) = width {
                                    // col 索引从 1 开始，转为 0-based
                                    for col in min..=max {
                                        dims.col_widths.insert(col - 1, w);
                                    }
                                }
                            }
                        }
                    }
                    "row" => {
                        current_row = get_rel_attr(e, "r").and_then(|v| v.parse::<u32>().ok());
                        if let Some(row) = current_row {
                            if let Some(ht) =
                                get_rel_attr(e, "ht").and_then(|v| v.parse::<f64>().ok())
                            {
                                // row 索引从 1 开始，转为 0-based
                                dims.row_heights.insert(row - 1, ht);
                            }
                        }
                    }
                    "c" => {
                        // 单元格样式索引
                        if let Some(r_attr) = get_rel_attr(e, "r") {
                            if let Some((row, col)) = parse_cell_ref(&r_attr) {
                                if let Some(s) =
                                    get_rel_attr(e, "s").and_then(|v| v.parse::<usize>().ok())
                                {
                                    dims.cell_styles.insert((row, col), s);
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => return Err(format!("解析 sheet xml 失败: {e}")),
            _ => {}
        }
        buf.clear();
    }
    let _ = current_row;

    Ok(dims)
}

/// 解析单元格引用（如 "A1" → (0, 0)、"B3" → (2, 1)）
fn parse_cell_ref(cell_ref: &str) -> Option<(u32, u32)> {
    let mut col: u32 = 0;
    let mut row_str = String::new();

    for ch in cell_ref.chars() {
        if ch.is_ascii_alphabetic() {
            col = col * 26 + (ch.to_ascii_uppercase() as u32 - b'A' as u32 + 1);
        } else if ch.is_ascii_digit() {
            row_str.push(ch);
        }
    }

    if col == 0 || row_str.is_empty() {
        return None;
    }

    let row = row_str.parse::<u32>().ok()?;
    // 转为 0-based
    Some((row - 1, col - 1))
}

/// 获取 XML 元素的本地名称
fn local_name_str(full_name: &[u8]) -> &str {
    let name = std::str::from_utf8(full_name).unwrap_or("");
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

// ============================================================================
// 单元测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cell_ref() {
        assert_eq!(parse_cell_ref("A1"), Some((0, 0)));
        assert_eq!(parse_cell_ref("B3"), Some((2, 1)));
        assert_eq!(parse_cell_ref("Z1"), Some((0, 25)));
        assert_eq!(parse_cell_ref("AA1"), Some((0, 26)));
        assert_eq!(parse_cell_ref("AZ10"), Some((9, 51)));
        assert_eq!(parse_cell_ref(""), None);
        assert_eq!(parse_cell_ref("123"), None);
    }

    #[test]
    fn test_find_data_bounds_empty() {
        let range = calamine::Range::<Data>::empty();
        assert_eq!(find_data_bounds(&range), (0, 0));
    }

    #[test]
    fn test_build_merge_map() {
        let regions = vec![MergeRegion {
            start_row: 0,
            start_col: 0,
            end_row: 1,
            end_col: 2,
        }];
        let map = build_merge_map(&regions, 10, 10);
        assert_eq!(map.get(&(0, 0)), Some(&(2, 3)));
    }

    #[test]
    fn test_build_skip_set() {
        let regions = vec![MergeRegion {
            start_row: 0,
            start_col: 0,
            end_row: 1,
            end_col: 1,
        }];
        let set = build_skip_set(&regions, 10, 10);
        assert!(!set.contains(&(0, 0))); // 主单元格不跳过
        assert!(set.contains(&(0, 1))); // 被占用
        assert!(set.contains(&(1, 0))); // 被占用
        assert!(set.contains(&(1, 1))); // 被占用
    }

    #[test]
    fn test_cell_value_to_string_basic() {
        let dims = SheetDimensions::default();

        assert_eq!(
            cell_value_to_string(Some(&Data::String("hello".into())), &dims, 0, 0, None),
            "hello"
        );
        assert_eq!(
            cell_value_to_string(Some(&Data::Float(3.14)), &dims, 0, 0, None),
            "3.14"
        );
        assert_eq!(
            cell_value_to_string(Some(&Data::Int(42)), &dims, 0, 0, None),
            "42"
        );
        assert_eq!(
            cell_value_to_string(Some(&Data::Bool(true)), &dims, 0, 0, None),
            "TRUE"
        );
        assert_eq!(cell_value_to_string(None, &dims, 0, 0, None), "");
    }

    #[test]
    fn test_cell_value_integer_display() {
        let dims = SheetDimensions::default();
        // 整数应该不带小数点
        assert_eq!(
            cell_value_to_string(Some(&Data::Float(100.0)), &dims, 0, 0, None),
            "100"
        );
    }

    #[test]
    fn test_resolve_sheet_index() {
        let names = vec!["Sheet1".to_string(), "Sheet2".to_string()];
        let empty_hidden = std::collections::HashSet::new();
        let opts = PreviewOptions::default();
        assert_eq!(
            resolve_sheet_index(&names, &empty_hidden, &opts).unwrap(),
            0
        );

        let opts = PreviewOptions {
            sheet_index: Some(1),
            ..Default::default()
        };
        assert_eq!(
            resolve_sheet_index(&names, &empty_hidden, &opts).unwrap(),
            1
        );

        let opts = PreviewOptions {
            sheet_name: Some("Sheet2".to_string()),
            ..Default::default()
        };
        assert_eq!(
            resolve_sheet_index(&names, &empty_hidden, &opts).unwrap(),
            1
        );

        let opts = PreviewOptions {
            sheet_index: Some(5),
            ..Default::default()
        };
        assert!(resolve_sheet_index(&names, &empty_hidden, &opts).is_err());

        // 测试 skip_hidden：自动选择第一个可见 sheet
        let names3 = vec![
            "Hidden1".to_string(),
            "Visible".to_string(),
            "Hidden2".to_string(),
        ];
        let mut hidden_set = std::collections::HashSet::new();
        hidden_set.insert("Hidden1".to_string());
        hidden_set.insert("Hidden2".to_string());
        let opts = PreviewOptions {
            skip_hidden: true,
            ..Default::default()
        };
        assert_eq!(resolve_sheet_index(&names3, &hidden_set, &opts).unwrap(), 1);
    }

    /// 使用 rust_xlsxwriter 生成测试 xlsx 文件并解析
    #[test]
    fn test_parse_excel_basic() {
        let xlsx_data = create_test_xlsx_basic();
        let options = PreviewOptions {
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        assert_eq!(workbook.sheets.len(), 1);
        let sheet = &workbook.sheets[0];
        assert!(!sheet.rows.is_empty());

        // 验证第一行数据
        let first_row = &sheet.rows[0];
        assert!(first_row.cells.len() >= 2);
        assert_eq!(
            first_row.cells[0].as_ref().map(|c| c.value.as_str()),
            Some("名称")
        );
        assert_eq!(
            first_row.cells[1].as_ref().map(|c| c.value.as_str()),
            Some("数值")
        );
    }

    #[test]
    fn test_get_sheet_list() {
        let xlsx_data = create_test_xlsx_basic();
        let sheets = get_sheet_list(&xlsx_data).unwrap();
        assert!(!sheets.is_empty());
        assert_eq!(sheets[0].name, "Sheet1");
    }

    #[test]
    fn test_parse_excel_empty_data() {
        // 空数据应返回错误
        let result = parse_excel(&[], &PreviewOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_excel_invalid_data() {
        // 无效数据应返回错误
        let result = parse_excel(b"not an excel file", &PreviewOptions::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_excel_max_rows() {
        let xlsx_data = create_test_xlsx_many_rows(100);
        let options = PreviewOptions {
            max_rows: Some(10),
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let sheet = &workbook.sheets[0];
        assert!(sheet.rows.len() <= 10);
        assert_eq!(sheet.truncated, Some(true));
    }

    /// 创建基础测试 xlsx 文件
    fn create_test_xlsx_basic() -> Vec<u8> {
        let mut wb = rust_xlsxwriter::Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Sheet1").unwrap();
        ws.write_string(0, 0, "名称").unwrap();
        ws.write_string(0, 1, "数值").unwrap();
        ws.write_string(1, 0, "测试A").unwrap();
        ws.write_number(1, 1, 100.0).unwrap();
        ws.write_string(2, 0, "测试B").unwrap();
        ws.write_number(2, 1, 200.0).unwrap();
        wb.save_to_buffer().unwrap()
    }

    /// 创建多行测试 xlsx 文件
    fn create_test_xlsx_many_rows(count: usize) -> Vec<u8> {
        let mut wb = rust_xlsxwriter::Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Sheet1").unwrap();
        ws.write_string(0, 0, "序号").unwrap();
        ws.write_string(0, 1, "数据").unwrap();
        for i in 1..=count {
            ws.write_number(i as u32, 0, i as f64).unwrap();
            ws.write_string(i as u32, 1, &format!("行{i}")).unwrap();
        }
        wb.save_to_buffer().unwrap()
    }

    /// 创建带样式的测试 xlsx 文件
    fn create_test_xlsx_with_styles() -> Vec<u8> {
        use rust_xlsxwriter::{Color, Format};

        let mut wb = rust_xlsxwriter::Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Sheet1").unwrap();

        // 粗体单元格
        let bold_fmt = Format::new().set_bold();
        ws.write_string_with_format(0, 0, "粗体标题", &bold_fmt)
            .unwrap();

        // 带字体颜色的单元格
        let red_fmt = Format::new().set_font_color(Color::Red);
        ws.write_string_with_format(0, 1, "红色文字", &red_fmt)
            .unwrap();

        // 带背景色的单元格
        let bg_fmt = Format::new().set_background_color(Color::Yellow);
        ws.write_string_with_format(1, 0, "黄色背景", &bg_fmt)
            .unwrap();

        // 普通单元格（无样式）
        ws.write_string(1, 1, "普通文字").unwrap();

        wb.save_to_buffer().unwrap()
    }

    /// 创建带合并单元格的测试 xlsx 文件
    fn create_test_xlsx_with_merge() -> Vec<u8> {
        let mut wb = rust_xlsxwriter::Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Sheet1").unwrap();

        // 合并 A1:B2（2行2列）
        let fmt = rust_xlsxwriter::Format::new();
        ws.merge_range(0, 0, 1, 1, "合并区域", &fmt).unwrap();

        // 在合并区域外写入数据
        ws.write_string(0, 2, "C1").unwrap();
        ws.write_string(1, 2, "C2").unwrap();
        ws.write_string(2, 0, "A3").unwrap();
        ws.write_string(2, 1, "B3").unwrap();
        ws.write_string(2, 2, "C3").unwrap();

        wb.save_to_buffer().unwrap()
    }

    /// 创建多工作表测试 xlsx 文件
    fn create_test_xlsx_multi_sheet() -> Vec<u8> {
        let mut wb = rust_xlsxwriter::Workbook::new();

        let ws1 = wb.add_worksheet();
        ws1.set_name("销售数据").unwrap();
        ws1.write_string(0, 0, "产品").unwrap();
        ws1.write_string(0, 1, "金额").unwrap();
        ws1.write_string(1, 0, "商品A").unwrap();
        ws1.write_number(1, 1, 1000.0).unwrap();

        let ws2 = wb.add_worksheet();
        ws2.set_name("库存数据").unwrap();
        ws2.write_string(0, 0, "仓库").unwrap();
        ws2.write_string(0, 1, "数量").unwrap();
        ws2.write_string(1, 0, "仓库1").unwrap();
        ws2.write_number(1, 1, 500.0).unwrap();

        let ws3 = wb.add_worksheet();
        ws3.set_name("汇总").unwrap();
        ws3.write_string(0, 0, "总计").unwrap();

        wb.save_to_buffer().unwrap()
    }

    /// 创建包含多种数据类型的测试 xlsx 文件
    fn create_test_xlsx_data_types() -> Vec<u8> {
        let mut wb = rust_xlsxwriter::Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Sheet1").unwrap();

        // 表头
        ws.write_string(0, 0, "类型").unwrap();
        ws.write_string(0, 1, "值").unwrap();

        // 字符串
        ws.write_string(1, 0, "字符串").unwrap();
        ws.write_string(1, 1, "Hello 世界").unwrap();

        // 整数（作为浮点数写入）
        ws.write_string(2, 0, "整数").unwrap();
        ws.write_number(2, 1, 42.0).unwrap();

        // 浮点数
        ws.write_string(3, 0, "浮点数").unwrap();
        ws.write_number(3, 1, 3.14159).unwrap();

        // 布尔值
        ws.write_string(4, 0, "布尔值").unwrap();
        ws.write_boolean(4, 1, true).unwrap();

        // 空单元格（不写入 B6）
        ws.write_string(5, 0, "空值").unwrap();

        // 特殊字符
        ws.write_string(6, 0, "特殊字符").unwrap();
        ws.write_string(6, 1, "<script>alert('xss')</script>")
            .unwrap();

        // 长文本
        ws.write_string(7, 0, "长文本").unwrap();
        ws.write_string(7, 1, &"测试".repeat(100)).unwrap();

        wb.save_to_buffer().unwrap()
    }

    // ========================================================================
    // 新增测试：样式、合并单元格、多工作表、数据类型、HTML 构建
    // ========================================================================

    /// 测试带样式的 xlsx 解析（粗体、颜色、背景色）
    #[test]
    fn test_parse_excel_with_styles() {
        let xlsx_data = create_test_xlsx_with_styles();

        // include_styles = true 时应生成样式信息
        let options = PreviewOptions {
            include_styles: true,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let sheet = &workbook.sheets[0];
        assert!(!sheet.rows.is_empty());

        // 粗体单元格应该有 style
        let bold_cell = sheet.rows[0].cells[0].as_ref().unwrap();
        assert_eq!(bold_cell.value, "粗体标题");
        // 样式字符串应包含 font-weight:bold
        if let Some(ref style) = bold_cell.style {
            assert!(
                style.contains("font-weight:bold"),
                "粗体样式缺失，实际样式: {style}"
            );
        }

        // 普通单元格
        let plain_cell = sheet.rows[1].cells[1].as_ref().unwrap();
        assert_eq!(plain_cell.value, "普通文字");

        // include_styles = false 时所有单元格不应有样式
        let options_no_style = PreviewOptions {
            include_styles: false,
            ..Default::default()
        };
        let wb_no_style = parse_excel(&xlsx_data, &options_no_style).unwrap();
        for row in &wb_no_style.sheets[0].rows {
            for cell in row.cells.iter().flatten() {
                assert!(
                    cell.style.is_none(),
                    "include_styles=false 时不应包含样式: {:?}",
                    cell.style
                );
            }
        }
    }

    /// 测试合并单元格的解析
    #[test]
    fn test_parse_excel_merge_cells() {
        let xlsx_data = create_test_xlsx_with_merge();
        let options = PreviewOptions {
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let sheet = &workbook.sheets[0];

        // 验证合并区域已被记录
        assert!(!sheet.merged_cells.is_empty(), "应包含合并区域信息");

        // 验证合并区域范围：A1:B2 即 (0,0)-(1,1)
        let merge = &sheet.merged_cells[0];
        assert_eq!(merge.start_row, 0);
        assert_eq!(merge.start_col, 0);
        assert_eq!(merge.end_row, 1);
        assert_eq!(merge.end_col, 1);

        // 验证合并主单元格的 span 属性
        let master_cell = sheet.rows[0].cells[0].as_ref().unwrap();
        assert_eq!(master_cell.value, "合并区域");
        assert_eq!(master_cell.col_span, Some(2));
        assert_eq!(master_cell.row_span, Some(2));

        // 验证被合并占用的单元格为 None
        assert!(
            sheet.rows[0].cells[1].is_none(),
            "被合并占用的 B1 应为 None"
        );
        assert!(
            sheet.rows[1].cells[0].is_none(),
            "被合并占用的 A2 应为 None"
        );
        assert!(
            sheet.rows[1].cells[1].is_none(),
            "被合并占用的 B2 应为 None"
        );

        // 验证合并区域外的数据正常
        let c1 = sheet.rows[0].cells[2].as_ref().unwrap();
        assert_eq!(c1.value, "C1");
        let a3 = sheet.rows[2].cells[0].as_ref().unwrap();
        assert_eq!(a3.value, "A3");
    }

    /// 测试按工作表名称选择
    #[test]
    fn test_parse_excel_sheet_name_selection() {
        let xlsx_data = create_test_xlsx_multi_sheet();

        // 按名称选择第二个工作表
        let options = PreviewOptions {
            sheet_name: Some("库存数据".to_string()),
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let sheet = &workbook.sheets[0];
        assert_eq!(sheet.name, "库存数据");

        // 验证数据内容
        let header = sheet.rows[0].cells[0].as_ref().unwrap();
        assert_eq!(header.value, "仓库");

        // 选择不存在的工作表名称应报错
        let bad_options = PreviewOptions {
            sheet_name: Some("不存在的表".to_string()),
            include_styles: false,
            ..Default::default()
        };
        let result = parse_excel(&xlsx_data, &bad_options);
        assert!(result.is_err());
        assert!(
            result.unwrap_err().contains("未找到工作表"),
            "错误信息应包含中文提示"
        );
    }

    /// 测试多种数据类型的解析
    #[test]
    fn test_parse_excel_multiple_data_types() {
        let xlsx_data = create_test_xlsx_data_types();
        let options = PreviewOptions {
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let sheet = &workbook.sheets[0];

        // 字符串
        let str_cell = sheet.rows[1].cells[1].as_ref().unwrap();
        assert_eq!(str_cell.value, "Hello 世界");

        // 整数（应不带小数点）
        let int_cell = sheet.rows[2].cells[1].as_ref().unwrap();
        assert_eq!(int_cell.value, "42");

        // 浮点数
        let float_cell = sheet.rows[3].cells[1].as_ref().unwrap();
        assert!(
            float_cell.value.starts_with("3.14"),
            "浮点数值应以 3.14 开头，实际: {}",
            float_cell.value
        );

        // 布尔值
        let bool_cell = sheet.rows[4].cells[1].as_ref().unwrap();
        assert_eq!(bool_cell.value, "TRUE");

        // 空单元格
        let empty_row = &sheet.rows[5];
        if empty_row.cells.len() > 1 {
            match &empty_row.cells[1] {
                None => {} // 空单元格为 None，符合预期
                Some(cell) => assert!(
                    cell.value.is_empty(),
                    "空单元格值应为空，实际: {}",
                    cell.value
                ),
            }
        }

        // 特殊字符（原始值保留，HTML 转义在 html_builder 层处理）
        let special_cell = sheet.rows[6].cells[1].as_ref().unwrap();
        assert_eq!(special_cell.value, "<script>alert('xss')</script>");
    }

    /// 测试解析后构建 HTML 表格
    #[test]
    fn test_build_html_from_parsed_data() {
        use crate::core::html_builder::build_html_table;

        let xlsx_data = create_test_xlsx_basic();
        let options = PreviewOptions {
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let html = build_html_table(&workbook.sheets[0]);

        // 验证 HTML 结构
        assert!(html.starts_with("<style>"), "应以 <style> 开头");
        assert!(html.ends_with("</table>"), "应以 </table> 结尾");
        assert!(
            html.contains("<table class=\"bsg-preview-table\">"),
            "应包含带 class 的 <table>"
        );
        assert!(html.contains("<tbody>"), "应包含 <tbody>");
        assert!(html.contains("<tr"), "应包含 <tr>");
        assert!(html.contains("<td>"), "应包含 <td>");

        // 验证数据内容出现在 HTML 中
        assert!(html.contains("名称"), "HTML 中应包含表头 '名称'");
        assert!(html.contains("数值"), "HTML 中应包含表头 '数值'");
        assert!(html.contains("测试A"), "HTML 中应包含数据 '测试A'");
        assert!(html.contains("100"), "HTML 中应包含数值 '100'");
    }

    /// 测试合并单元格解析后生成的 HTML 包含正确的 colspan/rowspan
    #[test]
    fn test_build_html_merge_cells_attributes() {
        use crate::core::html_builder::build_html_table;

        let xlsx_data = create_test_xlsx_with_merge();
        let options = PreviewOptions {
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let html = build_html_table(&workbook.sheets[0]);

        // 验证合并属性
        assert!(
            html.contains("colspan=\"2\""),
            "HTML 应包含 colspan=\"2\"，实际: {html}"
        );
        assert!(
            html.contains("rowspan=\"2\""),
            "HTML 应包含 rowspan=\"2\"，实际: {html}"
        );
        assert!(html.contains("合并区域"), "HTML 应包含合并区域的文本");
    }

    /// 测试获取多工作表列表
    #[test]
    fn test_get_sheet_list_multiple() {
        let xlsx_data = create_test_xlsx_multi_sheet();
        let sheets = get_sheet_list(&xlsx_data).unwrap();

        assert_eq!(sheets.len(), 3, "应有 3 个工作表");

        // 验证名称和索引
        assert_eq!(sheets[0].name, "销售数据");
        assert_eq!(sheets[0].index, 0);
        assert_eq!(sheets[1].name, "库存数据");
        assert_eq!(sheets[1].index, 1);
        assert_eq!(sheets[2].name, "汇总");
        assert_eq!(sheets[2].index, 2);

        // 验证行列数大于 0（有数据的工作表）
        assert!(sheets[0].rows > 0, "销售数据表应有数据行");
        assert!(sheets[0].cols > 0, "销售数据表应有数据列");
        assert!(sheets[2].rows > 0, "汇总表应有数据行");
    }

    /// 测试按索引选择工作表
    #[test]
    fn test_parse_excel_sheet_index_selection() {
        let xlsx_data = create_test_xlsx_multi_sheet();

        // 选择第三个工作表（索引 2）
        let options = PreviewOptions {
            sheet_index: Some(2),
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        assert_eq!(workbook.sheets[0].name, "汇总");

        let first_cell = workbook.sheets[0].rows[0].cells[0].as_ref().unwrap();
        assert_eq!(first_cell.value, "总计");
    }

    /// 测试 XSS 内容经 HTML 构建后被转义
    #[test]
    fn test_build_html_xss_escape() {
        use crate::core::html_builder::build_html_table;

        let xlsx_data = create_test_xlsx_data_types();
        let options = PreviewOptions {
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let html = build_html_table(&workbook.sheets[0]);

        // script 标签应被 HTML 转义
        assert!(
            !html.contains("<script>"),
            "HTML 不应包含未转义的 <script> 标签"
        );
        assert!(
            html.contains("&lt;script&gt;"),
            "HTML 应包含转义后的 script 标签"
        );
    }

    /// 测试 trim_empty 配置（裁剪空白区域）
    #[test]
    fn test_parse_excel_trim_empty() {
        // 创建一个有大量空行空列的 xlsx
        let mut wb = rust_xlsxwriter::Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Sheet1").unwrap();
        ws.write_string(0, 0, "数据").unwrap();
        // 在远处写入一个空字符串（模拟空白区域）
        // calamine 的 range 会包含到最后有数据的位置
        let xlsx_data = wb.save_to_buffer().unwrap();

        let options = PreviewOptions {
            trim_empty: true,
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let sheet = &workbook.sheets[0];

        // 启用 trim_empty 后应裁剪空白区域
        assert!(!sheet.rows.is_empty(), "应至少有一行数据");
        // 第一行第一列应该是 "数据"
        let cell = sheet.rows[0].cells[0].as_ref().unwrap();
        assert_eq!(cell.value, "数据");
    }

    /// 测试 max_cols 限制
    #[test]
    fn test_parse_excel_max_cols() {
        // 创建一个宽表
        let mut wb = rust_xlsxwriter::Workbook::new();
        let ws = wb.add_worksheet();
        ws.set_name("Sheet1").unwrap();
        for col in 0..20u16 {
            ws.write_string(0, col, &format!("列{col}")).unwrap();
        }
        let xlsx_data = wb.save_to_buffer().unwrap();

        let options = PreviewOptions {
            max_cols: Some(5),
            include_styles: false,
            ..Default::default()
        };
        let workbook = parse_excel(&xlsx_data, &options).unwrap();
        let sheet = &workbook.sheets[0];

        // 每行的单元格数不应超过 max_cols
        for row in &sheet.rows {
            assert!(
                row.cells.len() <= 5,
                "列数应不超过 5，实际: {}",
                row.cells.len()
            );
        }
    }
}
