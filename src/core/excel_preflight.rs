//! Excel 解码前的资源预检，避免底层库按稀疏坐标分配巨大矩形。
use std::collections::{HashMap, HashSet};
use std::io::{BufReader, Cursor, Read, Seek};

use quick_xml::events::{BytesStart, Event};
use zip::ZipArchive;

const MAX_INPUT_BYTES: usize = 50 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 50 * 1024 * 1024;
const MAX_TOTAL_BYTES: u64 = 100 * 1024 * 1024;
const MAX_ZIP_ENTRIES: usize = 10_000;
/// calamine 的 Range 为稠密矩形，限制面积而非实际非空单元格数。
pub(super) const MAX_RANGE_CELLS: u64 = 1_000_000;
/// `sharedStrings.uniqueCount` 会在读取正文前直接 `Vec::reserve`。
const MAX_SHARED_STRINGS: u64 = 1_000_000;

type PrecheckedArchive<'a> = (ZipArchive<Cursor<&'a [u8]>>, Vec<String>);

/// 预检输入并返回可供样式和尺寸解析继续复用的 ZIP 及工作表路径。
pub(super) fn preflight(data: &[u8]) -> Result<Option<PrecheckedArchive<'_>>, String> {
    if data.len() > MAX_INPUT_BYTES {
        return Err("Excel 文件大小超过安全上限（50 MiB）".into());
    }
    if !data.starts_with(b"PK") {
        // BIFF/XLS 由 calamine 在打开时完整解码；此处只能限制原始文件大小。
        return Ok(None);
    }
    let mut archive =
        ZipArchive::new(Cursor::new(data)).map_err(|e| format!("无法打开 Excel 压缩包: {e}"))?;
    if archive.len() > MAX_ZIP_ENTRIES {
        return Err("Excel 压缩包文件数量超过安全上限".into());
    }
    let mut total = 0_u64;
    let mut names = HashSet::new();
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|e| format!("读取 Excel 压缩包失败: {e}"))?;
        // calamine 按忽略大小写、统一斜线的路径查找；拒绝其可能覆盖的别名。
        if !names.insert(file.name().replace('\\', "/").to_ascii_lowercase()) {
            return Err("Excel 压缩包含重复或歧义文件名".into());
        }
        if file.size() > MAX_ENTRY_BYTES {
            return Err("Excel 压缩包单文件解压大小超过安全上限（50 MiB）".into());
        }
        total = total
            .checked_add(file.size())
            .filter(|size| *size <= MAX_TOTAL_BYTES)
            .ok_or("Excel 压缩包总解压大小超过安全上限（100 MiB）")?;
    }
    // 先检查全部目录项，再解压；总量超限时不做任何大块读取。
    for index in 0..archive.len() {
        let file = archive
            .by_index(index)
            .map_err(|e| format!("读取 Excel 压缩包失败: {e}"))?;
        // 不只信任中央目录的声明；有界解压并核对实际大小，也验证 CRC。
        let expected = file.size();
        let actual = std::io::copy(&mut file.take(MAX_ENTRY_BYTES + 1), &mut std::io::sink())
            .map_err(|e| format!("校验 Excel 压缩包失败: {e}"))?;
        if actual != expected || actual > MAX_ENTRY_BYTES {
            return Err("Excel 压缩包实际解压大小异常".into());
        }
    }
    check_shared_strings(&mut archive)?;
    let paths = sheet_paths(&mut archive)?;
    for path in &paths {
        scan_sheet(&mut archive, path)?;
    }
    Ok(Some((archive, paths)))
}

/// 关系标识和目标保留原始属性值，与 calamine 0.34 的查找语义一致。
fn attr(event: &BytesStart<'_>, key: &[u8]) -> Option<String> {
    event.attributes().flatten().find_map(|a| {
        (a.key.as_ref() == key)
            .then(|| String::from_utf8(a.value.into_owned()).ok())
            .flatten()
    })
}

/// 按 calamine 相同的忽略 ASCII 大小写/斜线规则取得实际 ZIP 项名。
fn archive_path<R: Read + Seek>(archive: &ZipArchive<R>, path: &str) -> Result<String, String> {
    archive
        .file_names()
        .find(|name| name.replace('\\', "/").eq_ignore_ascii_case(path))
        .map(str::to_owned)
        .ok_or_else(|| format!("Excel 压缩包缺少文件: {path}"))
}

