/* tslint:disable */
/* eslint-disable */

/**
 * 导出格式枚举
 */
export enum ExportFormat {
    /**
     * CSV 格式（默认）
     */
    Csv = 0,
    /**
     * Excel XLSX 格式
     */
    Xlsx = 1,
}

/**
 * 从 JavaScript 数组直接导出为文件（不依赖 DOM）
 *
 * 接收二维数组数据或对象数组（配合列配置），直接生成 CSV 或 XLSX 文件并触发下载。
 * 当提供 `columns` 时，支持嵌套表头（自动生成多行表头和合并单元格）。
 *
 * # 参数
 * * `data` - JS 数组 (二维数组 `Array<Array<any>>` 或对象数组 `Array<Object>`)
 * * `options` - 可选的配置对象，包含以下字段：
 *   - `columns`: 表头配置数组，支持嵌套 children 实现多级表头
 *   - `filename`: 导出文件名
 *   - `format`: 导出格式（ExportFormat.Csv 或 ExportFormat.Xlsx），默认 Csv
 *   - `progressCallback`: 进度回调函数，接收 0-100 的进度值
 *   - `indentColumn`: 树形模式下需要缩进的列的 key
 *   - `childrenKey`: 传入此参数启用树形数据模式，指定子节点字段名
 *   - `withBom`: 是否添加 UTF-8 BOM（仅 CSV 有效）
 *   - `strictProgressCallback`: 回调失败是否立刻中断导出（默认 false）
 *
 * # 返回值
 * * `Ok(())` - 导出成功
 * * `Err(JsValue)` - 导出失败，包含错误信息
 *
 * # 示例
 * ```javascript
 * import init, { export_data, ExportFormat } from './pkg/belobog_stellar_grid.js';
 * await init();
 *
 * // 1. 二维数组导出（无需 options）
 * const arrayData = [['姓名', '年龄'], ['张三', 28]];
 * export_data(arrayData);
 * export_data(arrayData, { filename: '用户.csv' });
 *
 * // 2. 对象数组 + 简单表头
 * const data = [{ name: '张三', age: 28 }];
 * const columns = [
 *   { title: '姓名', key: 'name' },
 *   { title: '年龄', key: 'age' }
 * ];
 * export_data(data, { columns, filename: '用户.xlsx', format: ExportFormat.Xlsx });
 *
 * // 3. 对象数组 + 嵌套表头（多行表头 + 合并单元格）
 * const nestedColumns = [
 *   { title: '姓名', key: 'name' },
 *   { title: '其他信息', children: [
 *     { title: '年龄', key: 'age' },
 *     { title: '住址', key: 'address' }
 *   ]}
 * ];
 * export_data(data, { columns: nestedColumns, filename: '用户.xlsx', format: ExportFormat.Xlsx });
 *
 * // 4. 树形数据导出（递归拍平 children + 层级缩进）
 * const treeData = [
 *   { name: 'CEO', children: [
 *     { name: 'CTO' },
 *     { name: 'CFO', children: [{ name: '会计' }] }
 *   ]}
 * ];
 * export_data(treeData, {
 *   columns, filename: '组织架构.xlsx', format: ExportFormat.Xlsx,
 *   indentColumn: 'name', childrenKey: 'children'
 * });
 * ```
 */
export function export_data(data: any, options?: any | null): void;

