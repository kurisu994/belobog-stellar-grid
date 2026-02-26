/**
 * useWorkerExporter - Worker 线程导出管理 Hook
 *
 * 将 CSV/XLSX 生成移至 Worker 线程，主线程不阻塞。
 * 用户需要传入一个已创建的 Worker 实例。
 *
 * @example
 * ```tsx
 * // Vite
 * import ExportWorkerScript from '@bsg-export/worker/worker?worker';
 * const { initialized, loading, progress, exportData } = useWorkerExporter(() => new ExportWorkerScript());
 *
 * // Webpack 5
 * const { exportData } = useWorkerExporter(
 *   () => new Worker(new URL('@bsg-export/worker/worker', import.meta.url), { type: 'module' })
 * );
 * ```
 */

import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  ExportDataOptions,
  ProgressCallback,
  DataRow,
} from '@bsg-export/types';

/** useWorkerExporter Hook 的返回值 */
export interface UseWorkerExporterReturn {
  /** Worker 中的 WASM 是否已初始化完成 */
  initialized: boolean;
  /** 是否正在导出 */
  loading: boolean;
  /** 导出进度 (0-100) */
  progress: number;
  /** 错误信息 */
  error: Error | null;
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
 * Worker 线程导出管理 Hook
 *
 * 接收一个 Worker 工厂函数，自动管理 Worker 生命周期和 WASM 初始化。
 * 导出计算在 Worker 线程执行，主线程保持响应。
 *
 * @param createWorker - 创建 Worker 实例的工厂函数
 */
export function useWorkerExporter(createWorker: () => Worker): UseWorkerExporterReturn {
  const [initialized, setInitialized] = useState(false);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [error, setError] = useState<Error | null>(null);

  const workerRef = useRef<Worker | null>(null);
  const pendingRef = useRef<Map<string, PendingRequest>>(new Map());
  const mountedRef = useRef(true);

  // 处理 Worker 消息
  const handleMessage = useCallback((event: MessageEvent<WorkerResponse>) => {
    if (!mountedRef.current) return;
    const { type, id, bytes, message, progress: prog } = event.data;
    const pending = pendingRef.current.get(id);
    if (!pending) return;

    switch (type) {
      case 'ready':
        pendingRef.current.delete(id);
        pending.resolve(new ArrayBuffer(0));
        break;
      case 'result':
        pendingRef.current.delete(id);
        pending.resolve(bytes!);
        break;
      case 'error':
        pendingRef.current.delete(id);
        pending.reject(new Error(message ?? '未知错误'));
        break;
      case 'progress':
        if (pending.onProgress && prog !== undefined) {
          pending.onProgress(prog);
        }
        break;
    }
  }, []);

  // 处理 Worker 错误
  const handleError = useCallback((event: ErrorEvent) => {
    if (!mountedRef.current) return;
    const err = new Error(`Worker 错误: ${event.message}`);
    for (const [id, pending] of pendingRef.current) {
      pending.reject(err);
      pendingRef.current.delete(id);
    }
    setError(err);
  }, []);

  // 初始化 Worker 和 WASM
  useEffect(() => {
    mountedRef.current = true;

    const worker = createWorker();
    workerRef.current = worker;
    worker.addEventListener('message', handleMessage);
    worker.addEventListener('error', handleError);

    // 发送 init 消息
    const id = generateId();
    const initPromise = new Promise<ArrayBuffer>((resolve, reject) => {
      pendingRef.current.set(id, { resolve, reject });
    });

    const request: WorkerRequest = { type: 'init', id };
    worker.postMessage(request);

    initPromise
      .then(() => {
        if (mountedRef.current) setInitialized(true);
      })
      .catch((err) => {
        if (mountedRef.current) setError(err instanceof Error ? err : new Error(String(err)));
      });

    return () => {
      mountedRef.current = false;
      // 拒绝所有待处理请求
      for (const [, pending] of pendingRef.current) {
        pending.reject(new Error('组件已卸载'));
      }
      pendingRef.current.clear();
      worker.terminate();
      workerRef.current = null;
    };
  }, []); // eslint-disable-line react-hooks/exhaustive-deps

  /** 向 Worker 发送生成请求 */
  const sendGenerate = useCallback(
    (data: DataRow[], options?: Omit<ExportDataOptions, 'progressCallback'>): Promise<ArrayBuffer> => {
      const worker = workerRef.current;
      if (!worker) return Promise.reject(new Error('Worker 未创建'));

      return new Promise<ArrayBuffer>((resolve, reject) => {
        const id = generateId();
        pendingRef.current.set(id, {
          resolve,
          reject,
          onProgress: (p: number) => {
            if (mountedRef.current) setProgress(p);
          },
        });

        const request: WorkerRequest = {
          type: 'generate',
          id,
          data,
          options: (options ?? {}) as Record<string, unknown>,
        };
        worker.postMessage(request);
      });
    },
    [],
  );

  /** 在主线程触发文件下载 */
  const downloadFile = useCallback(
    (bytes: Uint8Array, filename?: string, format?: number) => {
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
    },
    [],
  );

  /** 在 Worker 中生成文件并触发下载 */
  const exportData = useCallback(
    async (data: DataRow[], options?: Omit<ExportDataOptions, 'progressCallback'>): Promise<boolean> => {
      if (!initialized) return false;
      setLoading(true);
      setProgress(0);
      setError(null);
      try {
        const buffer = await sendGenerate(data, options);
        const bytes = new Uint8Array(buffer);
        downloadFile(bytes, options?.filename, options?.format);
        if (mountedRef.current) setProgress(100);
        return true;
      } catch (err) {
        if (mountedRef.current) setError(err instanceof Error ? err : new Error(String(err)));
        return false;
      } finally {
        if (mountedRef.current) setLoading(false);
      }
    },
    [initialized, sendGenerate, downloadFile],
  );

  /** 在 Worker 中生成文件字节（不触发下载） */
  const generateBytes = useCallback(
    async (data: DataRow[], options?: Omit<ExportDataOptions, 'progressCallback'>): Promise<Uint8Array | null> => {
      if (!initialized) return null;
      setLoading(true);
      setProgress(0);
      setError(null);
      try {
        const buffer = await sendGenerate(data, options);
        if (mountedRef.current) setProgress(100);
        return new Uint8Array(buffer);
      } catch (err) {
        if (mountedRef.current) setError(err instanceof Error ? err : new Error(String(err)));
        return null;
      } finally {
        if (mountedRef.current) setLoading(false);
      }
    },
    [initialized, sendGenerate],
  );

  /** 手动销毁 Worker */
  const terminate = useCallback(() => {
    for (const [, pending] of pendingRef.current) {
      pending.reject(new Error('Worker 已被手动销毁'));
    }
    pendingRef.current.clear();
    workerRef.current?.terminate();
    workerRef.current = null;
    if (mountedRef.current) {
      setInitialized(false);
    }
  }, []);

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
