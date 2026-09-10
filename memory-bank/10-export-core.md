---
name: 10-export-core
description: 导出主干架构、统一写入/下载路径、安全上限常量与公开 WASM API
paths:
  - "src/lib.rs"
  - "src/utils.rs"
  - "src/core/mod.rs"
  - "src/core/export_*.rs"
  - "src/core/table_extractor.rs"
  - "src/core/data_export.rs"
  - "src/core/style.rs"
  - "src/batch_export*.rs"
  - "src/streaming_export.rs"
---

# 导出主干 (Export Core)

> 改 `src/` 下的导出代码前先读这里。架构约定、放置位置、上限常量、负向约束。

## 架构分层

```
┌─────────────────────────────────────────────────────────┐
│  JS / 框架子包 (packages/react|vue|svelte|solid|worker) │
└────────────────────────┬────────────────────────────────┘
                         │ wasm-bindgen
┌────────────────────────▼────────────────────────────────┐
│  src/lib.rs  — 公开 API 再导出                          │
├─────────────────────────────────────────────────────────┤
│  入口层                                                 │
│   core/mod.rs          统一导出入口 + 选项解析          │
│   batch_export.rs      CSV 分批异步                     │
│   batch_export_xlsx.rs XLSX 分批异步（单表/多表）       │
│   streaming_export.rs  流式 CSV                         │
├─────────────────────────────────────────────────────────┤
│  数据层                                                 │
│   core/table_extractor.rs  DOM 提取 + TableRowSources   │
│   core/data_export.rs      JS 数组/树形 → TableData     │
├─────────────────────────────────────────────────────────┤
│  生成层                                                 │
│   core/export_csv.rs   CSV 编码 + 下载                  │
│   core/export_xlsx.rs  write_sheet + 下载               │
│   core/style.rs        StyleSheet 三级样式              │
├─────────────────────────────────────────────────────────┤
│  基础设施                                               │
│   resource.rs   Blob 下载助手 + URL 生命周期（见 12）   │
│   validation.rs 文件名校验（见 12）                     │
│   utils.rs      进度回调 / 让出事件循环 / CSV 转义      │
└─────────────────────────────────────────────────────────┘
```

## 目录约定

```
src/
├── lib.rs                  只做模块声明与公开再导出，不写业务
├── core/
│   ├── mod.rs              wasm_bindgen 入口、选项解析
│   ├── data_export.rs      JS 数据 → TableData（表头/数据/合并/样式）
│   ├── table_extractor.rs  DOM 提取、TableData、TableRowSources
│   ├── export_csv.rs       CSV 字节生成与下载
│   ├── export_xlsx.rs      XLSX 写入与下载
│   └── style.rs            CellStyle / StyleSheet / 颜色归一化
├── batch_export.rs         CSV 分批
├── batch_export_xlsx.rs    XLSX 分批
├── streaming_export.rs     流式 CSV
├── resource.rs             UrlGuard / schedule_url_revoke / 下载助手
├── utils.rs                通用工具
└── validation.rs           文件名安全
```

> `src/core/` 下的预览解析与 HTML 构建见 `11-excel-preview.md`。

仓库级目录：

```
tests/     集成测试 test_<领域>.rs
e2e/       Playwright 浏览器测试
benches/   Criterion 基准
examples/  可直接打开的 HTML 示例
packages/  TS 类型与框架封装
pkg/ target/  构建产物，禁止手改
```

## 关键设计模式

### 下载统一走 resource.rs

```rust
resource::trigger_bytes_download(bytes, mime, filename, default_name, ext)
resource::trigger_blob_download(parts, mime, filename, default_name, ext)
```

内部顺序固定：**先校验文件名 → 再创建 Blob → 再创建 Object URL → 点击 → 延迟 revoke**。
新增导出格式时复用它，不要各自复制一份 DOM 锚点逻辑。

### XLSX 写入统一走 write_sheet

`src/core/export_xlsx.rs`：

- `write_sheet(worksheet, table_data, freeze_pane, progress)` — 单表写入的唯一实现
- `write_sheet_with_progress(...)` — 供 `batch_export_xlsx` 映射进度区间
- 单表 / 多表 / 分批三条路径都调用它，保证行列上限、合并、冻结行为一致

### 样式三级合并 + 列级缓存

`src/core/style.rs`：

