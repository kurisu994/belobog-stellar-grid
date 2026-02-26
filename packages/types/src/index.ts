/**
 * belobog-stellar-grid 严格类型定义
 *
 * 为 WASM 导出 API 提供类型安全的接口声明，
 * 替代 wasm-bindgen 自动生成的 `any` 类型。
 *
 * @packageDocumentation
 */

// =============================================================================
// 基础类型
// =============================================================================

/** 导出格式枚举 */
export enum ExportFormat {
  /** CSV 格式（默认） */
  Csv = 0,
  /** Excel XLSX 格式 */
  Xlsx = 1,
}

/** 进度回调函数，接收 0-100 的进度值 */
export type ProgressCallback = (progress: number) => void;

// =============================================================================
// 列配置
// =============================================================================

/** 列配置（支持嵌套子列形成多级表头） */
export interface Column {
  /** 表头标题文本 */
  title: string;
  /**
   * 数据字段 key，对应数据对象中的属性名。
   * 叶子列（无 children）必须提供 key。
   */
  key?: string;
  /** 子列配置，用于形成嵌套多级表头（自动生成合并单元格） */
  children?: Column[];
}

// =============================================================================
// 单元格与数据类型
// =============================================================================

/** 合并单元格配置 */
export interface MergeCellValue {
  /** 单元格显示值 */
  value: string | number | boolean | null;
  /** 列合并数（默认 1） */
  colSpan?: number;
  /** 行合并数（默认 1） */
  rowSpan?: number;
}

/** 单元格值类型 */
export type CellValue = string | number | boolean | null | undefined;

/** 支持合并的单元格值 */
export type MergeableCellValue = CellValue | MergeCellValue;

/**
 * 数据行类型
 *
 * - 二维数组模式：`CellValue[]`
 * - 对象数组模式：`Record<string, MergeableCellValue>`
 */
export type DataRow = CellValue[] | Record<string, MergeableCellValue>;

/** 树形数据行（含可选子节点） */
export type TreeDataRow<TChildrenKey extends string = 'children'> = Record<
  string,
  MergeableCellValue
> & {
  [K in TChildrenKey]?: TreeDataRow<TChildrenKey>[];
};

// =============================================================================
// export_data 配置
// =============================================================================

/** `export_data()` 的配置选项 */
export interface ExportDataOptions {
  /** 列配置数组（对象数组模式必需，二维数组模式不需要） */
  columns?: Column[];
  /** 导出文件名（默认 'export.csv'） */
  filename?: string;
  /** 导出格式（默认 ExportFormat.Csv） */
  format?: ExportFormat;
  /** 进度回调函数 */
  progressCallback?: ProgressCallback;
  /**
   * 树形模式：指定需要缩进的列的 key。
   * 需配合 `childrenKey` 使用。
   */
  indentColumn?: string;
  /**
   * 传入此参数启用树形数据模式。
   * 指定子节点字段名（如 'children'、'subCategories'）。
   */
  childrenKey?: string;
  /** 是否添加 UTF-8 BOM 头（仅 CSV 有效，解决 Excel 中文乱码） */
  withBom?: boolean;
}

// =============================================================================
// Sheet 配置（多工作表导出）
// =============================================================================

/** 多工作表同步导出的 Sheet 配置 */
export interface SheetConfig {
  /** 要导出的 HTML 表格元素的 ID */
  tableId: string;
  /** 工作表名称（默认使用 tableId） */
  sheetName?: string;
  /** 是否排除隐藏行/列（默认 false） */
  excludeHidden?: boolean;
}

/** 多工作表分批异步导出的 Sheet 配置 */
export interface BatchSheetConfig extends SheetConfig {
  /**
   * 可选的数据表格体 ID（用于分离表头和数据）。
   * 注意：此 ID 应指向不在 tableId 所指表格内部的独立 `<tbody>` 元素。
   */
  tbodyId?: string;
}

// =============================================================================
// 函数签名（类型安全版）
// =============================================================================

