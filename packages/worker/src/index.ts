/**
 * ExportWorker - Web Worker 导出管理器
 *
 * 将导出计算移至 Worker 线程，彻底避免主线程阻塞。
 * 适用于大数据量场景下的 CSV/XLSX 导出。
 *
 * @example
 * ```typescript
 * import { ExportWorker } from '@bsg-export/worker';
 * import MyWorker from './my-worker?worker';
 *
 * const worker = new ExportWorker(new MyWorker());
 * await worker.init();
 *
 * // 导出数据（在 Worker 线程中生成，主线程触发下载）
 * await worker.exportData(
 *   [{ name: '张三', age: 28 }],
 *   {
 *     columns: [{ title: '姓名', key: 'name' }, { title: '年龄', key: 'age' }],
 *     filename: '用户.xlsx',
 *     format: 1, // ExportFormat.Xlsx
 *   }
 * );
 *
 * // 使用完毕后销毁
 * worker.terminate();
 * ```
 */

import type {
  ExportDataOptions,
  DataRow,
  ProgressCallback,
} from '@bsg-export/types';

/** Worker 消息类型 */
interface WorkerRequest {
  type: 'init' | 'generate';
  id: string;
  data?: unknown;
  options?: Record<string, unknown>;
}

/** Worker 响应类型 */
interface WorkerResponse {
  type: 'ready' | 'result' | 'error' | 'progress';
  id: string;
  bytes?: ArrayBuffer;
  message?: string;
  progress?: number;
}

/** ExportWorker 配置选项 */
export interface ExportWorkerOptions {
  /** 进度回调函数 */
  onProgress?: ProgressCallback;
}

/** 待处理的请求 */
interface PendingRequest {
  resolve: (value: ArrayBuffer) => void;
  reject: (reason: Error) => void;
  onProgress?: ProgressCallback;
}

/**
 * 生成唯一请求 ID
 */
let requestCounter = 0;
function generateId(): string {
  return `req_${++requestCounter}_${Date.now()}`;
}

/**
 * Web Worker 导出管理器
 *
 * 封装 Worker 生命周期管理、消息协议、文件下载触发。
 */
export class ExportWorker {
  private worker: Worker;
  private pending = new Map<string, PendingRequest>();
  private _initialized = false;

  /**
   * 创建 ExportWorker 实例
   *
   * @param worker - Worker 实例，由用户根据构建工具自行创建
   *
   * @example
   * ```typescript
   * // Vite
   * import MyWorker from './export-worker?worker';
   * const ew = new ExportWorker(new MyWorker());
   *
   * // Webpack 5
   * const worker = new Worker(new URL('./export-worker', import.meta.url));
   * const ew = new ExportWorker(worker);
   * ```
   */
  constructor(worker: Worker) {
    this.worker = worker;
    this.worker.addEventListener('message', this.handleMessage.bind(this));
    this.worker.addEventListener('error', this.handleError.bind(this));
  }

  /** WASM 是否已初始化完成 */
  get initialized(): boolean {
    return this._initialized;
  }

  /**
   * 初始化 Worker 中的 WASM 模块
   *
   * 必须在调用 exportData 之前调用。
   */
  async init(): Promise<void> {
    if (this._initialized) return;

    return new Promise<void>((resolve, reject) => {
      const id = generateId();

      this.pending.set(id, {
        resolve: () => {
          this._initialized = true;
          resolve();
        },
        reject,
      });

      const request: WorkerRequest = { type: 'init', id };
      this.worker.postMessage(request);
    });
  }

  /**
   * 在 Worker 中生成文件并触发下载
   *
   * @param data - 导出数据（二维数组或对象数组）
   * @param options - 导出选项（同 ExportDataOptions，但 progressCallback 通过 onProgress 传入）
   * @param workerOptions - Worker 特有选项
   *
   * @returns Promise，导出完成时 resolve
   * @throws 导出失败时 reject
   */
  async exportData(
    data: DataRow[],
    options?: Omit<ExportDataOptions, 'progressCallback'>,
    workerOptions?: ExportWorkerOptions,
  ): Promise<void> {
    if (!this._initialized) {
      throw new Error('ExportWorker 未初始化，请先调用 init()');
    }

    const bytes = await this.generateBytes(data, options, workerOptions);

    // 在主线程触发下载
    this.downloadFile(
      bytes,
      options?.filename,
      options?.format,
      options?.withBom,
    );
  }

