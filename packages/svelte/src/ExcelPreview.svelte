<!--
  ExcelPreview - Excel 文件预览组件 (Svelte 4/5)

  渲染 Excel 解析后的 HTML 表格，支持多 Sheet 切换。

  @example
  ```svelte
  <script>
  import ExcelPreview from '@bsg-export/svelte/ExcelPreview.svelte';
  import { createExcelPreview } from '@bsg-export/svelte';

  const preview = createExcelPreview({ ... });
  </script>

  <ExcelPreview
    loading={$preview.loading}
    html={$preview.html}
    sheets={$preview.sheets}
    activeSheet={$preview.activeSheet}
    on:sheetChange={(e) => preview.switchSheet(e.detail)}
  />
  ```
-->
<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import type { SheetInfo } from '@bsg-export/types';

  /** 是否正在解析 */
  export let loading: boolean = false;
  /** 错误信息 */
  export let error: string | null = null;
  /** HTML 输出 */
  export let html: string | null = null;
  /** 工作表列表 */
  export let sheets: SheetInfo[] = [];
  /** 当前活动 Sheet 索引 */
  export let activeSheet: number = 0;
  /** 表格容器最大高度 */
  export let maxHeight: string | number = '600px';

  const dispatch = createEventDispatcher<{ sheetChange: number }>();

  $: showSheetTabs = sheets.length > 1;
  $: maxHeightStyle = typeof maxHeight === 'number' ? `${maxHeight}px` : maxHeight;
</script>

{#if loading}
  <div>加载中...</div>
{:else if error}
  <div style="color: red">{error}</div>
{:else if html}
  <div>
    <!-- 表格预览区域 -->
    <div
      style="overflow: auto; max-height: {maxHeightStyle}; border: 1px solid #e0e0e0; border-radius: 4px;"
    >
      {@html html}
    </div>

    <!-- Sheet 标签栏 -->
    {#if showSheetTabs}
      <div
        style="display: flex; gap: 2px; margin-top: 8px; border-top: 1px solid #e0e0e0; padding-top: 8px;"
      >
        {#each sheets as sheet, index (sheet.name)}
          <button
            style="padding: 4px 12px; border: 1px solid #ccc; border-radius: 4px 4px 0 0; background: {index === activeSheet ? '#fff' : '#f5f5f5'}; font-weight: {index === activeSheet ? 'bold' : 'normal'}; cursor: pointer; font-size: 12px;"
            on:click={() => dispatch('sheetChange', index)}
          >
            {sheet.name}
          </button>
        {/each}
      </div>
    {/if}
  </div>
{/if}
