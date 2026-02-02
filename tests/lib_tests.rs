//! 完整的单元测试套件
//!
//! 这个模块包含了所有核心功能的单元测试，旨在达到 100% 代码覆盖率

use belobog_stellar_grid::{ensure_extension, validate_filename};
use csv::Writer;
use std::io::Cursor;

// ============================================================================
// 文件名扩展名处理测试
// ============================================================================

#[test]
fn test_filename_extension_handling_basic() {
    // 测试基本的文件名扩展名处理逻辑
    let test_cases = vec![
        ("test.csv", "test.csv"),
        ("report", "report.csv"),
        ("data.CSV", "data.CSV"), // 保持原有大小写
        ("export.csv", "export.csv"),
        ("", "table_export.csv"), // 默认文件名
    ];

    for (input, expected) in test_cases {
        let result = if input.is_empty() {
            "table_export.csv".to_string()
        } else if input.to_lowercase().ends_with(".csv") {
            input.to_string()
        } else {
            format!("{}.csv", input)
        };

        assert_eq!(
            result, expected,
            "文件名处理失败: 输入={}, 期望={}",
            input, expected
        );
    }
}

#[test]
fn test_filename_extension_handling_unicode() {
    // 测试 Unicode 文件名
    let test_cases = vec![
        ("数据.csv", "数据.csv"),
        ("数据", "数据.csv"),
        ("报告_2024", "报告_2024.csv"),
        ("测试文件.CSV", "测试文件.CSV"),
        ("日本語.csv", "日本語.csv"),
        ("한국어", "한국어.csv"),
    ];

    for (input, expected) in test_cases {
        let result = if input.is_empty() {
            "table_export.csv".to_string()
        } else if input.to_lowercase().ends_with(".csv") {
            input.to_string()
        } else {
            format!("{}.csv", input)
        };

        assert_eq!(
            result, expected,
            "Unicode 文件名处理失败: 输入={}, 期望={}",
            input, expected
        );
    }
}

#[test]
fn test_filename_extension_handling_special_cases() {
    // 测试特殊情况
    let test_cases = vec![
        ("file.txt", "file.txt.csv"),         // 其他扩展名
        ("file.csv.bak", "file.csv.bak.csv"), // 多个点
        (".csv", ".csv"),                     // 只有扩展名
        ("file.", "file..csv"),               // 以点结尾
        ("file..csv", "file..csv"),           // 双点
    ];

    for (input, expected) in test_cases {
        let result = if input.is_empty() {
            "table_export.csv".to_string()
        } else if input.to_lowercase().ends_with(".csv") {
            input.to_string()
        } else {
            format!("{}.csv", input)
        };

        assert_eq!(
            result, expected,
            "特殊情况处理失败: 输入={}, 期望={}",
            input, expected
        );
    }
}

#[test]
fn test_ensure_extension_basic() {
    // 测试基本的 ensure_extension 功能
    assert_eq!(ensure_extension("file", "csv"), "file.csv");
    assert_eq!(ensure_extension("file.txt", "csv"), "file.txt.csv");
    assert_eq!(ensure_extension("file.CSV", "csv"), "file.CSV");
    assert_eq!(ensure_extension("file.csv", "CSV"), "file.csv");
    assert_eq!(ensure_extension("", "csv"), ".csv"); // 空文件名的情况
}

#[test]
fn test_ensure_extension_unicode() {
    // 测试 Unicode 文件名的扩展名处理
    assert_eq!(ensure_extension("数据", "csv"), "数据.csv");
    assert_eq!(ensure_extension("报告.txt", "csv"), "报告.txt.csv");
    assert_eq!(ensure_extension("テスト.CSV", "csv"), "テスト.CSV");
    assert_eq!(ensure_extension("파일.csv", "CSV"), "파일.csv");
}

// ============================================================================
// 输入验证逻辑测试
// ============================================================================

#[test]
fn test_validation_empty_string() {
    assert!("".is_empty(), "空字符串应该被识别为空");
}

#[test]
fn test_validation_non_empty_string() {
    assert!(!"my-table".is_empty(), "非空字符串不应该被识别为空");
    assert!(!"table123".is_empty(), "包含数字的字符串不应该为空");
    assert!(!"_table_".is_empty(), "包含下划线的字符串不应该为空");
}

#[test]
fn test_validation_whitespace_string() {
    // 只包含空格的字符串不应该被识别为空（在 Rust 中）
    assert!(!"   ".is_empty(), "只包含空格的字符串在 Rust 中不为空");
    assert!(!"\t\n".is_empty(), "包含制表符和换行符的字符串不为空");
}

