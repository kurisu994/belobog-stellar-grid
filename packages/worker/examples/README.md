# @bsg-export/worker 示例

演示 `ExportWorker` 类的基本用法（React + TypeScript）。

## 运行步骤

```bash
# 1. 在项目根目录构建 WASM
wasm-pack build --target web

# 2. 构建子包
just build-packages

# 3. 安装依赖并启动开发服务器
cd packages/worker/examples
pnpm install
pnpm dev
```

## 功能演示

- **Worker 导出 CSV/XLSX** — 使用 `ExportWorker.exportData()` 在后台线程生成文件并下载
- **生成字节** — 使用 `ExportWorker.generateBytes()` 仅生成文件字节，不触发下载
