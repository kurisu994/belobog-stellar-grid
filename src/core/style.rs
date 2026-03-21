/// Excel 单元格样式模块
///
/// 提供三级样式体系：全局样式 → 列级样式 → 单元格样式
/// 支持字体、颜色、边框、背景色、对齐等常用样式属性
use rust_xlsxwriter::{Format, FormatAlign, FormatBorder};
use std::collections::HashMap;

/// 水平对齐方式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HAlign {
    /// 左对齐
    Left,
    /// 居中
    Center,
    /// 右对齐
    Right,
}

/// 垂直对齐方式
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VAlign {
    /// 顶部对齐
    Top,
    /// 居中
    Center,
    /// 底部对齐
    Bottom,
}

/// 边框线条类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorderLine {
    /// 细线
    Thin,
    /// 中等
    Medium,
    /// 粗线
    Thick,
    /// 虚线
    Dashed,
    /// 点线
    Dotted,
    /// 双线
    Double,
}

impl BorderLine {
    /// 转换为 rust_xlsxwriter 的 FormatBorder
    fn to_format_border(&self) -> FormatBorder {
        match self {
            BorderLine::Thin => FormatBorder::Thin,
            BorderLine::Medium => FormatBorder::Medium,
            BorderLine::Thick => FormatBorder::Thick,
            BorderLine::Dashed => FormatBorder::Dashed,
            BorderLine::Dotted => FormatBorder::Dotted,
            BorderLine::Double => FormatBorder::Double,
        }
    }

    /// 从字符串解析边框线条类型（兼容方法，内部委托给 FromStr）
    pub fn from_str(s: &str) -> Option<Self> {
        s.parse().ok()
    }
}

impl std::str::FromStr for BorderLine {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "thin" => Ok(BorderLine::Thin),
            "medium" => Ok(BorderLine::Medium),
            "thick" => Ok(BorderLine::Thick),
            "dashed" => Ok(BorderLine::Dashed),
            "dotted" => Ok(BorderLine::Dotted),
            "double" => Ok(BorderLine::Double),
            _ => Err(()),
        }
    }
}

/// 边框样式配置
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorderConfig {
    /// 四边统一细线边框
    All,
    /// 分别指定四边
    Individual {
        top: Option<BorderLine>,
        bottom: Option<BorderLine>,
        left: Option<BorderLine>,
        right: Option<BorderLine>,
    },
}

/// 单元格样式定义
///
/// 所有字段均为 Option，未设置的属性不会覆盖更低优先级的样式
#[derive(Debug, Clone, Default)]
pub struct CellStyle {
    /// 粗体
    pub bold: Option<bool>,
    /// 斜体
    pub italic: Option<bool>,
    /// 字号
    pub font_size: Option<f64>,
    /// 字体名称
    pub font_name: Option<String>,
    /// 字体颜色 (hex: "#RRGGBB" 或 "#RGB")
    pub font_color: Option<String>,
    /// 背景色 (hex: "#RRGGBB" 或 "#RGB")
    pub background_color: Option<String>,
    /// 水平对齐
    pub align: Option<HAlign>,
    /// 垂直对齐
    pub vertical_align: Option<VAlign>,
    /// 边框
    pub border: Option<BorderConfig>,
    /// 数字格式 (如 "#,##0.00")
    pub number_format: Option<String>,
    /// 自动换行
    pub text_wrap: Option<bool>,
}

impl CellStyle {
    /// 合并两个样式：`other` 中已设置的属性覆盖 `self` 中的属性
    pub fn merge(&self, other: &CellStyle) -> CellStyle {
        CellStyle {
            bold: other.bold.or(self.bold),
            italic: other.italic.or(self.italic),
            font_size: other.font_size.or(self.font_size),
            font_name: other.font_name.clone().or_else(|| self.font_name.clone()),
            font_color: other.font_color.clone().or_else(|| self.font_color.clone()),
            background_color: other
                .background_color
                .clone()
                .or_else(|| self.background_color.clone()),
            align: other.align.clone().or_else(|| self.align.clone()),
            vertical_align: other
                .vertical_align
                .clone()
                .or_else(|| self.vertical_align.clone()),
            border: other.border.clone().or_else(|| self.border.clone()),
            number_format: other
                .number_format
                .clone()
                .or_else(|| self.number_format.clone()),
            text_wrap: other.text_wrap.or(self.text_wrap),
        }
    }

