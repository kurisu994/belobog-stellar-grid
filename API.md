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
    with_bom: Option<bool>,
    strict_progress_callback: Option<bool>,
) -> Result<(), JsValue>
```

**参数**
- `table_id`: HTML 表格元素的 ID。
- `filename`: 导出文件名（可选）。不包含扩展名时会自动根据格式添加。
- `format`: 导出格式（可选）。默认为 `ExportFormat.Csv`。
- `exclude_hidden`: 是否排除隐藏（`display: none`）的行和列（可选）。默认为 `false`。
- `progress_callback`: 进度回调函数（可选）。接收一个 0-100 的数字。
- `with_bom`: CSV 导出时是否添加 UTF-8 BOM（可选）。默认为 `false`。添加 BOM 可解决 Excel 打开 CSV 中文乱码问题。
- `strict_progress_callback`: 是否启用严格进度回调模式（可选）。默认为 `false`。启用后，进度回调失败将中止导出并返回错误；未启用时仅 `console.warn`。

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
    - `format`: 导出格式。默认 CSV。只接受 `ExportFormat.Csv`(0) 和 `ExportFormat.Xlsx`(1)，传入其他值将报错。
    - `progressCallback`: 进度回调函数。
    - `indentColumn`: 树形数据模式下，需要缩进的列的 key。
    - `childrenKey`: 指定子节点字段名，启用树形数据模式。
    - `withBom`: CSV 导出时是否添加 UTF-8 BOM。默认 `false`。
    - `strictProgressCallback`: 是否启用严格进度回调模式。默认 `false`。启用后进度回调失败将中止导出。

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
    strict_progress_callback: Option<bool>,
) -> Result<(), JsValue>
```

**参数**
- `sheets`: 配置数组。每个元素包含 `{ tableId: string, sheetName?: string, excludeHidden?: boolean }`。
- `filename`: 导出文件名（可选）。
- `progress_callback`: 进度回调函数（可选）。
- `strict_progress_callback`: 是否启用严格进度回调模式（可选）。默认为 `false`。

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
pub async fn export_table_to_csv_batch(
    table_id: String,
    tbody_id: Option<String>,
    filename: Option<String>,
    batch_size: Option<u32>,
    exclude_hidden: Option<bool>,
    progress_callback: Option<js_sys::Function>,
    with_bom: Option<bool>,
    strict_progress_callback: Option<bool>,
) -> Result<JsValue, JsValue>
```

**参数**
- `table_id`: 表格 ID。
- `tbody_id`: 外部 tbody 元素 ID（可选）。用于虚拟滚动等场景，指定包含实际数据行的 tbody。会在运行时验证该 tbody 是否属于目标 table 内部。
- `filename`: 文件名（可选）。
- `batch_size`: 每批处理行数（可选，默认 1000）。
- `exclude_hidden`: 是否排除隐藏行列（可选，默认 `false`）。
- `progress_callback`: 进度回调。
- `with_bom`: CSV 导出时是否添加 UTF-8 BOM（可选，默认 `false`）。
- `strict_progress_callback`: 是否启用严格进度回调模式（可选）。默认为 `false`。

**返回值**
- `Promise`: 导出完成时 resolve。

---

### `export_table_to_xlsx_batch`

分批异步导出 XLSX。

```rust
pub async fn export_table_to_xlsx_batch(
    table_id: String,
    tbody_id: Option<String>,
    filename: Option<String>,
    batch_size: Option<u32>,
    exclude_hidden: Option<bool>,
    progress_callback: Option<js_sys::Function>,
    strict_progress_callback: Option<bool>,
) -> Result<JsValue, JsValue>
```

**参数**
- `table_id`: 表格 ID。
- `tbody_id`: 外部 tbody 元素 ID（可选）。同 `export_table_to_csv_batch`。
- `filename`: 文件名（可选）。
- `batch_size`: 每批处理行数（可选，默认 1000）。
- `exclude_hidden`: 是否排除隐藏行列（可选，默认 `false`）。
- `progress_callback`: 进度回调。
- `strict_progress_callback`: 是否启用严格进度回调模式（可选）。默认为 `false`。

---

### `export_tables_to_xlsx_batch`

多工作表分批异步导出 XLSX。

```rust
pub async fn export_tables_to_xlsx_batch(
    sheets: JsValue,
    filename: Option<String>,
    batch_size: Option<u32>,
    progress_callback: Option<js_sys::Function>,
    strict_progress_callback: Option<bool>,
) -> Result<JsValue, JsValue>
```

**参数**
- `sheets`: 配置数组。每个元素包含 `{ tableId: string, sheetName?: string, excludeHidden?: boolean, tbodyId?: string }`。
- `filename`: 导出文件名（可选）。
- `batch_size`: 每批处理行数（可选，默认 1000）。
- `progress_callback`: 进度回调函数（可选）。
- `strict_progress_callback`: 是否启用严格进度回调模式（可选）。默认为 `false`。

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

---

## 框架集成包

### `@bsg-export/types`

严格 TypeScript 类型定义（零运行时），替代 wasm-bindgen 自动生成的 `any` 类型。

**核心接口**：`Column`、`ExportDataOptions`、`SheetConfig`、`BatchSheetConfig`、`MergeCellValue`、`ProgressCallback`

```typescript
import type { Column, ExportDataOptions } from '@bsg-export/types';
import { ExportFormat } from '@bsg-export/types';
```

---

### `@bsg-export/react`

React 封装，提供 Hook 和组件。

#### `useExporter(wasmUrl?)`

自动管理 WASM 初始化，返回状态和类型安全的导出方法。

```tsx
import { useExporter, ExportFormat } from '@bsg-export/react';

