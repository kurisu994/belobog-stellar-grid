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
