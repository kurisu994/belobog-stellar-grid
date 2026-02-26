/**
 * Web Worker 脚本 - 在 Worker 线程中加载 WASM 并执行导出
 *
 * 此脚本运行在 Web Worker 环境中，负责：
 * 1. 初始化 WASM 模块
 * 2. 接收主线程发来的导出请求
 * 3. 调用 generate_data_bytes 生成文件字节
 * 4. 通过 Transferable 将字节传回主线程
 */

/// <reference lib="webworker" />

/** 声明 belobog-stellar-grid 模块类型 */
declare module 'belobog-stellar-grid' {
  export function generate_data_bytes(data: unknown, options?: unknown): Uint8Array;
  export default function init(): Promise<void>;
}

/** Worker 接收的消息类型 */
interface WorkerRequest {
  /** 消息类型 */
  type: 'init' | 'generate';
  /** 请求唯一标识 */
  id: string;
  /** WASM 模块路径（init 时使用） */
  wasmUrl?: string;
  /** 导出数据（generate 时使用） */
  data?: unknown;
  /** 导出选项（generate 时使用） */
  options?: Record<string, unknown>;
}

/** Worker 发送的消息类型 */
interface WorkerResponse {
  /** 消息类型 */
  type: 'ready' | 'result' | 'error' | 'progress';
  /** 对应的请求标识 */
  id: string;
  /** 生成的文件字节（result 时） */
  bytes?: ArrayBuffer;
  /** 错误信息（error 时） */
  message?: string;
  /** 进度值 0-100（progress 时） */
  progress?: number;
}

/** WASM 模块实例 */
let wasmModule: typeof import('belobog-stellar-grid') | null = null;

/**
 * 初始化 WASM 模块
 */
async function initWasm(id: string): Promise<void> {
  try {
    const mod = await import('belobog-stellar-grid');
    await mod.default();
    wasmModule = mod;

    const response: WorkerResponse = { type: 'ready', id };
    self.postMessage(response);
  } catch (err) {
    const response: WorkerResponse = {
      type: 'error',
      id,
      message: `WASM 初始化失败: ${err instanceof Error ? err.message : String(err)}`,
    };
    self.postMessage(response);
  }
}

/**
 * 生成导出文件字节
 */
function generateBytes(
  id: string,
  data: unknown,
  options?: Record<string, unknown>,
): void {
  if (!wasmModule) {
    const response: WorkerResponse = {
      type: 'error',
      id,
      message: 'WASM 模块未初始化，请先发送 init 消息',
    };
    self.postMessage(response);
    return;
  }

  try {
    // 构建选项，注入进度回调（通过 postMessage 传回主线程）
    const workerOptions: Record<string, unknown> = { ...options };
    workerOptions.progressCallback = (progress: number) => {
      const response: WorkerResponse = { type: 'progress', id, progress };
      self.postMessage(response);
    };

    // 调用 WASM 的 generate_data_bytes
    const bytes: Uint8Array = wasmModule.generate_data_bytes(data, workerOptions);

    // 通过 Transferable 传输字节，避免拷贝
    const buffer = bytes.buffer as ArrayBuffer;
    const response: WorkerResponse = { type: 'result', id, bytes: buffer };
    self.postMessage(response, [buffer]);
  } catch (err) {
    const response: WorkerResponse = {
      type: 'error',
      id,
      message: `导出失败: ${err instanceof Error ? err.message : String(err)}`,
    };
    self.postMessage(response);
  }
}

/**
 * 监听主线程消息
 */
self.addEventListener('message', (event: MessageEvent<WorkerRequest>) => {
  const { type, id, data, options } = event.data;

  switch (type) {
    case 'init':
      initWasm(id);
      break;
    case 'generate':
      generateBytes(id, data, options);
      break;
    default:
      self.postMessage({
        type: 'error',
        id,
        message: `未知的消息类型: ${type}`,
      } satisfies WorkerResponse);
  }
});
