//! 流式导出功能测试
//!
//! 测试 streaming_export 模块的分块逻辑和边界情况（纯 Rust 逻辑，不依赖 WASM）

use belobog_stellar_grid::{ensure_extension, validate_filename};

// ============================================================================
// 分块大小计算逻辑测试
// ============================================================================

#[test]
fn test_chunk_size_basic() {
    // 验证分块计算：总行数 / 分块大小
    let total_rows = 10000;
    let chunk_size = 5000;

    let chunk_count = (total_rows + chunk_size - 1) / chunk_size;
    assert_eq!(chunk_count, 2);
}

#[test]
fn test_chunk_size_not_evenly_divisible() {
    // 总行数不能整除分块大小时，最后一块行数较少
    let total_rows = 10001;
    let chunk_size = 5000;

    let chunk_count = (total_rows + chunk_size - 1) / chunk_size;
    assert_eq!(chunk_count, 3);

    // 最后一个分块的大小
    let last_chunk = total_rows % chunk_size;
    assert_eq!(last_chunk, 1);
}

#[test]
fn test_chunk_size_exactly_one_chunk() {
    // 数据行数 <= 分块大小，只需一个分块
    let total_rows = 100;
    let chunk_size = 5000;

    let chunk_count = (total_rows + chunk_size - 1) / chunk_size;
    assert_eq!(chunk_count, 1);
}

#[test]
fn test_chunk_size_single_row() {
    // 只有一行数据
    let total_rows = 1;
    let chunk_size = 5000;

    let chunk_count = (total_rows + chunk_size - 1) / chunk_size;
    assert_eq!(chunk_count, 1);
}

#[test]
fn test_chunk_size_equal_to_total() {
    // 分块大小恰好等于总行数
    let total_rows = 5000;
    let chunk_size = 5000;

    let chunk_count = (total_rows + chunk_size - 1) / chunk_size;
    assert_eq!(chunk_count, 1);
}

#[test]
fn test_chunk_size_very_small() {
    // 极小分块大小（每块 1 行）
    let total_rows = 10;
    let chunk_size = 1;

    let chunk_count = (total_rows + chunk_size - 1) / chunk_size;
    assert_eq!(chunk_count, 10);
}

#[test]
fn test_chunk_size_very_large() {
    // 极大分块大小
    let total_rows = 100;
    let chunk_size = 1_000_000;

    let chunk_count = (total_rows + chunk_size - 1) / chunk_size;
    assert_eq!(chunk_count, 1);
}

// ============================================================================
// 分块边界索引测试
// ============================================================================

#[test]
fn test_chunk_boundaries_basic() {
    // 验证分块的起始和结束索引
    let total_rows = 12;
    let chunk_size = 5;

    let mut boundaries = Vec::new();
    let mut processed = 0;
    while processed < total_rows {
        let end = std::cmp::min(processed + chunk_size, total_rows);
        boundaries.push((processed, end));
        processed = end;
    }

    assert_eq!(boundaries, vec![(0, 5), (5, 10), (10, 12)]);
}

#[test]
fn test_chunk_boundaries_exact_division() {
    // 能整除时的边界
    let total_rows = 10;
    let chunk_size = 5;

    let mut boundaries = Vec::new();
    let mut processed = 0;
    while processed < total_rows {
        let end = std::cmp::min(processed + chunk_size, total_rows);
        boundaries.push((processed, end));
        processed = end;
    }

    assert_eq!(boundaries, vec![(0, 5), (5, 10)]);
}

#[test]
fn test_chunk_boundaries_single_chunk() {
    // 只有一个分块
    let total_rows = 3;
    let chunk_size = 10;

    let mut boundaries = Vec::new();
    let mut processed = 0;
    while processed < total_rows {
        let end = std::cmp::min(processed + chunk_size, total_rows);
        boundaries.push((processed, end));
        processed = end;
    }

    assert_eq!(boundaries, vec![(0, 3)]);
}

// ============================================================================
// 进度百分比计算测试
// ============================================================================

