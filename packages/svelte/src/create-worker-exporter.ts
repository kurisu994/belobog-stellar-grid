/**
 * createWorkerExporter - Worker 线程导出管理 Store
 *
 * 将 CSV/XLSX 生成移至 Worker 线程，主线程不阻塞。
 * 用户需要传入一个创建 Worker 实例的工厂函数。
 *
 * @example
 * ```svelte
 * <script>
 * import { createWorkerExporter } from '@bsg-export/svelte';
 *
 * // Vite
 * import ExportWorkerScript from '@bsg-export/worker/worker?worker';
 * const { initialized, loading, progress, exportData, destroy } =
 *   createWorkerExporter(() => new ExportWorkerScript());
 *
 * // 组件卸载时调用 destroy()
 * import { onDestroy } from 'svelte';
 * onDestroy(() => destroy());
 * </script>
 *
 * <button disabled={!$initialized || $loading}
 *         on:click={() => exportData(data, { columns, filename: '报表.xlsx', format: 1 })}>
 *   {$loading ? `导出中 ${Math.round($progress)}%` : '导出'}
 * </button>
 * ```
 */

import { writable, get, type Readable } from 'svelte/store';
import type {
  ExportDataOptions,
  ProgressCallback,
  DataRow,
} from '@bsg-export/types';

/** createWorkerExporter 返回值 */
export interface WorkerExporterStore {
  /** Worker 中的 WASM 是否已初始化完成 */
  initialized: Readable<boolean>;
  /** 是否正在导出 */
  loading: Readable<boolean>;
  /** 导出进度 (0-100) */
  progress: Readable<number>;
  /** 错误信息 */
  error: Readable<Error | null>;
  /** 在 Worker 中生成文件并触发下载 */
  exportData: (data: DataRow[], options?: Omit<ExportDataOptions, 'progressCallback'>) => Promise<boolean>;
  /** 在 Worker 中生成文件字节（不触发下载） */
  generateBytes: (data: DataRow[], options?: Omit<ExportDataOptions, 'progressCallback'>) => Promise<Uint8Array | null>;
  /** 销毁 Worker 和清理资源 */
  destroy: () => void;
}

// ---- Worker 消息协议（与 @bsg-export/worker 的 worker.ts 对齐） ----

/** Worker 请求消息 */
interface WorkerRequest {
  type: 'init' | 'generate';
  id: string;
  data?: unknown;
  options?: Record<string, unknown>;
}

/** Worker 响应消息 */
interface WorkerResponse {
  type: 'ready' | 'result' | 'error' | 'progress';
  id: string;
  bytes?: ArrayBuffer;
  message?: string;
  progress?: number;
}

/** 待处理的请求 */
interface PendingRequest {
  resolve: (value: ArrayBuffer) => void;
  reject: (reason: Error) => void;
  onProgress?: ProgressCallback;
}

let requestCounter = 0;
function generateId(): string {
  return `req_${++requestCounter}_${Date.now()}`;
}

/**
 * Worker 线程导出管理 Store
 *
 * 接收一个 Worker 工厂函数，自动管理 Worker 生命周期和 WASM 初始化。
 * 导出计算在 Worker 线程执行，主线程保持响应。
 *
 * 必须在组件卸载时调用 `destroy()` 清理资源。
 *
 * @param createWorker - 创建 Worker 实例的工厂函数
 */
