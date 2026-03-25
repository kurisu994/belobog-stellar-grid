/// Excel 样式解析与 CSS 映射模块
///
/// 负责从 xlsx 文件的 xl/styles.xml 和 xl/theme/theme1.xml 中
/// 提取样式信息，并将 OOXML 样式转换为 CSS 内联样式字符串。
use std::collections::HashMap;
use std::io::{Read, Seek};

/// OOXML 默认主题色（Office 2016+ 主题）
const DEFAULT_THEME_COLORS: [&str; 12] = [
    "#FFFFFF", "#000000", "#E7E6E6", "#44546A", "#4472C4", "#ED7D31", "#A5A5A5", "#FFC000",
    "#5B9BD5", "#70AD47", "#0563C1", "#954F72",
];

/// 单元格样式信息
#[derive(Debug, Clone, Default)]
pub struct ExcelCellStyle {
    pub font_name: Option<String>,
    pub font_size: Option<f64>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_color: Option<String>,
    pub bg_color: Option<String>,
    pub h_align: Option<String>,
    pub v_align: Option<String>,
    pub wrap_text: bool,
    pub border_top: Option<BorderDef>,
    pub border_bottom: Option<BorderDef>,
    pub border_left: Option<BorderDef>,
    pub border_right: Option<BorderDef>,
    pub number_format: Option<String>,
}

/// 边框定义
#[derive(Debug, Clone)]
pub struct BorderDef {
    pub style: String,
    pub color: Option<String>,
}

// 内部解析结构
#[derive(Debug, Clone, Default)]
struct FontDef {
    name: Option<String>,
    size: Option<f64>,
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    color: Option<ColorRef>,
}

#[derive(Debug, Clone, Default)]
struct FillDef {
    pattern_type: Option<String>,
    fg_color: Option<ColorRef>,
}

#[derive(Debug, Clone, Default)]
struct BorderSetDef {
    top: Option<RawBorderDef>,
    bottom: Option<RawBorderDef>,
    left: Option<RawBorderDef>,
    right: Option<RawBorderDef>,
}

#[derive(Debug, Clone)]
struct RawBorderDef {
    style: String,
    color: Option<ColorRef>,
}

#[derive(Debug, Clone, Default)]
struct AlignmentDef {
    horizontal: Option<String>,
    vertical: Option<String>,
    wrap_text: bool,
}

#[derive(Debug, Clone)]
enum ColorRef {
    Rgb(String),
    Theme(usize, f64),
    Indexed(usize),
}

#[derive(Debug, Clone, Default)]
struct CellXf {
    font_id: usize,
    fill_id: usize,
    border_id: usize,
    num_fmt_id: usize,
    alignment: AlignmentDef,
}

/// 样式表（从 xl/styles.xml 解析的完整样式信息）
#[derive(Debug)]
pub struct ExcelStyleSheet {
    theme_colors: Vec<String>,
    fonts: Vec<FontDef>,
    fills: Vec<FillDef>,
    borders: Vec<BorderSetDef>,
    num_fmts: HashMap<usize, String>,
    cell_xfs: Vec<CellXf>,
    /// 差异格式表（条件格式使用）
    dxfs: Vec<ExcelCellStyle>,
}

impl Default for ExcelStyleSheet {
    fn default() -> Self {
        Self {
            theme_colors: DEFAULT_THEME_COLORS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            fonts: Vec::new(),
            fills: Vec::new(),
            borders: Vec::new(),
            num_fmts: Self::builtin_num_fmts(),
            cell_xfs: Vec::new(),
            dxfs: Vec::new(),
        }
    }
}

impl ExcelStyleSheet {
    /// 内置数字格式
    fn builtin_num_fmts() -> HashMap<usize, String> {
        let mut m = HashMap::new();
        m.insert(0, "General".into());
        m.insert(1, "0".into());
        m.insert(2, "0.00".into());
        m.insert(3, "#,##0".into());
        m.insert(4, "#,##0.00".into());
        m.insert(9, "0%".into());
        m.insert(10, "0.00%".into());
        m.insert(11, "0.00E+00".into());
        m.insert(14, "yyyy-mm-dd".into());
        m.insert(22, "yyyy-mm-dd hh:mm".into());
        m
    }

    /// 从 xlsx zip 文件中解析样式信息
    pub fn from_xlsx_zip<R: Read + Seek>(reader: R) -> Result<Self, String> {
        let mut archive =
            zip::ZipArchive::new(reader).map_err(|e| format!("无法读取 zip 文件: {e}"))?;

        let theme_colors = Self::parse_theme(&mut archive).unwrap_or_else(|_| {
            DEFAULT_THEME_COLORS
                .iter()
                .map(|s| (*s).to_string())
                .collect()
        });

        let styles_xml = Self::read_zip_entry(&mut archive, "xl/styles.xml")?;
        let mut sheet = Self::parse_styles_xml(&styles_xml)?;
        sheet.theme_colors = theme_colors;
        Ok(sheet)
    }

    /// XML 文件最大读取限制（50MB），防止 Zip Bomb 攻击
    const MAX_XML_SIZE: u64 = 50 * 1024 * 1024;

    fn read_zip_entry<R: Read + Seek>(
        archive: &mut zip::ZipArchive<R>,
        name: &str,
    ) -> Result<String, String> {
        let file = archive
            .by_name(name)
            .map_err(|e| format!("无法找到 {name}: {e}"))?;
        let mut content = String::new();
        file.take(Self::MAX_XML_SIZE)
            .read_to_string(&mut content)
            .map_err(|e| format!("读取 {name} 失败: {e}"))?;
        Ok(content)
    }

