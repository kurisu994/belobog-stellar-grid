import { SvelteComponent } from 'svelte';
import type { SheetInfo } from '@bsg-export/types';

/** ExcelPreview 组件的 Props */
export interface ExcelPreviewProps {
  /** 是否正在解析 */
  loading?: boolean;
  /** 错误信息 */
  error?: string | null;
  /** HTML 输出 */
  html?: string | null;
  /** 工作表列表 */
  sheets?: SheetInfo[];
  /** 当前活动 Sheet 索引 */
  activeSheet?: number;
  /** 表格容器最大高度 */
  maxHeight?: string | number;
}

/** ExcelPreview 组件的事件 */
export interface ExcelPreviewEvents {
  /** Sheet 切换事件 */
  sheetChange: CustomEvent<number>;
}

/** Excel 文件预览组件 */
export default class ExcelPreview extends SvelteComponent<
  ExcelPreviewProps,
  ExcelPreviewEvents
> {}