/**
 * 流式导出 JavaScript 数据为 CSV 文件（异步，降低内存峰值）
 *
 * 与 `export_data` 功能相同，但采用分块写入策略：
 * 将 CSV 输出按 `chunkSize` 行分块写入，每块转为 `Uint8Array` 后立即释放 Rust 侧内存，
 * 最后用所有分块拼接成单个 `Blob` 触发下载。
 *
 * **内存优化**：Rust 侧内存峰值仅为一个分块大小，而非全部数据。
 *
 * **XLSX 限制**：当 `format=Xlsx` 时，由于 XLSX 库不支持流式写入，
 * 会自动回退到 `export_data` 的同步逻辑。
 *
 * # 参数
 * * `data` - JS 数组（二维数组或对象数组）
 * * `options` - 配置对象（同 `export_data`，额外支持 `chunkSize` 字段）
 *   - `chunkSize`: 每个分块包含的行数（默认 5000）
 *   - 其他字段同 `export_data` 的 options
 *
 * # 返回值
 * * `Promise<void>` - 异步导出完成
 *
 * # 示例
 * ```javascript
 * import init, { export_data_streaming, ExportFormat } from './pkg/belobog_stellar_grid.js';
 * await init();
 *
 * // 流式 CSV 导出（适合超大数据量）
 * const largeData = generateLargeData(100000); // 10 万行
 * await export_data_streaming(largeData, {
 *   columns: [{ title: '姓名', key: 'name' }, { title: '年龄', key: 'age' }],
 *   filename: '大数据.csv',
 *   chunkSize: 10000, // 每块 1 万行
 *   progressCallback: (progress) => {
 *     console.log(`进度: ${Math.round(progress)}%`);
 *   },
 * });
 *
 * // XLSX 格式会自动回退到同步导出
 * await export_data_streaming(largeData, {
 *   columns: [{ title: '姓名', key: 'name' }],
 *   filename: '报表.xlsx',
 *   format: ExportFormat.Xlsx,
 * });
 * ```
 */
export function export_data_streaming(data: any, options?: any | null): Promise<any>;

/**
 * 统一的表格导出函数（带进度回调）
 *
 * 支持导出为 CSV 或 Excel 格式，通过 format 参数控制，支持进度回调
 *
 * # 参数
 * * `table_id` - 要导出的 HTML 表格元素的 ID
 * * `filename` - 可选的导出文件名（不包含扩展名时会自动添加）
 * * `format` - 导出格式（Csv 或 Xlsx），默认为 Csv
 * * `exclude_hidden` - 可选，是否排除隐藏的行和列（默认为 false）
 * * `progress_callback` - 可选的进度回调函数，接收 0-100 的进度值
 * * `with_bom` - 可选，是否添加 UTF-8 BOM（默认为 false，仅对 CSV 有效）
 * * `strict_progress_callback` - 可选，是否启用严格进度回调模式（默认 false）。
 *   启用后回调失败会中断导出并返回错误，否则仅 console.warn
 *
 * # 返回值
 * * `Ok(())` - 导出成功
 * * `Err(JsValue)` - 导出失败，包含错误信息
 *
 * # 示例
 * ```javascript
 * import init, { export_table, ExportFormat } from './pkg/belobog_stellar_grid.js';
 * await init();
 *
 * // 导出为 CSV（默认，无进度回调）
 * export_table('my-table');
 * export_table('my-table', '数据.csv');
 *
 * // 导出为 CSV（带进度回调，不排除隐藏行）
 * export_table('my-table', '数据', ExportFormat.Csv, false, (progress) => {
 *     console.log(`进度: ${progress.toFixed(1)}%`);
 * });
 *
 * // 导出为 Excel（带进度回调，排除隐藏行）
 * export_table('my-table', '报表', ExportFormat.Xlsx, true, (progress) => {
 *     document.getElementById('progress').style.width = `${progress}%`;
 * });
 * ```
 */
export function export_table(table_id: string, filename?: string | null, format?: ExportFormat | null, exclude_hidden?: boolean | null, progress_callback?: Function | null, with_bom?: boolean | null, strict_progress_callback?: boolean | null): void;

