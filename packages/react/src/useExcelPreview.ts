/**
 * useExcelPreview - Excel 文件预览 Hook
 *
 * 提供 Excel 文件解析与预览能力，支持 HTML 渲染和 JSON 数据提取。
 *
 * @example
 * ```tsx
 * const preview = useExcelPreview({
 *   init: () => import('belobog-stellar-grid').then(m => m.default()),
 *   parseExcelToHtml: (data, opts) => wasmModule.parseExcelToHtml(data, opts),
 *   parseExcelToJson: (data, opts) => wasmModule.parseExcelToJson(data, opts),
 *   getExcelSheetList: (data) => wasmModule.getExcelSheetList(data),
 * });
 *
 * return <ExcelPreview state={preview} onSheetChange={preview.switchSheet} />;
 * ```
 */

import { useState, useCallback, useRef } from 'react';
import type { PreviewOptions, ParsedWorkbook, SheetInfo } from '@bsg-export/types';

/** Excel 预览状态 */
export interface ExcelPreviewState {
  /** 是否正在解析 */
  loading: boolean;
  /** 错误信息 */
  error: string | null;
  /** HTML 输出（HTML 模式） */
  html: string | null;
  /** JSON 数据（JSON 模式） */
  data: ParsedWorkbook | null;
  /** 工作表列表 */
  sheets: SheetInfo[];
  /** 当前活动 Sheet 索引 */
  activeSheet: number;
}

/** Excel 预览 Hook 配置 */
export interface UseExcelPreviewOptions {
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

/** useExcelPreview Hook 的返回值 */
export interface UseExcelPreviewReturn extends ExcelPreviewState {
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

/** 按核心规则选择工作表，保留显式选中的隐藏表以确保活动标签与内容一致。 */
function selectSheets(allSheets: SheetInfo[], options: PreviewOptions) {
  const selected = options.sheetName !== undefined
    ? allSheets.find(s => s.name === options.sheetName)
    : options.sheetIndex !== undefined
      ? allSheets.find(s => s.index === options.sheetIndex)
      : allSheets.find(s => options.skipHidden === false || !s.hidden) ?? allSheets[0];
  if (!selected) throw new Error('未找到指定工作表');
  const visible = allSheets.filter(s => options.skipHidden === false || !s.hidden || s.index === selected.index);
  return {
    visible,
    // 将名称选择固化为真实索引，后续切表不能继续沿用旧名称。
    options: { ...options, sheetName: undefined, sheetIndex: selected.index },
    activeSheet: visible.findIndex(s => s.index === selected.index),
  };
}

/** Excel 文件预览 Hook，管理解析与 Sheet 切换的生命周期。 */
export function useExcelPreview(config: UseExcelPreviewOptions): UseExcelPreviewReturn {
  const [state, setState] = useState<ExcelPreviewState>({
    loading: false,
    error: null,
    html: null,
    data: null,
    sheets: [],
    activeSheet: 0,
  });

  const wasmReady = useRef(false);
  const fileDataRef = useRef<Uint8Array | null>(null);
  /** 可见 sheets 缓存（用于 switchSheet 中解析真实索引） */
  const visibleSheetsRef = useRef<SheetInfo[]>([]);
  const currentOptionsRef = useRef<PreviewOptions>({});
  // 代际保护不依赖渲染闭包，重置后旧请求不得恢复状态。
  const generationRef = useRef(0);

  /** 确保 WASM 已初始化 */
  const ensureInit = useCallback(async () => {
    if (!wasmReady.current) {
      await config.init();
      wasmReady.current = true;
    }
  }, [config]);

  /** 解析成功后统一提交文件、选项和工作表状态。 */
  const applySheets = useCallback((data: Uint8Array, options: PreviewOptions) => {
    const selection = selectSheets(config.getExcelSheetList(data), options);
    const html = config.parseExcelToHtml(data, selection.options);
    fileDataRef.current = data;
    visibleSheetsRef.current = selection.visible;
    currentOptionsRef.current = selection.options;
    setState({
      loading: false,
      error: null,
      html,
      data: null,
      sheets: selection.visible,
      activeSheet: selection.activeSheet,
    });
  }, [config]);

  /** 加载 Excel 文件（从 File 对象） */
  const loadFile = useCallback(async (file: File, options?: PreviewOptions) => {
    const request = ++generationRef.current;
    const mergedOptions = { ...config.defaultOptions, ...options };
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      await ensureInit();
      if (request !== generationRef.current) return;
      const buffer = await file.arrayBuffer();
      const data = new Uint8Array(buffer);
      if (request !== generationRef.current) return;
      applySheets(data, mergedOptions);
    } catch (e) {
      if (request !== generationRef.current) return;
      setState(prev => ({
        ...prev,
        loading: false,
        error: e instanceof Error ? e.message : String(e),
      }));
    }
  }, [config, ensureInit, applySheets]);

