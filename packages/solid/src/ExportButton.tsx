/**
 * ExportButton - 开箱即用的导出按钮组件 (Solid.js)
 *
 * 自动管理 WASM 初始化、导出状态和进度显示。
 *
 * @example
 * ```tsx
 * import { ExportButton, ExportFormat } from '@bsg-export/solid';
 *
 * <ExportButton tableId="my-table" filename="报表.xlsx" format={ExportFormat.Xlsx}>
 *   导出 Excel
 * </ExportButton>
 * ```
 */

import { createEffect, type JSX, type ParentProps } from 'solid-js';
import type { ExportFormat } from '@bsg-export/types';
import { createExporter } from './create-exporter';

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
  /** 导出成功回调 */
  onExportSuccess?: () => void;
  /** 导出失败回调 */
  onExportError?: (error: Error) => void;
  /** 进度变化回调 */
  onExportProgress?: (progress: number) => void;
  /** 初始化中的提示文本 */
  initializingText?: string;
  /** 导出中的提示文本（支持 {progress} 占位符） */
  loadingText?: string;
  /** 按钮的 CSS class */
  class?: string;
  /** 按钮的内联样式 */
  style?: string | JSX.CSSProperties;
}

/**
 * 导出按钮组件
 *
 * 封装了 WASM 初始化和导出逻辑，通过 props 配置导出参数。
 */
export function ExportButton(props: ParentProps<ExportButtonProps>) {
  const { initialized, loading, progress, error, exportTable } = createExporter();

  const initText = () => props.initializingText ?? '初始化中...';
  const loadText = () => props.loadingText ?? '导出中 {progress}%';

  // 监听错误并触发回调
  createEffect(() => {
    const err = error();
    if (err && props.onExportError) {
      props.onExportError(err);
    }
  });

  // 监听进度变化并触发回调
  createEffect(() => {
    const p = progress();
    if (props.onExportProgress) {
      props.onExportProgress(p);
    }
  });

  /** 处理点击 */
  const handleClick = () => {
    const success = exportTable({
      tableId: props.tableId,
      filename: props.filename,
      format: props.format,
      excludeHidden: props.excludeHidden,
      withBom: props.withBom,
    });

    if (success && props.onExportSuccess) {
      props.onExportSuccess();
    }
  };

  /** 渲染按钮文本 */
  const renderText = () => {
    if (!initialized()) return initText();
    if (loading()) return loadText().replace('{progress}', Math.round(progress()).toString());
    return props.children ?? '导出';
  };

  return (
    <button
      class={props.class}
      style={props.style}
      disabled={props.disabled || !initialized() || loading()}
      onClick={handleClick}
    >
      {renderText()}
    </button>
  );
}
