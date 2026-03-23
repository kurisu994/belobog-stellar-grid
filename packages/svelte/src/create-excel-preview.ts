/**
 * createExcelPreview - Excel 文件预览 Store 工厂函数
 *
 * 提供 Excel 文件解析与预览能力，支持 HTML 渲染和 JSON 数据提取。
 * 使用 Svelte writable/readable store 管理响应式状态。
 *
 * @example
 * ```svelte
 * <script>
 * import { createExcelPreview } from '@bsg-export/svelte';
 *
 * const preview = createExcelPreview({
 *   init: () => import('belobog-stellar-grid').then(m => m.default()),
 *   parseExcelToHtml: (data, opts) => wasmModule.parseExcelToHtml(data, opts),
 *   parseExcelToJson: (data, opts) => wasmModule.parseExcelToJson(data, opts),
 *   getExcelSheetList: (data) => wasmModule.getExcelSheetList(data),
 * });
 * </script>
 * ```
 */

import { writable, readonly, type Readable } from 'svelte/store';
import type { PreviewOptions, ParsedWorkbook, SheetInfo } from '@bsg-export/types';

/** Excel 预览 Store 配置 */
export interface CreateExcelPreviewOptions {
  /** WASM 模块初始化函数 */
  init: () => Promise<void>;
  /** parseExcelToHtml 函数 */
  parseExcelToHtml: (data: Uint8Array, options?: PreviewOptions) => string;
  /** parseExcelToJson 函数 */
  parseExcelToJson: (data: Uint8Array, options?: PreviewOptions) => ParsedWorkbook;
  /** getExcelSheetList 函数 */
  getExcelSheetList: (data: Uint8Array) => SheetInfo[];
  /** 默认预览配置 */
  defaultOptions?: PreviewOptions;
}

/** createExcelPreview 返回的 Store 接口 */
export interface ExcelPreviewStore {
  /** 是否正在解析 */
  loading: Readable<boolean>;
  /** 错误信息 */
  error: Readable<string | null>;
  /** HTML 输出（HTML 模式） */
  html: Readable<string | null>;
  /** JSON 数据（JSON 模式） */
  data: Readable<ParsedWorkbook | null>;
  /** 工作表列表 */
  sheets: Readable<SheetInfo[]>;
  /** 当前活动 Sheet 索引 */
  activeSheet: Readable<number>;
  /** 加载 Excel 文件（从 File 对象） */
  loadFile: (file: File, options?: PreviewOptions) => Promise<void>;
  /** 加载 Excel 文件（从 Uint8Array） */
  loadData: (data: Uint8Array, options?: PreviewOptions) => Promise<void>;
  /** 加载远程 Excel 文件（从 URL） */
  loadUrl: (url: string, options?: PreviewOptions, fetchInit?: RequestInit) => Promise<void>;
  /** 切换 Sheet */
  switchSheet: (sheetIndex: number) => Promise<void>;
  /** 获取 JSON 数据 */
  getJsonData: (options?: PreviewOptions) => Promise<ParsedWorkbook | null>;
  /** 重置状态 */
  reset: () => void;
}

/**
 * Excel 文件预览 Store 工厂函数
 *
 * 管理 WASM 初始化、文件解析、Sheet 切换等完整预览生命周期。
 */
