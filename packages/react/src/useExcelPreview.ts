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
  /** 切换 Sheet */
  switchSheet: (sheetIndex: number) => Promise<void>;
  /** 获取 JSON 数据 */
  getJsonData: (options?: PreviewOptions) => Promise<ParsedWorkbook | null>;
  /** 重置状态 */
  reset: () => void;
}

/**
 * Excel 文件预览 Hook
 *
 * 管理 WASM 初始化、文件解析、Sheet 切换等完整预览生命周期。
 */
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

  /** 确保 WASM 已初始化 */
  const ensureInit = useCallback(async () => {
    if (!wasmReady.current) {
      await config.init();
      wasmReady.current = true;
    }
  }, [config]);

  /** 加载 Excel 文件（从 File 对象） */
  const loadFile = useCallback(async (file: File, options?: PreviewOptions) => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      await ensureInit();
      const buffer = await file.arrayBuffer();
      const data = new Uint8Array(buffer);
      fileDataRef.current = data;

      const mergedOptions = { ...config.defaultOptions, ...options };
      const sheets = config.getExcelSheetList(data);
      const html = config.parseExcelToHtml(data, mergedOptions);

      setState({
        loading: false,
        error: null,
        html,
        data: null,
        sheets,
        activeSheet: mergedOptions.sheetIndex ?? 0,
      });
    } catch (e) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: e instanceof Error ? e.message : String(e),
      }));
    }
  }, [config, ensureInit]);

  /** 加载 Excel 文件（从 Uint8Array） */
  const loadData = useCallback(async (data: Uint8Array, options?: PreviewOptions) => {
    setState(prev => ({ ...prev, loading: true, error: null }));
    try {
      await ensureInit();
      fileDataRef.current = data;

      const mergedOptions = { ...config.defaultOptions, ...options };
      const sheets = config.getExcelSheetList(data);
      const html = config.parseExcelToHtml(data, mergedOptions);

      setState({
        loading: false,
        error: null,
        html,
        data: null,
        sheets,
        activeSheet: mergedOptions.sheetIndex ?? 0,
      });
    } catch (e) {
      setState(prev => ({
        ...prev,
        loading: false,
        error: e instanceof Error ? e.message : String(e),
      }));
    }
  }, [config, ensureInit]);

  /** 切换 Sheet */
  const switchSheet = useCallback(async (sheetIndex: number) => {
    if (!fileDataRef.current) return;
    setState(prev => ({ ...prev, loading: true }));
    try {
      const options = { ...config.defaultOptions, sheetIndex };
      const html = config.parseExcelToHtml(fileDataRef.current!, options);
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

  /** 获取 JSON 数据 */
  const getJsonData = useCallback(async (options?: PreviewOptions): Promise<ParsedWorkbook | null> => {
    if (!fileDataRef.current) return null;
    try {
      await ensureInit();
      const mergedOptions = { ...config.defaultOptions, ...options, sheetIndex: state.activeSheet };
      const data = config.parseExcelToJson(fileDataRef.current!, mergedOptions);
      setState(prev => ({ ...prev, data }));
      return data;
    } catch (e) {
      setState(prev => ({
        ...prev,
        error: e instanceof Error ? e.message : String(e),
      }));
      return null;
    }
  }, [config, ensureInit, state.activeSheet]);

  /** 重置状态 */
  const reset = useCallback(() => {
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
    switchSheet,
    getJsonData,
    reset,
  };
}
