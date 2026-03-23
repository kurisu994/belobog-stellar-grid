/**
 * @bsg-export/svelte - belobog-stellar-grid 的 Svelte 官方封装
 *
 * 提供基于 Svelte store 的导出功能封装，简化在 Svelte 项目中使用表格导出功能。
 * 兼容 Svelte 4 和 Svelte 5。
 *
 * @packageDocumentation
 */

// Store 封装
export { createExporter } from './create-exporter';
export type { ExporterStore } from './create-exporter';

export { createWorkerExporter } from './create-worker-exporter';
export type { WorkerExporterStore } from './create-worker-exporter';

// Excel 预览
export { createExcelPreview } from './create-excel-preview';
export type { CreateExcelPreviewOptions, ExcelPreviewStore } from './create-excel-preview';

// 组件类型（组件本身通过子路径导入，如 '@bsg-export/svelte/ExportButton.svelte'）
export type { ExportButtonProps } from './ExportButton.svelte';
export type { ExcelPreviewProps, ExcelPreviewEvents } from './ExcelPreview.svelte';

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
  // 预览相关类型
  PreviewOptions,
  SheetInfo,
  ParsedWorkbook,
  ParsedSheet,
  ParsedRow,
  ParsedCell,
  MergeRegion,
} from '@bsg-export/types';

export { ExportFormat } from '@bsg-export/types';
