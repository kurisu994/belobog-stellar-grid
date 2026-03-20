/**
 * @bsg-export/vue - belobog-stellar-grid 的 Vue 3 官方封装
 *
 * 提供 Composable 和组件，简化在 Vue 项目中使用表格导出功能。
 *
 * @packageDocumentation
 */

// Composable
export { useExporter } from './use-exporter';
export { useWorkerExporter } from './use-worker-exporter';
export { useExcelPreview } from './useExcelPreview';
export type {
  UseExcelPreviewOptions,
  UseExcelPreviewReturn,
} from './useExcelPreview';

// 组件
export { default as ExportButton } from './ExportButton.vue';
export { default as ExcelPreview } from './ExcelPreview.vue';

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
  PreviewOptions,
  SheetInfo,
  ParsedWorkbook,
  ParsedSheet,
  ParsedRow,
  ParsedCell,
  MergeRegion,
} from '@bsg-export/types';

export { ExportFormat } from '@bsg-export/types';