- `StyleSheet::resolve(row, col, header_row_count)` — 完整三级解析（含单元格覆盖）
- `StyleSheet::resolve_column(is_header, col)` — 只算「全局 + 列级」，**可按列缓存**

`write_sheet` 中：无单元格覆盖的格子走缓存路径，避免逐格 clone + 重建 `Format`。

### DOM 行源统一抽象

`src/core/table_extractor.rs::TableRowSources`：封装 table + 可选外部 tbody 的解析、
`ensure_external_tbody` 校验、跨源 `get_row(index)`。分批 CSV / XLSX 共用。

## 安全上限常量

| 常量 | 值 | 位置 |
| ---- | -- | ---- |
| `MAX_DEPTH` | 64 | `src/core/data_export.rs`（递归深度） |
| `MAX_HEADER_CELLS` | 100_000 | `src/core/data_export.rs` |
| `MAX_DATA_CELLS` | 5_000_000 | `src/core/data_export.rs` |
| `EXCEL_MAX_ROW` / `EXCEL_MAX_COL` | 1_048_575 / 16_383 | `src/core/export_xlsx.rs` |
| `DEFAULT_CHUNK_SIZE` | 5000 | `src/streaming_export.rs` |

> 预览侧的上限常量（`MAX_ROWS_LIMIT` / `MAX_COLS_LIMIT` / `MAX_MERGE_EXPAND_CELLS` /
> `MAX_XML_SIZE` / `MAX_SHEET_XML_SIZE`）见 `11-excel-preview.md`。

## 公开 API（`src/lib.rs`）

**导出**

- `ExportFormat`（`Csv` 默认 / `Xlsx`）
- `export_table` — DOM 表格导出
- `export_tables_xlsx` — 多表 → 多 Sheet
- `export_data` — JS 数组 / 对象 / 树形数据导出
- `generate_data_bytes` — 仅生成字节（Worker 场景）
- `export_table_to_csv_batch` — CSV 分批异步
- `export_table_to_xlsx_batch` / `export_tables_to_xlsx_batch` — XLSX 分批异步
- `export_data_streaming` — 流式 CSV

**Excel 预览**（实现见 `11-excel-preview.md`）

- `get_excel_sheet_list` / `parse_excel_to_html` / `parse_excel_to_json`

**工具**

- `UrlGuard`、`validate_filename`、`ensure_extension`、`escape_csv_injection`、`set_panic_hook`

**内部（`#[doc(hidden)] bench_exports`）**

- `generate_csv_bytes`、`generate_xlsx_bytes`、`generate_xlsx_multi_bytes`、`MergeRange`、`TableData`

## `export_data` 选项字段

来自 `src/core/mod.rs::ExportDataOptions`：

`columns` / `filename` / `format` / `progressCallback` / `indentColumn` / `childrenKey` /
`withBom` / `strictProgressCallback` / `freezeRows` / `freezeCols` / `headerStyle` / `cellStyle`

流式额外支持 `chunkSize`（默认 5000，最小 1）。

## 负向约束（❌ 不要这么做）

- ❌ **不要在生产路径用 `panic!` / `unwrap()` / `expect()`**，公开 API 返回 `Result<T, JsValue>`
- ❌ **不要先创建 Blob URL 再校验文件名**——错误路径会泄漏 URL
- ❌ **不要按字节长度对 UTF-8 字符串分支**（`"中".len() == 3` 会误入 3 字符分支导致越界）
- ❌ **不要用裸乘法做上限检查**——wasm32 上 `usize` 是 u32，用 `checked_mul`
- ❌ **不要先 `as u16` / `as u32` 再比较上限**——截断会漏检，应在 `usize` 下比较
- ❌ **不要用 `write_formula` 或任何会让 Excel 求值的写入**——一律 `write_string`
- ❌ **不要用 `Reflect::get(...).ok()` 静默吞掉 getter 异常**——用 `get_object_property` 并 `?` 传播
- ❌ **不要重复打开 zip / workbook**——复用已有 `ZipArchive` 与 calamine 句柄
- ❌ **不要手改 `pkg/`、`target/`**——构建产物

## 编码约定

- Rust 2024 + `rustfmt` 默认配置；标识符英文 `snake_case`，注释/文档/错误消息中文
- 完整协作与提交规范见仓库根 `AGENTS.md`（此处不重复），测试命名与运行方式见 `13-testing.md`