/**
 * 从 JavaScript 数组直接导出为文件（不依赖 DOM）
 *
 * @param data - 二维数组 `CellValue[][]` 或对象数组 `Record<string, MergeableCellValue>[]`
 * @param options - 配置选项
 * @throws 导出失败时抛出错误
 *
 * @example
 * ```typescript
 * // 二维数组
 * export_data([['姓名', '年龄'], ['张三', 28]]);
 *
 * // 对象数组 + 列配置
 * export_data(
 *   [{ name: '张三', age: 28 }],
 *   { columns: [{ title: '姓名', key: 'name' }], format: ExportFormat.Xlsx }
 * );
 * ```
 */
export declare function export_data(
  data: DataRow[],
  options?: ExportDataOptions,
): void;

/**
 * 导出 HTML 表格为 CSV 或 Excel 文件
 *
 * @param tableId - 要导出的 HTML 表格元素的 ID
 * @param filename - 导出文件名（默认 'table.csv'）
 * @param format - 导出格式（默认 Csv）
 * @param excludeHidden - 是否排除隐藏行/列（默认 false）
 * @param progressCallback - 进度回调函数
 * @param withBom - 是否添加 UTF-8 BOM（仅 CSV 有效）
 * @param strictProgressCallback - 回调失败是否中断导出（默认 false）
 * @throws 导出失败时抛出错误
 */
export declare function export_table(
  tableId: string,
  filename?: string | null,
  format?: ExportFormat | null,
  excludeHidden?: boolean | null,
  progressCallback?: ProgressCallback | null,
  withBom?: boolean | null,
  strictProgressCallback?: boolean | null,
): void;

/**
 * 多工作表导出为 Excel 文件（同步）
 *
 * @param sheets - Sheet 配置数组
 * @param filename - 导出文件名（默认 'table_export.xlsx'）
 * @param progressCallback - 进度回调函数
 * @throws 导出失败时抛出错误
 */
export declare function export_tables_xlsx(
  sheets: SheetConfig[],
  filename?: string | null,
  progressCallback?: ProgressCallback | null,
): void;

/**
 * 分批异步导出 HTML 表格为 CSV 文件
 *
 * @param tableId - 要导出的 HTML 表格元素的 ID
 * @param tbodyId - 可选的独立 tbody ID
 * @param filename - 导出文件名（默认 'table_export.csv'）
 * @param batchSize - 每批处理行数（默认 1000）
 * @param excludeHidden - 是否排除隐藏行/列
 * @param progressCallback - 进度回调函数
 * @param withBom - 是否添加 UTF-8 BOM
 */
export declare function export_table_to_csv_batch(
  tableId: string,
  tbodyId?: string | null,
  filename?: string | null,
  batchSize?: number | null,
  excludeHidden?: boolean | null,
  progressCallback?: ProgressCallback | null,
  withBom?: boolean | null,
): Promise<void>;

/**
 * 分批异步导出 HTML 表格为 XLSX 文件
 *
 * @param tableId - 要导出的 HTML 表格元素的 ID
 * @param tbodyId - 可选的独立 tbody ID
 * @param filename - 导出文件名（默认 'table_export.xlsx'）
 * @param batchSize - 每批处理行数（默认 1000）
 * @param excludeHidden - 是否排除隐藏行/列
 * @param progressCallback - 进度回调函数
 */
export declare function export_table_to_xlsx_batch(
  tableId: string,
  tbodyId?: string | null,
  filename?: string | null,
  batchSize?: number | null,
  excludeHidden?: boolean | null,
  progressCallback?: ProgressCallback | null,
): Promise<void>;

/**
 * 多工作表分批异步导出为 XLSX 文件
 *
 * @param sheets - Sheet 配置数组
 * @param filename - 导出文件名（默认 'table_export.xlsx'）
 * @param batchSize - 每批处理行数（默认 1000）
 * @param progressCallback - 进度回调函数
 */
export declare function export_tables_to_xlsx_batch(
  sheets: BatchSheetConfig[],
  filename?: string | null,
  batchSize?: number | null,
  progressCallback?: ProgressCallback | null,
): Promise<void>;
