---
name: 11-excel-preview
description: Excel 在线预览的 OOXML 解析、合并坐标、条件格式与安全上限
paths:
  - "src/core/excel_reader.rs"
  - "src/core/excel_style.rs"
  - "src/core/html_builder.rs"
---

# Excel 预览 (Excel Preview)

> 改 `src/core/excel_reader.rs` / `excel_style.rs` / `html_builder.rs` 前先读这里。

## 技术选型

预览**不引入重量级解析器**，而是手工分层：

| 层 | 依赖 | 职责 |
| -- | ---- | ---- |
| 数据 + 合并区域 | `calamine` | 单元格值、工作表列表、合并区域 |
| 容器解包 | `zip`（`default-features = false` + `deflate`） | 取 `styles.xml`、sheet XML |
| OOXML 解析 | `quick-xml` | 样式表、维度、条件格式 |

这样做的目的是控制 WASM 体积：全量 OOXML 解析器会把包体积推高到不可接受。

## 入口

`src/core/excel_reader.rs::parse_excel` 是预览的唯一入口，
`parse_excel_to_html` 与 `parse_excel_to_json` 都走它，区别只在 `src/core/html_builder.rs` 的输出层。

`get_excel_sheet_list` 单独提供工作表列表（含隐藏过滤）。

## 关键设计模式

### 一次打开，多处复用

- calamine workbook 打开 **1 次**（数据 + 合并区域）
- `ZipArchive` 打开 **1 次**（`styles.xml` + sheet XML），由 `ExcelStyleSheet::from_zip_archive`
  与 `parse_sheet_dimensions_from_archive` 共享
- 合并区域从已打开的 calamine workbook 读取，不重新解包

> 历史教训：早期版本打开 4 次 zip/workbook，是明显的性能与内存浪费。

### 合并区域坐标是工作表绝对坐标

`build_merge_map` 与 `build_skip_set` 的边界比较必须加 `range.start()` 偏移，
写作 `data_* + start`。`range.start() != (0,0)` 时若用相对坐标，后半段合并会被误丢。

### 条件格式用范围表示，不展开坐标

```rust
CellRange { first_row, first_col, last_row, last_col }
```

整列条件格式 `sqref="A1:D1048576"` 若展开成坐标集合，极端情况生成上百亿个坐标，瞬间撑爆 WASM 内存。
保留矩形范围后，内存与命中判断都从 O(单元格数) 降为 O(范围数)。

`parse_cell_ref` 也要防两处溢出：`"A0"` 下溢、超长列名的 u32 溢出。

### dxf 填充的 `patternType` 语义

Excel 条件格式常省略 `patternType`，缺省即视为 `patternType="solid"`，仅显式 `"none"` 才不上色。

### 自闭合元素

`<xf ... />` 这类自闭合标签必须与 `<xf>...</xf>` 一并收集，否则样式索引整体偏移。
`parse_sheet_xml` 已把 Start 与 Empty 事件合并处理。

### 数字格式

`Data::Int` 也要套用数字格式；`format_number` 需处理 NaN / Inf 与 `"0,"` 缩放；
`apply_tint` 注意溢出。

## 隐藏行列与 xls 兼容

- 隐藏行列（xlsx `hidden="1"` / 导出侧 `display:none`）不渲染；跨度计算要扣除隐藏列，
  仅当合并原点可见时才保留该合并
- 含隐藏 Sheet 时 `activeSheet` 存的是**可见列表位置**，传给 WASM 前必须区分
  可见列表索引与原始 workbook 索引，否则预览错位
- `.xls` 走 `open_workbook_auto_from_rs`，降级路径与 `.xlsx` 不同

## 安全上限常量

| 常量 | 值 | 位置 |
| ---- | -- | ---- |
| `MAX_ROWS_LIMIT` / `MAX_COLS_LIMIT` | 100_000 / 16_384 | `src/core/excel_reader.rs`（预览） |
| `MAX_MERGE_EXPAND_CELLS` | 1_000_000 | `src/core/excel_reader.rs` |
| `MAX_SHEET_XML_SIZE` | 50 MB | `src/core/excel_reader.rs` |
| `MAX_XML_SIZE` | 50 MB | `src/core/excel_style.rs` |

## 负向约束（❌ 不要这么做）

- ❌ **不要展开超大合并区域到 HashSet**——面积超 `MAX_MERGE_EXPAND_CELLS` 直接跳过
- ❌ **不要把条件格式 sqref 展开成坐标集合**——整列会炸内存，用范围表示
- ❌ **不要重复打开 zip / workbook**——复用已有 `ZipArchive` 与 calamine 句柄
- ❌ **不要用相对坐标比较合并边界**——一律加 `range.start()` 转绝对坐标
- ❌ **dxf 填充不要要求 `patternType="solid"`**——缺省即 solid，仅 `"none"` 不上色
- ❌ **不要把预览 HTML 的文本/样式直出**——文本实体转义，`style` 值过滤 `;{}<>`