    /// 检查样式是否全部为空（无任何属性设置）
    pub fn is_empty(&self) -> bool {
        self.bold.is_none()
            && self.italic.is_none()
            && self.font_size.is_none()
            && self.font_name.is_none()
            && self.font_color.is_none()
            && self.background_color.is_none()
            && self.align.is_none()
            && self.vertical_align.is_none()
            && self.border.is_none()
            && self.number_format.is_none()
            && self.text_wrap.is_none()
    }

    /// 转换为 rust_xlsxwriter 的 Format 对象
    pub fn to_format(&self) -> Format {
        let mut format = Format::new();

        if let Some(true) = self.bold {
            format = format.set_bold();
        }

        if let Some(true) = self.italic {
            format = format.set_italic();
        }

        if let Some(size) = self.font_size {
            format = format.set_font_size(size);
        }

        if let Some(ref name) = self.font_name {
            format = format.set_font_name(name);
        }

        if let Some(ref color) = self.font_color {
            let normalized = normalize_hex_color(color);
            format = format.set_font_color(normalized.as_str());
        }

        if let Some(ref color) = self.background_color {
            let normalized = normalize_hex_color(color);
            format = format.set_background_color(normalized.as_str());
        }

        if let Some(ref align) = self.align {
            format = match align {
                HAlign::Left => format.set_align(FormatAlign::Left),
                HAlign::Center => format.set_align(FormatAlign::Center),
                HAlign::Right => format.set_align(FormatAlign::Right),
            };
        }

        if let Some(ref valign) = self.vertical_align {
            format = match valign {
                VAlign::Top => format.set_align(FormatAlign::Top),
                VAlign::Center => format.set_align(FormatAlign::VerticalCenter),
                VAlign::Bottom => format.set_align(FormatAlign::Bottom),
            };
        }

        if let Some(ref border) = self.border {
            match border {
                BorderConfig::All => {
                    format = format.set_border(FormatBorder::Thin);
                }
                BorderConfig::Individual {
                    top,
                    bottom,
                    left,
                    right,
                } => {
                    if let Some(line) = top {
                        format = format.set_border_top(line.to_format_border());
                    }
                    if let Some(line) = bottom {
                        format = format.set_border_bottom(line.to_format_border());
                    }
                    if let Some(line) = left {
                        format = format.set_border_left(line.to_format_border());
                    }
                    if let Some(line) = right {
                        format = format.set_border_right(line.to_format_border());
                    }
                }
            }
        }

        if let Some(ref fmt) = self.number_format {
            format = format.set_num_format(fmt);
        }

        if let Some(true) = self.text_wrap {
            format = format.set_text_wrap();
        }

        format
    }
}

/// 样式表，汇总一个 TableData 的所有样式信息
///
/// 解析优先级：全局 → 列级 → 单元格级
#[derive(Debug, Clone, Default)]
pub struct StyleSheet {
    /// 表头行默认样式
    pub header_style: Option<CellStyle>,
    /// 数据行默认样式
    pub data_style: Option<CellStyle>,
    /// 列级数据样式（索引 = 列号）
    pub column_styles: Vec<Option<CellStyle>>,
    /// 列级表头样式（索引 = 列号）
    pub column_header_styles: Vec<Option<CellStyle>>,
    /// 单元格级样式覆盖 (row, col) → style
    pub cell_overrides: HashMap<(u32, u16), CellStyle>,
    /// 列宽配置（索引 = 列号）
    pub column_widths: Vec<Option<f64>>,
}

impl StyleSheet {
    /// 检查样式表是否全部为空（无任何样式配置）
    pub fn is_empty(&self) -> bool {
        self.header_style.is_none()
            && self.data_style.is_none()
            && self.column_styles.iter().all(|s| s.is_none())
            && self.column_header_styles.iter().all(|s| s.is_none())
            && self.cell_overrides.is_empty()
            && self.column_widths.iter().all(|w| w.is_none())
    }

