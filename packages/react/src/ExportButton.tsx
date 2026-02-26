/**
 * ExportButton - 开箱即用的导出按钮组件
 *
 * 自动管理 WASM 初始化、导出状态和进度显示。
 *
 * @example
 * ```tsx
 * <ExportButton tableId="my-table" filename="报表.xlsx" format={ExportFormat.Xlsx}>
 *   导出 Excel
 * </ExportButton>
 * ```
 */

import React from 'react';
import type { ExportFormat } from '@bsg-export/types';
import { useExporter } from './use-exporter';

/** ExportButton 组件的 Props */
export interface ExportButtonProps
  extends Omit<React.ButtonHTMLAttributes<HTMLButtonElement>, 'onError'> {
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
}

/**
 * 导出按钮组件
 *
 * 封装了 WASM 初始化和导出逻辑，通过 props 配置导出参数。
 */
export function ExportButton({
  tableId,
  filename,
  format,
  excludeHidden,
  withBom,
  onExportSuccess,
  onExportError,
  onExportProgress,
  initializingText = '初始化中...',
  loadingText = '导出中 {progress}%',
  children = '导出',
  disabled,
  ...buttonProps
}: ExportButtonProps) {
  const { initialized, loading, progress, error, exportTable } = useExporter();

  React.useEffect(() => {
    if (error && onExportError) {
      onExportError(error);
    }
  }, [error, onExportError]);

  React.useEffect(() => {
    if (onExportProgress) {
      onExportProgress(progress);
    }
  }, [progress, onExportProgress]);

  const handleClick = React.useCallback(
    (e: React.MouseEvent<HTMLButtonElement>) => {
      buttonProps.onClick?.(e);
      if (e.defaultPrevented) return;

      exportTable({
        tableId,
        filename,
        format,
        excludeHidden,
        withBom,
      });

      // 导出成功回调（同步导出完成后立即触发）
      if (onExportSuccess && !error) {
        onExportSuccess();
      }
    },
    [tableId, filename, format, excludeHidden, withBom, exportTable, onExportSuccess, error, buttonProps],
  );

  /** 渲染按钮文本 */
  const renderText = () => {
    if (!initialized) return initializingText;
    if (loading) return loadingText.replace('{progress}', Math.round(progress).toString());
    return children;
  };

  return (
    <button
      {...buttonProps}
      disabled={disabled || !initialized || loading}
      onClick={handleClick}
    >
      {renderText()}
    </button>
  );
}
