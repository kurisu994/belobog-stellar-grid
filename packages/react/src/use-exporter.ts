/**
 * useExporter - WASM 导出管理 Hook
 *
 * 自动管理 WASM 初始化生命周期，提供类型安全的导出方法和状态追踪。
 *
 * @example
 * ```tsx
 * const { initialized, loading, progress, exportTable } = useExporter();
 *
 * return (
 *   <button onClick={() => exportTable({ tableId: 'my-table', filename: '报表.xlsx' })}
 *           disabled={!initialized || loading}>
 *     {loading ? `导出中 ${progress}%` : '导出 Excel'}
 *   </button>
 * );
 * ```
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  ExportFormat,
  ExportDataOptions,
  SheetConfig,
  BatchSheetConfig,
  ProgressCallback,
  DataRow,
} from '@bsg-export/types';

/** export_table 的参数配置 */
export interface ExportTableOptions {
  /** 要导出的 HTML 表格元素的 ID */
  tableId: string;
  /** 导出文件名 */
  filename?: string;
  /** 导出格式 */
  format?: ExportFormat;
  /** 是否排除隐藏行/列 */
  excludeHidden?: boolean;
  /** 是否添加 UTF-8 BOM（仅 CSV 有效） */
  withBom?: boolean;
  /** 回调失败是否中断导出 */
  strictProgressCallback?: boolean;
}

/** 多工作表导出的参数配置 */
export interface ExportTablesXlsxOptions {
  /** Sheet 配置数组 */
  sheets: SheetConfig[];
  /** 导出文件名 */
  filename?: string;
}

/** 分批导出 CSV 的参数配置 */
export interface ExportCsvBatchOptions {
  /** 要导出的 HTML 表格元素的 ID */
  tableId: string;
  /** 可选的独立 tbody ID */
  tbodyId?: string;
  /** 导出文件名 */
  filename?: string;
  /** 每批处理行数 */
  batchSize?: number;
  /** 是否排除隐藏行/列 */
  excludeHidden?: boolean;
  /** 是否添加 UTF-8 BOM */
  withBom?: boolean;
}

/** 分批导出 XLSX 的参数配置 */
export interface ExportXlsxBatchOptions {
  /** 要导出的 HTML 表格元素的 ID */
  tableId: string;
  /** 可选的独立 tbody ID */
  tbodyId?: string;
  /** 导出文件名 */
  filename?: string;
  /** 每批处理行数 */
  batchSize?: number;
  /** 是否排除隐藏行/列 */
  excludeHidden?: boolean;
}

/** 多工作表分批导出的参数配置 */
export interface ExportTablesBatchOptions {
  /** Sheet 配置数组 */
  sheets: BatchSheetConfig[];
  /** 导出文件名 */
  filename?: string;
  /** 每批处理行数 */
  batchSize?: number;
}

/** useExporter Hook 的返回值 */
export interface UseExporterReturn {
  /** WASM 是否已初始化完成 */
  initialized: boolean;
  /** 是否正在导出 */
  loading: boolean;
  /** 导出进度 (0-100) */
  progress: number;
  /** 错误信息 */
  error: Error | null;
  /** 导出 HTML 表格 */
  exportTable: (options: ExportTableOptions) => void;
  /** 从 JS 数组直接导出 */
  exportData: (data: DataRow[], options?: ExportDataOptions) => void;
  /** 多工作表同步导出 */
  exportTablesXlsx: (options: ExportTablesXlsxOptions) => void;
  /** 分批异步导出 CSV */
  exportCsvBatch: (options: ExportCsvBatchOptions) => Promise<void>;
  /** 分批异步导出 XLSX */
  exportXlsxBatch: (options: ExportXlsxBatchOptions) => Promise<void>;
  /** 多工作表分批异步导出 */
  exportTablesBatch: (options: ExportTablesBatchOptions) => Promise<void>;
}

/** WASM 模块缓存 */
let wasmModule: typeof import('belobog-stellar-grid') | null = null;
let wasmInitPromise: Promise<typeof import('belobog-stellar-grid')> | null = null;

/**
 * 初始化 WASM 模块（单例模式）
 */
async function initWasm(): Promise<typeof import('belobog-stellar-grid')> {
  if (wasmModule) return wasmModule;

  if (!wasmInitPromise) {
    wasmInitPromise = import('belobog-stellar-grid').then(async (mod) => {
      await mod.default();
      wasmModule = mod;
      return mod;
    });
  }

  return wasmInitPromise;
}

