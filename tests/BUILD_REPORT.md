# belobog-stellar-grid 构建报告

## 基本信息
- **包版本**: 1.0.3
- **Rust Edition**: 2024
- **最低 Rust 版本**: 1.85.0

## 文件检查
- ✅ belobog_stellar_grid.js
- ✅ belobog_stellar_grid_bg.wasm
- ✅ belobog_stellar_grid.d.ts
- ✅ package.json

## API 检查
- ✅ `export_table` — 统一导出（CSV/XLSX）
- ✅ `export_data` — 纯数据导出（二维数组/对象数组/树形/合并单元格）
- ✅ `export_tables_xlsx` — 多工作表同步导出
- ✅ `export_table_to_csv_batch` — CSV 分批异步导出
- ✅ `export_table_to_xlsx_batch` — XLSX 分批异步导出
- ✅ `export_tables_to_xlsx_batch` — 多工作表分批异步导出

## 测试页面
- 🌐 手动功能验证页面: `fixtures/test-page.html`
- 📱 可在浏览器中打开进行端到端功能测试

## 使用方法
```javascript
import init, {
    export_table,
    export_data,
    export_tables_xlsx,
    export_table_to_csv_batch,
    export_table_to_xlsx_batch,
    ExportFormat
} from './pkg/belobog_stellar_grid.js';

await init();

// 统一导出
export_table('table-id', '文件名.csv');
export_table('table-id', '文件名.xlsx', ExportFormat.Xlsx);

// 纯数据导出
export_data(arrayData, { filename: '数据.xlsx', format: ExportFormat.Xlsx });

// 分批异步导出
await export_table_to_csv_batch('table-id', null, 'filename.csv', 1000, false, (progress) => {
    console.log(`进度: ${progress}%`);
});
```

## 命令行
```bash
# 构建
wasm-pack build --target web

# 运行测试
cargo test

# 格式化和代码检查
cargo fmt
cargo clippy -- -D warnings
```