# @bsg-export/vue 示例

演示 `useExporter` 和 `useWorkerExporter` Composable 的基本用法。

## 运行步骤

```bash
# 1. 在项目根目录构建 WASM
wasm-pack build --target web

# 2. 构建子包
just build-packages

# 3. 安装依赖并启动开发服务器
cd packages/vue/examples
pnpm install
pnpm dev
```

## 功能演示

- **DOM 表格导出** — 使用 `useExporter` 从 HTML 表格导出 CSV/XLSX
- **纯数据导出** — 使用 `exportData()` 从 JS 数组直接导出
- **Worker 后台导出** — 使用 `useWorkerExporter` 在后台线程处理大数据导出