export function createExcelPreview(config: CreateExcelPreviewOptions): ExcelPreviewStore {
  const loading = writable(false);
  const error = writable<string | null>(null);
  const html = writable<string | null>(null);
  const data = writable<ParsedWorkbook | null>(null);
  const sheets = writable<SheetInfo[]>([]);
  const activeSheet = writable(0);

  let wasmReady = false;
  let fileData: Uint8Array | null = null;
  /** 可见 sheets 缓存（用于 switchSheet 中解析真实索引） */
  let visibleSheets: SheetInfo[] = [];

  /** 确保 WASM 已初始化 */
  async function ensureInit() {
    if (!wasmReady) {
      await config.init();
      wasmReady = true;
    }
  }

  /** 内部加载逻辑（统一处理 hidden sheet 过滤） */
  function applySheets(bytes: Uint8Array, mergedOptions: PreviewOptions) {
    const allSheets = config.getExcelSheetList(bytes);
    visibleSheets = allSheets.filter(s => !s.hidden);
    sheets.set(visibleSheets);
    html.set(config.parseExcelToHtml(bytes, mergedOptions));
    data.set(null);
    activeSheet.set(mergedOptions.sheetIndex ?? 0);
  }

  /** 加载 Excel 文件（从 File 对象） */
  async function loadFile(file: File, options?: PreviewOptions) {
    loading.set(true);
    error.set(null);
    try {
      await ensureInit();
      const buffer = await file.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      fileData = bytes;
      applySheets(bytes, { ...config.defaultOptions, ...options });
    } catch (e) {
      error.set(e instanceof Error ? e.message : String(e));
    } finally {
      loading.set(false);
    }
  }

  /** 加载 Excel 文件（从 Uint8Array） */
  async function loadData(bytes: Uint8Array, options?: PreviewOptions) {
    loading.set(true);
    error.set(null);
    try {
      await ensureInit();
      fileData = bytes;
      applySheets(bytes, { ...config.defaultOptions, ...options });
    } catch (e) {
      error.set(e instanceof Error ? e.message : String(e));
    } finally {
      loading.set(false);
    }
  }

  /** 加载远程 Excel 文件（从 URL） */
  async function loadUrl(url: string, options?: PreviewOptions, fetchInit?: RequestInit) {
    loading.set(true);
    error.set(null);
    try {
      await ensureInit();
      const response = await fetch(url, fetchInit);
      if (!response.ok) {
        throw new Error(`远程文件加载失败: HTTP ${response.status} ${response.statusText}`);
      }
      const buffer = await response.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      fileData = bytes;
      applySheets(bytes, { ...config.defaultOptions, ...options });
    } catch (e) {
      error.set(e instanceof Error ? e.message : String(e));
    } finally {
      loading.set(false);
    }
  }

  /** 切换 Sheet（传入可见 sheets 列表中的位置） */
  async function switchSheet(sheetIndex: number) {
    if (!fileData) return;
    loading.set(true);
    try {
      // 将可见列表位置映射为原始工作簿索引
      const realIndex = visibleSheets[sheetIndex]?.index ?? sheetIndex;
      const options = { ...config.defaultOptions, sheetIndex: realIndex };
      html.set(config.parseExcelToHtml(fileData, options));
      data.set(null);
      activeSheet.set(sheetIndex);
    } catch (e) {
      error.set(e instanceof Error ? e.message : String(e));
    } finally {
      loading.set(false);
    }
  }

  /** 获取 JSON 数据 */
  async function getJsonData(options?: PreviewOptions): Promise<ParsedWorkbook | null> {
    if (!fileData) return null;
    try {
      await ensureInit();
      let currentActiveSheet = 0;
      activeSheet.subscribe(v => { currentActiveSheet = v; })();
      const mergedOptions = { ...config.defaultOptions, ...options, sheetIndex: currentActiveSheet };
      const result = config.parseExcelToJson(fileData, mergedOptions);
      data.set(result);
      return result;
    } catch (e) {
      error.set(e instanceof Error ? e.message : String(e));
      return null;
    }
  }

  /** 重置状态 */
  function reset() {
    fileData = null;
    visibleSheets = [];
    loading.set(false);
    error.set(null);
    html.set(null);
    data.set(null);
    sheets.set([]);
    activeSheet.set(0);
  }

  return {
    loading: readonly(loading),
    error: readonly(error),
    html: readonly(html),
    data: readonly(data),
    sheets: readonly(sheets),
    activeSheet: readonly(activeSheet),
    loadFile,
    loadData,
    loadUrl,
    switchSheet,
    getJsonData,
    reset,
  };
}
