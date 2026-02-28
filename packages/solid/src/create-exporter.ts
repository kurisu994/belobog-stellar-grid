/**
 * createExporter - WASM 导出管理 Primitive
 *
 * 自动管理 WASM 初始化生命周期，提供类型安全的导出方法和响应式信号。
 *
 * @example
 * ```tsx
 * import { createExporter } from '@bsg-export/solid';
 *
 * function App() {
 *   const { initialized, loading, progress, exportTable } = createExporter();
 *
 *   return (
 *     <button onClick={() => exportTable({ tableId: 'my-table', filename: '报表.xlsx' })}
 *             disabled={!initialized() || loading()}>
 *       {loading() ? `导出中 ${progress()}%` : '导出 Excel'}
 *     </button>
 *   );
 * }
 * ```
 */

import { createSignal, onMount, onCleanup, type Accessor } from 'solid-js';
import type {
  ExportDataOptions,
  ExportTableOptions,
  ExportTablesXlsxOptions,
  ExportCsvBatchOptions,
  ExportXlsxBatchOptions,
  ExportTablesBatchOptions,
  ProgressCallback,
  DataRow,
} from '@bsg-export/types';

/** createExporter 返回值 */
export interface CreateExporterReturn {
  /** WASM 是否已初始化完成 */
  initialized: Accessor<boolean>;
  /** 是否正在导出 */
  loading: Accessor<boolean>;
  /** 导出进度 (0-100) */
  progress: Accessor<number>;
  /** 错误信息 */
  error: Accessor<Error | null>;
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
 * WASM 导出管理 Primitive
 *
 * 自动初始化 WASM 模块，提供类型安全的导出方法，
 * 管理 loading / progress / error 响应式信号。
 */
export function createExporter(): CreateExporterReturn {
  const [initialized, setInitialized] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [progress, setProgress] = createSignal(0);
  const [error, setError] = createSignal<Error | null>(null);
  let alive = true;

  onMount(() => {
    initWasm()
      .then(() => {
        if (alive) setInitialized(true);
      })
      .catch((err) => {
        if (alive) setError(err instanceof Error ? err : new Error(String(err)));
      });
  });

  onCleanup(() => {
    alive = false;
  });

  /** 创建进度回调（自动更新 progress 状态） */
  const createProgressCallback = (): ProgressCallback => {
    return (p: number) => {
      if (alive) setProgress(p);
    };
  };

  /** 包装同步导出操作 */
  const wrapSync = (fn: () => void): boolean => {
    if (!initialized() || !wasmModule) return false;
    setLoading(true);
    setProgress(0);
    setError(null);
    try {
      fn();
      if (alive) setProgress(100);
      return true;
    } catch (err) {
      if (alive) setError(err instanceof Error ? err : new Error(String(err)));
      return false;
    } finally {
      if (alive) setLoading(false);
    }
  };

  /** 包装异步导出操作 */
  const wrapAsync = async (fn: () => Promise<void>): Promise<boolean> => {
    if (!initialized() || !wasmModule) return false;
    setLoading(true);
    setProgress(0);
    setError(null);
    try {
      await fn();
      if (alive) setProgress(100);
      return true;
    } catch (err) {
      if (alive) setError(err instanceof Error ? err : new Error(String(err)));
      return false;
    } finally {
      if (alive) setLoading(false);
    }
  };

  /** 导出 HTML 表格 */
  const exportTable = (options: ExportTableOptions) => {
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
  };

  /** 从 JS 数组直接导出 */
  const exportData = (data: DataRow[], options?: ExportDataOptions) => {
    return wrapSync(() => {
      const opts = options
        ? { ...options, progressCallback: options.progressCallback ?? createProgressCallback() }
        : { progressCallback: createProgressCallback() };
      wasmModule!.export_data(data, opts);
    });
  };

  /** 多工作表同步导出 */
  const exportTablesXlsx = (options: ExportTablesXlsxOptions) => {
    return wrapSync(() => {
      wasmModule!.export_tables_xlsx(
        options.sheets,
        options.filename,
        createProgressCallback(),
        options.strictProgressCallback,
      );
    });
  };

  /** 分批异步导出 CSV */
  const exportCsvBatch = async (options: ExportCsvBatchOptions) => {
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
  };

  /** 分批异步导出 XLSX */
  const exportXlsxBatch = async (options: ExportXlsxBatchOptions) => {
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
  };

  /** 多工作表分批异步导出 */
  const exportTablesBatch = async (options: ExportTablesBatchOptions) => {
    return await wrapAsync(async () => {
      await wasmModule!.export_tables_to_xlsx_batch(
        options.sheets,
        options.filename,
        options.batchSize,
        createProgressCallback(),
        options.strictProgressCallback,
      );
    });
  };

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