    /// 解析主题色
    fn parse_theme<R: Read + Seek>(
        archive: &mut zip::ZipArchive<R>,
    ) -> Result<Vec<String>, String> {
        let xml = Self::read_zip_entry(archive, "xl/theme/theme1.xml")?;
        let mut reader = quick_xml::Reader::from_str(&xml);
        let mut buf = Vec::new();
        let mut in_theme_elements = false;
        let mut in_clr_scheme = false;

        let order_map: &[(&str, usize)] = &[
            ("dk1", 1),
            ("lt1", 0),
            ("dk2", 3),
            ("lt2", 2),
            ("accent1", 4),
            ("accent2", 5),
            ("accent3", 6),
            ("accent4", 7),
            ("accent5", 8),
            ("accent6", 9),
            ("hlink", 10),
            ("folHlink", 11),
        ];

        let mut theme_map: HashMap<usize, String> = HashMap::new();
        let mut current_element_index: Option<usize> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    let name_bytes = e.name().as_ref().to_vec();
                    let local = local_name(&name_bytes);
                    match local {
                        "themeElements" => in_theme_elements = true,
                        "clrScheme" if in_theme_elements => in_clr_scheme = true,
                        _ if in_clr_scheme => {
                            for &(name, idx) in order_map {
                                if local == name {
                                    current_element_index = Some(idx);
                                    break;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                Ok(quick_xml::events::Event::Empty(ref e)) => {
                    if in_clr_scheme {
                        let name_bytes = e.name().as_ref().to_vec();
                        let local = local_name(&name_bytes);

                        // 检查是否是颜色元素名称
                        for &(name, idx) in order_map {
                            if local == name {
                                current_element_index = Some(idx);
                                break;
                            }
                        }

                        if current_element_index.is_some() {
                            if local == "srgbClr" {
                                if let Some(val) = get_attr(e, "val") {
                                    if let Some(idx) = current_element_index.take() {
                                        theme_map.insert(idx, format!("#{val}"));
                                    }
                                }
                            } else if local == "sysClr" {
                                if let Some(val) = get_attr(e, "lastClr") {
                                    if let Some(idx) = current_element_index.take() {
                                        theme_map.insert(idx, format!("#{val}"));
                                    }
                                }
                            }
                        }
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    let name_bytes = e.name().as_ref().to_vec();
                    let local = local_name(&name_bytes);
                    match local {
                        "themeElements" => in_theme_elements = false,
                        "clrScheme" => in_clr_scheme = false,
                        _ => {
                            for &(name, _) in order_map {
                                if local == name {
                                    current_element_index = None;
                                    break;
                                }
                            }
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(format!("解析 theme1.xml 失败: {e}")),
                _ => {}
            }
            buf.clear();
        }

        let mut colors = Vec::with_capacity(12);
        for (i, default_color) in DEFAULT_THEME_COLORS.iter().enumerate().take(12) {
            let color = theme_map
                .get(&i)
                .cloned()
                .unwrap_or_else(|| (*default_color).to_string());
            colors.push(color);
        }
        Ok(colors)
    }

    /// 解析 styles.xml
    fn parse_styles_xml(xml: &str) -> Result<Self, String> {
        let mut reader = quick_xml::Reader::from_str(xml);
        let mut buf = Vec::new();

        let mut fonts: Vec<FontDef> = Vec::new();
        let mut fills: Vec<FillDef> = Vec::new();
        let mut borders: Vec<BorderSetDef> = Vec::new();
        let mut num_fmts = Self::builtin_num_fmts();
        let mut cell_xfs: Vec<CellXf> = Vec::new();
        let mut dxfs: Vec<ExcelCellStyle> = Vec::new();

        let mut section = Section::None;

        let mut current_font: Option<FontDef> = None;
        let mut current_fill: Option<FillDef> = None;
        let mut current_border: Option<BorderSetDef> = None;
        let mut current_xf: Option<CellXf> = None;
        let mut in_pattern_fill = false;
        let mut current_border_side: Option<String> = None;
        let mut current_border_style: Option<String> = None;
        // dxf 解析状态
        let mut current_dxf: Option<DxfParseState> = None;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(quick_xml::events::Event::Start(ref e)) => {
                    let name_bytes = e.name().as_ref().to_vec();
                    let local = local_name(&name_bytes);
                    // dxf 段内使用独立解析逻辑
                    if section == Section::Dxfs {
                        handle_dxf_start(local, e, &mut current_dxf);
                    } else if local == "dxfs" {
                        section = Section::Dxfs;
                    } else {
                        handle_start_element(
                            local,
                            e,
                            &mut section,
                            &mut current_font,
                            &mut current_fill,
                            &mut current_border,
                            &mut current_xf,
                            &mut in_pattern_fill,
                            &mut current_border_side,
                            &mut current_border_style,
                            &mut num_fmts,
                        );
                    }
                }
                Ok(quick_xml::events::Event::Empty(ref e)) => {
                    let name_bytes = e.name().as_ref().to_vec();
                    let local = local_name(&name_bytes);
                    if section == Section::Dxfs {
                        handle_dxf_start(local, e, &mut current_dxf);
                    } else {
                        handle_start_element(
                            local,
                            e,
                            &mut section,
                            &mut current_font,
                            &mut current_fill,
                            &mut current_border,
                            &mut current_xf,
                            &mut in_pattern_fill,
                            &mut current_border_side,
                            &mut current_border_style,
                            &mut num_fmts,
                        );
                        // 自闭合元素没有 End 事件，需立即提交到集合中
                        match local {
                            "xf" if section == Section::CellXfs => {
                                if let Some(xf) = current_xf.take() {
                                    cell_xfs.push(xf);
                                }
                            }
                            "font" if section == Section::Fonts => {
                                if let Some(f) = current_font.take() {
                                    fonts.push(f);
                                }
                            }
                            "fill" if section == Section::Fills => {
                                if let Some(f) = current_fill.take() {
                                    fills.push(f);
                                }
                                in_pattern_fill = false;
                            }
                            "border" if section == Section::Borders => {
                                if let Some(b) = current_border.take() {
                                    borders.push(b);
                                }
                                current_border_side = None;
                                current_border_style = None;
                            }
                            _ => {}
                        }
                    }
                }
                Ok(quick_xml::events::Event::End(ref e)) => {
                    let name_bytes = e.name().as_ref().to_vec();
                    let local = local_name(&name_bytes);
                    if section == Section::Dxfs {
                        match local {
                            "dxfs" => section = Section::None,
                            "dxf" => {
                                if let Some(state) = current_dxf.take() {
                                    dxfs.push(state.into_style());
                                }
                            }
                            "font" => {
                                if let Some(s) = current_dxf.as_mut() {
                                    s.in_font = false;
                                }
                            }
                            "fill" => {
                                if let Some(s) = current_dxf.as_mut() {
                                    s.in_fill = false;
                                }
                            }
                            _ => {}
                        }
                    } else {
                        match local {
                            "fonts" | "fills" | "borders" | "numFmts" | "cellXfs" => {
                                section = Section::None;
                            }
                            "font" if section == Section::Fonts => {
                                if let Some(f) = current_font.take() {
                                    fonts.push(f);
                                }
                            }
                            "fill" if section == Section::Fills => {
                                if let Some(f) = current_fill.take() {
                                    fills.push(f);
                                }
                                in_pattern_fill = false;
                            }
                            "patternFill" => {
                                in_pattern_fill = false;
                            }
                            "border" if section == Section::Borders => {
                                if let Some(b) = current_border.take() {
                                    borders.push(b);
                                }
                                current_border_side = None;
                                current_border_style = None;
                            }
                            "left" | "right" | "top" | "bottom"
                                if section == Section::Borders && current_border.is_some() =>
                            {
                                // 有样式但没颜色子元素时使用默认色
                                if let Some(border) = &mut current_border {
                                    if let Some(side) = &current_border_side {
                                        if let Some(style) = &current_border_style {
                                            if !style.is_empty() {
                                                let has_it = match side.as_str() {
                                                    "left" => border.left.is_some(),
                                                    "right" => border.right.is_some(),
                                                    "top" => border.top.is_some(),
                                                    "bottom" => border.bottom.is_some(),
                                                    _ => true,
                                                };
                                                if !has_it {
                                                    let raw = RawBorderDef {
                                                        style: style.clone(),
                                                        color: None,
                                                    };
                                                    match side.as_str() {
                                                        "left" => border.left = Some(raw),
                                                        "right" => border.right = Some(raw),
                                                        "top" => border.top = Some(raw),
                                                        "bottom" => border.bottom = Some(raw),
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                current_border_side = None;
                                current_border_style = None;
                            }
                            "xf" if section == Section::CellXfs => {
                                if let Some(xf) = current_xf.take() {
                                    cell_xfs.push(xf);
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Ok(quick_xml::events::Event::Eof) => break,
                Err(e) => return Err(format!("解析 styles.xml 失败: {e}")),
                _ => {}
            }
            buf.clear();
        }

        Ok(Self {
            theme_colors: DEFAULT_THEME_COLORS
                .iter()
                .map(|s| (*s).to_string())
                .collect(),
            fonts,
            fills,
            borders,
            num_fmts,
            cell_xfs,
            dxfs,
        })
    }

    /// 根据样式索引获取单元格样式
    pub fn get_cell_style(&self, style_index: usize) -> ExcelCellStyle {
        let xf = match self.cell_xfs.get(style_index) {
            Some(xf) => xf,
            None => return ExcelCellStyle::default(),
        };
        let mut style = ExcelCellStyle::default();

        if let Some(font) = self.fonts.get(xf.font_id) {
            style.font_name = font.name.clone();
            style.font_size = font.size;
            style.bold = font.bold;
            style.italic = font.italic;
            style.underline = font.underline;
            style.strikethrough = font.strikethrough;
            style.font_color = font.color.as_ref().and_then(|c| self.resolve_color(c));
        }
        if let Some(fill) = self.fills.get(xf.fill_id) {
            // 仅 patternType="solid" 时才应用填充色
            let is_solid = fill.pattern_type.as_deref().is_some_and(|t| t == "solid");
            if is_solid {
                style.bg_color = fill.fg_color.as_ref().and_then(|c| self.resolve_color(c));
            }
        }
        if let Some(border) = self.borders.get(xf.border_id) {
            style.border_top = border.top.as_ref().map(|b| self.resolve_border(b));
            style.border_bottom = border.bottom.as_ref().map(|b| self.resolve_border(b));
            style.border_left = border.left.as_ref().map(|b| self.resolve_border(b));
            style.border_right = border.right.as_ref().map(|b| self.resolve_border(b));
        }
        style.h_align = xf.alignment.horizontal.clone();
        style.v_align = xf.alignment.vertical.clone();
        style.wrap_text = xf.alignment.wrap_text;
        style.number_format = self.num_fmts.get(&xf.num_fmt_id).cloned();
        style
    }

    fn resolve_color(&self, color: &ColorRef) -> Option<String> {
        match color {
            ColorRef::Rgb(rgb) => Some(normalize_color(rgb)),
            ColorRef::Theme(idx, tint) => {
                let base = self.theme_colors.get(*idx)?;
                if *tint == 0.0 {
                    Some(base.clone())
                } else {
                    Some(apply_tint(base, *tint))
                }
            }
            ColorRef::Indexed(idx) => indexed_color(*idx).map(|s| s.to_string()),
        }
    }

    fn resolve_border(&self, raw: &RawBorderDef) -> BorderDef {
        BorderDef {
            style: raw.style.clone(),
            color: raw.color.as_ref().and_then(|c| self.resolve_color(c)),
        }
    }

    /// 样式索引数量
    pub fn xf_count(&self) -> usize {
        self.cell_xfs.len()
    }

    /// 获取差异格式样式（条件格式使用）
    pub fn get_dxf_style(&self, dxf_id: usize) -> Option<&ExcelCellStyle> {
        self.dxfs.get(dxf_id)
    }
}

/// 处理 XML Start/Empty 元素（抽取为函数解决借用问题）
#[allow(clippy::too_many_arguments)]
fn handle_start_element(
    local: &str,
    e: &quick_xml::events::BytesStart,
    section: &mut Section,
    current_font: &mut Option<FontDef>,
    current_fill: &mut Option<FillDef>,
    current_border: &mut Option<BorderSetDef>,
    current_xf: &mut Option<CellXf>,
    in_pattern_fill: &mut bool,
    current_border_side: &mut Option<String>,
    current_border_style: &mut Option<String>,
    num_fmts: &mut HashMap<usize, String>,
) {
    match local {
        "fonts" => *section = Section::Fonts,
        "fills" => *section = Section::Fills,
        "borders" => *section = Section::Borders,
        "numFmts" => *section = Section::NumFmts,
        "cellXfs" => *section = Section::CellXfs,

        "font" if *section == Section::Fonts => {
            *current_font = Some(FontDef::default());
        }
        "b" if current_font.is_some() => {
            if let Some(f) = current_font {
                let val = get_attr(e, "val");
                f.bold = val.as_deref().is_none_or(|v| v != "0" && v != "false");
            }
        }
        "i" if current_font.is_some() => {
            if let Some(f) = current_font {
                let val = get_attr(e, "val");
                f.italic = val.as_deref().is_none_or(|v| v != "0" && v != "false");
            }
        }
        "u" if current_font.is_some() => {
            if let Some(f) = current_font {
                f.underline = true;
            }
        }
        "strike" if current_font.is_some() => {
            if let Some(f) = current_font {
                let val = get_attr(e, "val");
                f.strikethrough = val.as_deref().is_none_or(|v| v != "0" && v != "false");
            }
        }
        "sz" if current_font.is_some() => {
            if let Some(f) = current_font {
                f.size = get_attr(e, "val").and_then(|v| v.parse::<f64>().ok());
            }
        }
        "name" if current_font.is_some() => {
            if let Some(f) = current_font {
                f.name = get_attr(e, "val");
            }
        }
        "color" if current_font.is_some() => {
            if let Some(f) = current_font {
                f.color = parse_color_ref(e);
            }
        }

        "fill" if *section == Section::Fills => {
            *current_fill = Some(FillDef::default());
            *in_pattern_fill = false;
        }
        "patternFill" if current_fill.is_some() => {
            *in_pattern_fill = true;
            // 记录 patternType 属性（仅 "solid" 才应用填充色）
            if let Some(f) = current_fill {
                f.pattern_type = get_attr(e, "patternType");
            }
        }
        "fgColor" if *in_pattern_fill && current_fill.is_some() => {
            if let Some(f) = current_fill {
                f.fg_color = parse_color_ref(e);
            }
        }

        "border" if *section == Section::Borders => {
            *current_border = Some(BorderSetDef::default());
        }
        "left" | "right" | "top" | "bottom"
            if *section == Section::Borders && current_border.is_some() =>
        {
            *current_border_side = Some(local.to_string());
            *current_border_style = get_attr(e, "style");
        }
        "color"
            if *section == Section::Borders
                && current_border.is_some()
                && current_border_side.is_some() =>
        {
            let color_ref = parse_color_ref(e);
            if let Some(border) = current_border {
                if let Some(side) = current_border_side {
                    if let Some(style) = current_border_style {
                        let raw = RawBorderDef {
                            style: style.clone(),
                            color: color_ref,
                        };
                        match side.as_str() {
                            "left" => border.left = Some(raw),
                            "right" => border.right = Some(raw),
                            "top" => border.top = Some(raw),
                            "bottom" => border.bottom = Some(raw),
                            _ => {}
                        }
                    }
                }
            }
        }

        "numFmt" if *section == Section::NumFmts => {
            if let (Some(id_str), Some(code)) = (get_attr(e, "numFmtId"), get_attr(e, "formatCode"))
            {
                if let Ok(id) = id_str.parse::<usize>() {
                    num_fmts.insert(id, code);
                }
            }
        }

        "xf" if *section == Section::CellXfs => {
            let xf = CellXf {
                font_id: get_attr(e, "fontId")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                fill_id: get_attr(e, "fillId")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                border_id: get_attr(e, "borderId")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                num_fmt_id: get_attr(e, "numFmtId")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(0),
                ..Default::default()
            };
            *current_xf = Some(xf);
        }
        "alignment" if current_xf.is_some() => {
            if let Some(xf) = current_xf {
                xf.alignment.horizontal = get_attr(e, "horizontal");
                xf.alignment.vertical = get_attr(e, "vertical");
                xf.alignment.wrap_text = get_attr(e, "wrapText")
                    .as_deref()
                    .is_some_and(|v| v == "1" || v == "true");
            }
        }
        _ => {}
    }
}

// 需要顶层可见的 Section 枚举
#[derive(PartialEq)]
enum Section {
    None,
    Fonts,
    Fills,
    Borders,
    NumFmts,
    CellXfs,
    Dxfs,
}

/// dxf 解析中间状态（差异格式内联了 font/fill/border）
#[derive(Debug, Default)]
struct DxfParseState {
    font_color: Option<ColorRef>,
    bold: bool,
    italic: bool,
    bg_color: Option<ColorRef>,
    in_font: bool,
    in_fill: bool,
}

impl DxfParseState {
    /// 将解析状态转为最终样式（颜色需要后续解析为 RGB）
    fn into_style(self) -> ExcelCellStyle {
        ExcelCellStyle {
            font_color: self.font_color.and_then(|c| resolve_color_standalone(&c)),
            bold: self.bold,
            italic: self.italic,
            bg_color: self.bg_color.and_then(|c| resolve_color_standalone(&c)),
            ..Default::default()
        }
    }
}

/// 独立的颜色解析（dxf 中不依赖主题色，直接使用 RGB）
fn resolve_color_standalone(color: &ColorRef) -> Option<String> {
    match color {
        ColorRef::Rgb(rgb) => Some(normalize_color(rgb)),
        ColorRef::Indexed(idx) => indexed_color(*idx).map(|s| s.to_string()),
        // dxf 中的主题色暂不支持 tint 计算，使用默认主题色
        ColorRef::Theme(idx, _) => DEFAULT_THEME_COLORS.get(*idx).map(|s| (*s).to_string()),
    }
}

/// 处理 dxf 内部元素
fn handle_dxf_start(
    local: &str,
    e: &quick_xml::events::BytesStart,
    state: &mut Option<DxfParseState>,
) {
    match local {
        "dxf" => {
            *state = Some(DxfParseState::default());
        }
        "font" if state.is_some() => {
            if let Some(s) = state.as_mut() {
                s.in_font = true;
            }
        }
        "fill" if state.is_some() => {
            if let Some(s) = state.as_mut() {
                s.in_fill = true;
            }
        }
        "b" if state.as_ref().is_some_and(|s| s.in_font) => {
            if let Some(s) = state.as_mut() {
                // <b/> 或 <b val="1"/> 表示加粗
                let val = get_attr(e, "val");
                s.bold = val.as_deref() != Some("0");
            }
        }
        "i" if state.as_ref().is_some_and(|s| s.in_font) => {
            if let Some(s) = state.as_mut() {
                let val = get_attr(e, "val");
                s.italic = val.as_deref() != Some("0");
            }
        }
        "color" if state.as_ref().is_some_and(|s| s.in_font) => {
            if let Some(s) = state.as_mut() {
                s.font_color = parse_color_ref(e);
            }
        }
        "bgColor" if state.as_ref().is_some_and(|s| s.in_fill) => {
            if let Some(s) = state.as_mut() {
                s.bg_color = parse_color_ref(e);
            }
        }
        "fgColor" if state.as_ref().is_some_and(|s| s.in_fill) => {
            // 某些文件用 fgColor 而非 bgColor
            if let Some(s) = state.as_mut() {
                if s.bg_color.is_none() {
                    s.bg_color = parse_color_ref(e);
                }
            }
        }
        _ => {}
    }
}

/// 从 XML 元素解析颜色引用
fn parse_color_ref(e: &quick_xml::events::BytesStart) -> Option<ColorRef> {
    if let Some(rgb) = get_attr(e, "rgb") {
        Some(ColorRef::Rgb(rgb))
    } else if let Some(theme) = get_attr(e, "theme").and_then(|v| v.parse::<usize>().ok()) {
        let tint = get_attr(e, "tint")
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        Some(ColorRef::Theme(theme, tint))
    } else {
        get_attr(e, "indexed")
            .and_then(|v| v.parse::<usize>().ok())
            .map(ColorRef::Indexed)
    }
}

/// 将 ExcelCellStyle 转换为 CSS 内联样式字符串
pub fn cell_style_to_css(style: &ExcelCellStyle) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(ref name) = style.font_name {
        let safe_name = sanitize_css_value(name);
        parts.push(format!("font-family:{safe_name}"));
    }
    if let Some(size) = style.font_size {
        parts.push(format!("font-size:{size}pt"));
    }
    if style.bold {
        parts.push("font-weight:bold".to_string());
    }
    if style.italic {
        parts.push("font-style:italic".to_string());
    }

    let mut decorations = Vec::new();
    if style.underline {
        decorations.push("underline");
    }
    if style.strikethrough {
        decorations.push("line-through");
    }
    if !decorations.is_empty() {
        parts.push(format!("text-decoration:{}", decorations.join(" ")));
    }

    if let Some(ref color) = style.font_color {
        parts.push(format!("color:{}", sanitize_hex_color(color)));
    }
    if let Some(ref bg) = style.bg_color {
        parts.push(format!("background-color:{}", sanitize_hex_color(bg)));
    }

    if let Some(ref align) = style.h_align {
        let css = match align.as_str() {
            "center" | "centerContinuous" => "center",
            "right" => "right",
            "justify" | "distributed" => "justify",
            _ => "left",
        };
        parts.push(format!("text-align:{css}"));
    }
    if let Some(ref align) = style.v_align {
        let css = match align.as_str() {
            "center" => "middle",
            "top" => "top",
            _ => "bottom",
        };
        parts.push(format!("vertical-align:{css}"));
    }

    if let Some(ref b) = style.border_top {
        parts.push(format!("border-top:{}", border_to_css(b)));
    }
    if let Some(ref b) = style.border_bottom {
        parts.push(format!("border-bottom:{}", border_to_css(b)));
    }
    if let Some(ref b) = style.border_left {
        parts.push(format!("border-left:{}", border_to_css(b)));
    }
    if let Some(ref b) = style.border_right {
        parts.push(format!("border-right:{}", border_to_css(b)));
    }

    parts.join(";")
}

fn border_to_css(border: &BorderDef) -> String {
    let css_style = match border.style.as_str() {
        "thin" => "1px solid",
        "medium" => "2px solid",
        "thick" => "3px solid",
        "dotted" => "1px dotted",
        "dashed" => "1px dashed",
        "double" => "3px double",
        "hair" => "0.5px solid",
        "mediumDashed" => "2px dashed",
        "dashDot" | "slantDashDot" => "1px dashed",
        "mediumDashDot" => "2px dashed",
        "dashDotDot" | "mediumDashDotDot" => "1px dotted",
        _ => "1px solid",
    };
    let color = border.color.as_deref().unwrap_or("#000000");
    // 校验边框颜色值合法性，防止 CSS 注入
    let safe_color = sanitize_hex_color(color);
    format!("{css_style} {safe_color}")
}

/// 应用 tint 色调偏移
pub fn apply_tint(hex: &str, tint: f64) -> String {
    let hex_str = hex.trim_start_matches('#');
    let (r, g, b) = match parse_hex_rgb(hex_str) {
        Some(rgb) => rgb,
        None => return format!("#{hex_str}"),
    };

    let (r, g, b) = if tint > 0.0 {
        let r = (r as f64 + (255.0 - r as f64) * tint).clamp(0.0, 255.0);
        let g = (g as f64 + (255.0 - g as f64) * tint).clamp(0.0, 255.0);
        let b = (b as f64 + (255.0 - b as f64) * tint).clamp(0.0, 255.0);
        (r.round() as u8, g.round() as u8, b.round() as u8)
    } else {
        let factor = 1.0 + tint;
        (
            (r as f64 * factor).clamp(0.0, 255.0).round() as u8,
            (g as f64 * factor).clamp(0.0, 255.0).round() as u8,
            (b as f64 * factor).clamp(0.0, 255.0).round() as u8,
        )
    };

    format!("#{r:02X}{g:02X}{b:02X}")
}

fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    let rgb = if hex.len() == 8 {
        &hex[2..]
    } else if hex.len() == 6 {
        hex
    } else {
        return None;
    };
    let r = u8::from_str_radix(&rgb[0..2], 16).ok()?;
    let g = u8::from_str_radix(&rgb[2..4], 16).ok()?;
    let b = u8::from_str_radix(&rgb[4..6], 16).ok()?;
    Some((r, g, b))
}

fn normalize_color(color: &str) -> String {
    let c = color.trim_start_matches('#');
    // ARGB 格式：取后 6 位 RGB
    let rgb = if c.len() == 8 { &c[2..] } else { c };
    // 校验是否为合法十六进制颜色，防止 CSS 注入
    if (rgb.len() == 6 || rgb.len() == 3) && rgb.chars().all(|ch| ch.is_ascii_hexdigit()) {
        format!("#{rgb}")
    } else {
        // 非法值回退为黑色
        "#000000".to_string()
    }
}

/// 校验十六进制颜色值合法性，防止 CSS 注入
///
/// 仅允许 `#RGB` 或 `#RRGGBB` 格式，非法值回退为默认值。
fn sanitize_hex_color(color: &str) -> &str {
    let c = color.trim_start_matches('#');
    if (c.len() == 6 || c.len() == 3) && c.chars().all(|ch| ch.is_ascii_hexdigit()) {
        color
    } else {
        "#000000"
    }
}

fn sanitize_css_value(value: &str) -> String {
    let lower = value.to_lowercase();
    if lower.contains("expression")
        || lower.contains("url(")
        || lower.contains("javascript:")
        || lower.contains("import")
        || lower.contains("\\")
    {
        return "sans-serif".to_string();
    }
    value.to_string()
}

fn indexed_color(index: usize) -> Option<&'static str> {
    const INDEXED_COLORS: &[&str] = &[
        "#000000", "#FFFFFF", "#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF", "#00FFFF",
        "#000000", "#FFFFFF", "#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF", "#00FFFF",
        "#800000", "#008000", "#000080", "#808000", "#800080", "#008080", "#C0C0C0", "#808080",
        "#9999FF", "#993366", "#FFFFCC", "#CCFFFF", "#660066", "#FF8080", "#0066CC", "#CCCCFF",
        "#000080", "#FF00FF", "#FFFF00", "#00FFFF", "#800080", "#800000", "#008080", "#0000FF",
        "#00CCFF", "#CCFFFF", "#CCFFCC", "#FFFF99", "#99CCFF", "#FF99CC", "#CC99FF", "#FFCC99",
        "#3366FF", "#33CCCC", "#99CC00", "#FFCC00", "#FF9900", "#FF6600", "#666699", "#969696",
        "#003366", "#339966", "#003300", "#333300", "#993300", "#993366", "#333399", "#333333",
    ];
    INDEXED_COLORS.get(index).copied()
}

fn get_attr(event: &quick_xml::events::BytesStart, name: &str) -> Option<String> {
    for attr in event.attributes().flatten() {
        if attr.key.as_ref() == name.as_bytes() {
            return String::from_utf8(attr.value.to_vec()).ok();
        }
    }
    None
}

fn local_name(full_name: &[u8]) -> &str {
    let name = std::str::from_utf8(full_name).unwrap_or("");
    name.rsplit_once(':').map_or(name, |(_, local)| local)
}

/// 格式化数字值（动态解析格式字符串）
pub fn format_number(value: f64, format_str: &str) -> String {
    let fmt = format_str.trim();

    // General / 空格式
    if fmt.is_empty() || fmt.eq_ignore_ascii_case("General") {
        return format_general(value);
    }

    // 处理分号分隔的正/负/零格式（如 "0.00;-0.00;0"）
    let section = select_format_section(fmt, value);

    // 去除颜色标记（如 [Red]、[Green]）
    let clean = strip_color_markers(section);

    // 日期/时间格式 - 此处不做日期序列号转换，直接返回智能格式
    if is_date_format(&clean) {
        return format_general(value);
    }

    // 百分比格式
    if clean.contains('%') {
        let pct = value * 100.0;
        let decimals = count_decimal_places(&clean);
        let has_thousands = clean.contains(',');
        return if has_thousands {
            format!("{}%", format_with_thousands(pct, decimals))
        } else {
            format!("{pct:.prec$}%", prec = decimals)
        };
    }

    // 科学计数法
    if clean.contains("E+") || clean.contains("E-") || clean.contains("e+") || clean.contains("e-")
    {
        let decimals = count_decimal_places(&clean);
        return format!("{value:.prec$E}", prec = decimals);
    }

    // 带千位分隔符
    if clean.contains(',') {
        let decimals = count_decimal_places(&clean);
        return format_with_thousands(value, decimals);
    }

    // 固定小数位格式（如 "0", "0.00", "0.0", "0.000", "#.##" 等）
    if clean.contains('.') {
        let decimals = count_decimal_places(&clean);
        return format!("{value:.prec$}", prec = decimals);
    }

    // 纯整数格式（如 "0", "#"）
    if clean.contains('0') || clean.contains('#') {
        // 超出 i64 精确表示范围时回退到通用格式
        if value.abs() < 1e18 {
            return format!("{}", value.round() as i64);
        }
        return format_general(value);
    }

    // 未知格式，使用智能默认
    format_general(value)
}

/// General 格式的智能格式化（限制精度，去除浮点噪声）
fn format_general(value: f64) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    if value == value.floor() && value.abs() < 1e15 {
        return format!("{}", value as i64);
    }
    // 非常大或非常小的数用科学计数法
    if value.abs() >= 1e11 || value.abs() < 1e-9 {
        let s = format!("{value:.6E}");
        // 去除尾部零
        if let Some((mantissa, exp)) = s.split_once('E') {
            let clean = mantissa.trim_end_matches('0').trim_end_matches('.');
            return format!("{clean}E{exp}");
        }
        return s;
    }
    // 普通数字：10 位小数精度，去除尾部零
    let s = format!("{value:.10}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    s.to_string()
}

/// 处理正/负/零格式分段（如 "0.00;-0.00;0"）
fn select_format_section(fmt: &str, value: f64) -> &str {
    let sections: Vec<&str> = fmt.split(';').collect();
    match sections.len() {
        1 => sections[0],
        2 => {
            if value >= 0.0 {
                sections[0]
            } else {
                sections[1]
            }
        }
        _ => {
            if value > 0.0 {
                sections[0]
            } else if value < 0.0 {
                sections[1]
            } else {
                sections.get(2).copied().unwrap_or(sections[0])
            }
        }
    }
}

/// 统计格式字符串中小数位数
fn count_decimal_places(fmt: &str) -> usize {
    if let Some(dot_pos) = fmt.find('.') {
        fmt[dot_pos + 1..]
            .chars()
            .take_while(|c| *c == '0' || *c == '#' || *c == '?')
            .count()
    } else {
        0
    }
}

/// 去除格式字符串中的颜色标记（如 [Red]、[Green]、[Color1]）
///
/// 仅移除已知的颜色标记，保留条件格式、区域设置等方括号内容。
fn strip_color_markers(fmt: &str) -> String {
    /// 判断方括号内的内容是否为已知颜色标记
    fn is_color_marker(content: &str) -> bool {
        let lower = content.to_ascii_lowercase();
        matches!(
            lower.as_str(),
            "red" | "green" | "blue" | "yellow" | "cyan" | "magenta" | "white" | "black"
        ) || lower.starts_with("color") && lower["color".len()..].trim().parse::<u32>().is_ok()
    }

    let mut result = String::with_capacity(fmt.len());
    let mut chars = fmt.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            // 收集方括号内的内容
            let mut bracket_content = String::new();
            for inner in chars.by_ref() {
                if inner == ']' {
                    break;
                }
                bracket_content.push(inner);
            }
            // 仅移除颜色标记，保留其他方括号内容
            if !is_color_marker(&bracket_content) {
                result.push('[');
                result.push_str(&bracket_content);
                result.push(']');
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// 判断是否为日期/时间格式
fn is_date_format(fmt: &str) -> bool {
    let lower = fmt.to_ascii_lowercase();
    lower.contains("yy")
        || lower.contains("mm")
        || lower.contains("dd")
        || lower.contains("hh")
        || lower.contains("ss")
        || lower.contains("am/pm")
}

fn format_with_thousands(value: f64, decimals: usize) -> String {
    let is_negative = value < 0.0;
    let abs_val = value.abs();
    let formatted = if decimals > 0 {
        format!("{abs_val:.prec$}", prec = decimals)
    } else if abs_val < 1e18 {
        format!("{}", abs_val.round() as i64)
    } else {
        // 超大值回退到通用格式
        return format_general(value);
    };
    let parts: Vec<&str> = formatted.splitn(2, '.').collect();
    let int_part = parts[0];
    let dec_part = parts.get(1);
    let chars: Vec<char> = int_part.chars().collect();
    let mut result = String::new();
    for (i, ch) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(*ch);
    }
    if let Some(dec) = dec_part {
        result.push('.');
        result.push_str(dec);
    }
    if is_negative {
        format!("-{result}")
    } else {
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_tint_positive() {
        let result = apply_tint("#4472C4", 0.5);
        assert!(result.starts_with('#'));
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn test_apply_tint_negative() {
        let result = apply_tint("#4472C4", -0.5);
        assert!(result.starts_with('#'));
        assert_eq!(result.len(), 7);
    }

    #[test]
    fn test_apply_tint_zero() {
        assert_eq!(apply_tint("#4472C4", 0.0), "#4472C4");
    }

    #[test]
    fn test_normalize_color_argb() {
        assert_eq!(normalize_color("FF4472C4"), "#4472C4");
    }

    #[test]
    fn test_format_number_general() {
        assert_eq!(format_number(42.0, "General"), "42");
        assert_eq!(format_number(3.14, "General"), "3.14");
    }

    #[test]
    fn test_format_number_percentage() {
        assert_eq!(format_number(0.5, "0%"), "50%");
        assert_eq!(format_number(0.1234, "0.00%"), "12.34%");
    }

    #[test]
    fn test_format_number_thousands() {
        assert_eq!(format_with_thousands(1234567.89, 2), "1,234,567.89");
        assert_eq!(format_with_thousands(1000.0, 0), "1,000");
    }

    #[test]
    fn test_border_to_css() {
        let border = BorderDef {
            style: "thin".to_string(),
            color: Some("#000000".to_string()),
        };
        assert_eq!(border_to_css(&border), "1px solid #000000");
    }

    #[test]
    fn test_cell_style_to_css() {
        let style = ExcelCellStyle {
            bold: true,
            italic: true,
            font_size: Some(12.0),
            ..Default::default()
        };
        let css = cell_style_to_css(&style);
        assert!(css.contains("font-weight:bold"));
        assert!(css.contains("font-style:italic"));
        assert!(css.contains("font-size:12pt"));
    }

    #[test]
    fn test_sanitize_css_value() {
        assert_eq!(sanitize_css_value("Arial"), "Arial");
        assert_eq!(sanitize_css_value("expression(evil)"), "sans-serif");
        assert_eq!(sanitize_css_value("url(http://evil)"), "sans-serif");
    }

    #[test]
    fn test_indexed_color() {
        assert_eq!(indexed_color(0), Some("#000000"));
        assert_eq!(indexed_color(1), Some("#FFFFFF"));
        assert_eq!(indexed_color(999), None);
    }

    #[test]
    fn test_parse_hex_rgb() {
        assert_eq!(parse_hex_rgb("FF4472C4"), Some((0x44, 0x72, 0xC4)));
        assert_eq!(parse_hex_rgb("4472C4"), Some((0x44, 0x72, 0xC4)));
        assert_eq!(parse_hex_rgb("123"), None);
    }

    #[test]
    fn test_self_closing_xf_elements_are_collected() {
        // 自闭合 <xf .../> 元素必须正确收集，不能丢失导致索引偏移
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <numFmts count="0"/>
  <fonts count="1"><font><sz val="11"/></font></fonts>
  <fills count="1"><fill><patternFill patternType="none"/></fill></fills>
  <borders count="1"><border><left/><right/><top/><bottom/></border></borders>
  <cellXfs count="4">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0"><alignment horizontal="center"/></xf>
    <xf numFmtId="9" fontId="0" fillId="0" borderId="0"/>
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0"><alignment wrapText="1"/></xf>
    <xf numFmtId="10" fontId="0" fillId="0" borderId="0"/>
  </cellXfs>
</styleSheet>"#;
        let ss = ExcelStyleSheet::parse_styles_xml(xml).unwrap();
        assert_eq!(ss.xf_count(), 4, "应正确收集 4 个 xf（含自闭合）");
        // xf[0] General，xf[1] 0%（自闭合），xf[2] General，xf[3] 0.00%（自闭合）
        let s0 = ss.get_cell_style(0);
        assert_eq!(s0.number_format.as_deref(), Some("General"));
        let s1 = ss.get_cell_style(1);
        assert_eq!(s1.number_format.as_deref(), Some("0%"));
        let s2 = ss.get_cell_style(2);
        assert_eq!(s2.number_format.as_deref(), Some("General"));
        let s3 = ss.get_cell_style(3);
        assert_eq!(s3.number_format.as_deref(), Some("0.00%"));
    }

    #[test]
    fn test_demo_xlsx_style_count() {
        // 验证 demo.xlsx 的 337 个 cellXfs 全部正确解析（含 6 个自闭合元素）
        let data = std::fs::read("demo.xlsx");
        if let Ok(data) = data {
            let cursor = std::io::Cursor::new(&data);
            let ss = ExcelStyleSheet::from_xlsx_zip(cursor).unwrap();
            assert_eq!(ss.xf_count(), 337, "demo.xlsx 应有 337 个 cellXfs");

            // 验证关键样式索引的数字格式正确
            // style 96: numFmtId=0 (General) — 普通数字，不是百分比
            let s96 = ss.get_cell_style(96);
            assert_eq!(s96.number_format.as_deref(), Some("General"));
            // style 97: numFmtId=9 (0%) — 百分比
            let s97 = ss.get_cell_style(97);
            assert_eq!(s97.number_format.as_deref(), Some("0%"));
            // style 108: numFmtId=10 (0.00%) — 带小数的百分比
            let s108 = ss.get_cell_style(108);
            assert_eq!(s108.number_format.as_deref(), Some("0.00%"));
        }
    }

    #[test]
    fn test_dxf_parsing_basic() {
        let styles_xml = r#"<?xml version="1.0"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dxfs count="2">
    <dxf>
      <font><color rgb="FF9C0006"/></font>
      <fill><patternFill patternType="solid"><bgColor rgb="FFFFC7CE"/></patternFill></fill>
    </dxf>
    <dxf>
      <font><color rgb="FF006100"/></font>
      <fill><patternFill patternType="solid"><bgColor rgb="FFC6EFCE"/></patternFill></fill>
    </dxf>
  </dxfs>
</styleSheet>"#;
        let ss = ExcelStyleSheet::parse_styles_xml(styles_xml).unwrap();
        // 验证 dxf 解析
        let dxf0 = ss.get_dxf_style(0).expect("dxf[0] 应存在");
        assert_eq!(dxf0.font_color.as_deref(), Some("#9C0006"));
        assert_eq!(dxf0.bg_color.as_deref(), Some("#FFC7CE"));
        let dxf1 = ss.get_dxf_style(1).expect("dxf[1] 应存在");
        assert_eq!(dxf1.font_color.as_deref(), Some("#006100"));
        assert_eq!(dxf1.bg_color.as_deref(), Some("#C6EFCE"));
        // 不存在的索引
        assert!(ss.get_dxf_style(2).is_none());
    }

    #[test]
    fn test_dxf_to_css() {
        let styles_xml = r#"<?xml version="1.0"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dxfs count="1">
    <dxf>
      <font><color rgb="FF9C0006"/><b/></font>
      <fill><patternFill patternType="solid"><bgColor rgb="FFFFC7CE"/></patternFill></fill>
    </dxf>
  </dxfs>
</styleSheet>"#;
        let ss = ExcelStyleSheet::parse_styles_xml(styles_xml).unwrap();
        let dxf0 = ss.get_dxf_style(0).unwrap();
        let css = cell_style_to_css(dxf0);
        assert!(css.contains("color:#9C0006"), "应包含字体颜色, 实际: {css}");
        assert!(
            css.contains("background-color:#FFC7CE"),
            "应包含背景颜色, 实际: {css}"
        );
        assert!(css.contains("font-weight:bold"), "应包含加粗, 实际: {css}");
    }
}
