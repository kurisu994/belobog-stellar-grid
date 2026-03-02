/// 流式 CSV 数据导出模块
///
/// 提供基于分块 Blob 拼接的流式 CSV 导出功能，
/// 将 Rust 侧内存峰值从「全部数据的 CSV 字节」降低为「一个分块大小」。
///
/// **注意**：XLSX 格式受 `rust_xlsxwriter` 库限制无法真正流式化，
/// 当 `format=Xlsx` 时会自动回退到 `export_data` 的同步逻辑。
use crate::core::{
    ExportDataOptions, ExportFormat, build_table_data_from_array, build_table_data_from_tree,
    export_data_impl, parse_export_data_options, parse_js_array_data,
};
use crate::utils::{report_progress, yield_to_browser};
use crate::validation::{ensure_extension, validate_filename};
use csv::Writer;
use std::io::Cursor;
use wasm_bindgen::prelude::*;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

/// 默认分块大小（每块行数）
const DEFAULT_CHUNK_SIZE: usize = 5000;

/// 流式导出 JavaScript 数据为 CSV 文件（异步，降低内存峰值）
///
/// 与 `export_data` 功能相同，但采用分块写入策略：
/// 将 CSV 输出按 `chunkSize` 行分块写入，每块转为 `Uint8Array` 后立即释放 Rust 侧内存，
/// 最后用所有分块拼接成单个 `Blob` 触发下载。
///
/// **内存优化**：Rust 侧内存峰值仅为一个分块大小，而非全部数据。
///
/// **XLSX 限制**：当 `format=Xlsx` 时，由于 XLSX 库不支持流式写入，
/// 会自动回退到 `export_data` 的同步逻辑。
///
/// # 参数
/// * `data` - JS 数组（二维数组或对象数组）
/// * `options` - 配置对象（同 `export_data`，额外支持 `chunkSize` 字段）
///   - `chunkSize`: 每个分块包含的行数（默认 5000）
///   - 其他字段同 `export_data` 的 options
///
/// # 返回值
/// * `Promise<void>` - 异步导出完成
///
/// # 示例
/// ```javascript
/// import init, { export_data_streaming, ExportFormat } from './pkg/belobog_stellar_grid.js';
/// await init();
///
/// // 流式 CSV 导出（适合超大数据量）
/// const largeData = generateLargeData(100000); // 10 万行
/// await export_data_streaming(largeData, {
///   columns: [{ title: '姓名', key: 'name' }, { title: '年龄', key: 'age' }],
///   filename: '大数据.csv',
///   chunkSize: 10000, // 每块 1 万行
///   progressCallback: (progress) => {
///     console.log(`进度: ${Math.round(progress)}%`);
///   },
/// });
///
/// // XLSX 格式会自动回退到同步导出
/// await export_data_streaming(largeData, {
///   columns: [{ title: '姓名', key: 'name' }],
///   filename: '报表.xlsx',
///   format: ExportFormat.Xlsx,
/// });
/// ```
#[wasm_bindgen]
pub async fn export_data_streaming(
    data: JsValue,
    options: Option<JsValue>,
) -> Result<JsValue, JsValue> {
    // 解析分块大小（从 options 中提取 chunkSize，默认 5000）
    let chunk_size = extract_chunk_size(&options);

    // 解析其他配置项（复用 export_data 的解析逻辑）
    let opts = parse_export_data_options(options)?;

    // XLSX 不支持流式写入，回退到同步逻辑
    if opts.format == ExportFormat::Xlsx {
        export_data_impl(data, opts)?;
        return Ok(JsValue::UNDEFINED);
    }

    // CSV 流式导出
    let sp = opts.strict_progress;
    let with_bom = opts.with_bom;

    // 构建表格数据（解析 JS 对象 → Rust 二维数组）
    let rows = build_rows_from_data(data, &opts)?;

    let total_rows = rows.len();
    if total_rows == 0 {
        return Err(JsValue::from_str("没有可导出的数据"));
    }

    // 报告初始进度
    if let Some(ref callback) = opts.progress_callback {
        report_progress(callback, 0.0, sp)?;
    }

    // 分块写入 CSV，收集 Uint8Array 片段
    let blob_parts = js_sys::Array::new();

    // 第一个分块包含 BOM（如果需要）
    if with_bom {
        let bom = js_sys::Uint8Array::from(&[0xEF_u8, 0xBB, 0xBF][..]);
        blob_parts.push(&bom);
    }

    let mut processed_rows = 0;

    while processed_rows < total_rows {
        let chunk_end = std::cmp::min(processed_rows + chunk_size, total_rows);

        // 创建当前分块的 CSV Writer
        let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

        // 写入当前分块的行数据
        for row_data in &rows[processed_rows..chunk_end] {
            // 转义 CSV 注入字符
            let safe_row: Vec<_> = row_data
                .iter()
                .map(|cell| crate::utils::escape_csv_injection(cell))
                .collect();
            wtr.write_record(safe_row.iter().map(|s| s.as_ref()))
                .map_err(|e| JsValue::from_str(&format!("写入 CSV 数据失败: {}", e)))?;
        }

        // 完成当前分块的写入
        wtr.flush()
            .map_err(|e| JsValue::from_str(&format!("完成 CSV 分块写入失败: {}", e)))?;

        let csv_data = wtr
            .into_inner()
            .map_err(|e| JsValue::from_str(&format!("获取 CSV 分块数据失败: {}", e)))?;

        let raw = csv_data.into_inner();

        // 将当前分块字节转为 Uint8Array 并加入 Blob 片段列表
        // 此后 raw (Vec<u8>) 被 drop，释放 Rust 侧内存
        if !raw.is_empty() {
            let uint8_array = js_sys::Uint8Array::from(raw.as_slice());
            blob_parts.push(&uint8_array);
        }

        processed_rows = chunk_end;

        // 报告进度
        if let Some(ref callback) = opts.progress_callback {
            let progress = (processed_rows as f64 / total_rows as f64) * 100.0;
            report_progress(callback, progress, sp)?;
        }

        // 分块之间让出控制权给浏览器事件循环
        if processed_rows < total_rows {
            yield_to_browser().await?;
        }
    }

    // 用所有分块片段创建 CSV Blob 并触发下载
    create_and_download_csv_from_parts(&blob_parts, opts.filename)?;

    Ok(JsValue::UNDEFINED)
}