export function createWorkerExporter(createWorker: () => Worker): WorkerExporterStore {
  const initialized = writable(false);
  const loading = writable(false);
  const progress = writable(0);
  const error = writable<Error | null>(null);

  let worker: Worker | null = null;
  const pending = new Map<string, PendingRequest>();
  let alive = true;

  /** 处理 Worker 消息 */
  const handleMessage = (event: MessageEvent<WorkerResponse>) => {
    if (!alive) return;
    const { type, id, bytes, message, progress: prog } = event.data;
    const req = pending.get(id);
    if (!req) return;

    switch (type) {
      case 'ready':
        pending.delete(id);
        req.resolve(new ArrayBuffer(0));
        break;
      case 'result':
        pending.delete(id);
        req.resolve(bytes!);
        break;
      case 'error':
        pending.delete(id);
        req.reject(new Error(message ?? '未知错误'));
        break;
      case 'progress':
        if (req.onProgress && prog !== undefined) {
          req.onProgress(prog);
        }
        break;
    }
  };

  /** 处理 Worker 错误 */
  const handleError = (event: ErrorEvent) => {
    if (!alive) return;
    const err = new Error(`Worker 错误: ${event.message}`);
    for (const [id, req] of pending) {
      req.reject(err);
      pending.delete(id);
    }
    error.set(err);
  };

  // 立即创建 Worker 并初始化
  worker = createWorker();
  worker.addEventListener('message', handleMessage);
  worker.addEventListener('error', handleError);

  // 发送 init 消息
  const initId = generateId();
  const initPromise = new Promise<ArrayBuffer>((resolve, reject) => {
    pending.set(initId, { resolve, reject });
  });

  const request: WorkerRequest = { type: 'init', id: initId };
  worker.postMessage(request);

  initPromise
    .then(() => {
      if (alive) initialized.set(true);
    })
    .catch((err) => {
      if (alive) error.set(err instanceof Error ? err : new Error(String(err)));
    });

  /** 向 Worker 发送生成请求 */
  const sendGenerate = (
    data: DataRow[],
    options?: Omit<ExportDataOptions, 'progressCallback'>,
  ): Promise<ArrayBuffer> => {
    if (!worker) return Promise.reject(new Error('Worker 未创建'));

    return new Promise<ArrayBuffer>((resolve, reject) => {
      const id = generateId();
      pending.set(id, {
        resolve,
        reject,
        onProgress: (p: number) => {
          if (alive) progress.set(p);
        },
      });

      const req: WorkerRequest = {
        type: 'generate',
        id,
        data,
        options: (options ?? {}) as Record<string, unknown>,
      };
      worker!.postMessage(req);
    });
  };

  /** 在主线程触发文件下载 */
  const downloadFile = (
    bytes: Uint8Array,
    filename?: string,
    format?: number,
  ) => {
    const isXlsx = format === 1;
    const mimeType = isXlsx
      ? 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
      : 'text/csv;charset=utf-8';
    const defaultExt = isXlsx ? 'xlsx' : 'csv';
    let finalFilename = filename ?? `export.${defaultExt}`;
    if (!finalFilename.endsWith(`.${defaultExt}`)) {
      finalFilename = `${finalFilename}.${defaultExt}`;
    }

    const blob = new Blob([bytes.buffer as ArrayBuffer], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = finalFilename;
    anchor.click();
    setTimeout(() => URL.revokeObjectURL(url), 10_000);
  };

  /** 在 Worker 中生成文件并触发下载 */
  const exportData = async (
    data: DataRow[],
    options?: Omit<ExportDataOptions, 'progressCallback'>,
  ): Promise<boolean> => {
    if (!get(initialized)) return false;

    loading.set(true);
    progress.set(0);
    error.set(null);
    try {
      const buffer = await sendGenerate(data, options);
      const bytes = new Uint8Array(buffer);
      downloadFile(bytes, options?.filename, options?.format);
      if (alive) progress.set(100);
      return true;
    } catch (err) {
      if (alive) error.set(err instanceof Error ? err : new Error(String(err)));
      return false;
    } finally {
      if (alive) loading.set(false);
    }
  };

  /** 在 Worker 中生成文件字节（不触发下载） */
  const generateBytes = async (
    data: DataRow[],
    options?: Omit<ExportDataOptions, 'progressCallback'>,
  ): Promise<Uint8Array | null> => {
    if (!get(initialized)) return null;

    loading.set(true);
    progress.set(0);
    error.set(null);
    try {
      const buffer = await sendGenerate(data, options);
      if (alive) progress.set(100);
      return new Uint8Array(buffer);
    } catch (err) {
      if (alive) error.set(err instanceof Error ? err : new Error(String(err)));
      return null;
    } finally {
      if (alive) loading.set(false);
    }
  };

  /** 销毁 Worker 和清理资源 */
  const destroy = () => {
    alive = false;
    // 拒绝所有待处理请求
    for (const [, req] of pending) {
      req.reject(new Error('组件已卸载'));
    }
    pending.clear();
    worker?.terminate();
    worker = null;
  };

  return {
    initialized: { subscribe: initialized.subscribe },
    loading: { subscribe: loading.subscribe },
    progress: { subscribe: progress.subscribe },
    error: { subscribe: error.subscribe },
    exportData,
    generateBytes,
    destroy,
  };
}
