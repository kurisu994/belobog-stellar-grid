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
  ExportTableOptions,
  ExportTablesXlsxOptions,
  ExportCsvBatchOptions,
  ExportXlsxBatchOptions,
  ExportTablesBatchOptions,
  ProgressCallback,
  DataRow,
} from '@bsg-export/types';



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
  exportTable: (options: ExportTableOptions) => boolean;
  /** 从 JS 数组直接导出 */
  exportData: (data: DataRow[], options?: ExportDataOptions) => boolean;
  /** 多工作表同步导出 */
  exportTablesXlsx: (options: ExportTablesXlsxOptions) => boolean;
  /** 分批异步导出 CSV */
  exportCsvBatch: (options: ExportCsvBatchOptions) => Promise<boolean>;
  /** 分批异步导出 XLSX */
  exportXlsxBatch: (options: ExportXlsxBatchOptions) => Promise<boolean>;
  /** 多工作表分批异步导出 */
  exportTablesBatch: (options: ExportTablesBatchOptions) => Promise<boolean>;
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
    wasmInitPromise = import('belobog-stellar-grid')
      .then(async (mod) => {
        await mod.default();
        wasmModule = mod;
        return mod;
      })
      .catch((err) => {
        // 初始化失败时重置 Promise，允许后续调用重试
        wasmInitPromise = null;
        throw err;
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
    (fn: () => void): boolean => {
      if (!initialized || !wasmModule) return false;
      setLoading(true);
      setProgress(0);
      setError(null);
      try {
        fn();
        if (mountedRef.current) setProgress(100);
        return true;
      } catch (err) {
        if (mountedRef.current) setError(err instanceof Error ? err : new Error(String(err)));
        return false;
      } finally {
        if (mountedRef.current) setLoading(false);
      }
    },
    [initialized],
  );

  /** 包装异步导出操作 */
  const wrapAsync = useCallback(
    async (fn: () => Promise<void>): Promise<boolean> => {
      if (!initialized || !wasmModule) return false;
      setLoading(true);
      setProgress(0);
      setError(null);
      try {
        await fn();
        if (mountedRef.current) setProgress(100);
        return true;
      } catch (err) {
        if (mountedRef.current) setError(err instanceof Error ? err : new Error(String(err)));
        return false;
      } finally {
        if (mountedRef.current) setLoading(false);
      }
    },
    [initialized],
  );

  const exportTable = useCallback(
    (options: ExportTableOptions) => {
      return wrapSync(() => {
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
      return wrapSync(() => {
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
      return wrapSync(() => {
        wasmModule!.export_tables_xlsx(
          options.sheets,
          options.filename,
          createProgressCallback(),
          options.strictProgressCallback,
        );
      });
    },
    [wrapSync, createProgressCallback],
  );

  const exportCsvBatch = useCallback(
    async (options: ExportCsvBatchOptions) => {
      return await wrapAsync(async () => {
        await wasmModule!.export_table_to_csv_batch(
          options.tableId,
          options.tbodyId,
          options.filename,
          options.batchSize,
          options.excludeHidden,
          createProgressCallback(),
          options.withBom,
          options.strictProgressCallback,
        );
      });
    },
    [wrapAsync, createProgressCallback],
  );

  const exportXlsxBatch = useCallback(
    async (options: ExportXlsxBatchOptions) => {
      return await wrapAsync(async () => {
        await wasmModule!.export_table_to_xlsx_batch(
          options.tableId,
          options.tbodyId,
          options.filename,
          options.batchSize,
          options.excludeHidden,
          createProgressCallback(),
          options.strictProgressCallback,
        );
      });
    },
    [wrapAsync, createProgressCallback],
  );

  const exportTablesBatch = useCallback(
    async (options: ExportTablesBatchOptions) => {
      return await wrapAsync(async () => {
        await wasmModule!.export_tables_to_xlsx_batch(
          options.sheets,
          options.filename,
          options.batchSize,
          createProgressCallback(),
          options.strictProgressCallback,
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
