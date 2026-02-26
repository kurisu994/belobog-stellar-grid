# @bsg-export/types

[![npm](https://img.shields.io/npm/v/@bsg-export/types)](https://www.npmjs.com/package/@bsg-export/types)

> [belobog-stellar-grid](https://github.com/kurisu994/belobog-stellar-grid) 的严格 TypeScript 类型定义

## 简介

为 `belobog-stellar-grid` WASM 导出 API 提供类型安全的接口声明，替代 wasm-bindgen 自动生成的 `any` 类型。

**零运行时开销** — 仅包含类型定义和 `ExportFormat` 枚举。

## 安装

```bash
npm install @bsg-export/types
# 或
pnpm add @bsg-export/types
```

## 核心类型

| 类型 | 说明 |
|------|------|
| `ExportFormat` | 导出格式枚举（`Csv = 0`, `Xlsx = 1`） |
| `Column` | 列配置，支持嵌套 `children` 多级表头 |
| `ExportDataOptions` | `export_data()` 配置选项 |
| `SheetConfig` | 多工作表同步导出配置 |
| `BatchSheetConfig` | 多工作表分批异步导出配置 |
| `MergeCellValue` | 合并单元格值 `{ value, colSpan?, rowSpan? }` |
| `ProgressCallback` | 进度回调函数签名 |
| `DataRow` | 数据行类型（二维数组 / 对象数组） |
| `TreeDataRow` | 树形数据行（含 children） |

## 使用

```typescript
import { ExportFormat } from '@bsg-export/types';
import type { Column, ExportDataOptions } from '@bsg-export/types';

const columns: Column[] = [
  { title: '姓名', key: 'name' },
  {
    title: '联系方式',
    children: [
      { title: '电话', key: 'phone' },
      { title: '邮箱', key: 'email' },
    ],
  },
];

const options: ExportDataOptions = {
  columns,
  filename: '用户列表.xlsx',
  format: ExportFormat.Xlsx,
};
```

## 许可证

MIT OR Apache-2.0
