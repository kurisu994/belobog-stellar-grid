/**
 * ExcelPreview - Excel 文件预览组件
 *
 * 渲染 Excel 解析后的 HTML 表格，支持多 Sheet 切换。
 *
 * @example
 * ```tsx
 * const preview = useExcelPreview({ ... });
 *
 * <ExcelPreview
 *   state={preview}
 *   onSheetChange={preview.switchSheet}
 *   maxHeight="500px"
 * />
 * ```
 */

import React from 'react';
import type { ExcelPreviewState } from './useExcelPreview';

/** ExcelPreview 组件属性 */
export interface ExcelPreviewProps {
  /** 预览状态（来自 useExcelPreview） */
  state: ExcelPreviewState;
  /** 切换 Sheet 回调 */
  onSheetChange?: (index: number) => void;
  /** 容器类名 */
  className?: string;
  /** 容器样式 */
  style?: React.CSSProperties;
  /** 表格容器最大高度 */
  maxHeight?: string | number;
}

/** Excel 预览组件 */
export const ExcelPreview: React.FC<ExcelPreviewProps> = ({
  state,
  onSheetChange,
  className,
  style,
  maxHeight = '600px',
}) => {
  if (state.loading) {
    return <div className={className} style={style}>加载中...</div>;
  }

  if (state.error) {
    return <div className={className} style={{ color: 'red', ...style }}>{state.error}</div>;
  }

  if (!state.html) {
    return null;
  }

  return (
    <div className={className} style={style}>
      {/* 表格预览区域 */}
      <div
        style={{
          overflow: 'auto',
          maxHeight,
          border: '1px solid #e0e0e0',
          borderRadius: '4px',
        }}
        dangerouslySetInnerHTML={{ __html: state.html }}
      />

      {/* Sheet 标签栏 */}
      {state.sheets.length > 1 && (
        <div style={{
          display: 'flex',
          gap: '2px',
          marginTop: '8px',
          borderTop: '1px solid #e0e0e0',
          paddingTop: '8px',
        }}>
          {state.sheets.map((sheet, index) => (
            <button
              key={sheet.name}
              onClick={() => onSheetChange?.(index)}
              style={{
                padding: '4px 12px',
                border: '1px solid #ccc',
                borderRadius: '4px 4px 0 0',
                background: index === state.activeSheet ? '#fff' : '#f5f5f5',
                fontWeight: index === state.activeSheet ? 'bold' : 'normal',
                cursor: 'pointer',
                fontSize: '12px',
              }}
            >
              {sheet.name}
            </button>
          ))}
        </div>
      )}
    </div>
  );
};