/// 从 workbook 的 r:id 定位关系，不能用 rels 的物理顺序猜工作表。
fn sheet_paths<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<Vec<String>, String> {
    let workbook_path = archive_path(archive, "xl/workbook.xml")?;
    let rels_path = archive_path(archive, "xl/_rels/workbook.xml.rels")?;
    let mut ids = Vec::new();
    let mut reader = quick_xml::Reader::from_reader(BufReader::new(
        archive
            .by_name(&workbook_path)
            .map_err(|e| format!("读取工作簿信息失败: {e}"))?,
    ));
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) if e.local_name().as_ref() == b"sheet" => {
                let id = match (attr(&e, b"r:id"), attr(&e, b"relationships:id")) {
                    (Some(id), None) | (None, Some(id)) => id,
                    _ => return Err("工作表关系标识缺失或存在歧义".into()),
                };
                ids.push(id);
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("解析工作簿信息失败: {e}")),
            _ => {}
        }
        buf.clear();
    }
    drop(reader);
    let mut relations = HashMap::new();
    let mut reader = quick_xml::Reader::from_reader(BufReader::new(
        archive
            .by_name(&rels_path)
            .map_err(|e| format!("读取工作簿关系失败: {e}"))?,
    ));
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) if e.local_name().as_ref() == b"Relationship" => {
                // 不按 Type 过滤：底层库按 Id 查找，其他类型的重复 Id 也会覆盖工作表。
                if let (Some(id), Some(target)) = (attr(&e, b"Id"), attr(&e, b"Target")) {
                    if relations.insert(id, target).is_some() {
                        return Err("工作簿含重复关系标识".into());
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("解析工作簿关系失败: {e}")),
            _ => {}
        }
        buf.clear();
    }
    drop(reader);
    ids.into_iter()
        .map(|id| {
            let target = relations
                .get(&id)
                .ok_or_else(|| format!("找不到工作表关系: {id}"))?;
            let path = normalize_path(target)?;
            let calamine_path = if let Some(path) = target.strip_prefix("/xl/") {
                format!("xl/{path}")
            } else if target.starts_with("xl/") {
                target.clone()
            } else {
                format!("xl/{target}")
            };
            if path != calamine_path {
                return Err("工作表路径不规范，无法安全预检".into());
            }
            archive_path(archive, &path)
        })
        .collect()
}

/// 按 OPC 相对路径规则解析 workbook.xml 的目标路径。
fn normalize_path(target: &str) -> Result<String, String> {
    let mut parts = if target.starts_with('/') || target.starts_with("xl/") {
        Vec::new()
    } else {
        vec!["xl"]
    };
    for part in target.split('/') {
        match part {
            "" | "." => {}
            ".." => {
                parts.pop().ok_or("工作表路径越出压缩包根目录")?;
            }
            _ if part.contains(['\\', ':', '\0']) => return Err("工作表路径不合法".into()),
            _ => parts.push(part),
        }
    }
    Ok(parts.join("/"))
}

/// 严格解析 A1 坐标；既避免下溢，也拒绝超出 XLSX 网格的坐标。
pub(super) fn cell_ref(value: &str) -> Option<(u32, u32)> {
    let split = value.find(|c: char| !c.is_ascii_alphabetic())?;
    if split == 0 {
        return None;
    }
    let (letters, digits) = value.split_at(split);
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let col = letters.bytes().try_fold(0_u32, |col, b| {
        col.checked_mul(26)?
            .checked_add(u32::from(b.to_ascii_uppercase() - b'A' + 1))
    })?;
    let row = digits.parse::<u32>().ok()?;
    if !(1..=1_048_576).contains(&row) || !(1..=16_384).contains(&col) {
        return None;
    }
    Some((row - 1, col - 1))
}