/**
 * 分批异步导出 HTML 表格到 CSV 文件
 *
 * 这个函数将表格数据分批处理，在批次之间让出控制权给浏览器事件循环，
 * 从而避免在处理大量数据时阻塞主线程导致页面卡死。
 * 支持合并单元格（colspan/rowspan）的正确处理。
 *
 * **内存优化**：采用分块 Blob 片段策略，每个批次的 CSV 字节在转为
 * `Uint8Array` 后立即释放 Rust 侧内存，峰值仅为一个批次大小。
 *
 * # 参数
 * * `table_id` - 要导出的 HTML 表格元素的 ID
 * * `tbody_id` - 可选的数据表格体 ID（用于分离表头和数据）。**注意**：此 ID 应指向**不在** `table_id` 所指表格内部的独立 `<tbody>` 元素。如果传入的 `tbody` 在 `table` 内部，会导致该部分数据被重复导出（一次作为 table 的一部分，一次作为独立 tbody）。
 * * `filename` - 可选的导出文件名（可选，默认为 "table_export.csv"）
 * * `batch_size` - 每批处理的行数（默认 1000）
 * * `exclude_hidden` - 可选，是否排除隐藏的行和列（默认为 false）
 * * `progress_callback` - 进度回调函数，接收进度百分比 (0-100)
 * * `with_bom` - 可选，是否添加 UTF-8 BOM（默认为 false）
 * * `strict_progress_callback` - 可选，是否严格报告进度（默认为 false）。如果为 true，则每次进度更新都会触发回调；如果为 false，则可能跳过一些更新以提高性能。
 *
 * # 返回值
 * * `Promise<void>` - 异步操作的 Promise
 *
 * # 示例
 * ```javascript
 * import { export_table_to_csv_batch } from './pkg/belobog_stellar_grid.js';
 *
 * await export_table_to_csv_batch(
 *     'my-table',
 *     'my-tbody',  // 可选的 tbody ID
 *     'data.csv',
 *     1000,  // 每批 1000 行
 *     false, // 不排除隐藏行
 *     (progress) => {
 *         console.log(`进度: ${progress}%`);
 *     },
 *     true // 添加 BOM
 * );
 * ```
 */
export function export_table_to_csv_batch(table_id: string, tbody_id?: string | null, filename?: string | null, batch_size?: number | null, exclude_hidden?: boolean | null, progress_callback?: Function | null, with_bom?: boolean | null, strict_progress_callback?: boolean | null): Promise<any>;

/**
 * 分批异步导出 HTML 表格到 XLSX 文件
 *
 * 采用两阶段策略避免阻塞主线程：
 * 1. 分批读取 DOM 数据（异步，可 yield 让出控制权）
 * 2. 同步生成 XLSX 文件（内存操作，快速）
 *
 * # 参数
 * * `table_id` - 要导出的 HTML 表格元素的 ID
 * * `tbody_id` - 可选的数据表格体 ID（用于分离表头和数据）。**注意**：此 ID 应指向**不在** `table_id` 所指表格内部的独立 `<tbody>` 元素。如果传入的 `tbody` 在 `table` 内部，会导致该部分数据被重复导出（一次作为 table 的一部分，一次作为独立 tbody）。
 * * `filename` - 可选的导出文件名（默认为 "table_export.xlsx"）
 * * `batch_size` - 每批处理的行数（默认 1000）
 * * `exclude_hidden` - 可选，是否排除隐藏的行和列（默认为 false）
 * * `progress_callback` - 进度回调函数，接收进度百分比 (0-100)
 *
 * # 返回值
 * * `Promise<void>` - 异步操作的 Promise
 *
 * # 示例
 * ```javascript
 * import { export_table_to_xlsx_batch } from './pkg/belobog_stellar_grid.js';
 *
 * await export_table_to_xlsx_batch(
 *     'my-table',
 *     'my-tbody',  // 可选的 tbody ID
 *     'data.xlsx',
 *     1000,  // 每批 1000 行
 *     false, // 不排除隐藏行
 *     (progress) => {
 *         console.log(`进度: ${progress}%`);
 *     }
 * );
 * ```
 */
export function export_table_to_xlsx_batch(table_id: string, tbody_id?: string | null, filename?: string | null, batch_size?: number | null, exclude_hidden?: boolean | null, progress_callback?: Function | null, strict_progress_callback?: boolean | null): Promise<any>;

