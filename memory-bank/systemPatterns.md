# 系统模式 (System Patterns)

> 写代码前先读这里。架构约定、代码放置位置、负向约束。

## 架构总览

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
│  预览层                                                 │
│   core/excel_reader.rs  calamine + zip + quick-xml      │
│   core/excel_style.rs   OOXML 样式/主题/dxf             │
│   core/html_builder.rs  HTML 渲染 + 转义                │
├─────────────────────────────────────────────────────────┤
│  基础设施                                               │
│   resource.rs   Blob 下载助手 + URL 生命周期            │
│   validation.rs 文件名校验                              │
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
│   ├── style.rs            CellStyle / StyleSheet / 颜色归一化
│   ├── excel_reader.rs     Excel 解析（预览）
│   ├── excel_style.rs      OOXML 样式表解析
│   └── html_builder.rs     预览 HTML 构建
├── batch_export.rs         CSV 分批
├── batch_export_xlsx.rs    XLSX 分批
├── streaming_export.rs     流式 CSV
├── resource.rs             UrlGuard / schedule_url_revoke / 下载助手
├── utils.rs                通用工具
└── validation.rs           文件名安全

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

`core/export_xlsx.rs`：

- `write_sheet(worksheet, table_data, freeze_pane, progress)` — 单表写入的唯一实现
- `write_sheet_with_progress(...)` — 供 `batch_export_xlsx` 映射进度区间
- 单表 / 多表 / 分批三条路径都调用它，保证行列上限、合并、冻结行为一致

### 样式三级合并 + 列级缓存

`core/style.rs`：

- `StyleSheet::resolve(row, col, header_row_count)` — 完整三级解析（含单元格覆盖）
- `StyleSheet::resolve_column(is_header, col)` — 只算「全局 + 列级」，**可按列缓存**

`write_sheet` 中：无单元格覆盖的格子走缓存路径，避免逐格 clone + 重建 `Format`。

### DOM 行源统一抽象

`core/table_extractor.rs::TableRowSources`：封装 table + 可选外部 tbody 的解析、
`ensure_external_tbody` 校验、跨源 `get_row(index)`。分批 CSV/XLSX 共用。

### Excel 预览：一次打开，多处复用

`core/excel_reader.rs::parse_excel`：

- calamine workbook 打开 **1 次**（数据 + 合并区域）
- `ZipArchive` 打开 **1 次**（styles.xml + sheet XML）
- 合并区域坐标是**工作表绝对坐标**，边界比较必须加 `range.start()` 偏移

### 安全上限常量

| 常量 | 值 | 位置 |
| ---- | -- | ---- |
| `MAX_DEPTH` | 64 | data_export（递归深度） |
| `MAX_HEADER_CELLS` | 100_000 | data_export |
| `MAX_DATA_CELLS` | 5_000_000 | data_export |
| `EXCEL_MAX_ROW` / `EXCEL_MAX_COL` | 1_048_575 / 16_383 | export_xlsx |
| `MAX_ROWS_LIMIT` / `MAX_COLS_LIMIT` | 100_000 / 16_384 | excel_reader（预览） |
| `MAX_MERGE_EXPAND_CELLS` | 1_000_000 | excel_reader |
| `MAX_XML_SIZE` / `MAX_SHEET_XML_SIZE` | 50 MB | excel_style / excel_reader |
| `DEFAULT_CHUNK_SIZE` | 5000 | streaming_export |

## 负向约束（❌ 不要这么做）

- ❌ **不要在生产路径用 `panic!` / `unwrap()` / `expect()`**，公开 API 返回 `Result<T, JsValue>`
- ❌ **不要先创建 Blob URL 再校验文件名**——错误路径会泄漏 URL
- ❌ **不要按字节长度对 UTF-8 字符串分支**（`"中".len() == 3` 会误入 3 字符分支导致越界）
- ❌ **不要用裸乘法做上限检查**——wasm32 上 `usize` 是 u32，用 `checked_mul`
- ❌ **不要先 `as u16` / `as u32` 再比较上限**——截断会漏检，应在 `usize` 下比较
- ❌ **不要展开超大合并区域到 HashSet**——面积超 `MAX_MERGE_EXPAND_CELLS` 直接跳过
- ❌ **不要用 `write_formula` 或任何会让 Excel 求值的写入**——一律 `write_string`
- ❌ **不要用 `Reflect::get(...).ok()` 静默吞掉 getter 异常**——用 `get_object_property` 并 `?` 传播
- ❌ **不要把条件格式 sqref 展开成坐标集合**——整列会炸内存，用范围表示
- ❌ **不要重复打开 zip / workbook**——复用已有 `ZipArchive` 与 calamine 句柄
- ❌ **不要手改 `pkg/`、`target/`**——构建产物
- ❌ **dxf 填充不要要求 `patternType="solid"`**——Excel 条件格式常省略该属性，缺省即视为 solid，仅 `"none"` 不上色

## 编码约定

- Rust 2024，`rustfmt` 默认配置
- 标识符英文 `snake_case`；注释、文档、错误消息中文
- 公开类型与函数需中文文档注释
- 测试文件 `test_<领域>.rs`，测试函数 `test_<模块>_<函数>_<场景>`
- 仅 wasm32 安全的 `JsValue` 测试要加 `#[cfg(target_arch = "wasm32")]`
- 新功能需覆盖：正常输入、边界值、Unicode、恶意输入
