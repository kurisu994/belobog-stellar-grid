/**
 * createExcelPreview - Excel 文件预览 Primitive
 *
 * 提供 Excel 文件解析与预览能力，支持 HTML 渲染和 JSON 数据提取。
 * 使用 Solid.js createSignal 管理响应式状态。
 *
 * @example
 * ```tsx
 * import { createExcelPreview } from '@bsg-export/solid';
 *
 * const preview = createExcelPreview({
 *   init: () => import('belobog-stellar-grid').then(m => m.default()),
 *   parseExcelToHtml: (data, opts) => wasmModule.parseExcelToHtml(data, opts),
 *   parseExcelToJson: (data, opts) => wasmModule.parseExcelToJson(data, opts),
 *   getExcelSheetList: (data) => wasmModule.getExcelSheetList(data),
 * });
 * ```
 */

import { createSignal, type Accessor } from 'solid-js';
import type { PreviewOptions, ParsedWorkbook, SheetInfo } from '@bsg-export/types';

/** Excel 预览 Primitive 配置 */
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

/** createExcelPreview 返回的接口 */
export interface ExcelPreviewReturn {
  /** 是否正在解析 */
  loading: Accessor<boolean>;
  /** 错误信息 */
  error: Accessor<string | null>;
  /** HTML 输出（HTML 模式） */
  html: Accessor<string | null>;
  /** JSON 数据（JSON 模式） */
  data: Accessor<ParsedWorkbook | null>;
  /** 工作表列表 */
  sheets: Accessor<SheetInfo[]>;
  /** 当前活动 Sheet 索引 */
  activeSheet: Accessor<number>;
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
 * 将可见列表位置映射为原始工作簿索引
 *
 * `PreviewOptions.sheetIndex` 是原始工作簿索引，而 `activeSheet` / `switchSheet`
 * 用的是可见列表位置，存在隐藏 Sheet 时两者不相等，必须显式转换。
 */
function toRealIndex(visible: SheetInfo[], pos: number): number {
  return visible[pos]?.index ?? pos;
}

/** 将原始工作簿索引反查为可见列表位置（指向隐藏 Sheet 时回退到 0） */
function toVisiblePos(visible: SheetInfo[], realIndex: number): number {
  const pos = visible.findIndex(s => s.index === realIndex);
  return pos >= 0 ? pos : 0;
}

/**
 * Excel 文件预览 Primitive
 *
 * 管理 WASM 初始化、文件解析、Sheet 切换等完整预览生命周期。
 */
export function createExcelPreview(config: CreateExcelPreviewOptions): ExcelPreviewReturn {
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [html, setHtml] = createSignal<string | null>(null);
  const [data, setData] = createSignal<ParsedWorkbook | null>(null);
  const [sheets, setSheets] = createSignal<SheetInfo[]>([]);
  const [activeSheet, setActiveSheet] = createSignal(0);

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
    setSheets(visibleSheets);
    setHtml(config.parseExcelToHtml(bytes, mergedOptions));
    setData(null);
    setActiveSheet(toVisiblePos(visibleSheets, mergedOptions.sheetIndex ?? 0));
  }

  /** 加载 Excel 文件（从 File 对象） */
  async function loadFile(file: File, options?: PreviewOptions) {
    setLoading(true);
    setError(null);
    try {
      await ensureInit();
      const buffer = await file.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      fileData = bytes;
      applySheets(bytes, { ...config.defaultOptions, ...options });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  /** 加载 Excel 文件（从 Uint8Array） */
  async function loadData(bytes: Uint8Array, options?: PreviewOptions) {
    setLoading(true);
    setError(null);
    try {
      await ensureInit();
      fileData = bytes;
      applySheets(bytes, { ...config.defaultOptions, ...options });
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  /** 加载远程 Excel 文件（从 URL） */
  async function loadUrl(url: string, options?: PreviewOptions, fetchInit?: RequestInit) {
    setLoading(true);
    setError(null);
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
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  /** 切换 Sheet（传入可见 sheets 列表中的位置） */
  async function switchSheet(sheetIndex: number) {
    if (!fileData) return;
    setLoading(true);
    try {
      // 将可见列表位置映射为原始工作簿索引
      const realIndex = toRealIndex(visibleSheets, sheetIndex);
      const options = { ...config.defaultOptions, sheetIndex: realIndex };
      setHtml(config.parseExcelToHtml(fileData, options));
      setData(null);
      setActiveSheet(sheetIndex);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  /**
   * 获取 JSON 数据
   *
   * 默认解析当前活动 Sheet；`options.sheetIndex`（原始工作簿索引）可覆盖此默认值。
   */
  async function getJsonData(options?: PreviewOptions): Promise<ParsedWorkbook | null> {
    if (!fileData) return null;
    try {
      await ensureInit();
      // activeSheet 是可见列表位置，需转换为原始工作簿索引后才能传给 WASM
      const realIndex = options?.sheetIndex ?? toRealIndex(visibleSheets, activeSheet());
      const mergedOptions = { ...config.defaultOptions, ...options, sheetIndex: realIndex };
      const result = config.parseExcelToJson(fileData, mergedOptions);
      setData(result);
      return result;
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      return null;
    }
  }

  /** 重置状态 */
  function reset() {
    fileData = null;
    visibleSheets = [];
    setLoading(false);
    setError(null);
    setHtml(null);
    setData(null);
    setSheets([]);
    setActiveSheet(0);
  }

  return {
    loading,
    error,
    html,
    data,
    sheets,
    activeSheet,
    loadFile,
    loadData,
    loadUrl,
    switchSheet,
    getJsonData,
    reset,
  };
}
