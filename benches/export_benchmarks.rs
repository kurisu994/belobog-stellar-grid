//! 导出性能基准测试
//!
//! 使用 criterion 框架测试 CSV/XLSX 生成在不同数据规模下的性能

use belobog_stellar_grid::bench_exports::{
    MergeRange, TableData, generate_csv_bytes, generate_xlsx_bytes,
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

/// 生成测试用的二维字符串数据
fn generate_rows(row_count: usize, col_count: usize) -> Vec<Vec<String>> {
    (0..row_count)
        .map(|r| {
            (0..col_count)
                .map(|c| format!("数据_R{}_C{}", r, c))
                .collect()
        })
        .collect()
}

/// 生成包含合并区域的 TableData（模拟嵌套表头场景）
fn generate_table_data_with_merges(
    row_count: usize,
    col_count: usize,
) -> TableData {
    // 1 行表头 + row_count 行数据
    let mut rows = Vec::with_capacity(row_count + 1);

    // 表头行
    let header: Vec<String> = (0..col_count).map(|c| format!("列{}", c + 1)).collect();
    rows.push(header);

    // 数据行
    rows.extend(generate_rows(row_count, col_count));

    // 模拟合并区域：每 10 行第一列 rowSpan=2
    let mut merge_ranges = Vec::new();
    for r in (1..=row_count).step_by(10) {
        if r + 1 <= row_count {
            merge_ranges.push(MergeRange::new(r as u32, 0, (r + 1) as u32, 0));
        }
    }

    TableData {
        rows,
        merge_ranges,
        header_row_count: 1,
    }
}

// =============================================================================
// CSV 生成 Benchmark
// =============================================================================

fn bench_csv_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("csv_generation");

    // 测试不同数据规模
    let configs: Vec<(usize, usize, &str)> = vec![
        (100, 10, "100行x10列"),
        (1_000, 10, "1000行x10列"),
        (5_000, 10, "5000行x10列"),
        (1_000, 50, "1000行x50列"),
    ];

    for (rows, cols, label) in configs {
        let data = generate_rows(rows, cols);

        group.bench_with_input(
            BenchmarkId::new("无BOM", label),
            &data,
            |b, data| {
                b.iter(|| {
                    generate_csv_bytes(data.clone(), None, false, false)
                        .expect("CSV 生成不应失败")
                })
            },
        );
    }

    // BOM 对比测试（使用中等数据集）
    let data_bom = generate_rows(1_000, 10);
    group.bench_with_input(
        BenchmarkId::new("带BOM", "1000行x10列"),
        &data_bom,
        |b, data| {
            b.iter(|| {
                generate_csv_bytes(data.clone(), None, false, true)
                    .expect("CSV 生成不应失败")
            })
        },
    );

    group.finish();
}

// =============================================================================
// XLSX 生成 Benchmark
// =============================================================================

fn bench_xlsx_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("xlsx_generation");
    // XLSX 生成较慢，减少采样时间
    group.sample_size(20);

    let configs: Vec<(usize, usize, &str)> = vec![
        (100, 10, "100行x10列"),
        (1_000, 10, "1000行x10列"),
        (5_000, 10, "5000行x10列"),
        (1_000, 50, "1000行x50列"),
    ];

    for (rows, cols, label) in configs {
        // 无合并区域的简单 TableData
        let table_data = TableData {
            rows: generate_rows(rows, cols),
            merge_ranges: Vec::new(),
            header_row_count: 0,
        };

        group.bench_with_input(
            BenchmarkId::new("无合并", label),
            &table_data,
            |b, data| {
                b.iter(|| {
                    generate_xlsx_bytes(data, None, false, None)
                        .expect("XLSX 生成不应失败")
                })
            },
        );
    }

    // 含合并区域测试
    let merge_data = generate_table_data_with_merges(1_000, 10);
    group.bench_with_input(
        BenchmarkId::new("含合并", "1000行x10列"),
        &merge_data,
        |b, data| {
            b.iter(|| {
                generate_xlsx_bytes(data, None, false, Some((1, 0)))
                    .expect("XLSX 生成不应失败")
            })
        },
    );

    group.finish();
}

// =============================================================================
// CSV 注入转义 Benchmark
// =============================================================================

fn bench_csv_escape(c: &mut Criterion) {
    use belobog_stellar_grid::escape_csv_injection;

    let mut group = c.benchmark_group("csv_escape");

    // 普通文本（无需转义，走零拷贝路径）
    let normal = "普通文本数据";
    group.bench_function("普通文本", |b| {
        b.iter(|| escape_csv_injection(normal))
    });

    // 需要转义的文本
    let dangerous = "=SUM(A1:A10)";
    group.bench_function("公式文本", |b| {
        b.iter(|| escape_csv_injection(dangerous))
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_csv_generation,
    bench_xlsx_generation,
    bench_csv_escape,
);
criterion_main!(benches);
