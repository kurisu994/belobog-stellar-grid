/* @ts-self-types="./belobog_stellar_grid.d.ts" */

/**
 * 导出格式枚举
 * @enum {0 | 1}
 */
export const ExportFormat = Object.freeze({
    /**
     * CSV 格式（默认）
     */
    Csv: 0, "0": "Csv",
    /**
     * Excel XLSX 格式
     */
    Xlsx: 1, "1": "Xlsx",
});

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
 * @param {any} data
 * @param {any | null} [options]
 */
export function export_data(data, options) {
    const ret = wasm.export_data(data, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * 流式导出 JavaScript 数据为 CSV 文件（异步，降低内存峰值）
 *
 * 与 `export_data` 功能相同，但采用分块写入策略：
 * 将 CSV 输出按 `chunkSize` 行分块写入，每块转为 `Uint8Array` 后立即释放 Rust 侧内存，
 * 最后用所有分块拼接成单个 `Blob` 触发下载。
 *
 * **内存优化**：CSV 编码缓冲峰值仅为一个分块大小（源数据仍完整驻留内存中）。
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
 * @param {any} data
 * @param {any | null} [options]
 * @returns {Promise<any>}
 */
export function export_data_streaming(data, options) {
    const ret = wasm.export_data_streaming(data, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    return ret;
}

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
 * @param {string} table_id
 * @param {string | null} [filename]
 * @param {ExportFormat | null} [format]
 * @param {boolean | null} [exclude_hidden]
 * @param {Function | null} [progress_callback]
 * @param {boolean | null} [with_bom]
 * @param {boolean | null} [strict_progress_callback]
 * @param {any | null} [header_style]
 * @param {any | null} [cell_style]
 */
export function export_table(table_id, filename, format, exclude_hidden, progress_callback, with_bom, strict_progress_callback, header_style, cell_style) {
    const ptr0 = passStringToWasm0(table_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    var ptr1 = isLikeNone(filename) ? 0 : passStringToWasm0(filename, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    const ret = wasm.export_table(ptr0, len0, ptr1, len1, isLikeNone(format) ? 2 : format, isLikeNone(exclude_hidden) ? 0xFFFFFF : exclude_hidden ? 1 : 0, isLikeNone(progress_callback) ? 0 : addToExternrefTable0(progress_callback), isLikeNone(with_bom) ? 0xFFFFFF : with_bom ? 1 : 0, isLikeNone(strict_progress_callback) ? 0xFFFFFF : strict_progress_callback ? 1 : 0, isLikeNone(header_style) ? 0 : addToExternrefTable0(header_style), isLikeNone(cell_style) ? 0 : addToExternrefTable0(cell_style));
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

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
 * @param {string} table_id
 * @param {string | null} [tbody_id]
 * @param {string | null} [filename]
 * @param {number | null} [batch_size]
 * @param {boolean | null} [exclude_hidden]
 * @param {Function | null} [progress_callback]
 * @param {boolean | null} [with_bom]
 * @param {boolean | null} [strict_progress_callback]
 * @returns {Promise<any>}
 */
export function export_table_to_csv_batch(table_id, tbody_id, filename, batch_size, exclude_hidden, progress_callback, with_bom, strict_progress_callback) {
    const ptr0 = passStringToWasm0(table_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    var ptr1 = isLikeNone(tbody_id) ? 0 : passStringToWasm0(tbody_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    var ptr2 = isLikeNone(filename) ? 0 : passStringToWasm0(filename, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len2 = WASM_VECTOR_LEN;
    const ret = wasm.export_table_to_csv_batch(ptr0, len0, ptr1, len1, ptr2, len2, isLikeNone(batch_size) ? 0x100000001 : (batch_size) >>> 0, isLikeNone(exclude_hidden) ? 0xFFFFFF : exclude_hidden ? 1 : 0, isLikeNone(progress_callback) ? 0 : addToExternrefTable0(progress_callback), isLikeNone(with_bom) ? 0xFFFFFF : with_bom ? 1 : 0, isLikeNone(strict_progress_callback) ? 0xFFFFFF : strict_progress_callback ? 1 : 0);
    return ret;
}

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
 * @param {string} table_id
 * @param {string | null} [tbody_id]
 * @param {string | null} [filename]
 * @param {number | null} [batch_size]
 * @param {boolean | null} [exclude_hidden]
 * @param {Function | null} [progress_callback]
 * @param {boolean | null} [strict_progress_callback]
 * @param {any | null} [header_style]
 * @param {any | null} [cell_style]
 * @returns {Promise<any>}
 */
export function export_table_to_xlsx_batch(table_id, tbody_id, filename, batch_size, exclude_hidden, progress_callback, strict_progress_callback, header_style, cell_style) {
    const ptr0 = passStringToWasm0(table_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    var ptr1 = isLikeNone(tbody_id) ? 0 : passStringToWasm0(tbody_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    var ptr2 = isLikeNone(filename) ? 0 : passStringToWasm0(filename, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len2 = WASM_VECTOR_LEN;
    const ret = wasm.export_table_to_xlsx_batch(ptr0, len0, ptr1, len1, ptr2, len2, isLikeNone(batch_size) ? 0x100000001 : (batch_size) >>> 0, isLikeNone(exclude_hidden) ? 0xFFFFFF : exclude_hidden ? 1 : 0, isLikeNone(progress_callback) ? 0 : addToExternrefTable0(progress_callback), isLikeNone(strict_progress_callback) ? 0xFFFFFF : strict_progress_callback ? 1 : 0, isLikeNone(header_style) ? 0 : addToExternrefTable0(header_style), isLikeNone(cell_style) ? 0 : addToExternrefTable0(cell_style));
    return ret;
}

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
 * @param {any} sheets
 * @param {string | null} [filename]
 * @param {number | null} [batch_size]
 * @param {Function | null} [progress_callback]
 * @param {boolean | null} [strict_progress_callback]
 * @param {any | null} [header_style]
 * @param {any | null} [cell_style]
 * @returns {Promise<any>}
 */
export function export_tables_to_xlsx_batch(sheets, filename, batch_size, progress_callback, strict_progress_callback, header_style, cell_style) {
    var ptr0 = isLikeNone(filename) ? 0 : passStringToWasm0(filename, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    const ret = wasm.export_tables_to_xlsx_batch(sheets, ptr0, len0, isLikeNone(batch_size) ? 0x100000001 : (batch_size) >>> 0, isLikeNone(progress_callback) ? 0 : addToExternrefTable0(progress_callback), isLikeNone(strict_progress_callback) ? 0xFFFFFF : strict_progress_callback ? 1 : 0, isLikeNone(header_style) ? 0 : addToExternrefTable0(header_style), isLikeNone(cell_style) ? 0 : addToExternrefTable0(cell_style));
    return ret;
}

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
 * @param {any} sheets
 * @param {string | null} [filename]
 * @param {Function | null} [progress_callback]
 * @param {boolean | null} [strict_progress_callback]
 * @param {any | null} [header_style]
 * @param {any | null} [cell_style]
 */
export function export_tables_xlsx(sheets, filename, progress_callback, strict_progress_callback, header_style, cell_style) {
    var ptr0 = isLikeNone(filename) ? 0 : passStringToWasm0(filename, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len0 = WASM_VECTOR_LEN;
    const ret = wasm.export_tables_xlsx(sheets, ptr0, len0, isLikeNone(progress_callback) ? 0 : addToExternrefTable0(progress_callback), isLikeNone(strict_progress_callback) ? 0xFFFFFF : strict_progress_callback ? 1 : 0, isLikeNone(header_style) ? 0 : addToExternrefTable0(header_style), isLikeNone(cell_style) ? 0 : addToExternrefTable0(cell_style));
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

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
 * @param {any} data
 * @param {any | null} [options]
 * @returns {Uint8Array}
 */
export function generate_data_bytes(data, options) {
    const ret = wasm.generate_data_bytes(data, isLikeNone(options) ? 0 : addToExternrefTable0(options));
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * 获取 Excel 文件的工作表列表
 *
 * 返回各工作表的名称、索引和行列数信息，用于 Sheet 切换 UI。
 *
 * # 参数
 * * `data` - Excel 文件的二进制数据（Uint8Array）
 *
 * # 返回值
 * SheetInfo 数组的 JSON 对象
 * @param {Uint8Array} data
 * @returns {any}
 */
export function getExcelSheetList(data) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.getExcelSheetList(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * 解析 Excel 文件并返回 HTML Table 字符串
 *
 * 在 WASM 侧完成全部解析和拼装，返回可直接挂载的 `<table>` HTML。
 * 前端通过 `dangerouslySetInnerHTML` (React) 或 `v-html` (Vue) 直接使用。
 *
 * # 参数
 * * `data` - Excel 文件的二进制数据（Uint8Array）
 * * `options` - 可选的预览配置（JsValue 对象）
 *
 * # 返回值
 * 包含完整 HTML table 的字符串
 * @param {Uint8Array} data
 * @param {any} options
 * @returns {any}
 */
export function parseExcelToHtml(data, options) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.parseExcelToHtml(ptr0, len0, options);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * 解析 Excel 文件并返回结构化 JSON 数据
 *
 * 返回 ParsedWorkbook 结构化数据，前端可自行渲染。
 * 适合需要自定义渲染逻辑（虚拟滚动、交互等）的场景。
 *
 * # 参数
 * * `data` - Excel 文件的二进制数据（Uint8Array）
 * * `options` - 可选的预览配置（JsValue 对象）
 *
 * # 返回值
 * ParsedWorkbook 结构的 JSON 对象
 * @param {Uint8Array} data
 * @param {any} options
 * @returns {any}
 */
export function parseExcelToJson(data, options) {
    const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.parseExcelToJson(ptr0, len0, options);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_boolean_get_c0f3f60bac5a78d1: function(arg0) {
            const v = arg0;
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_5398f5bb970e0daa: function(arg0, arg1) {
            const ret = debugString(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_is_function_3c846841762788c1: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_null_0b605fc6b167c56f: function(arg0) {
            const ret = arg0 === null;
            return ret;
        },
        __wbg___wbindgen_is_object_781bc9f159099513: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_undefined_52709e72fb9f179c: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_number_get_34bb9d9dcfa21373: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_395e606bd0ee4427: function(arg0, arg1) {
            const obj = arg1;
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_6ddd609b62940d55: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg__wbg_cb_unref_6b5b6b8576d35cb1: function(arg0) {
            arg0._wbg_cb_unref();
        },
        __wbg_call_2d781c1f4d5c0ef8: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_cells_90c232e1905b79ce: function(arg0) {
            const ret = arg0.cells;
            return ret;
        },
        __wbg_click_14a2543ed4ab7b86: function(arg0) {
            arg0.click();
        },
        __wbg_colSpan_2b68718aa2cc50ed: function(arg0) {
            const ret = arg0.colSpan;
            return ret;
        },
        __wbg_contains_6b23671a193f58e5: function(arg0, arg1) {
            const ret = arg0.contains(arg1);
            return ret;
        },
        __wbg_createElement_9b0aab265c549ded: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.createElement(getStringFromWasm0(arg1, arg2));
            return ret;
        }, arguments); },
        __wbg_createObjectURL_f141426bcc1f70aa: function() { return handleError(function (arg0, arg1) {
            const ret = URL.createObjectURL(arg1);
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_document_c0320cd4183c6d9b: function(arg0) {
            const ret = arg0.document;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_from_4bdf88943703fd48: function(arg0) {
            const ret = Array.from(arg0);
            return ret;
        },
        __wbg_getComputedStyle_b12e52450a4be72c: function() { return handleError(function (arg0, arg1) {
            const ret = arg0.getComputedStyle(arg1);
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_getElementById_d1f25d287b19a833: function(arg0, arg1, arg2) {
            const ret = arg0.getElementById(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_getPropertyValue_d2181532557839cf: function() { return handleError(function (arg0, arg1, arg2, arg3) {
            const ret = arg1.getPropertyValue(getStringFromWasm0(arg2, arg3));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        }, arguments); },
        __wbg_get_3ef1eba1850ade27: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_get_a8ee5c45dabc1b3b: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return ret;
        },
        __wbg_get_with_index_51afe72d3653615f: function(arg0, arg1) {
            const ret = arg0[arg1 >>> 0];
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_innerText_39b56e7a85eaff79: function(arg0, arg1) {
            const ret = arg1.innerText;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_instanceof_HtmlAnchorElement_085fbdab589de474: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLAnchorElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlTableCellElement_76ee822ee2df6d78: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLTableCellElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlTableElement_df8809c21c7fcad6: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLTableElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlTableRowElement_f5a68a0215737673: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLTableRowElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_HtmlTableSectionElement_4146c9419496e3b5: function(arg0) {
            let result;
            try {
                result = arg0 instanceof HTMLTableSectionElement;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Window_23e677d2c6843922: function(arg0) {
            let result;
            try {
                result = arg0 instanceof Window;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_33b91feb269ff46e: function(arg0) {
            const ret = Array.isArray(arg0);
            return ret;
        },
        __wbg_length_8e55f95606d8e278: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_length_b3416cf66a5452c8: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_new_a70fbab9066b301f: function() {
            const ret = new Array();
            return ret;
        },
        __wbg_new_ab79df5bd7c26067: function() {
            const ret = new Object();
            return ret;
        },
        __wbg_new_d098e265629cd10f: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__h00d297b34697abe8(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = state0.b = 0;
            }
        },
        __wbg_new_from_slice_22da9388ac046e50: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return ret;
        },
        __wbg_new_typed_aaaeaf29cf802876: function(arg0, arg1) {
            try {
                var state0 = {a: arg0, b: arg1};
                var cb0 = (arg0, arg1) => {
                    const a = state0.a;
                    state0.a = 0;
                    try {
                        return wasm_bindgen__convert__closures_____invoke__h00d297b34697abe8(a, state0.b, arg0, arg1);
                    } finally {
                        state0.a = a;
                    }
                };
                const ret = new Promise(cb0);
                return ret;
            } finally {
                state0.a = state0.b = 0;
            }
        },
        __wbg_new_with_u8_array_sequence_and_options_de38f663e19ad899: function() { return handleError(function (arg0, arg1) {
            const ret = new Blob(arg0, arg1);
            return ret;
        }, arguments); },
        __wbg_now_16f0c993d5dd6c27: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_of_8bf7ed3eca00ea43: function(arg0) {
            const ret = Array.of(arg0);
            return ret;
        },
        __wbg_parse_e9eddd2a82c706eb: function() { return handleError(function (arg0, arg1) {
            const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
            return ret;
        }, arguments); },
        __wbg_push_e87b0e732085a946: function(arg0, arg1) {
            const ret = arg0.push(arg1);
            return ret;
        },
        __wbg_querySelector_332d8dfa3e191085: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.querySelector(getStringFromWasm0(arg1, arg2));
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        }, arguments); },
        __wbg_queueMicrotask_0c399741342fb10f: function(arg0) {
            const ret = arg0.queueMicrotask;
            return ret;
        },
        __wbg_queueMicrotask_a082d78ce798393e: function(arg0) {
            queueMicrotask(arg0);
        },
        __wbg_resolve_ae8d83246e5bcc12: function(arg0) {
            const ret = Promise.resolve(arg0);
            return ret;
        },
        __wbg_revokeObjectURL_c4a7ed8e1908b794: function() { return handleError(function (arg0, arg1) {
            URL.revokeObjectURL(getStringFromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_rowSpan_6f470c9c8e3c6d59: function(arg0) {
            const ret = arg0.rowSpan;
            return ret;
        },
        __wbg_rows_43c2c367cc313d4c: function(arg0) {
            const ret = arg0.rows;
            return ret;
        },
        __wbg_rows_71a2f6be15bb9391: function(arg0) {
            const ret = arg0.rows;
            return ret;
        },
        __wbg_setTimeout_7f7035ad0b026458: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.setTimeout(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_set_download_c59352398d4fe8c5: function(arg0, arg1, arg2) {
            arg0.download = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_href_de1379dfb6df96a6: function(arg0, arg1, arg2) {
            arg0.href = getStringFromWasm0(arg1, arg2);
        },
        __wbg_set_type_33e79f1b45a78c37: function(arg0, arg1, arg2) {
            arg0.type = getStringFromWasm0(arg1, arg2);
        },
        __wbg_static_accessor_GLOBAL_8adb955bd33fac2f: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_ad356e0db91c7913: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_f207c857566db248: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_bb9f1ba69d61b386: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_tHead_407d72c52918404c: function(arg0) {
            const ret = arg0.tHead;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_then_098abe61755d12f6: function(arg0, arg1) {
            const ret = arg0.then(arg1);
            return ret;
        },
        __wbg_then_9e335f6dd892bc11: function(arg0, arg1, arg2) {
            const ret = arg0.then(arg1, arg2);
            return ret;
        },
        __wbg_warn_69424c2d92a2fa73: function(arg0) {
            console.warn(arg0);
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 173, function: Function { arguments: [Externref], shim_idx: 974, ret: Result(Unit), inner_ret: Some(Result(Unit)) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__hb1f1e222d193ce94, wasm_bindgen__convert__closures_____invoke__h4c21a5017d209281);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Closure(Closure { dtor_idx: 3, function: Function { arguments: [], shim_idx: 4, ret: Unit, inner_ret: Some(Unit) }, mutable: true }) -> Externref`.
            const ret = makeMutClosure(arg0, arg1, wasm.wasm_bindgen__closure__destroy__h47a02344c072fbfb, wasm_bindgen__convert__closures_____invoke__h493839daf82ac541);
            return ret;
        },
        __wbindgen_cast_0000000000000003: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return ret;
        },
        __wbindgen_cast_0000000000000004: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./belobog_stellar_grid_bg.js": import0,
    };
}

function wasm_bindgen__convert__closures_____invoke__h493839daf82ac541(arg0, arg1) {
    wasm.wasm_bindgen__convert__closures_____invoke__h493839daf82ac541(arg0, arg1);
}

function wasm_bindgen__convert__closures_____invoke__h4c21a5017d209281(arg0, arg1, arg2) {
    const ret = wasm.wasm_bindgen__convert__closures_____invoke__h4c21a5017d209281(arg0, arg1, arg2);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function wasm_bindgen__convert__closures_____invoke__h00d297b34697abe8(arg0, arg1, arg2, arg3) {
    wasm.wasm_bindgen__convert__closures_____invoke__h00d297b34697abe8(arg0, arg1, arg2, arg3);
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => state.dtor(state.a, state.b));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {

        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            state.a = a;
            real._wbg_cb_unref();
        }
    };
    real._wbg_cb_unref = () => {
        if (--state.cnt === 0) {
            state.dtor(state.a, state.b);
            state.a = 0;
            CLOSURE_DTORS.unregister(state);
        }
    };
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasm;
function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    wasmModule = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('belobog_stellar_grid_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