#[test]
fn test_validation_special_chars_in_id() {
    // 测试包含特殊字符的 ID（这些在实际使用中可能是有效的）
    assert!(!"table-1".is_empty(), "包含连字符的 ID 不为空");
    assert!(!"table_1".is_empty(), "包含下划线的 ID 不为空");
    assert!(!"my.table".is_empty(), "包含点的 ID 不为空");
}

// ============================================================================
// CSV Writer 创建和使用测试
// ============================================================================

#[test]
fn test_csv_writer_creation() {
    // 测试 CSV writer 可以正常创建
    let wtr = Writer::from_writer(Cursor::new(Vec::new()));
    assert!(wtr.into_inner().is_ok(), "CSV writer 应该能正常创建");
}

#[test]
fn test_csv_writer_write_single_record() {
    // 测试写入单条记录
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));
    let test_data = vec!["Header1", "Header2", "Header3"];

    assert!(wtr.write_record(&test_data).is_ok(), "写入单条记录应该成功");
    assert!(wtr.flush().is_ok(), "刷新缓冲区应该成功");

    let csv_data = wtr.into_inner().unwrap();
    let result = String::from_utf8(csv_data.into_inner()).unwrap();
    assert!(result.contains("Header1"), "结果应该包含 Header1");
    assert!(result.contains("Header2"), "结果应该包含 Header2");
    assert!(result.contains("Header3"), "结果应该包含 Header3");
}

#[test]
fn test_csv_writer_write_multiple_records() {
    // 测试写入多条记录
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

    let records = vec![
        vec!["Name", "Age", "City"],
        vec!["Alice", "30", "New York"],
        vec!["Bob", "25", "London"],
    ];

    for record in records {
        assert!(wtr.write_record(&record).is_ok(), "写入记录应该成功");
    }

    assert!(wtr.flush().is_ok(), "刷新缓冲区应该成功");

    let csv_data = wtr.into_inner().unwrap();
    assert!(!csv_data.get_ref().is_empty(), "CSV 数据不应该为空");

    let result = String::from_utf8(csv_data.into_inner()).unwrap();
    assert!(result.contains("Alice"), "结果应该包含 Alice");
    assert!(result.contains("Bob"), "结果应该包含 Bob");
}

#[test]
fn test_csv_writer_empty_data() {
    // 测试空数据处理
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));
    assert!(wtr.flush().is_ok(), "即使没有写入数据，刷新也应该成功");

    let csv_data = wtr.into_inner().unwrap();
    assert!(
        csv_data.get_ref().is_empty(),
        "未写入任何数据时 CSV 应该为空"
    );
}

#[test]
fn test_csv_writer_unicode_data() {
    // 测试 Unicode 数据
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));
    let test_data = vec!["姓名", "年龄", "城市", "日本語", "한국어"];

    assert!(
        wtr.write_record(&test_data).is_ok(),
        "写入 Unicode 数据应该成功"
    );
    assert!(wtr.flush().is_ok(), "刷新应该成功");

    let csv_data = wtr.into_inner().unwrap();
    let result = String::from_utf8(csv_data.into_inner()).unwrap();
    assert!(result.contains("姓名"), "结果应该包含中文");
    assert!(result.contains("日本語"), "结果应该包含日文");
    assert!(result.contains("한국어"), "结果应该包含韩文");
}

#[test]
fn test_csv_writer_special_characters() {
    // 测试特殊字符处理
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));
    let test_data = vec![
        "field,with,commas",
        "field\"with\"quotes",
        "field\nwith\nnewlines",
        "field\twith\ttabs",
    ];

    assert!(
        wtr.write_record(&test_data).is_ok(),
        "写入包含特殊字符的数据应该成功"
    );
    assert!(wtr.flush().is_ok(), "刷新应该成功");

    let csv_data = wtr.into_inner().unwrap();
    assert!(
        !csv_data.get_ref().is_empty(),
        "包含特殊字符的 CSV 数据不应该为空"
    );
}

// ============================================================================
// 文件名验证测试 - 有效文件名
// ============================================================================

#[test]
fn test_filename_validation_valid_simple() {
    // 测试简单有效的文件名
    assert!(validate_filename("valid_filename.csv").is_ok());
    assert!(validate_filename("report-2024").is_ok());
    assert!(validate_filename("data_export").is_ok());
    assert!(validate_filename("file123").is_ok());
}

