/**
 * createWorkerExporter - Worker 线程导出管理 Primitive
 *
 * 将 CSV/XLSX 生成移至 Worker 线程，主线程不阻塞。
 * 用户需要传入一个创建 Worker 实例的工厂函数。
 *
 * @example
 * ```tsx
 * import { createWorkerExporter } from '@bsg-export/solid';
 *
 * // Vite
 * import ExportWorkerScript from '@bsg-export/worker/worker?worker';
 *
 * function App() {
 *   const { initialized, loading, progress, exportData } =
 *     createWorkerExporter(() => new ExportWorkerScript());
 *
 *   return (
 *     <button disabled={!initialized() || loading()}
 *             onClick={() => exportData(data, { columns, filename: '报表.xlsx', format: 1 })}>
 *       {loading() ? `导出中 ${Math.round(progress())}%` : '导出'}
 *     </button>
 *   );
 * }
 * ```
 */

import { createSignal, onMount, onCleanup, type Accessor } from 'solid-js';
import type {
  ExportDataOptions,
  ProgressCallback,
  DataRow,
} from '@bsg-export/types';

/** createWorkerExporter 返回值 */
export interface CreateWorkerExporterReturn {
  /** Worker 中的 WASM 是否已初始化完成 */
  initialized: Accessor<boolean>;
  /** 是否正在导出 */
  loading: Accessor<boolean>;
  /** 导出进度 (0-100) */
  progress: Accessor<number>;
  /** 错误信息 */
  error: Accessor<Error | null>;
  /** 在 Worker 中生成文件并触发下载 */
  exportData: (data: DataRow[], options?: Omit<ExportDataOptions, 'progressCallback'>) => Promise<boolean>;
  /** 在 Worker 中生成文件字节（不触发下载） */
  generateBytes: (data: DataRow[], options?: Omit<ExportDataOptions, 'progressCallback'>) => Promise<Uint8Array | null>;
  /** 销毁 Worker */
  terminate: () => void;
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
 * Worker 线程导出管理 Primitive
 *
 * 接收一个 Worker 工厂函数，自动管理 Worker 生命周期和 WASM 初始化。
 * 导出计算在 Worker 线程执行，主线程保持响应。
 *
 * @param createWorker - 创建 Worker 实例的工厂函数
 */
export function createWorkerExporter(createWorker: () => Worker): CreateWorkerExporterReturn {
  const [initialized, setInitialized] = createSignal(false);
  const [loading, setLoading] = createSignal(false);
  const [progress, setProgress] = createSignal(0);
  const [error, setError] = createSignal<Error | null>(null);

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
    setError(err);
  };

  onMount(() => {
    worker = createWorker();
    worker.addEventListener('message', handleMessage);
    worker.addEventListener('error', handleError);

    // 发送 init 消息
    const id = generateId();
    const initPromise = new Promise<ArrayBuffer>((resolve, reject) => {
      pending.set(id, { resolve, reject });
    });

    const request: WorkerRequest = { type: 'init', id };
    worker.postMessage(request);

    initPromise
      .then(() => {
        if (alive) setInitialized(true);
      })
      .catch((err) => {
        if (alive) setError(err instanceof Error ? err : new Error(String(err)));
      });
  });

  onCleanup(() => {
    alive = false;
    // 拒绝所有待处理请求
    for (const [, req] of pending) {
      req.reject(new Error('组件已卸载'));
    }
    pending.clear();
    worker?.terminate();
    worker = null;
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
          if (alive) setProgress(p);
        },
      });

      const request: WorkerRequest = {
        type: 'generate',
        id,
        data,
        options: (options ?? {}) as Record<string, unknown>,
      };
      worker!.postMessage(request);
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
    if (!initialized()) return false;
    setLoading(true);
    setProgress(0);
    setError(null);
    try {
      const buffer = await sendGenerate(data, options);
      const bytes = new Uint8Array(buffer);
      downloadFile(bytes, options?.filename, options?.format);
      if (alive) setProgress(100);
      return true;
    } catch (err) {
      if (alive) setError(err instanceof Error ? err : new Error(String(err)));
      return false;
    } finally {
      if (alive) setLoading(false);
    }
  };

  /** 在 Worker 中生成文件字节（不触发下载） */
  const generateBytes = async (
    data: DataRow[],
    options?: Omit<ExportDataOptions, 'progressCallback'>,
  ): Promise<Uint8Array | null> => {
    if (!initialized()) return null;
    setLoading(true);
    setProgress(0);
    setError(null);
    try {
      const buffer = await sendGenerate(data, options);
      if (alive) setProgress(100);
      return new Uint8Array(buffer);
    } catch (err) {
      if (alive) setError(err instanceof Error ? err : new Error(String(err)));
      return null;
    } finally {
      if (alive) setLoading(false);
    }
  };

  /** 手动销毁 Worker */
  const terminate = () => {
    for (const [, req] of pending) {
      req.reject(new Error('Worker 已被手动销毁'));
    }
    pending.clear();
    worker?.terminate();
    worker = null;
    if (alive) setInitialized(false);
  };

  return {
    initialized,
    loading,
    progress,
    error,
    exportData,
    generateBytes,
    terminate,
  };
}
