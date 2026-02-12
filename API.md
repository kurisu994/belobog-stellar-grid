# API Reference

所有核心功能都通过 `belobog-stellar-grid` 模块导出。

## 核心函数

### `export_table`

统一的表格导出函数，支持 CSV 和 XLSX 格式。

```rust
pub fn export_table(
    table_id: &str,
    filename: Option<String>,
    format: Option<ExportFormat>,
    exclude_hidden: Option<bool>,
    progress_callback: Option<js_sys::Function>,
) -> Result<(), JsValue>
```

**参数**
- `table_id`: HTML 表格元素的 ID。
- `filename`: 导出文件名（可选）。不包含扩展名时会自动根据格式添加。
- `format`: 导出格式（可选）。默认为 `ExportFormat.Csv`。
- `exclude_hidden`: 是否排除隐藏（`display: none`）的行和列（可选）。默认为 `false`。
- `progress_callback`: 进度回调函数（可选）。接收一个 0-100 的数字。

**返回值**
- `Result<(), JsValue>`: 成功返回 `Ok(())`，失败返回错误信息。

**示例**
```javascript
import { export_table, ExportFormat } from 'belobog-stellar-grid';

// 导出为 CSV
export_table('my-table', 'data.csv');

// 导出为 Excel，带进度条
export_table('my-table', 'report', ExportFormat.Xlsx, true, (progress) => {
    console.log(`Progress: ${progress}%`);
});
```

---

### `export_data`

不依赖 DOM，直接将 JavaScript 数组导出为文件。支持二维数组和对象数组（配合 `columns` 配置）。

```rust
pub fn export_data(data: JsValue, options: Option<JsValue>) -> Result<(), JsValue>
```

**参数**
- `data`: JS 数组。可以是二维数组 `Array<Array<any>>` 或对象数组 `Array<Object>`。
- `options`: 配置对象（可选）。
    - `columns`: 表头配置数组。导出对象数组时必填。支持嵌套 `children` 实现多级表头。
    - `filename`: 导出文件名。
    - `format`: 导出格式。默认 CSV。
    - `progressCallback`: 进度回调函数。
    - `indentColumn`: 树形数据模式下，需要缩进的列的 key。
    - `childrenKey`: 指定子节点字段名，启用树形数据模式。

**返回值**
- `Result<(), JsValue>`

**示例**
```javascript
import { export_data, ExportFormat } from 'belobog-stellar-grid';

// 二维数组
const data = [['Name', 'Age'], ['Alice', 20]];
export_data(data, { filename: 'users.csv' });

// 对象数组 + 树形结构
const treeData = [{ name: 'Root', children: [{ name: 'Child' }] }];
const columns = [{ title: 'Name', key: 'name' }];
export_data(treeData, {
    columns,
    childrenKey: 'children',
    indentColumn: 'name',
    format: ExportFormat.Xlsx
});
```

---

### `export_tables_xlsx`

将多个 HTML 表格导出到同一个 Excel 文件的不同工作表。

```rust
pub fn export_tables_xlsx(
    sheets: JsValue,
    filename: Option<String>,
    progress_callback: Option<js_sys::Function>,
) -> Result<(), JsValue>
```

**参数**
- `sheets`: 配置数组。每个元素包含 `{ tableId: string, sheetName?: string, excludeHidden?: boolean }`。
- `filename`: 导出文件名（可选）。
- `progress_callback`: 进度回调函数（可选）。

**示例**
```javascript
import { export_tables_xlsx } from 'belobog-stellar-grid';

export_tables_xlsx([
    { tableId: 't1', sheetName: 'Summary' },
    { tableId: 't2', sheetName: 'Details' }
], 'report.xlsx');
```

---

### `export_table_to_csv_batch`

分批异步导出 CSV，适用于大数据量，避免阻塞 UI。

```rust
pub fn export_table_to_csv_batch(
    table_id: String,
    filename: String,
    batch_size: Option<usize>,
    progress_callback: Option<js_sys::Function>,
) -> Result<Promise, JsValue>
```

**参数**
- `table_id`: 表格 ID。
- `filename`: 文件名。
- `batch_size`: 每批处理行数（可选，默认 1000）。
- `progress_callback`: 进度回调。

**返回值**
- `Promise`: 导出完成时 resolve。

---

### `export_table_to_xlsx_batch`

分批异步导出 XLSX。

```rust
pub fn export_table_to_xlsx_batch(
    table_id: String,
    filename: String,
    batch_size: Option<usize>,
    progress_callback: Option<js_sys::Function>,
) -> Result<Promise, JsValue>
```

---

### `export_tables_to_xlsx_batch`

多工作表分批异步导出 XLSX。

```rust
pub fn export_tables_to_xlsx_batch(
    sheets: JsValue,
    filename: String,
    batch_size: Option<usize>,
    progress_callback: Option<js_sys::Function>,
) -> Result<Promise, JsValue>
```

## 类型定义

### `ExportFormat`

导出格式枚举。

- `Csv` (0)
- `Xlsx` (1)

可以通过模块导出的常量使用：
```javascript
import { ExportFormat } from 'belobog-stellar-grid';
console.log(ExportFormat.Csv); // 0
console.log(ExportFormat.Xlsx); // 1
```