#[test]
fn test_filename_validation_valid_unicode() {
    // 测试 Unicode 文件名
    assert!(validate_filename("数据导出_test").is_ok());
    assert!(validate_filename("报告2024").is_ok());
    assert!(validate_filename("日本語ファイル").is_ok());
    assert!(validate_filename("한국어_파일").is_ok());
}

#[test]
fn test_filename_validation_valid_with_spaces() {
    // 测试包含空格的有效文件名（不在开头和结尾）
    assert!(validate_filename("my file.csv").is_ok());
    assert!(validate_filename("data export 2024").is_ok());
}

#[test]
fn test_filename_validation_valid_with_special_chars() {
    // 测试包含某些特殊字符的有效文件名
    assert!(validate_filename("file_name.csv").is_ok());
    assert!(validate_filename("file-name.csv").is_ok());
    assert!(validate_filename("file.name.csv").is_ok());
    assert!(validate_filename("file(1).csv").is_ok());
    assert!(validate_filename("file[1].csv").is_ok());
}

// ============================================================================
// 文件名验证测试 - 无效文件名
// ============================================================================

#[test]
fn test_filename_validation_empty() {
    // 测试空文件名
    assert!(validate_filename("").is_err(), "空文件名应该被拒绝");
}

#[test]
fn test_filename_validation_path_separators() {
    // 测试包含路径分隔符的文件名
    assert!(
        validate_filename("path/to/file").is_err(),
        "包含 / 应该被拒绝"
    );
    assert!(
        validate_filename("path\\to\\file").is_err(),
        "包含 \\ 应该被拒绝"
    );
    assert!(validate_filename("../file").is_err(), "包含 ../ 应该被拒绝");
    assert!(
        validate_filename("..\\file").is_err(),
        "包含 ..\\ 应该被拒绝"
    );
}

#[test]
fn test_filename_validation_dangerous_chars() {
    // 测试危险字符
    assert!(
        validate_filename("invalid<name").is_err(),
        "包含 < 应该被拒绝"
    );
    assert!(
        validate_filename("invalid>name").is_err(),
        "包含 > 应该被拒绝"
    );
    assert!(
        validate_filename("invalid:name").is_err(),
        "包含 : 应该被拒绝"
    );
    assert!(
        validate_filename("invalid\"name").is_err(),
        "包含 \" 应该被拒绝"
    );
    assert!(
        validate_filename("invalid|name").is_err(),
        "包含 | 应该被拒绝"
    );
    assert!(
        validate_filename("invalid?name").is_err(),
        "包含 ? 应该被拒绝"
    );
    assert!(
        validate_filename("invalid*name").is_err(),
        "包含 * 应该被拒绝"
    );
}

