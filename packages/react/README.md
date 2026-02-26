# @bsg-export/react

[![npm](https://img.shields.io/npm/v/@bsg-export/react)](https://www.npmjs.com/package/@bsg-export/react)

> [belobog-stellar-grid](https://github.com/kurisu994/belobog-stellar-grid) 的 React 官方封装

## 简介

提供 `useExporter` Hook 和 `ExportButton` 组件，自动管理 WASM 初始化、导出状态和进度追踪。

## 安装

```bash
npm install @bsg-export/react belobog-stellar-grid
# 或
pnpm add @bsg-export/react belobog-stellar-grid
```

**前置依赖**：`react >= 17.0.0`、`belobog-stellar-grid >= 1.0.0`

## 快速开始

### useExporter Hook

```tsx
import { useExporter, ExportFormat } from '@bsg-export/react';

function App() {
  const { initialized, loading, progress, error, exportTable, exportData } = useExporter();

  return (
    <button
      disabled={!initialized || loading}
      onClick={() =>
        exportTable({
          tableId: 'my-table',
          filename: '报表.xlsx',
          format: ExportFormat.Xlsx,
        })
      }
    >
      {loading ? `导出中 ${Math.round(progress)}%` : '导出 Excel'}
    </button>
  );
}
```

### ExportButton 组件

```tsx
import { ExportButton, ExportFormat } from '@bsg-export/react';

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

### `useExporter()` 返回值

| 属性/方法 | 类型 | 说明 |
|-----------|------|------|
| `initialized` | `boolean` | WASM 是否初始化完成 |
| `loading` | `boolean` | 是否正在导出 |
| `progress` | `number` | 导出进度 (0-100) |
| `error` | `Error \| null` | 错误信息 |
| `exportTable` | `(options) => void` | DOM 表格导出 |
| `exportData` | `(data, options?) => void` | 纯数据导出 |
| `exportTablesXlsx` | `(options) => void` | 多 Sheet 导出 |
| `exportCsvBatch` | `(options) => Promise` | CSV 分批导出 |
| `exportXlsxBatch` | `(options) => Promise` | XLSX 分批导出 |
| `exportTablesBatch` | `(options) => Promise` | 多 Sheet 分批导出 |

### `<ExportButton>` Props

继承所有 `<button>` HTML 属性，额外支持：

| Prop | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `tableId` | `string` | — | 要导出的表格 ID（必填） |
| `filename` | `string` | — | 导出文件名 |
| `format` | `ExportFormat` | `Csv` | 导出格式 |
| `excludeHidden` | `boolean` | `false` | 排除隐藏行/列 |
| `withBom` | `boolean` | `false` | 添加 UTF-8 BOM |
| `onExportSuccess` | `() => void` | — | 导出成功回调 |
| `onExportError` | `(error) => void` | — | 导出失败回调 |
| `onExportProgress` | `(progress) => void` | — | 进度变化回调 |
| `initializingText` | `string` | `'初始化中...'` | 初始化中按钮文本 |
| `loadingText` | `string` | `'导出中 {progress}%'` | 导出中按钮文本 |

## 许可证

MIT OR Apache-2.0