/// 拒绝声明巨量共享字符串。calamine 会按 `uniqueCount` 预分配，与实际 XML 体积无关。
fn check_shared_strings<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<(), String> {
    let Some(path) = archive
        .file_names()
        .find(|name| {
            name.replace('\\', "/")
                .eq_ignore_ascii_case("xl/sharedStrings.xml")
        })
        .map(str::to_owned)
    else {
        return Ok(());
    };
    let file = archive
        .by_name(&path)
        .map_err(|e| format!("读取共享字符串失败: {e}"))?;
    let mut reader = quick_xml::Reader::from_reader(BufReader::new(file));
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e) | Event::Empty(e)) if e.local_name().as_ref() == b"sst" => {
                if let Some(count) = attr(&e, b"uniqueCount") {
                    if count.parse::<u64>().unwrap_or(0) > MAX_SHARED_STRINGS {
                        return Err(format!(
                            "共享字符串数量超过安全上限（{MAX_SHARED_STRINGS}）"
                        ));
                    }
                }
                return Ok(());
            }
            Ok(Event::Eof) => return Ok(()),
            Err(e) => return Err(format!("预检共享字符串失败: {e}")),
            _ => {}
        }
        buf.clear();
    }
}

struct OpenCell {
    row: u32,
    col: u32,
    type_name: Option<String>,
    /// 与 calamine 一致：后出现的 `v` / `is` / `f` 覆盖前一个值。
    counts: bool,
}

struct SheetScan {
    row: u32,
    col: u32,
    in_sheet_data: bool,
    open: Option<OpenCell>,
    in_value: bool,
    in_inline: bool,
    value_text: String,
    bounds: Option<(u32, u32, u32, u32)>,
}

/// 空的共享字符串索引和数字不会进入稠密矩形；其余类型连空文本也会。
fn value_occupies(type_name: Option<&str>, text: &str) -> bool {
    match type_name {
        Some("s") | Some("n") => !text.is_empty(),
        _ => true,
    }
}

fn include_used_cell(
    bounds: &mut Option<(u32, u32, u32, u32)>,
    row: u32,
    col: u32,
) -> Result<(), String> {
    let (min_row, min_col, max_row, max_col) = match *bounds {
        Some((min_row, min_col, max_row, max_col)) => (
            min_row.min(row),
            min_col.min(col),
            max_row.max(row),
            max_col.max(col),
        ),
        None => (row, col, row, col),
    };
    let area = u64::from(max_row - min_row + 1) * u64::from(max_col - min_col + 1);
    if area > MAX_RANGE_CELLS {
        return Err(format!(
            "工作表已用区域矩形超过安全上限（{MAX_RANGE_CELLS} 个单元格）"
        ));
    }
    *bounds = Some((min_row, min_col, max_row, max_col));
    Ok(())
}

fn finish_open_cell(scan: &mut SheetScan) -> Result<(), String> {
    if let Some(cell) = scan.open.take() {
        if cell.counts {
            include_used_cell(&mut scan.bounds, cell.row, cell.col)?;
        }
    }
    scan.in_value = false;
    scan.in_inline = false;
    Ok(())
}

fn handle_sheet_tag(
    scan: &mut SheetScan,
    event: &BytesStart<'_>,
    is_empty: bool,
) -> Result<(), String> {
    match event.local_name().as_ref() {
        b"dimension" | b"mergeCell" => {
            if let Some(value) = attr(event, b"ref") {
                let (first, last) = value.split_once(':').unwrap_or((&value, &value));
                let (start, end) = (
                    cell_ref(first).ok_or("工作表范围起点不合法")?,
                    cell_ref(last).ok_or("工作表范围终点不合法")?,
                );
                // calamine 的 get_dimension 使用无符号减法，反向范围会下溢。
                if start.0 > end.0 || start.1 > end.1 {
                    return Err("工作表范围起止坐标颠倒".into());
                }
            }
        }
        b"sheetData" if !is_empty => scan.in_sheet_data = true,
        b"row" if scan.in_sheet_data => {
            if let Some(value) = attr(event, b"r") {
                scan.row = value
                    .parse::<u32>()
                    .ok()
                    .filter(|r| (1..=1_048_576).contains(r))
                    .ok_or("工作表行号不合法")?
                    - 1;
            }
            // 自闭合 <row/> 会被 calamine 展开，隐式行号仍要推进。
            if is_empty {
                finish_open_cell(scan)?;
                scan.row = scan.row.saturating_add(1);
                scan.col = 0;
            }
        }
        b"c" if scan.in_sheet_data => {
            let (row, col) = if let Some(value) = attr(event, b"r") {
                cell_ref(&value).ok_or("工作表单元格坐标不合法")?
            } else {
                (scan.row, scan.col)
            };
            if row >= 1_048_576 || col >= 16_384 {
                return Err("工作表单元格坐标超出安全范围".into());
            }
            scan.col = col.saturating_add(1);
            // 没有值的样式单元格不会扩大 calamine Range。
            if is_empty {
                scan.open = None;
            } else {
                scan.open = Some(OpenCell {
                    row,
                    col,
                    type_name: attr(event, b"t"),
                    counts: false,
                });
                scan.in_value = false;
                scan.in_inline = false;
                scan.value_text.clear();
            }
        }
        b"v" if scan.open.is_some() => {
            if is_empty {
                if let Some(cell) = scan.open.as_mut() {
                    cell.counts = value_occupies(cell.type_name.as_deref(), "");
                }
            } else {
                scan.in_value = true;
                scan.in_inline = false;
                scan.value_text.clear();
            }
        }
        b"is" if scan.open.is_some() => {
            if let Some(cell) = scan.open.as_mut() {
                // 空 inlineStr 仍是 String，不是 Empty。
                cell.counts = true;
            }
            scan.in_inline = !is_empty;
            scan.in_value = false;
        }
        b"f" if is_empty && scan.open.is_some() => {
            if let Some(cell) = scan.open.as_mut() {
                cell.counts = false;
            }
        }
        _ => {}
    }
    Ok(())
}