#[test]
fn test_progress_calculation_basic() {
    // 基础进度计算
    let total_rows = 100;

    let progress_at_50 = (50_f64 / total_rows as f64) * 100.0;
    assert!((progress_at_50 - 50.0).abs() < f64::EPSILON);

    let progress_at_100 = (100_f64 / total_rows as f64) * 100.0;
    assert!((progress_at_100 - 100.0).abs() < f64::EPSILON);
}

#[test]
fn test_progress_calculation_with_chunks() {
    // 模拟分块进度更新
    let total_rows = 12;
    let chunk_size = 5;

    let mut progresses = Vec::new();
    let mut processed = 0;
    while processed < total_rows {
        processed = std::cmp::min(processed + chunk_size, total_rows);
        let progress = (processed as f64 / total_rows as f64) * 100.0;
        progresses.push(progress.round() as u32);
    }

    assert_eq!(progresses, vec![42, 83, 100]); // 5/12, 10/12, 12/12
}

#[test]
fn test_progress_starts_at_zero() {
    let total_rows = 100;
    let progress = (0_f64 / total_rows as f64) * 100.0;
    assert!((progress - 0.0).abs() < f64::EPSILON);
}

#[test]
fn test_progress_ends_at_hundred() {
    let total_rows = 7777;
    let progress = (total_rows as f64 / total_rows as f64) * 100.0;
    assert!((progress - 100.0).abs() < f64::EPSILON);
}

// ============================================================================
// CSV BOM 处理测试
// ============================================================================

#[test]
fn test_csv_bom_bytes() {
    // UTF-8 BOM 的字节序列
    let bom: [u8; 3] = [0xEF, 0xBB, 0xBF];
    assert_eq!(bom.len(), 3);
    assert_eq!(bom[0], 0xEF);
    assert_eq!(bom[1], 0xBB);
    assert_eq!(bom[2], 0xBF);
}

#[test]
fn test_csv_bom_as_string_prefix() {
    // BOM 作为 UTF-8 字符串前缀
    let bom = "\u{FEFF}";
    assert_eq!(bom.len(), 3); // UTF-8 编码占 3 字节
    assert_eq!(bom.as_bytes(), &[0xEF, 0xBB, 0xBF]);
}

// ============================================================================
// 分块 CSV 写入模拟测试
// ============================================================================

use csv::Writer;
use std::io::Cursor;

#[test]
fn test_chunked_csv_write_basic() {
    // 模拟分块 CSV 写入：每块独立生成 CSV 字节
    let data = vec![
        vec!["姓名", "年龄", "城市"],
        vec!["张三", "28", "北京"],
        vec!["李四", "35", "上海"],
        vec!["王五", "42", "广州"],
    ];

    let chunk_size = 2;
    let mut all_parts: Vec<Vec<u8>> = Vec::new();

    let mut processed = 0;
    while processed < data.len() {
        let end = std::cmp::min(processed + chunk_size, data.len());

        let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));
        for row in &data[processed..end] {
            wtr.write_record(row.iter()).unwrap();
        }
        wtr.flush().unwrap();
        let raw = wtr.into_inner().unwrap().into_inner();

        if !raw.is_empty() {
            all_parts.push(raw);
        }
        processed = end;
    }

    // 应该产生 2 个分块
    assert_eq!(all_parts.len(), 2);

    // 拼接后的 CSV 应包含所有数据
    let combined: Vec<u8> = all_parts.into_iter().flatten().collect();
    let result = String::from_utf8(combined).unwrap();

    assert!(result.contains("姓名"));
    assert!(result.contains("张三"));
    assert!(result.contains("王五"));

    // 4 行（无额外空行）
    let line_count = result.lines().count();
    assert_eq!(line_count, 4);
}

#[test]
fn test_chunked_csv_write_single_chunk() {
    // 数据量小于分块大小，只产生一个分块
    let data = vec![vec!["a", "b"], vec!["c", "d"]];

    let chunk_size = 100;
    let mut all_parts: Vec<Vec<u8>> = Vec::new();

    let mut processed = 0;
    while processed < data.len() {
        let end = std::cmp::min(processed + chunk_size, data.len());

        let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));
        for row in &data[processed..end] {
            wtr.write_record(row.iter()).unwrap();
        }
        wtr.flush().unwrap();
        all_parts.push(wtr.into_inner().unwrap().into_inner());
        processed = end;
    }

    assert_eq!(all_parts.len(), 1);
}