/// 从 options 对象中提取 chunkSize 参数
fn extract_chunk_size(options: &Option<JsValue>) -> usize {
    options
        .as_ref()
        .filter(|opt| !opt.is_null() && !opt.is_undefined())
        .and_then(|opt| {
            js_sys::Reflect::get(opt, &JsValue::from_str("chunkSize"))
                .ok()
                .and_then(|v| v.as_f64())
                .map(|n| (n as usize).max(1)) // 至少为 1
        })
        .unwrap_or(DEFAULT_CHUNK_SIZE)
}

/// 根据配置构建行数据（统一处理三种数据模式）
fn build_rows_from_data(
    data: JsValue,
    opts: &ExportDataOptions,
) -> Result<Vec<Vec<String>>, JsValue> {
    if let Some(ref cols) = opts.columns {
        if let Some(ref ck) = opts.children_key {
            // 树形数据模式
            let table_data =
                build_table_data_from_tree(cols, &data, opts.indent_column.as_deref(), ck)?;
            Ok(table_data.rows)
        } else {
            // 对象数组 + columns 配置
            let table_data = build_table_data_from_array(cols, &data)?;
            Ok(table_data.rows)
        }
    } else {
        // 二维数组模式
        parse_js_array_data(&data)
    }
}

/// 从 Blob 片段数组创建 CSV 文件并触发下载
fn create_and_download_csv_from_parts(
    parts: &js_sys::Array,
    filename: Option<String>,
) -> Result<(), JsValue> {
    let window = web_sys::window().ok_or_else(|| JsValue::from_str("无法获取 window 对象"))?;
    let document = window
        .document()
        .ok_or_else(|| JsValue::from_str("无法获取 document 对象"))?;

    // 创建 CSV Blob 对象（从多个 Uint8Array 片段拼接）
    let blob_property_bag = BlobPropertyBag::new();
    blob_property_bag.set_type("text/csv;charset=utf-8");

    let blob = Blob::new_with_u8_array_sequence_and_options(parts, &blob_property_bag)
        .map_err(|e| JsValue::from_str(&format!("创建 Blob 对象失败: {:?}", e)))?;

    // 创建下载链接
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| JsValue::from_str(&format!("创建下载链接失败: {:?}", e)))?;

    // 设置文件名
    let final_filename = filename.unwrap_or_else(|| "streaming_export.csv".to_string());

    // 验证文件名安全性
    if let Err(e) = validate_filename(&final_filename) {
        return Err(JsValue::from_str(&format!("文件名验证失败: {}", e)));
    }

    let final_filename = ensure_extension(&final_filename, "csv");

    // 创建下载链接元素
    let anchor = document
        .create_element("a")
        .map_err(|e| JsValue::from_str(&format!("创建下载链接元素失败: {:?}", e)))?;
    let anchor = anchor
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|_| JsValue::from_str("创建的元素不是有效的锚点元素"))?;

    anchor.set_href(&url);
    anchor.set_download(&final_filename);
    anchor.click();

    // 延迟释放 Blob URL
    crate::resource::schedule_url_revoke(&window, url);

    Ok(())
}