/// 流式扫描真实非空单元格坐标及省略坐标的隐式位置，dimension 不能作为可信边界。
fn scan_sheet<R: Read + Seek>(archive: &mut ZipArchive<R>, path: &str) -> Result<(), String> {
    let file = archive
        .by_name(path)
        .map_err(|e| format!("读取工作表 {path} 失败: {e}"))?;
    let mut reader = quick_xml::Reader::from_reader(BufReader::new(file));
    let mut buf = Vec::new();
    let mut scan = SheetScan {
        row: 0,
        col: 0,
        in_sheet_data: false,
        open: None,
        in_value: false,
        in_inline: false,
        value_text: String::new(),
        bounds: None,
    };
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(event)) => handle_sheet_tag(&mut scan, &event, false)?,
            Ok(Event::Empty(event)) => handle_sheet_tag(&mut scan, &event, true)?,
            Ok(Event::Text(text)) if scan.in_value || scan.in_inline => match text.unescape() {
                Ok(value) => scan.value_text.push_str(&value),
                Err(_) => scan.value_text.push('\u{FFFD}'),
            },
            Ok(Event::CData(text)) if scan.in_value || scan.in_inline => {
                scan.value_text
                    .push_str(&String::from_utf8_lossy(text.as_ref()));
            }
            Ok(Event::End(event)) => match event.local_name().as_ref() {
                b"sheetData" => {
                    finish_open_cell(&mut scan)?;
                    scan.in_sheet_data = false;
                }
                b"row" if scan.in_sheet_data => {
                    finish_open_cell(&mut scan)?;
                    scan.row = scan.row.saturating_add(1);
                    scan.col = 0;
                }
                b"v" => {
                    scan.in_value = false;
                    if let Some(cell) = scan.open.as_mut() {
                        cell.counts = value_occupies(cell.type_name.as_deref(), &scan.value_text);
                    }
                }
                b"is" => {
                    scan.in_inline = false;
                    if let Some(cell) = scan.open.as_mut() {
                        cell.counts = true;
                    }
                }
                b"f" => {
                    if let Some(cell) = scan.open.as_mut() {
                        cell.counts = false;
                    }
                }
                b"c" => finish_open_cell(&mut scan)?,
                _ => {}
            },
            Ok(Event::Eof) => break,
            Err(e) => return Err(format!("预检工作表 XML 失败: {e}")),
            _ => {}
        }
        buf.clear();
    }
    finish_open_cell(&mut scan)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::excel_reader::{PreviewOptions, get_sheet_list, parse_excel};
    use std::io::Write;

    const SHEET: &str = "xl/worksheets/sheet1.xml";
    const STYLES: &str = r#"<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><numFmts count="3"><numFmt numFmtId="164" formatCode="yyyy-mm-dd"/><numFmt numFmtId="165" formatCode="yyyy-mm-dd hh:mm:ss"/><numFmt numFmtId="166" formatCode="hh:mm:ss"/></numFmts><fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts><fills count="1"><fill><patternFill patternType="none"/></fill></fills><borders count="1"><border/></borders><cellXfs count="4"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/><xf numFmtId="164" fontId="0" fillId="0" borderId="0"/><xf numFmtId="165" fontId="0" fillId="0" borderId="0"/><xf numFmtId="166" fontId="0" fillId="0" borderId="0"/></cellXfs><dxfs count="3"><dxf><font><color rgb="FFFF0000"/></font></dxf><dxf><fill><patternFill patternType="solid"><fgColor rgb="FFFFFF00"/></patternFill></fill></dxf><dxf><font><color rgb="FF0000FF"/></font></dxf></dxfs></styleSheet>"#;

    /// 基于真实 Workbook 生成 ZIP，再定点替换 OOXML；不会分配巨型 Range。
    fn base_fixture() -> Vec<u8> {
        let mut workbook = rust_xlsxwriter::Workbook::new();
        workbook.add_worksheet().write_number(0, 0, 1).unwrap();
        workbook.add_worksheet().write_number(0, 0, 2).unwrap();
        workbook.save_to_buffer().unwrap()
    }

    fn replace_entries(data: &[u8], replacements: &[(&str, &str)]) -> Vec<u8> {
        let mut archive = ZipArchive::new(Cursor::new(data)).unwrap();
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).unwrap();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).unwrap();
            if let Some((_, replacement)) =
                replacements.iter().find(|(name, _)| *name == entry.name())
            {
                content = replacement.as_bytes().to_vec();
            }
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output.write_all(&content).unwrap();
        }
        output.finish().unwrap().into_inner()
    }

    fn upsert_entries(data: &[u8], updates: &[(&str, &str)]) -> Vec<u8> {
        let mut archive = ZipArchive::new(Cursor::new(data)).unwrap();
        let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let mut seen = Vec::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).unwrap();
            let name = entry.name().to_owned();
            let mut content = Vec::new();
            entry.read_to_end(&mut content).unwrap();
            if let Some((_, replacement)) = updates.iter().find(|(path, _)| *path == name) {
                content = replacement.as_bytes().to_vec();
                seen.push(name.clone());
            }
            output
                .start_file(name, zip::write::SimpleFileOptions::default())
                .unwrap();
            output.write_all(&content).unwrap();
        }
        for (name, content) in updates {
            if !seen.iter().any(|seen| seen == name) {
                output
                    .start_file(*name, zip::write::SimpleFileOptions::default())
                    .unwrap();
                output.write_all(content.as_bytes()).unwrap();
            }
        }
        output.finish().unwrap().into_inner()
    }

    fn entry_text(data: &[u8], path: &str) -> String {
        let mut archive = ZipArchive::new(Cursor::new(data)).unwrap();
        let mut text = String::new();
        archive
            .by_name(path)
            .unwrap()
            .read_to_string(&mut text)
            .unwrap();
        text
    }

    #[test]
    fn test_excel_preflight_sparse_rectangle_ignores_misleading_dimension() {
        for dimension in ["", r#"<dimension ref="A1"/>"#] {
            let xml = format!(
                r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">{dimension}<sheetData><row r="1"><c r="A1"><v>1</v></c><c r="XFD1048576"><v>2</v></c></row></sheetData></worksheet>"#
            );
            let data = replace_entries(&base_fixture(), &[(SHEET, &xml)]);
            let options = PreviewOptions {
                max_rows: Some(1),
                ..Default::default()
            };
            assert!(
                parse_excel(&data, &options)
                    .unwrap_err()
                    .contains("矩形超过安全上限")
            );
            assert!(
                get_sheet_list(&data)
                    .unwrap_err()
                    .contains("矩形超过安全上限")
            );
        }
    }

    #[test]
    fn test_excel_preflight_implicit_cell_coordinate_and_hidden_sheet() {
        let xml = r#"<worksheet><sheetData><row r="1"><c><v>1</v></c></row><row r="1048576"><c><v>2</v></c></row></sheetData></worksheet>"#;
        // 默认只预览 Sheet1，仍必须在 calamine 打开前检查其他工作表。
        let data = replace_entries(&base_fixture(), &[("xl/worksheets/sheet2.xml", xml)]);
        assert!(
            parse_excel(&data, &PreviewOptions::default())
                .unwrap_err()
                .contains("矩形")
        );
        assert!(get_sheet_list(&data).is_err());
    }

    #[test]
    fn test_excel_preflight_zip_entry_and_total_limits_before_decompression() {
        // 只改中央目录中的声明大小，验证解压之前的拒绝路径；fixture 保持数 KB。
        let base = base_fixture();
        for (count, size, expected) in [
            (1, MAX_ENTRY_BYTES + 1, "单文件"),
            (3, 40 * 1024 * 1024, "总解压"),
        ] {
            let mut data = base.clone();
            let mut patched = 0;
            for index in 0..data.len().saturating_sub(28) {
                if data[index..].starts_with(b"PK\x01\x02") && patched < count {
                    data[index + 24..index + 28].copy_from_slice(&(size as u32).to_le_bytes());
                    patched += 1;
                }
            }
            assert_eq!(patched, count);
            assert!(get_sheet_list(&data).unwrap_err().contains(expected));
        }
    }

    #[test]
    fn test_excel_preflight_real_rid_mapping_and_layout_without_styles() {
        let base = base_fixture();
        let workbook = entry_text(&base, "xl/workbook.xml")
            .replace("r:id=\"rId1\"", "r:id=\"temporary\"")
            .replace("r:id=\"rId2\"", "r:id=\"rId1\"")
            .replace("r:id=\"temporary\"", "r:id=\"rId2\"");
        let xml = r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetFormatPr defaultRowHeight="15"/><cols><col min="1" max="1" width="25"/><col min="2" max="2" hidden="1"/></cols><sheetData><row r="1" hidden="1"><c r="A1"><v>9</v></c><c r="B1"><v>8</v></c></row><row r="2" ht="30"><c r="A2"><v>7</v></c><c r="B2"><v>6</v></c></row></sheetData></worksheet>"#;
        let data = replace_entries(
            &base,
            &[
                ("xl/workbook.xml", &workbook),
                ("xl/worksheets/sheet2.xml", xml),
            ],
        );
        let parsed = parse_excel(
            &data,
            &PreviewOptions {
                include_styles: false,
                ..Default::default()
            },
        )
        .unwrap();
        let sheet = &parsed.sheets[0];
        assert_eq!(sheet.rows.len(), 1);
        assert_eq!(sheet.rows[0].cells.len(), 1);
        assert_eq!(sheet.rows[0].cells[0].as_ref().unwrap().value, "7");
        assert_eq!(sheet.rows[0].cells[0].as_ref().unwrap().style, None);
        assert!((sheet.rows[0].height.unwrap() - 39.99).abs() < 0.01);
        assert_eq!(sheet.col_widths, vec![187.0]);
    }

    #[test]
    fn test_excel_preflight_dates_datetime_time_and_1904() {
        for (date_1904, serial, date) in [(false, "45292", "2024-01-01"), (true, "0", "1904-01-01")]
        {
            let base = base_fixture();
            let workbook = entry_text(&base, "xl/workbook.xml").replace(
                "<workbookPr",
                &format!("<workbookPr date1904=\"{}\"", u8::from(date_1904)),
            );
            let serial_with_time = serial.parse::<f64>().unwrap() + 0.5;
            let xml = format!(
                r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1" s="1"><v>{serial}</v></c><c r="B1" s="2"><v>{serial_with_time}</v></c><c r="C1" s="3"><v>0.5</v></c></row></sheetData></worksheet>"#
            );
            let data = replace_entries(
                &base,
                &[
                    (SHEET, &xml),
                    ("xl/styles.xml", STYLES),
                    ("xl/workbook.xml", &workbook),
                ],
            );
            let parsed = parse_excel(&data, &PreviewOptions::default()).unwrap();
            let cells = &parsed.sheets[0].rows[0].cells;
            assert_eq!(cells[0].as_ref().unwrap().value, date);
            assert_eq!(cells[1].as_ref().unwrap().value, format!("{date} 12:00:00"));
            assert_eq!(cells[2].as_ref().unwrap().value, "12:00:00");
            let plain = parse_excel(
                &data,
                &PreviewOptions {
                    include_styles: false,
                    ..Default::default()
                },
            )
            .unwrap();
            assert_eq!(
                plain.sheets[0].rows[0].cells[0].as_ref().unwrap().value,
                date
            );
        }
    }

    #[test]
    fn test_excel_preflight_cf_global_priority_composition_and_stop() {
        let base = base_fixture();
        for stop in [false, true] {
            let xml = format!(
                r#"<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData><row r="1"><c r="A1"><v>5</v></c></row></sheetData><conditionalFormatting sqref="A1"><cfRule type="cellIs" operator="greaterThan" priority="3" dxfId="2"><formula>0</formula></cfRule><cfRule type="cellIs" operator="greaterThan" priority="2" dxfId="1"><formula>0</formula></cfRule></conditionalFormatting><conditionalFormatting sqref="A1"><cfRule type="cellIs" operator="greaterThan" priority="1" stopIfTrue="{}" dxfId="0"><formula>0</formula></cfRule></conditionalFormatting></worksheet>"#,
                u8::from(stop)
            );
            let data = replace_entries(&base, &[(SHEET, &xml), ("xl/styles.xml", STYLES)]);
            let parsed = parse_excel(&data, &PreviewOptions::default()).unwrap();
            let css = parsed.sheets[0].rows[0].cells[0]
                .as_ref()
                .unwrap()
                .style
                .as_ref()
                .unwrap();
            assert!(css.ends_with("color:#FF0000"), "{css}");
            assert_eq!(css.contains("background-color:#FFFF00"), !stop, "{css}");
        }
        // 无差异格式的停止规则也必须阻止低优先级样式。
        let xml = r#"<worksheet><sheetData><row><c><v>5</v></c></row></sheetData><conditionalFormatting sqref="A1"><cfRule type="cellIs" operator="greaterThan" priority="2" dxfId="0"><formula>0</formula></cfRule><cfRule type="cellIs" operator="greaterThan" priority="1" stopIfTrue="1"><formula>0</formula></cfRule></conditionalFormatting></worksheet>"#;
        let data = replace_entries(&base, &[(SHEET, xml), ("xl/styles.xml", STYLES)]);
        let parsed = parse_excel(&data, &PreviewOptions::default()).unwrap();
        assert_eq!(
            parsed.sheets[0].rows[0].cells[0].as_ref().unwrap().style,
            None
        );
    }

    #[test]
    fn test_excel_preflight_reversed_dimension_and_merge_are_rejected() {
        for node in [
            r#"<dimension ref="B2:A1"/>"#,
            r#"<mergeCells><mergeCell ref="B2:A1"/></mergeCells>"#,
        ] {
            let xml = format!(
                "<worksheet>{node}<sheetData><row><c><v>1</v></c></row></sheetData></worksheet>"
            );
            let data = replace_entries(&base_fixture(), &[(SHEET, &xml)]);
            assert!(
                parse_excel(&data, &PreviewOptions::default())
                    .unwrap_err()
                    .contains("颠倒")
            );
            assert!(get_sheet_list(&data).is_err());
        }
    }

    #[test]
    fn test_excel_preflight_prefixed_targets_and_duplicate_relation_ids() {
        let base = base_fixture();
        let rels = entry_text(&base, "xl/_rels/workbook.xml.rels");
        let prefixed = rels.replace("Target=\"worksheets/", "Target=\"xl/worksheets/");
        let data = replace_entries(&base, &[("xl/_rels/workbook.xml.rels", &prefixed)]);
        assert!(parse_excel(&data, &PreviewOptions::default()).is_ok());
        let duplicate = rels.replace("</Relationships>", r#"<Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="worksheets/sheet2.xml"/></Relationships>"#);
        let data = replace_entries(&base, &[("xl/_rels/workbook.xml.rels", &duplicate)]);
        assert!(get_sheet_list(&data).unwrap_err().contains("重复关系"));
    }

    #[test]
    fn test_excel_preflight_unicode_color_fixture() {
        let xml = r#"<worksheet><sheetData><row><c><v>5</v></c></row></sheetData><conditionalFormatting sqref="A1"><cfRule type="cellIs" operator="greaterThan" priority="1" dxfId="0"><formula>0</formula></cfRule></conditionalFormatting></worksheet>"#;
        let styles = STYLES.replace("FFFF0000", "中00000");
        let data = replace_entries(&base_fixture(), &[(SHEET, xml), ("xl/styles.xml", &styles)]);
        let parsed = parse_excel(&data, &PreviewOptions::default()).unwrap();
        let css = parsed.sheets[0].rows[0].cells[0]
            .as_ref()
            .unwrap()
            .style
            .as_ref()
            .unwrap();
        assert_eq!(css, "color:#000000");
    }

    #[test]
    fn test_excel_preflight_coordinate_bounds_unicode_and_paths() {
        for value in [
            "A0",
            "A1B2",
            "A-1",
            "中1",
            "XFE1",
            "A1048577",
            "AAAAAAAAAAAAAAAA1",
        ] {
            assert_eq!(cell_ref(value), None, "{value}");
        }
        assert_eq!(cell_ref("XFD1048576"), Some((1_048_575, 16_383)));
        assert_eq!(
            normalize_path("worksheets/../worksheets/表🌸.xml").unwrap(),
            "xl/worksheets/表🌸.xml"
        );
        assert_eq!(
            normalize_path("/xl/worksheets/a.xml").unwrap(),
            "xl/worksheets/a.xml"
        );
        assert!(normalize_path("../../evil.xml").is_err());
        let invalid = r#"<worksheet><sheetData><row r="0"><c r="A0"><v>1</v></c></row></sheetData></worksheet>"#;
        let data = replace_entries(&base_fixture(), &[(SHEET, invalid)]);
        assert!(
            parse_excel(&data, &PreviewOptions::default())
                .unwrap_err()
                .contains("行号不合法")
        );
    }

    #[test]
    fn test_excel_preflight_self_closing_row_advances_implicit_position() {
        let mut rows = String::from(r#"<row><c r="A1"><v>1</v></c></row>"#);
        for _ in 0..60 {
            rows.push_str("<row/>");
        }
        rows.push_str(r#"<row><c r="XFC2"/><c><v>2</v></c></row>"#);
        let xml = format!("<worksheet><sheetData>{rows}</sheetData></worksheet>");
        let data = replace_entries(&base_fixture(), &[(SHEET, &xml)]);
        // 只走预检：回归失效时也不能把这个文件交给 calamine 分配。
        assert!(preflight(&data).unwrap_err().contains("矩形超过安全上限"));
    }

    #[test]
    fn test_excel_preflight_empty_style_cell_does_not_expand_rectangle() {
        let xml = r#"<worksheet><sheetData><row r="1"><c r="A1"><v>1</v></c></row><row r="1048576"><c r="XFD1048576" s="1"/></row></sheetData></worksheet>"#;
        let data = replace_entries(&base_fixture(), &[(SHEET, xml)]);
        assert!(preflight(&data).is_ok());
        let parsed = parse_excel(&data, &PreviewOptions::default()).unwrap();
        assert_eq!(
            parsed.sheets[0].rows[0].cells[0].as_ref().unwrap().value,
            "1"
        );
        assert!(get_sheet_list(&data).is_ok());
    }

    #[test]
    fn test_excel_preflight_empty_number_formula_and_untyped_value() {
        for xml in [
            r#"<worksheet><sheetData><row r="1"><c r="A1"><v>1</v></c></row><row r="1048576"><c r="XFD1048576" t="n"><v></v></c></row></sheetData></worksheet>"#,
            r#"<worksheet><sheetData><row r="1"><c r="A1"><v>1</v></c></row><row r="1048576"><c r="XFD1048576"><f>1</f></c></row></sheetData></worksheet>"#,
            r#"<worksheet><sheetData><row r="1"><c r="A1"><v>1</v></c></row><row r="1048576"><c r="XFD1048576"><v>9</v><f>1</f></c></row></sheetData></worksheet>"#,
        ] {
            let data = replace_entries(&base_fixture(), &[(SHEET, xml)]);
            assert!(preflight(&data).is_ok(), "{xml}");
        }
        let untyped = r#"<worksheet><sheetData><row r="1"><c r="A1"><v>1</v></c></row><row r="1048576"><c r="XFD1048576"><v></v></c></row></sheetData></worksheet>"#;
        let data = replace_entries(&base_fixture(), &[(SHEET, untyped)]);
        assert!(preflight(&data).unwrap_err().contains("矩形"));
    }

    #[test]
    fn test_excel_preflight_shared_strings_unique_count_before_open() {
        let sst = r#"<sst xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" count="1" uniqueCount="4294967295"><si><t>x</t></si></sst>"#;
        let data = upsert_entries(&base_fixture(), &[("xl/sharedStrings.xml", sst)]);
        assert!(preflight(&data).unwrap_err().contains("共享字符串"));
    }
}
