/**
 * @bsg-export/react - belobog-stellar-grid 的 React 官方封装
 *
 * 提供 Hook 和组件，简化在 React 项目中使用表格导出功能。
 *
 * @packageDocumentation
 */

// Hook
export { useExporter } from './use-exporter';
export type {
  UseExporterReturn,
} from './use-exporter';

export { useWorkerExporter } from './use-worker-exporter';
export type {
  UseWorkerExporterReturn,
} from './use-worker-exporter';

export { useExcelPreview } from './useExcelPreview';
export type {
  ExcelPreviewState,
  UseExcelPreviewOptions,
  UseExcelPreviewReturn,
} from './useExcelPreview';

// 组件
export { ExportButton } from './ExportButton';
export { ExcelPreview } from './ExcelPreview';
export type { ExportButtonProps } from './ExportButton';
export type { ExcelPreviewProps } from './ExcelPreview';

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