    /// 解析指定位置 (row, col) 的最终合并样式，并转换为 Format
    ///
    /// 合并优先级：全局 → 列级 → 单元格级
    /// 返回 None 表示该位置无任何样式设置
    pub fn resolve(&self, row: u32, col: u16, header_row_count: usize) -> Option<Format> {
        let is_header = (row as usize) < header_row_count;

        // 第一级：全局样式
        let base = if is_header {
            self.header_style.as_ref()
        } else {
            self.data_style.as_ref()
        };

        // 第二级：列级样式
        let col_style = if is_header {
            self.column_header_styles
                .get(col as usize)
                .and_then(|s| s.as_ref())
        } else {
            self.column_styles
                .get(col as usize)
                .and_then(|s| s.as_ref())
        };

        // 第三级：单元格级覆盖
        let cell_style = self.cell_overrides.get(&(row, col));

        // 按优先级合并
        let merged = match (base, col_style, cell_style) {
            (None, None, None) => return None,
            (Some(s), None, None) | (None, Some(s), None) | (None, None, Some(s)) => s.clone(),
            (Some(base), Some(col), None) => base.merge(col),
            (Some(base), None, Some(cell)) => base.merge(cell),
            (None, Some(col), Some(cell)) => col.merge(cell),
            (Some(base), Some(col), Some(cell)) => {
                let intermediate = base.merge(col);
                intermediate.merge(cell)
            }
        };

        if merged.is_empty() {
            return None;
        }

        Some(merged.to_format())
    }
}

/// 将 3 位 hex 颜色扩展为 6 位，并确保以 "#" 开头
///
/// 例: "#F00" → "#FF0000", "ABC" → "#AABBCC"
pub fn normalize_hex_color(color: &str) -> String {
    let trimmed = color.trim().trim_start_matches('#');

    match trimmed.len() {
        3 => {
            // 扩展 3 位 → 6 位
            let chars: Vec<char> = trimmed.chars().collect();
            format!(
                "#{}{}{}{}{}{}",
                chars[0], chars[0], chars[1], chars[1], chars[2], chars[2]
            )
        }
        6 => format!("#{}", trimmed),
        _ => color.to_string(),
    }
}

/// 从 JsValue 解析单元格样式
///
/// 接受 JS 对象格式：
/// ```js
/// {
///   bold: true,
///   italic: false,
///   fontSize: 12,
///   fontName: "Arial",
///   fontColor: "#FF0000",
///   backgroundColor: "#F2F2F2",
///   align: "center",
///   verticalAlign: "center",
///   border: true, // 或 { top: "thin", bottom: "medium" }
///   numberFormat: "#,##0.00",
///   textWrap: true
/// }
/// ```
pub fn parse_cell_style(val: &wasm_bindgen::JsValue) -> Option<CellStyle> {
    if val.is_null() || val.is_undefined() || !val.is_object() {
        return None;
    }

    let bold = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("bold"))
        .ok()
        .and_then(|v| v.as_bool());

    let italic = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("italic"))
        .ok()
        .and_then(|v| v.as_bool());

    let font_size = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("fontSize"))
        .ok()
        .and_then(|v| v.as_f64());

    let font_name = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("fontName"))
        .ok()
        .and_then(|v| v.as_string());

    let font_color = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("fontColor"))
        .ok()
        .and_then(|v| v.as_string());

    let background_color =
        js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("backgroundColor"))
            .ok()
            .and_then(|v| v.as_string());

    let align = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("align"))
        .ok()
        .and_then(|v| v.as_string())
        .and_then(|s| match s.to_lowercase().as_str() {
            "left" => Some(HAlign::Left),
            "center" => Some(HAlign::Center),
            "right" => Some(HAlign::Right),
            _ => None,
        });

    let vertical_align =
        js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("verticalAlign"))
            .ok()
            .and_then(|v| v.as_string())
            .and_then(|s| match s.to_lowercase().as_str() {
                "top" => Some(VAlign::Top),
                "center" => Some(VAlign::Center),
                "bottom" => Some(VAlign::Bottom),
                _ => None,
            });

    let border = parse_border_config(val);

    let number_format = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("numberFormat"))
        .ok()
        .and_then(|v| v.as_string());

    let text_wrap = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("textWrap"))
        .ok()
        .and_then(|v| v.as_bool());

    let style = CellStyle {
        bold,
        italic,
        font_size,
        font_name,
        font_color,
        background_color,
        align,
        vertical_align,
        border,
        number_format,
        text_wrap,
    };

    if style.is_empty() { None } else { Some(style) }
}