/**
 * WASM 导出管理 Hook
 *
 * 自动初始化 WASM 模块，提供类型安全的导出方法，
 * 管理 loading / progress / error 状态。
 */
export function useExporter(): UseExporterReturn {
  const [initialized, setInitialized] = useState(false);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<Error | null>(null);
  const mountedRef = useRef(true);

  useEffect(() => {
    mountedRef.current = true;
    initWasm()
      .then(() => {
        if (mountedRef.current) setInitialized(true);
      })
      .catch((err) => {
        if (mountedRef.current) setError(err instanceof Error ? err : new Error(String(err)));
      });

    return () => {
      mountedRef.current = false;
    };
  }, []);

  /** 创建进度回调（自动更新 progress 状态） */
  const createProgressCallback = useCallback((): ProgressCallback => {
    return (p: number) => {
      if (mountedRef.current) setProgress(p);
    };
  }, []);

  /** 包装同步导出操作 */
  const wrapSync = useCallback(
    (fn: () => void) => {
      if (!initialized || !wasmModule) return;
      setLoading(true);
      setProgress(0);
      setError(null);
      try {
        fn();
        if (mountedRef.current) setProgress(100);
      } catch (err) {
        if (mountedRef.current) setError(err instanceof Error ? err : new Error(String(err)));
      } finally {
        if (mountedRef.current) setLoading(false);
      }
    },
    [initialized],
  );

  /** 包装异步导出操作 */
  const wrapAsync = useCallback(
    async (fn: () => Promise<void>) => {
      if (!initialized || !wasmModule) return;
      setLoading(true);
      setProgress(0);
      setError(null);
      try {
        await fn();
        if (mountedRef.current) setProgress(100);
      } catch (err) {
        if (mountedRef.current) setError(err instanceof Error ? err : new Error(String(err)));
      } finally {
        if (mountedRef.current) setLoading(false);
      }
    },
    [initialized],
  );

  const exportTable = useCallback(
    (options: ExportTableOptions) => {
      wrapSync(() => {
        wasmModule!.export_table(
          options.tableId,
          options.filename,
          options.format,
          options.excludeHidden,
          createProgressCallback(),
          options.withBom,
          options.strictProgressCallback,
        );
      });
    },
    [wrapSync, createProgressCallback],
  );

  const exportData = useCallback(
    (data: DataRow[], options?: ExportDataOptions) => {
      wrapSync(() => {
        const opts = options
          ? { ...options, progressCallback: options.progressCallback ?? createProgressCallback() }
          : { progressCallback: createProgressCallback() };
        wasmModule!.export_data(data, opts);
      });
    },
    [wrapSync, createProgressCallback],
  );

  const exportTablesXlsx = useCallback(
    (options: ExportTablesXlsxOptions) => {
      wrapSync(() => {
        wasmModule!.export_tables_xlsx(
          options.sheets,
          options.filename,
          createProgressCallback(),
        );
      });
    },
    [wrapSync, createProgressCallback],
  );

  const exportCsvBatch = useCallback(
    async (options: ExportCsvBatchOptions) => {
      await wrapAsync(async () => {
        await wasmModule!.export_table_to_csv_batch(
          options.tableId,
          options.tbodyId,
          options.filename,
          options.batchSize,
          options.excludeHidden,
          createProgressCallback(),
          options.withBom,
        );
      });
    },
    [wrapAsync, createProgressCallback],
  );

  const exportXlsxBatch = useCallback(
    async (options: ExportXlsxBatchOptions) => {
      await wrapAsync(async () => {
        await wasmModule!.export_table_to_xlsx_batch(
          options.tableId,
          options.tbodyId,
          options.filename,
          options.batchSize,
          options.excludeHidden,
          createProgressCallback(),
        );
      });
    },
    [wrapAsync, createProgressCallback],
  );

  const exportTablesBatch = useCallback(
    async (options: ExportTablesBatchOptions) => {
      await wrapAsync(async () => {
        await wasmModule!.export_tables_to_xlsx_batch(
          options.sheets,
          options.filename,
          options.batchSize,
          createProgressCallback(),
        );
      });
    },
    [wrapAsync, createProgressCallback],
  );

  return {
    initialized,
    loading,
    progress,
    error,
    exportTable,
    exportData,
    exportTablesXlsx,
    exportCsvBatch,
    exportXlsxBatch,
    exportTablesBatch,
  };
}