function App() {
  const { initialized, loading, progress, error, exportTable, exportData } = useExporter();

  return (
    <button
      disabled={!initialized || loading}
      onClick={() => exportTable({ tableId: 'my-table', filename: '报表.xlsx', format: ExportFormat.Xlsx })}
    >
      {loading ? `导出中 ${Math.round(progress)}%` : '导出'}
    </button>
  );
}
```

**返回值**：
- `initialized: boolean` — WASM 是否初始化完成
- `loading: boolean` — 是否正在导出
- `progress: number` — 导出进度 (0-100)
- `error: Error | null` — 错误信息
- `exportTable(options)` — DOM 表格导出
- `exportData(options)` — 纯数据导出
- `exportTablesXlsx(options)` — 多 Sheet 导出
- `exportTableToCsvBatch(options)` — CSV 分批导出
- `exportTableToXlsxBatch(options)` — XLSX 分批导出
- `exportTablesToXlsxBatch(options)` — 多 Sheet 分批导出

#### `<ExportButton>`

开箱即用的导出按钮，自动管理初始化和状态。

```tsx
import { ExportButton, ExportFormat } from '@bsg-export/react';

<ExportButton tableId="my-table" filename="报表.xlsx" format={ExportFormat.Xlsx}>
  导出 Excel
</ExportButton>
```

---

### `@bsg-export/vue`

Vue 3 封装，提供 Composable 和组件。

#### `useExporter(wasmUrl?)`

功能同 React 版本，使用 Vue 3 的 `ref` 响应式状态。

```vue
<script setup>
import { useExporter, ExportFormat } from '@bsg-export/vue';

const { initialized, loading, progress, exportTable } = useExporter();
</script>

<template>
  <button :disabled="!initialized || loading" @click="exportTable({ tableId: 'my-table' })">
    {{ loading ? `导出中 ${Math.round(progress)}%` : '导出' }}
  </button>
</template>
```

#### `<ExportButton>`

Vue 组件，支持插槽和事件。

```vue
<script setup>
import { ExportButton, ExportFormat } from '@bsg-export/vue';
</script>

<template>
  <ExportButton table-id="my-table" filename="报表.xlsx" :format="ExportFormat.Xlsx">
    导出 Excel
  </ExportButton>
</template>
```

---

### `@bsg-export/worker`

Web Worker 导出封装，将 CPU 密集的文件生成移至 Worker 线程，彻底避免主线程阻塞。

#### `generate_data_bytes(data, options?)`

WASM 底层函数。与 `export_data` 功能相同，但不创建 Blob 和下载链接，直接返回文件字节（`Uint8Array`）。

```rust
pub fn generate_data_bytes(
    data: JsValue,
    options: Option<JsValue>,
) -> Result<js_sys::Uint8Array, JsValue>
```

**参数**：同 `export_data`。

**返回值**：`Uint8Array` — 生成的 CSV 或 XLSX 文件字节。

**使用场景**：在 Web Worker 中调用，通过 `postMessage` + Transferable 将字节传回主线程触发下载。

---

#### `ExportWorker` 类

管理 Worker 生命周期的高层封装。

```typescript
import { ExportWorker } from '@bsg-export/worker';

// 1. 创建 Worker（根据构建工具选择方式）
// Vite:
import MyWorker from '@bsg-export/worker/worker?worker';
const worker = new ExportWorker(new MyWorker());

// Webpack 5:
const w = new Worker(new URL('@bsg-export/worker/worker', import.meta.url));
const worker = new ExportWorker(w);

// 2. 初始化 WASM
await worker.init();

// 3. 导出数据（Worker 生成 → 主线程下载）
await worker.exportData(
  [{ name: '张三', age: 28 }],
  {
    columns: [{ title: '姓名', key: 'name' }, { title: '年龄', key: 'age' }],
    filename: '用户.xlsx',
    format: 1, // ExportFormat.Xlsx
  },
  { onProgress: (p) => console.log(`${p}%`) }
);

// 4. 仅生成字节（不触发下载）
const bytes = await worker.generateBytes(data, options);

// 5. 销毁 Worker
worker.terminate();
```

**方法**：
- `init()` — 初始化 Worker 中的 WASM 模块
- `exportData(data, options?, workerOptions?)` — 生成文件并触发下载
- `generateBytes(data, options?, workerOptions?)` — 仅生成文件字节
- `terminate()` — 销毁 Worker

**属性**：
- `initialized: boolean` — WASM 是否初始化完成
