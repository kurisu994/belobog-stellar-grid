# @bsg-export/vue

[![npm](https://img.shields.io/npm/v/@bsg-export/vue)](https://www.npmjs.com/package/@bsg-export/vue)

> [belobog-stellar-grid](https://github.com/kurisu994/belobog-stellar-grid) 的 Vue 3 官方封装

## 简介

提供 `useExporter` Composable、`useWorkerExporter` Composable 和 `ExportButton` 组件，自动管理 WASM 初始化、导出状态和进度追踪。`useWorkerExporter` 支持将导出计算移至 Worker 线程。

## 安装

```bash
npm install @bsg-export/vue belobog-stellar-grid
# 或
pnpm add @bsg-export/vue belobog-stellar-grid
```

**前置依赖**：`vue >= 3.3.0`、`belobog-stellar-grid >= 1.0.0`

## 快速开始

### useExporter Composable

```vue
<script setup lang="ts">
import { useExporter, ExportFormat } from '@bsg-export/vue';

const { initialized, loading, progress, exportTable } = useExporter();
</script>

<template>
  <button
    :disabled="!initialized || loading"
    @click="exportTable({ tableId: 'my-table', filename: '报表.xlsx', format: ExportFormat.Xlsx })"
  >
    {{ loading ? `导出中 ${Math.round(progress)}%` : '导出 Excel' }}
  </button>
</template>
```

### ExportButton 组件

```vue
<script setup lang="ts">
import { ExportButton, ExportFormat } from '@bsg-export/vue';
</script>

<template>
  <ExportButton
    table-id="my-table"
    filename="报表.xlsx"
    :format="ExportFormat.Xlsx"
    @success="console.log('导出成功')"
    @error="(err) => console.error('导出失败', err)"
  >
    导出 Excel
  </ExportButton>
</template>
```

## API

### `useExporter()` 返回值

所有返回值均为 Vue 3 响应式 `Ref`。

| 属性/方法 | 类型 | 说明 |
|-----------|------|------|
| `initialized` | `Ref<boolean>` | WASM 是否初始化完成 |
| `loading` | `Ref<boolean>` | 是否正在导出 |
| `progress` | `Ref<number>` | 导出进度 (0-100) |
| `error` | `Ref<Error \| null>` | 错误信息 |
| `exportTable` | `(options) => void` | DOM 表格导出 |
| `exportData` | `(data, options?) => void` | 纯数据导出 |
| `exportTablesXlsx` | `(options) => void` | 多 Sheet 导出 |
| `exportCsvBatch` | `(options) => Promise` | CSV 分批导出 |
| `exportXlsxBatch` | `(options) => Promise` | XLSX 分批导出 |
| `exportTablesBatch` | `(options) => Promise` | 多 Sheet 分批导出 |

### `useWorkerExporter(createWorker)` 返回值

将导出计算移至 Worker 线程，主线程不阻塞。需要传入 Worker 工厂函数。

```vue
<script setup lang="ts">
import { useWorkerExporter } from '@bsg-export/vue';
import ExportWorkerScript from '@bsg-export/worker/worker?worker';

const { initialized, loading, progress, exportData } = useWorkerExporter(
  () => new ExportWorkerScript()
);
</script>
```

| 属性/方法 | 类型 | 说明 |
|-----------|------|------|
| `initialized` | `Ref<boolean>` | Worker 中 WASM 是否初始化完成 |
| `loading` | `Ref<boolean>` | 是否正在导出 |
| `progress` | `Ref<number>` | 导出进度 (0-100) |
| `error` | `Ref<Error \| null>` | 错误信息 |
| `exportData` | `(data, opts?) => Promise<boolean>` | Worker 生成并下载 |
| `generateBytes` | `(data, opts?) => Promise<Uint8Array>` | 仅生成字节 |
| `terminate` | `() => void` | 销毁 Worker |

### `<ExportButton>` Props

| Prop | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `table-id` | `string` | — | 要导出的表格 ID（必填） |
| `filename` | `string` | — | 导出文件名 |
| `format` | `ExportFormat` | `Csv` | 导出格式 |
| `exclude-hidden` | `boolean` | `false` | 排除隐藏行/列 |
| `with-bom` | `boolean` | `false` | 添加 UTF-8 BOM |
| `disabled` | `boolean` | `false` | 是否禁用按钮 |
| `initializing-text` | `string` | `'初始化中...'` | 初始化中按钮文本 |
| `loading-text` | `string` | `'导出中 {progress}%'` | 导出中按钮文本 |

### 事件

| 事件 | 参数 | 说明 |
|------|------|------|
| `success` | — | 导出成功 |
| `error` | `Error` | 导出失败 |
| `progress` | `number` | 进度变化 |

### 插槽

| 插槽 | 说明 |
|------|------|
| `default` | 按钮内容（默认显示"导出"） |

## 许可证

MIT OR Apache-2.0