  /**
   * 仅生成文件字节（不触发下载）
   *
   * 适用于需要自定义下载逻辑或进一步处理文件内容的场景。
   *
   * @param data - 导出数据
   * @param options - 导出选项
   * @param workerOptions - Worker 特有选项
   *
   * @returns 文件字节的 Uint8Array
   */
  async generateBytes(
    data: DataRow[],
    options?: Omit<ExportDataOptions, 'progressCallback'>,
    workerOptions?: ExportWorkerOptions,
  ): Promise<Uint8Array> {
    if (!this._initialized) {
      throw new Error('ExportWorker 未初始化，请先调用 init()');
    }

    const buffer = await new Promise<ArrayBuffer>((resolve, reject) => {
      const id = generateId();

      this.pending.set(id, {
        resolve,
        reject,
        onProgress: workerOptions?.onProgress,
      });

      // Worker 中不传 progressCallback（它不可序列化），
      // Worker 会自动注入 postMessage 版的进度回调
      const { ...safeOptions } = options ?? {};
      const request: WorkerRequest = {
        type: 'generate',
        id,
        data,
        options: safeOptions as Record<string, unknown>,
      };
      this.worker.postMessage(request);
    });

    return new Uint8Array(buffer);
  }

  /**
   * 销毁 Worker
   */
  terminate(): void {
    // 拒绝所有待处理的请求
    for (const [, pending] of this.pending) {
      pending.reject(new Error('Worker 已被销毁'));
    }
    this.pending.clear();

    this.worker.terminate();
    this._initialized = false;
  }

  /**
   * 处理 Worker 消息
   */
  private handleMessage(event: MessageEvent<WorkerResponse>): void {
    const { type, id, bytes, message, progress } = event.data;
    const pending = this.pending.get(id);

    if (!pending) return;

    switch (type) {
      case 'ready':
        // init 成功
        this.pending.delete(id);
        pending.resolve(new ArrayBuffer(0));
        break;

      case 'result':
        // 生成成功
        this.pending.delete(id);
        pending.resolve(bytes!);
        break;

      case 'error':
        // 出错
        this.pending.delete(id);
        pending.reject(new Error(message ?? '未知错误'));
        break;

      case 'progress':
        // 进度报告
        if (pending.onProgress && progress !== undefined) {
          pending.onProgress(progress);
        }
        break;
    }
  }

  /**
   * 处理 Worker 错误
   */
  private handleError(event: ErrorEvent): void {
    // 拒绝所有待处理的请求
    const error = new Error(`Worker 错误: ${event.message}`);
    for (const [id, pending] of this.pending) {
      pending.reject(error);
      this.pending.delete(id);
    }
  }

  /**
   * 在主线程触发文件下载
   */
  private downloadFile(
    data: Uint8Array,
    filename?: string,
    format?: number,
    withBom?: boolean,
  ): void {
    // 确定 MIME 类型和文件扩展名
    const isXlsx = format === 1;
    const mimeType = isXlsx
      ? 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet'
      : 'text/csv;charset=utf-8';
    const defaultExt = isXlsx ? 'xlsx' : 'csv';
    const defaultFilename = `export.${defaultExt}`;

    let finalFilename = filename ?? defaultFilename;

    // 确保文件有正确的扩展名
    if (!finalFilename.endsWith(`.${defaultExt}`)) {
      finalFilename = `${finalFilename}.${defaultExt}`;
    }

    // CSV 且需要 BOM 时，在字节前面加上 BOM（Worker 中 generate_data_bytes 已处理）
    // 这里无需额外处理，因为 withBom 选项已传给 Worker

    // 创建 Blob 并触发下载
    const blob = new Blob([data.buffer as ArrayBuffer], { type: mimeType });
    const url = URL.createObjectURL(blob);

    const anchor = document.createElement('a');
    anchor.href = url;
    anchor.download = finalFilename;
    anchor.click();

    // 延迟 10 秒后释放 Blob URL
    setTimeout(() => URL.revokeObjectURL(url), 10_000);
  }
}

export type { ExportWorkerOptions as WorkerOptions };
