<script setup lang="ts">
/**
 * ExportButton - 开箱即用的导出按钮组件
 *
 * @example
 * ```vue
 * <ExportButton table-id="my-table" filename="报表.xlsx" :format="ExportFormat.Xlsx">
 *   导出 Excel
 * </ExportButton>
 * ```
 */

import type { ExportFormat } from '@bsg-export/types';
import { useExporter } from './use-exporter';
import { computed } from 'vue';

/** 组件 Props */
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
}

const props = withDefaults(defineProps<ExportButtonProps>(), {
  initializingText: '初始化中...',
  loadingText: '导出中 {progress}%',
  disabled: false,
});

const emit = defineEmits<{
  /** 导出成功 */
  success: [];
  /** 导出失败 */
  error: [error: Error];
  /** 进度变化 */
  progress: [progress: number];
}>();

const { initialized, loading, progress, error, exportTable } = useExporter();

/** 按钮是否禁用 */
const isDisabled = computed(() => props.disabled || !initialized.value || loading.value);

/** 按钮显示文本 */
const buttonText = computed(() => {
  if (!initialized.value) return props.initializingText;
  if (loading.value) {
    return props.loadingText.replace('{progress}', Math.round(progress.value).toString());
  }
  return undefined; // 使用默认插槽
});

/** 处理点击 */
function handleClick() {
  exportTable({
    tableId: props.tableId,
    filename: props.filename,
    format: props.format,
    excludeHidden: props.excludeHidden,
    withBom: props.withBom,
  });

  // 导出完成后触发事件
  if (error.value) {
    emit('error', error.value);
  } else {
    emit('success');
  }
}
</script>

<template>
  <button :disabled="isDisabled" @click="handleClick">
    <template v-if="buttonText">{{ buttonText }}</template>
    <slot v-else>导出</slot>
  </button>
</template>
