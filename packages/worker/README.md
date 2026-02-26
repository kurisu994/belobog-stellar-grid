# @bsg-export/worker

[![npm](https://img.shields.io/npm/v/@bsg-export/worker)](https://www.npmjs.com/package/@bsg-export/worker)

> [belobog-stellar-grid](https://github.com/kurisu994/belobog-stellar-grid) 的 Web Worker 导出封装

## 简介

将 CSV/XLSX 文件生成移至 Worker 线程，主线程不阻塞。适用于大数据量导出（10000+ 行）。

## 安装

```bash
npm install @bsg-export/worker belobog-stellar-grid
# 或
pnpm add @bsg-export/worker belobog-stellar-grid
```

**前置依赖**：`belobog-stellar-grid >= 1.0.0`

## 使用方式

### 方式 1：ExportWorker 类（通用）

```typescript
import { ExportWorker } from '@bsg-export/worker';

// Vite
import ExportWorkerScript from '@bsg-export/worker/worker?worker';
const worker = new ExportWorker(new ExportWorkerScript());

// Webpack 5
const w = new Worker(new URL('@bsg-export/worker/worker', import.meta.url), { type: 'module' });
const worker = new ExportWorker(w);

// 初始化
await worker.init();

// 导出数据（Worker 生成 → 主线程下载）
await worker.exportData(
  [{ name: '张三', age: 28 }],
  {
    columns: [{ title: '姓名', key: 'name' }, { title: '年龄', key: 'age' }],
    filename: '用户.xlsx',
    format: 1,
  },
  { onProgress: (p) => console.log(`${p}%`) }
);

// 仅生成字节
const bytes = await worker.generateBytes(data, options);

// 销毁
worker.terminate();
```

### 方式 2：React Hook

```tsx
import { useWorkerExporter } from '@bsg-export/react';
import ExportWorkerScript from '@bsg-export/worker/worker?worker';

const { initialized, loading, progress, exportData } = useWorkerExporter(
  () => new ExportWorkerScript()
);
```

### 方式 3：Vue 3 Composable

```vue
<script setup>
import { useWorkerExporter } from '@bsg-export/vue';
import ExportWorkerScript from '@bsg-export/worker/worker?worker';

const { initialized, loading, progress, exportData } = useWorkerExporter(
  () => new ExportWorkerScript()
);
</script>
```

## API

### `ExportWorker`

| 方法 | 说明 |
|------|------|
| `init()` | 初始化 Worker 中的 WASM 模块 |
| `exportData(data, options?, workerOptions?)` | 生成文件并触发下载 |
| `generateBytes(data, options?, workerOptions?)` | 仅生成文件字节 |
| `terminate()` | 销毁 Worker |
| `initialized` | `boolean` — WASM 是否初始化完成 |

### Worker 创建方式

| 构建工具 | 创建方式 |
|---------|---------|
| **Vite** | `import W from '@bsg-export/worker/worker?worker'` |
| **Webpack 5** | `new Worker(new URL('@bsg-export/worker/worker', import.meta.url))` |

## 许可证

MIT OR Apache-2.0
