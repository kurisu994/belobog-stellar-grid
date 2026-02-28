<!--
  ExportButton - 开箱即用的导出按钮组件 (Svelte 4/5)

  自动管理 WASM 初始化、导出状态和进度显示。

  @example
  ```svelte
  <script>
  import ExportButton from '@bsg-export/svelte/ExportButton.svelte';
  import { ExportFormat } from '@bsg-export/svelte';
  </script>

  <ExportButton tableId="my-table" filename="报表.xlsx" format={ExportFormat.Xlsx}>
    导出 Excel
  </ExportButton>
  ```
-->
<script lang="ts">
  import { onDestroy } from 'svelte';
  import type { ExportFormat } from '@bsg-export/types';
  import { createExporter } from './create-exporter';

  /** 要导出的 HTML 表格元素的 ID */
  export let tableId: string;
  /** 导出文件名 */
  export let filename: string | undefined = undefined;
  /** 导出格式 */
  export let format: ExportFormat | undefined = undefined;
  /** 是否排除隐藏行/列 */
  export let excludeHidden: boolean | undefined = undefined;
  /** 是否添加 UTF-8 BOM（仅 CSV 有效） */
  export let withBom: boolean | undefined = undefined;
  /** 是否禁用按钮 */
  export let disabled: boolean = false;
  /** 初始化中的提示文本 */
  export let initializingText: string = '初始化中...';
  /** 导出中的提示文本（支持 {progress} 占位符） */
  export let loadingText: string = '导出中 {progress}%';
  /** 导出成功回调 */
  export let onExportSuccess: (() => void) | undefined = undefined;
  /** 导出失败回调 */
  export let onExportError: ((error: Error) => void) | undefined = undefined;
  /** 进度变化回调 */
  export let onExportProgress: ((progress: number) => void) | undefined = undefined;

  const store = createExporter();
  const { initialized, loading, progress, error, exportTable } = store;

  onDestroy(() => {
    store.destroy();
  });

  // 监听错误并触发回调
  $: if ($error && onExportError) {
    onExportError($error);
  }

  // 监听进度变化并触发回调
  $: if (onExportProgress) {
    onExportProgress($progress);
  }

  $: isDisabled = disabled || !$initialized || $loading;

  $: buttonText = !$initialized
    ? initializingText
    : $loading
      ? loadingText.replace('{progress}', Math.round($progress).toString())
      : undefined;

  /** 处理点击 */
  function handleClick() {
    const success = exportTable({
      tableId,
      filename,
      format,
      excludeHidden,
      withBom,
    });

    if (success && onExportSuccess) {
      onExportSuccess();
    }
  }
</script>

<button disabled={isDisabled} on:click={handleClick} {...$$restProps}>
  {#if buttonText}
    {buttonText}
  {:else}
    <slot>导出</slot>
  {/if}
</button>
