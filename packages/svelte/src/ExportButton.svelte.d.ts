import { SvelteComponent } from 'svelte';
import type { ExportFormat } from '@bsg-export/types';

/** ExportButton 组件的 Props */
export interface ExportButtonProps {
  /** 要导出的 HTML 表格元素的 ID */
  tableId: string;
  /** 导出文件名 */
  filename?: string;
  /** 导出格式 */
  format?: ExportFormat;
  /** 是否排除隐藏行/列 */
  excludeHidden?: boolean;
  /** 是否添加 UTF-8 BOM（仅 CSV 有效） */
  withBom?: boolean;
  /** 是否禁用按钮 */
  disabled?: boolean;
  /** 初始化中的提示文本 */
  initializingText?: string;
  /** 导出中的提示文本（支持 {progress} 占位符） */
  loadingText?: string;
  /** 导出成功回调 */
  onExportSuccess?: () => void;
  /** 导出失败回调 */
  onExportError?: (error: Error) => void;
  /** 进度变化回调 */
  onExportProgress?: (progress: number) => void;
}

/** 开箱即用的导出按钮组件 */
export default class ExportButton extends SvelteComponent<ExportButtonProps> {}
