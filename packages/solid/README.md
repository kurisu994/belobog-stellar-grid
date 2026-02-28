# @bsg-export/solid

[![npm](https://img.shields.io/npm/v/@bsg-export/solid)](https://www.npmjs.com/package/@bsg-export/solid)

> [belobog-stellar-grid](https://github.com/kurisu994/belobog-stellar-grid) 的 Solid.js 官方封装

## 简介

提供 `createExporter` Primitive、`createWorkerExporter` Primitive 和 `ExportButton` 组件，自动管理 WASM 初始化、导出状态和进度追踪。`createWorkerExporter` 支持将导出计算移至 Worker 线程。

## 安装

```bash
npm install @bsg-export/solid belobog-stellar-grid
# 或
pnpm add @bsg-export/solid belobog-stellar-grid
```

**前置依赖**：`solid-js >= 1.8.0`、`belobog-stellar-grid >= 1.0.0`

## 快速开始

### createExporter Primitive

```tsx
import { createExporter, ExportFormat } from '@bsg-export/solid';

function App() {
  const { initialized, loading, progress, error, exportTable, exportData } = createExporter();

  return (
    <button
      disabled={!initialized() || loading()}
      onClick={() =>
        exportTable({
          tableId: 'my-table',
          filename: '报表.xlsx',
          format: ExportFormat.Xlsx,
        })
      }
    >
      {loading() ? `导出中 ${Math.round(progress())}%` : '导出 Excel'}
    </button>
  );
}
```

### ExportButton 组件

```tsx
import { ExportButton, ExportFormat } from '@bsg-export/solid';

<ExportButton
  tableId="my-table"
  filename="报表.xlsx"
  format={ExportFormat.Xlsx}
  onExportSuccess={() => console.log('导出成功')}
  onExportError={(err) => console.error('导出失败', err)}
>
  导出 Excel
</ExportButton>
```

## API

### `createExporter()` 返回值

所有状态属性均为 Solid.js 响应式 `Accessor`。

| 属性/方法 | 类型 | 说明 |
|-----------|------|------|
| `initialized` | `Accessor<boolean>` | WASM 是否初始化完成 |
| `loading` | `Accessor<boolean>` | 是否正在导出 |
| `progress` | `Accessor<number>` | 导出进度 (0-100) |
| `error` | `Accessor<Error \| null>` | 错误信息 |
| `exportTable` | `(options) => boolean` | DOM 表格导出 |
| `exportData` | `(data, options?) => boolean` | 纯数据导出 |
| `exportTablesXlsx` | `(options) => boolean` | 多 Sheet 导出 |
| `exportCsvBatch` | `(options) => Promise` | CSV 分批导出 |
| `exportXlsxBatch` | `(options) => Promise` | XLSX 分批导出 |
| `exportTablesBatch` | `(options) => Promise` | 多 Sheet 分批导出 |

### `createWorkerExporter(createWorker)` 返回值

将导出计算移至 Worker 线程，主线程不阻塞。需要传入 Worker 工厂函数。

```tsx
import { createWorkerExporter } from '@bsg-export/solid';
import ExportWorkerScript from '@bsg-export/worker/worker?worker';

const { initialized, loading, progress, exportData } = createWorkerExporter(
  () => new ExportWorkerScript()
);
```

| 属性/方法 | 类型 | 说明 |
|-----------|------|------|
| `initialized` | `Accessor<boolean>` | Worker 中 WASM 是否初始化完成 |
| `loading` | `Accessor<boolean>` | 是否正在导出 |
| `progress` | `Accessor<number>` | 导出进度 (0-100) |
| `error` | `Accessor<Error \| null>` | 错误信息 |
| `exportData` | `(data, opts?) => Promise<boolean>` | Worker 生成并下载 |
| `generateBytes` | `(data, opts?) => Promise<Uint8Array>` | 仅生成字节 |
| `terminate` | `() => void` | 销毁 Worker |

### `<ExportButton>` Props

继承 `ParentProps`，额外支持：

| Prop | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `tableId` | `string` | — | 要导出的表格 ID（必填） |
| `filename` | `string` | — | 导出文件名 |
| `format` | `ExportFormat` | `Csv` | 导出格式 |
| `excludeHidden` | `boolean` | `false` | 排除隐藏行/列 |
| `withBom` | `boolean` | `false` | 添加 UTF-8 BOM |
| `disabled` | `boolean` | `false` | 是否禁用按钮 |
| `onExportSuccess` | `() => void` | — | 导出成功回调 |
| `onExportError` | `(error) => void` | — | 导出失败回调 |
| `onExportProgress` | `(progress) => void` | — | 进度变化回调 |
| `initializingText` | `string` | `'初始化中...'` | 初始化中按钮文本 |
| `loadingText` | `string` | `'导出中 {progress}%'` | 导出中按钮文本 |

## 许可证

MIT OR Apache-2.0
