/**
 * @bsg-export/solid - belobog-stellar-grid 的 Solid.js 官方封装
 *
 * 提供基于 Solid.js 信号的导出功能封装，简化在 Solid.js 项目中使用表格导出功能。
 *
 * @packageDocumentation
 */

// Hook
export { createExporter } from './create-exporter';
export type { CreateExporterReturn } from './create-exporter';

export { createWorkerExporter } from './create-worker-exporter';
export type { CreateWorkerExporterReturn } from './create-worker-exporter';

// 组件
export { ExportButton } from './ExportButton';
export type { ExportButtonProps } from './ExportButton';

// 重导出类型（方便用户不用额外安装 @bsg-export/types）
export type {
  Column,
  MergeCellValue,
  CellValue,
  MergeableCellValue,
  DataRow,
  ExportDataOptions,
  SheetConfig,
  BatchSheetConfig,
  ProgressCallback,
  ExportTableOptions,
  ExportTablesXlsxOptions,
  ExportCsvBatchOptions,
  ExportXlsxBatchOptions,
  ExportTablesBatchOptions,
} from '@bsg-export/types';

export { ExportFormat } from '@bsg-export/types';
