<script setup lang="ts">
/**
 * ExcelPreview - Excel 文件预览组件
 *
 * 渲染 Excel 解析后的 HTML 表格，支持多 Sheet 切换。
 *
 * @example
 * ```vue
 * <ExcelPreview
 *   :loading="preview.loading.value"
 *   :error="preview.error.value"
 *   :html="preview.html.value"
 *   :sheets="preview.sheets.value"
 *   :active-sheet="preview.activeSheet.value"
 *   @sheet-change="preview.switchSheet"
 * />
 * ```
 */

import { computed } from 'vue';
import type { SheetInfo } from '@bsg-export/types';

/** 组件 Props */
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

const props = withDefaults(defineProps<ExcelPreviewProps>(), {
  loading: false,
  error: null,
  html: null,
  sheets: () => [],
  activeSheet: 0,
  maxHeight: '600px',
});

const emit = defineEmits<{
  /** Sheet 切换 */
  sheetChange: [index: number];
}>();

/** 是否显示 Sheet 标签栏 */
const showSheetTabs = computed(() => props.sheets.length > 1);

/** 最大高度样式值 */
const maxHeightStyle = computed(() =>
  typeof props.maxHeight === 'number' ? `${props.maxHeight}px` : props.maxHeight
);
</script>

<template>
  <div>
    <!-- 加载中 -->
    <div v-if="loading">加载中...</div>

    <!-- 错误信息 -->
    <div v-else-if="error" style="color: red">{{ error }}</div>

    <!-- 预览内容 -->
    <template v-else-if="html">
      <!-- 表格预览区域 -->
      <div
        :style="{
          overflow: 'auto',
          maxHeight: maxHeightStyle,
          border: '1px solid #e0e0e0',
          borderRadius: '4px',
        }"
        v-html="html"
      />

      <!-- Sheet 标签栏 -->
      <div
        v-if="showSheetTabs"
        :style="{
          display: 'flex',
          gap: '2px',
          marginTop: '8px',
          borderTop: '1px solid #e0e0e0',
          paddingTop: '8px',
        }"
      >
        <button
          v-for="(sheet, index) in sheets"
          :key="sheet.name"
          :style="{
            padding: '4px 12px',
            border: '1px solid #ccc',
            borderRadius: '4px 4px 0 0',
            background: index === activeSheet ? '#fff' : '#f5f5f5',
            fontWeight: index === activeSheet ? 'bold' : 'normal',
            cursor: 'pointer',
            fontSize: '12px',
          }"
          @click="emit('sheetChange', index)"
        >
          {{ sheet.name }}
        </button>
      </div>
    </template>
  </div>
</template>