/// 解析边框配置
///
/// 支持两种格式：
/// - `border: true` → 四边细线
/// - `border: { top: "thin", bottom: "medium", left: "thick", right: "dashed" }` → 分别指定
fn parse_border_config(val: &wasm_bindgen::JsValue) -> Option<BorderConfig> {
    let border_val = js_sys::Reflect::get(val, &wasm_bindgen::JsValue::from_str("border"))
        .ok()
        .filter(|v| !v.is_undefined() && !v.is_null())?;

    // boolean: true → 四边细线
    if let Some(b) = border_val.as_bool() {
        return if b { Some(BorderConfig::All) } else { None };
    }

    // 对象: 分别指定四边
    if border_val.is_object() {
        let top = js_sys::Reflect::get(&border_val, &wasm_bindgen::JsValue::from_str("top"))
            .ok()
            .and_then(|v| v.as_string())
            .and_then(|s| BorderLine::from_str(&s));

        let bottom = js_sys::Reflect::get(&border_val, &wasm_bindgen::JsValue::from_str("bottom"))
            .ok()
            .and_then(|v| v.as_string())
            .and_then(|s| BorderLine::from_str(&s));

        let left = js_sys::Reflect::get(&border_val, &wasm_bindgen::JsValue::from_str("left"))
            .ok()
            .and_then(|v| v.as_string())
            .and_then(|s| BorderLine::from_str(&s));

        let right = js_sys::Reflect::get(&border_val, &wasm_bindgen::JsValue::from_str("right"))
            .ok()
            .and_then(|v| v.as_string())
            .and_then(|s| BorderLine::from_str(&s));

        if top.is_some() || bottom.is_some() || left.is_some() || right.is_some() {
            return Some(BorderConfig::Individual {
                top,
                bottom,
                left,
                right,
            });
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_hex_color_3digit() {
        assert_eq!(normalize_hex_color("#F00"), "#FF0000");
        assert_eq!(normalize_hex_color("ABC"), "#AABBCC");
        assert_eq!(normalize_hex_color("#abc"), "#aabbcc");
    }

    #[test]
    fn test_normalize_hex_color_6digit() {
        assert_eq!(normalize_hex_color("#FF0000"), "#FF0000");
        assert_eq!(normalize_hex_color("AABBCC"), "#AABBCC");
    }

    #[test]
    fn test_cell_style_is_empty() {
        let empty = CellStyle::default();
        assert!(empty.is_empty());

        let non_empty = CellStyle {
            bold: Some(true),
            ..Default::default()
        };
        assert!(!non_empty.is_empty());
    }

    #[test]
    fn test_cell_style_merge() {
        let base = CellStyle {
            bold: Some(true),
            font_size: Some(12.0),
            font_color: Some("#000000".to_string()),
            ..Default::default()
        };

        let override_style = CellStyle {
            bold: Some(false),
            italic: Some(true),
            font_color: Some("#FF0000".to_string()),
            ..Default::default()
        };

        let merged = base.merge(&override_style);
        assert_eq!(merged.bold, Some(false)); // 被覆盖
        assert_eq!(merged.italic, Some(true)); // 新增
        assert_eq!(merged.font_size, Some(12.0)); // 保留
        assert_eq!(merged.font_color, Some("#FF0000".to_string())); // 被覆盖
    }

    #[test]
    fn test_cell_style_to_format() {
        // 验证 to_format 不 panic
        let style = CellStyle {
            bold: Some(true),
            italic: Some(true),
            font_size: Some(14.0),
            font_name: Some("Arial".to_string()),
            font_color: Some("#FF0000".to_string()),
            background_color: Some("#F2F2F2".to_string()),
            align: Some(HAlign::Center),
            vertical_align: Some(VAlign::Center),
            border: Some(BorderConfig::All),
            number_format: Some("#,##0.00".to_string()),
            text_wrap: Some(true),
        };

        let _format = style.to_format();
    }

    #[test]
    fn test_cell_style_individual_border() {
        let style = CellStyle {
            border: Some(BorderConfig::Individual {
                top: Some(BorderLine::Thin),
                bottom: Some(BorderLine::Medium),
                left: None,
                right: Some(BorderLine::Thick),
            }),
            ..Default::default()
        };

        let _format = style.to_format();
    }

    #[test]
    fn test_border_line_from_str() {
        assert_eq!(BorderLine::from_str("thin"), Some(BorderLine::Thin));
        assert_eq!(BorderLine::from_str("MEDIUM"), Some(BorderLine::Medium));
        assert_eq!(BorderLine::from_str("Thick"), Some(BorderLine::Thick));
        assert_eq!(BorderLine::from_str("dashed"), Some(BorderLine::Dashed));
        assert_eq!(BorderLine::from_str("dotted"), Some(BorderLine::Dotted));
        assert_eq!(BorderLine::from_str("double"), Some(BorderLine::Double));
        assert_eq!(BorderLine::from_str("invalid"), None);
    }

    #[test]
    fn test_stylesheet_is_empty() {
        let empty = StyleSheet::default();
        assert!(empty.is_empty());
    }

    #[test]
    fn test_stylesheet_resolve_no_styles() {
        let sheet = StyleSheet::default();
        assert!(sheet.resolve(0, 0, 1).is_none());
        assert!(sheet.resolve(1, 0, 1).is_none());
    }

    #[test]
    fn test_stylesheet_resolve_header_style() {
        let sheet = StyleSheet {
            header_style: Some(CellStyle {
                bold: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };

        // 行 0 是表头（header_row_count=1）
        assert!(sheet.resolve(0, 0, 1).is_some());
        // 行 1 是数据，无样式
        assert!(sheet.resolve(1, 0, 1).is_none());
    }

    #[test]
    fn test_stylesheet_resolve_column_override() {
        let sheet = StyleSheet {
            data_style: Some(CellStyle {
                bold: Some(true),
                font_color: Some("#000000".to_string()),
                ..Default::default()
            }),
            column_styles: vec![
                None,
                Some(CellStyle {
                    font_color: Some("#FF0000".to_string()),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };

        // 列 0 应用全局数据样式
        assert!(sheet.resolve(1, 0, 1).is_some());
        // 列 1 应用列级覆盖（font_color 被覆盖）
        assert!(sheet.resolve(1, 1, 1).is_some());
    }

    #[test]
    fn test_stylesheet_resolve_cell_override() {
        let mut sheet = StyleSheet {
            data_style: Some(CellStyle {
                bold: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };
        sheet.cell_overrides.insert(
            (1, 0),
            CellStyle {
                font_color: Some("#FF0000".to_string()),
                ..Default::default()
            },
        );

        // (1,0) 有单元格级覆盖
        assert!(sheet.resolve(1, 0, 1).is_some());
        // (1,1) 只有全局样式
        assert!(sheet.resolve(1, 1, 1).is_some());
    }

    #[test]
    fn test_three_level_merge() {
        let mut sheet = StyleSheet {
            header_style: Some(CellStyle {
                bold: Some(true),
                font_size: Some(12.0),
                font_color: Some("#000000".to_string()),
                ..Default::default()
            }),
            column_header_styles: vec![
                None,
                Some(CellStyle {
                    font_color: Some("#0000FF".to_string()),
                    align: Some(HAlign::Center),
                    ..Default::default()
                }),
            ],
            ..Default::default()
        };
        sheet.cell_overrides.insert(
            (0, 1),
            CellStyle {
                background_color: Some("#FFFF00".to_string()),
                ..Default::default()
            },
        );

        // (0,1) 应合并三级样式：bold=true, fontSize=12, fontColor="#0000FF", align=center, bg="#FFFF00"
        let format = sheet.resolve(0, 1, 1);
        assert!(format.is_some());
    }
}