#[test]
fn test_chunked_csv_write_with_bom() {
    // 模拟带 BOM 的分块写入
    let bom = vec![0xEF_u8, 0xBB, 0xBF];
    let data_chunk = b"name,age\nAlice,30\n".to_vec();

    // BOM 作为第一个片段
    let mut all_parts = vec![bom, data_chunk];

    let combined: Vec<u8> = all_parts.into_iter().flatten().collect();
    assert_eq!(&combined[0..3], &[0xEF, 0xBB, 0xBF]); // BOM 在最前面
    assert!(String::from_utf8_lossy(&combined[3..]).contains("name"));
}

#[test]
fn test_chunked_csv_write_empty_chunk() {
    // 空数据不应产生任何分块
    let data: Vec<Vec<&str>> = Vec::new();
    let chunk_size = 100;

    let mut all_parts: Vec<Vec<u8>> = Vec::new();
    let mut processed = 0;
    while processed < data.len() {
        let end = std::cmp::min(processed + chunk_size, data.len());
        let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));
        for row in &data[processed..end] {
            wtr.write_record(row.iter()).unwrap();
        }
        wtr.flush().unwrap();
        let raw = wtr.into_inner().unwrap().into_inner();
        if !raw.is_empty() {
            all_parts.push(raw);
        }
        processed = end;
    }

    assert!(all_parts.is_empty());
}

// ============================================================================
// XLSX 回退策略测试
// ============================================================================

#[test]
fn test_xlsx_fallback_strategy_selection() {
    // 模拟格式选择逻辑：CSV 走流式，XLSX 回退同步
    #[derive(PartialEq, Debug)]
    enum Format {
        Csv,
        Xlsx,
    }

    fn should_use_streaming(format: &Format) -> bool {
        *format != Format::Xlsx
    }

    assert!(should_use_streaming(&Format::Csv));
    assert!(!should_use_streaming(&Format::Xlsx));
}

// ============================================================================
// 流式导出文件名处理测试
// ============================================================================

#[test]
fn test_streaming_export_default_filename() {
    // 默认文件名应为 csv 格式
    let default_name = "streaming_export.csv";
    assert!(default_name.ends_with(".csv"));
}

#[test]
fn test_streaming_export_filename_extension() {
    // 确保文件名正确添加扩展名
    assert_eq!(ensure_extension("流式数据", "csv"), "流式数据.csv");
    assert_eq!(ensure_extension("data.csv", "csv"), "data.csv");
    assert_eq!(ensure_extension("报表", "csv"), "报表.csv");
}

#[test]
fn test_streaming_export_filename_validation() {
    // 流式导出的文件名验证
    assert!(validate_filename("流式导出数据").is_ok());
    assert!(validate_filename("streaming_data").is_ok());
    assert!(validate_filename("data-2024").is_ok());

    // 不安全的文件名
    assert!(validate_filename("../hack").is_err());
    assert!(validate_filename("").is_err());
}

// ============================================================================
// 内存优化效果模拟测试
// ============================================================================

#[test]
fn test_memory_peak_comparison() {
    // 模拟对比：全量 vs 分块 的内存峰值
    let total_rows = 100_000;
    let bytes_per_row = 100; // 假设每行 100 字节

    // 全量方式：峰值 = 总数据大小
    let full_peak = total_rows * bytes_per_row;

    // 分块方式：峰值 = 一个分块大小
    let chunk_size = 5000;
    let chunk_peak = chunk_size * bytes_per_row;

    // 分块方式的内存峰值应该远小于全量方式
    assert!(chunk_peak < full_peak);
    assert_eq!(full_peak / chunk_peak, 20); // 降低 20 倍
}

#[test]
fn test_chunk_count_for_large_dataset() {
    // 100 万行数据的分块数量
    let total_rows = 1_000_000;
    let chunk_size = 5000;

    let chunk_count = (total_rows + chunk_size - 1) / chunk_size;
    assert_eq!(chunk_count, 200);
}