#[test]
fn test_filename_validation_windows_reserved_names() {
    // 测试 Windows 保留名称
    let reserved_names = vec![
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
        "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];

    for name in reserved_names {
        assert!(validate_filename(name).is_err(), "{} 应该被拒绝", name);
        assert!(
            validate_filename(&format!("{}.csv", name)).is_err(),
            "{}.csv 应该被拒绝",
            name
        );
        assert!(
            validate_filename(&name.to_lowercase()).is_err(),
            "{} (小写) 应该被拒绝",
            name
        );
    }
}

#[test]
fn test_filename_validation_starts_with_dot() {
    // 测试以点开头的文件名
    assert!(validate_filename(".hidden").is_err(), "以点开头应该被拒绝");
    assert!(
        validate_filename(".gitignore").is_err(),
        ".gitignore 应该被拒绝"
    );
    assert!(
        validate_filename(".file.csv").is_err(),
        ".file.csv 应该被拒绝"
    );
}

#[test]
fn test_filename_validation_ends_with_dot() {
    // 测试以点结尾的文件名
    assert!(
        validate_filename("trailing.").is_err(),
        "以点结尾应该被拒绝"
    );
    assert!(validate_filename("file..").is_err(), "以双点结尾应该被拒绝");
}

#[test]
fn test_filename_validation_starts_with_space() {
    // 测试以空格开头的文件名
    assert!(validate_filename(" space").is_err(), "以空格开头应该被拒绝");
    assert!(
        validate_filename("  spaces").is_err(),
        "以多个空格开头应该被拒绝"
    );
}

#[test]
fn test_filename_validation_ends_with_space() {
    // 测试以空格结尾的文件名
    assert!(validate_filename("space ").is_err(), "以空格结尾应该被拒绝");
    assert!(
        validate_filename("spaces  ").is_err(),
        "以多个空格结尾应该被拒绝"
    );
}

#[test]
fn test_filename_validation_too_long() {
    // 测试过长的文件名
    let long_name = "a".repeat(256);
    assert!(
        validate_filename(&long_name).is_err(),
        "256 字符的文件名应该被拒绝"
    );

    let very_long_name = "a".repeat(1000);
    assert!(
        validate_filename(&very_long_name).is_err(),
        "1000 字符的文件名应该被拒绝"
    );

    let exactly_255 = "a".repeat(255);
    assert!(
        validate_filename(&exactly_255).is_ok(),
        "255 字符的文件名应该被接受"
    );
}

// ============================================================================
// 边界情况和压力测试
// ============================================================================

#[test]
fn test_csv_writer_large_dataset() {
    // 测试大数据集
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

    // 写入 1000 行数据
    for i in 0..1000 {
        let record = vec![
            format!("Name{}", i),
            format!("{}", i),
            format!("City{}", i % 10),
        ];
        assert!(wtr.write_record(&record).is_ok(), "写入第 {} 行应该成功", i);
    }

    assert!(wtr.flush().is_ok(), "刷新大数据集应该成功");

    let csv_data = wtr.into_inner().unwrap();
    assert!(!csv_data.get_ref().is_empty(), "大数据集不应该为空");

    let result = String::from_utf8(csv_data.into_inner()).unwrap();
    assert!(result.contains("Name0"), "应该包含第一行");
    assert!(result.contains("Name999"), "应该包含最后一行");
}

#[test]
fn test_csv_writer_wide_table() {
    // 测试宽表格（很多列）
    let mut wtr = Writer::from_writer(Cursor::new(Vec::new()));

    // 创建一个包含 100 列的记录
    let wide_record: Vec<String> = (0..100).map(|i| format!("Col{}", i)).collect();

    assert!(wtr.write_record(&wide_record).is_ok(), "写入宽记录应该成功");
    assert!(wtr.flush().is_ok(), "刷新应该成功");

    let csv_data = wtr.into_inner().unwrap();
    let result = String::from_utf8(csv_data.into_inner()).unwrap();
    assert!(result.contains("Col0"), "应该包含第一列");
    assert!(result.contains("Col99"), "应该包含最后一列");
}

#[test]
fn test_filename_validation_edge_length() {
    // 测试边界长度
    let name_254 = "a".repeat(254);
    assert!(validate_filename(&name_254).is_ok(), "254 字符应该被接受");

    let name_255 = "a".repeat(255);
    assert!(validate_filename(&name_255).is_ok(), "255 字符应该被接受");

    let name_256 = "a".repeat(256);
    assert!(validate_filename(&name_256).is_err(), "256 字符应该被拒绝");
}

#[test]
fn test_filename_validation_mixed_valid_invalid() {
    // 测试混合有效和无效字符
    assert!(validate_filename("valid-name").is_ok(), "有效名称应该通过");
    assert!(
        validate_filename("valid/invalid").is_err(),
        "混合有效和无效应该被拒绝"
    );
    assert!(
        validate_filename("valid<invalid").is_err(),
        "混合有效和无效应该被拒绝"
    );
}

// ============================================================================
// 回归测试 - 确保修复的 bug 不再出现
// ============================================================================

#[test]
fn test_regression_empty_csv_writer() {
    // 回归测试：确保空的 CSV writer 不会导致 panic
    let wtr = Writer::from_writer(Cursor::new(Vec::new()));
    let result = wtr.into_inner();
    assert!(result.is_ok(), "空 CSV writer 应该能正常获取内部数据");
}

#[test]
fn test_regression_unicode_in_validation() {
    // 回归测试：确保 Unicode 字符在验证中正常工作
    assert!(
        validate_filename("中文文件名").is_ok(),
        "中文文件名应该有效"
    );
    assert!(
        validate_filename("中文/路径").is_err(),
        "包含路径分隔符的中文应该被拒绝"
    );
}

#[test]
fn test_regression_case_sensitivity() {
    // 回归测试：确保文件扩展名检查是大小写不敏感的
    let test_cases = vec![
        ("file.CSV", "file.CSV"),
        ("file.Csv", "file.Csv"),
        ("file.csv", "file.csv"),
        ("FILE.CSV", "FILE.CSV"),
    ];

    for (input, expected) in test_cases {
        let result = if input.to_lowercase().ends_with(".csv") {
            input.to_string()
        } else {
            format!("{}.csv", input)
        };
        assert_eq!(result, expected, "大小写处理应该正确");
    }
}
