/**
 * ExcelPreview - Excel 文件预览组件 (Solid.js)
 *
 * 渲染 Excel 解析后的 HTML 表格，支持多 Sheet 切换。
 *
 * @example
 * ```tsx
 * import { ExcelPreview, createExcelPreview } from '@bsg-export/solid';
 *
 * const preview = createExcelPreview({ ... });
 *
 * <ExcelPreview
 *   loading={preview.loading()}
 *   html={preview.html()}
 *   sheets={preview.sheets()}
 *   activeSheet={preview.activeSheet()}
 *   onSheetChange={(index) => preview.switchSheet(index)}
 * />
 * ```
 */

import { Show, For } from 'solid-js';
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
  /** Sheet 切换回调 */
  onSheetChange?: (sheetIndex: number) => void;
  /** 表格容器最大高度 */
  maxHeight?: string | number;
  /** CSS class */
  class?: string;
  /** 内联样式 */
  style?: string;
}

/**
 * Excel 文件预览组件
 *
 * 封装了加载/错误/HTML 渲染三种状态和 Sheet 标签栏。
 */
export function ExcelPreview(props: ExcelPreviewProps) {
  const maxHeightStyle = () => {
    const h = props.maxHeight ?? '600px';
    return typeof h === 'number' ? `${h}px` : h;
  };

  const showSheetTabs = () => (props.sheets?.length ?? 0) > 1;

  return (
    <Show when={!props.loading} fallback={<div>加载中...</div>}>
      <Show when={!props.error} fallback={<div style="color: red">{props.error}</div>}>
        <Show when={props.html}>
          <div class={props.class} style={props.style}>
            {/* 表格预览区域 */}
            <div
              style={`overflow: auto; max-height: ${maxHeightStyle()}; border: 1px solid #e0e0e0; border-radius: 4px;`}
              innerHTML={props.html!}
            />

            {/* Sheet 标签栏 */}
            <Show when={showSheetTabs()}>
              <div style="display: flex; gap: 2px; margin-top: 8px; border-top: 1px solid #e0e0e0; padding-top: 8px;">
                <For each={props.sheets}>
                  {(sheet, index) => (
                    <button
                      style={`padding: 4px 12px; border: 1px solid #ccc; border-radius: 4px 4px 0 0; background: ${index() === props.activeSheet ? '#fff' : '#f5f5f5'}; font-weight: ${index() === props.activeSheet ? 'bold' : 'normal'}; cursor: pointer; font-size: 12px;`}
                      onClick={() => props.onSheetChange?.(index())}
                    >
                      {sheet.name}
                    </button>
                  )}
                </For>
              </div>
            </Show>
          </div>
        </Show>
      </Show>
    </Show>
  );
}