  /** 加载 Excel 文件（从 Uint8Array） */
  const loadData = useCallback(async (data: Uint8Array, options?: PreviewOptions) => {
    const request = ++generationRef.current;
    const mergedOptions = { ...config.defaultOptions, ...options };
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      await ensureInit();
      if (request !== generationRef.current) return;
      applySheets(data, mergedOptions);
    } catch (e) {
      if (request !== generationRef.current) return;
      setState(prev => ({
        ...prev,
        loading: false,
        error: e instanceof Error ? e.message : String(e),
      }));
    }
  }, [config, ensureInit, applySheets]);

  /** 加载远程 Excel 文件（从 URL） */
  const loadUrl = useCallback(async (url: string, options?: PreviewOptions, fetchInit?: RequestInit) => {
    const request = ++generationRef.current;
    const mergedOptions = { ...config.defaultOptions, ...options };
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      await ensureInit();
      if (request !== generationRef.current) return;
      const response = await fetch(url, fetchInit);
      if (!response.ok) {
        throw new Error(`远程文件加载失败: HTTP ${response.status} ${response.statusText}`);
      }
      const buffer = await response.arrayBuffer();
      const data = new Uint8Array(buffer);
      if (request !== generationRef.current) return;
      applySheets(data, mergedOptions);
    } catch (e) {
      if (request !== generationRef.current) return;
      setState(prev => ({
        ...prev,
        loading: false,
        error: e instanceof Error ? e.message : String(e),
      }));
    }
  }, [config, ensureInit, applySheets]);

  /** 切换 Sheet（传入可见 sheets 列表中的位置） */
  const switchSheet = useCallback(async (sheetIndex: number) => {
    if (!fileDataRef.current || !visibleSheetsRef.current[sheetIndex]) return;
    ++generationRef.current;
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      // 将可见列表位置映射为原始工作簿索引
      const realIndex = visibleSheetsRef.current[sheetIndex].index;
      const options = { ...currentOptionsRef.current, sheetName: undefined, sheetIndex: realIndex };
      const html = config.parseExcelToHtml(fileDataRef.current, options);
      currentOptionsRef.current = options;
      setState(prev => ({
        ...prev,
        loading: false,
        html,
        data: null,
        activeSheet: sheetIndex,
      }));
    } catch (e) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: e instanceof Error ? e.message : String(e),
      }));
    }
  }, [config]);

  /**
   * 获取 JSON 数据
   *
   * 默认沿用当前 Sheet 与本次加载选项；显式 sheetName 优先于 sheetIndex 覆盖选择。
   */
  const getJsonData = useCallback(async (options?: PreviewOptions): Promise<ParsedWorkbook | null> => {
    if (!fileDataRef.current) return null;
    const request = generationRef.current;
    const bytes = fileDataRef.current;
    const mergedOptions = { ...currentOptionsRef.current, ...options };
    try {
      await ensureInit();
      if (request !== generationRef.current || bytes !== fileDataRef.current) return null;
      const data = config.parseExcelToJson(bytes, mergedOptions);
      setState(prev => ({ ...prev, data }));
      return data;
    } catch (e) {
      if (request !== generationRef.current || bytes !== fileDataRef.current) return null;
      setState(prev => ({
        ...prev,
        error: e instanceof Error ? e.message : String(e),
      }));
      return null;
    }
  }, [config, ensureInit]);

  /** 重置状态 */
  const reset = useCallback(() => {
    ++generationRef.current;
    currentOptionsRef.current = {};
    visibleSheetsRef.current = [];
    fileDataRef.current = null;
    setState({
      loading: false,
      error: null,
      html: null,
      data: null,
      sheets: [],
      activeSheet: 0,
    });
  }, []);

  return {
    ...state,
    loadFile,
    loadData,
    loadUrl,
    switchSheet,
    getJsonData,
    reset,
  };
}