/**
 * 多工作表分批异步导出 HTML 表格到 XLSX 文件
 *
 * 将页面上多个 HTML 表格分批异步提取后导出到同一 Excel 文件的不同工作表中
 *
 * # 参数
 * * `sheets` - JS 数组，每个元素为 `{ tableId: string, tbodyId?: string, sheetName?: string, excludeHidden?: boolean }`。**注意**：如果有 `tbodyId`，此 ID 应指向**不在** `tableId` 所指表格内部的独立 `<tbody>` 元素。如果传入的 `tbody` 在 `table` 内部，会导致该使用部分数据被重复导出（一次作为 table 的一部分，一次作为独立 tbody）。
 * * `filename` - 可选的导出文件名（默认为 "table_export.xlsx"）
 * * `batch_size` - 每批处理的行数（默认 1000）
 * * `progress_callback` - 进度回调函数，接收进度百分比 (0-100)
 *
 * # 返回值
 * * `Promise<void>` - 异步操作的 Promise
 *
 * # 示例
 * ```javascript
 * import { export_tables_to_xlsx_batch } from './pkg/belobog_stellar_grid.js';
 *
 * await export_tables_to_xlsx_batch(
 *   [
 *     { tableId: 'table1', sheetName: '订单列表', excludeHidden: true },
 *     { tableId: 'table2', tbodyId: 'tbody2', sheetName: '商品列表' },
 *   ],
 *   'report.xlsx',
 *   1000,
 *   (progress) => console.log(`进度: ${progress}%`)
 * );
 * ```
 */
export function export_tables_to_xlsx_batch(sheets: any, filename?: string | null, batch_size?: number | null, progress_callback?: Function | null, strict_progress_callback?: boolean | null): Promise<any>;

/**
 * 多工作表导出为 Excel XLSX 文件
 *
 * 将多个 HTML 表格导出到同一个 Excel 文件的不同工作表中
 *
 * # 参数
 * * `sheets` - JS 数组，每个元素包含 { tableId: string, sheetName?: string, excludeHidden?: boolean }
 * * `filename` - 可选的导出文件名
 * * `progress_callback` - 可选的进度回调函数，接收 0-100 的进度值
 *
 * # 示例
 * ```javascript
 * import { export_tables_xlsx } from './pkg/belobog_stellar_grid.js';
 *
 * export_tables_xlsx(
 *   [
 *     { tableId: 'table1', sheetName: '订单列表', excludeHidden: true },
 *     { tableId: 'table2', sheetName: '商品列表' },
 *   ],
 *   'report.xlsx',
 *   (progress) => console.log(`进度: ${progress}%`)
 * );
 * ```
 */
export function export_tables_xlsx(sheets: any, filename?: string | null, progress_callback?: Function | null, strict_progress_callback?: boolean | null): void;

/**
 * 从 JavaScript 数组生成文件字节（不触发下载，供 Web Worker 使用）
 *
 * 与 `export_data` 功能相同，但不创建 Blob 和下载链接，
 * 而是直接返回生成的文件字节（CSV 或 XLSX），
 * 适用于 Web Worker 场景：Worker 中生成字节，主线程触发下载。
 *
 * # 参数
 * * `data` - JS 数组 (二维数组或对象数组)
 * * `options` - 可选的配置对象（同 export_data，但进度回调在 Worker 中可能不可用）
 *
 * # 返回值
 * * `Ok(Uint8Array)` - 生成的文件字节
 * * `Err(JsValue)` - 生成失败
 *
 * # 示例
 * ```javascript
 * // 在 Web Worker 中：
 * import init, { generate_data_bytes, ExportFormat } from 'belobog-stellar-grid';
 * await init();
 *
 * const bytes = generate_data_bytes(
 *   [['姓名', '年龄'], ['张三', 28]],
 *   { format: ExportFormat.Xlsx }
 * );
 * // 通过 postMessage 将 bytes 传回主线程
 * self.postMessage({ type: 'result', bytes: bytes.buffer }, [bytes.buffer]);
 * ```
 */
export function generate_data_bytes(data: any, options?: any | null): Uint8Array;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly export_data: (a: any, b: number) => [number, number];
    readonly export_data_streaming: (a: any, b: number) => any;
    readonly export_table: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
    readonly export_table_to_csv_batch: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => any;
    readonly export_table_to_xlsx_batch: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => any;
    readonly export_tables_to_xlsx_batch: (a: any, b: number, c: number, d: number, e: number, f: number) => any;
    readonly export_tables_xlsx: (a: any, b: number, c: number, d: number, e: number) => [number, number];
    readonly generate_data_bytes: (a: any, b: number) => [number, number, number];
    readonly wasm_bindgen__closure__destroy__h4861cf75edd79f33: (a: number, b: number) => void;
    readonly wasm_bindgen__closure__destroy__hb1f1e222d193ce94: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h4c21a5017d209281: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h00d297b34697abe8: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h5825be643c3396fa: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
