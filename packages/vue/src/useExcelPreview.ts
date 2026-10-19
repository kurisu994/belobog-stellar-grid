/**
 * useExcelPreview - Excel 文件预览 Composable
 *
 * 提供 Excel 文件解析与预览能力，支持 HTML 渲染和 JSON 数据提取。
 *
 * @example
 * ```vue
 * <script setup lang="ts">
 * import { useExcelPreview } from '@bsg-export/vue';
 *
 * const preview = useExcelPreview({
 *   init: () => import('belobog-stellar-grid').then(m => m.default()),
 *   parseExcelToHtml: (data, opts) => wasmModule.parseExcelToHtml(data, opts),
 *   parseExcelToJson: (data, opts) => wasmModule.parseExcelToJson(data, opts),
 *   getExcelSheetList: (data) => wasmModule.getExcelSheetList(data),
 * });
 * </script>
 *
 * <template>
 *   <ExcelPreview :state="preview" @sheet-change="preview.switchSheet" />
 * </template>
 * ```
 */

import { ref, type Ref } from 'vue';
import type { PreviewOptions, ParsedWorkbook, SheetInfo } from '@bsg-export/types';

/** Excel 预览 Composable 配置 */
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

/** useExcelPreview Composable 的返回值 */
export interface UseExcelPreviewReturn {
  /** 是否正在解析 */
  loading: Ref<boolean>;
  /** 错误信息 */
  error: Ref<string | null>;
  /** HTML 输出（HTML 模式） */
  html: Ref<string | null>;
  /** JSON 数据（JSON 模式） */
  data: Ref<ParsedWorkbook | null>;
  /** 工作表列表 */
  sheets: Ref<SheetInfo[]>;
  /** 当前活动 Sheet 索引 */
  activeSheet: Ref<number>;
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

/** Excel 文件预览 Composable，管理解析与 Sheet 切换的生命周期。 */
export function useExcelPreview(config: UseExcelPreviewOptions): UseExcelPreviewReturn {
  const loading = ref(false);
  const error = ref<string | null>(null);
  const html = ref<string | null>(null);
  const data = ref<ParsedWorkbook | null>(null);
  const sheets = ref<SheetInfo[]>([]);
  const activeSheet = ref(0);

  let wasmReady = false;
  let fileData: Uint8Array | null = null;
  /** 可见 sheets 缓存（用于 switchSheet 中解析真实索引） */
  let visibleSheets: SheetInfo[] = [];
  let currentOptions: PreviewOptions = {};
  // 每次加载、切表和重置均使旧异步任务失效。
  let generation = 0;

  /** 确保 WASM 已初始化 */
  async function ensureInit() {
    if (!wasmReady) {
      await config.init();
      wasmReady = true;
    }
  }

  /** 解析成功后统一提交文件、选项和工作表状态。 */
  function applySheets(bytes: Uint8Array, mergedOptions: PreviewOptions) {
    const selection = selectSheets(config.getExcelSheetList(bytes), mergedOptions);
    const rendered = config.parseExcelToHtml(bytes, selection.options);
    fileData = bytes;
    currentOptions = selection.options;
    visibleSheets = selection.visible;
    sheets.value = selection.visible;
    html.value = rendered;
    data.value = null;
    activeSheet.value = selection.activeSheet;
  }

  /** 加载 Excel 文件（从 File 对象） */
  async function loadFile(file: File, options?: PreviewOptions) {
    const request = ++generation;
    const mergedOptions = { ...config.defaultOptions, ...options };
    loading.value = true;
    error.value = null;
    try {
      await ensureInit();
      if (request !== generation) return;
      const buffer = await file.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      if (request !== generation) return;
      applySheets(bytes, mergedOptions);
    } catch (e) {
      if (request !== generation) return;
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      if (request !== generation) return;
      loading.value = false;
    }
  }

  /** 加载 Excel 文件（从 Uint8Array） */
  async function loadData(bytes: Uint8Array, options?: PreviewOptions) {
    const request = ++generation;
    const mergedOptions = { ...config.defaultOptions, ...options };
    loading.value = true;
    error.value = null;
    try {
      await ensureInit();
      if (request !== generation) return;
      applySheets(bytes, mergedOptions);
    } catch (e) {
      if (request !== generation) return;
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      if (request !== generation) return;
      loading.value = false;
    }
  }

  /** 加载远程 Excel 文件（从 URL） */
  async function loadUrl(url: string, options?: PreviewOptions, fetchInit?: RequestInit) {
    const request = ++generation;
    const mergedOptions = { ...config.defaultOptions, ...options };
    loading.value = true;
    error.value = null;
    try {
      await ensureInit();
      if (request !== generation) return;
      const response = await fetch(url, fetchInit);
      if (!response.ok) {
        throw new Error(`远程文件加载失败: HTTP ${response.status} ${response.statusText}`);
      }
      const buffer = await response.arrayBuffer();
      const bytes = new Uint8Array(buffer);
      if (request !== generation) return;
      applySheets(bytes, mergedOptions);
    } catch (e) {
      if (request !== generation) return;
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      if (request !== generation) return;
      loading.value = false;
    }
  }

  /** 切换 Sheet（传入可见 sheets 列表中的位置） */
  async function switchSheet(sheetIndex: number) {
    if (!fileData || !visibleSheets[sheetIndex]) return;
    ++generation;
    loading.value = true;
    error.value = null;
    try {
      // 将可见列表位置映射为原始工作簿索引
      const realIndex = visibleSheets[sheetIndex].index;
      const options = { ...currentOptions, sheetName: undefined, sheetIndex: realIndex };
      html.value = config.parseExcelToHtml(fileData, options);
      currentOptions = options;
      data.value = null;
      activeSheet.value = sheetIndex;
    } catch (e) {
      error.value = e instanceof Error ? e.message : String(e);
    } finally {
      loading.value = false;
    }
  }

  /**
   * 获取 JSON 数据
   *
   * 默认沿用当前 Sheet 与本次加载选项；显式 sheetName 优先于 sheetIndex 覆盖选择。
   */
  async function getJsonData(options?: PreviewOptions): Promise<ParsedWorkbook | null> {
    if (!fileData) return null;
    const request = generation;
    const bytes = fileData;
    const mergedOptions = { ...currentOptions, ...options };
    try {
      await ensureInit();
      if (request !== generation || bytes !== fileData) return null;
      const result = config.parseExcelToJson(bytes, mergedOptions);
      data.value = result;
      return result;
    } catch (e) {
      if (request !== generation || bytes !== fileData) return null;
      error.value = e instanceof Error ? e.message : String(e);
      return null;
    }
  }

  /** 重置状态 */
  function reset() {
    ++generation;
    currentOptions = {};
    fileData = null;
    visibleSheets = [];
    loading.value = false;
    error.value = null;
    html.value = null;
    data.value = null;
    sheets.value = [];
    activeSheet.value = 0;
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
