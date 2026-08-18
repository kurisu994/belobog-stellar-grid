# 项目简介 (Project Brief)

## 是什么

`belobog-stellar-grid` 是一个 **Rust + WebAssembly 表格导出库**，让浏览器端把 HTML 表格或 JS 数据导出为 **CSV / XLSX** 文件，并支持 **Excel 文件在线预览**。

- 包名：`belobog-stellar-grid`
- 版本：`1.1.9`
- 许可：`MIT OR Apache-2.0`
- 仓库：<https://github.com/kurisu994/belobog-stellar-grid>
- 产物类型：`cdylib`（wasm）+ `rlib`

## 解决什么问题

传统前端导出方案（纯 JS 库）在**大数据量**下容易阻塞主线程、内存占用高，且缺少安全防护。本项目把导出计算下沉到 WASM：

1. **性能**：Rust 原生速度处理百万级行数据
2. **不卡页面**：分批异步 + 主动让出事件循环
3. **安全**：文件名校验、CSV/Excel 公式注入防护、Blob URL 生命周期管理
4. **保真**：合并单元格、多工作表、三级样式体系、冻结窗格

## 目标用户

- 需要在浏览器导出报表的前端开发者
- 使用 React / Vue / Svelte / Solid 的应用（提供官方封装子包）
- 需要 Web Worker 离主线程导出的场景
- 需要在页面内预览 Excel 文件的场景

## 交付物

| 交付物 | 说明 |
| ------ | ---- |
| WASM 核心包 | `wasm-pack build --target web --out-dir pkg` 产出 |
| `@bsg-export/types` | 共享 TypeScript 类型定义 |
| `@bsg-export/react` | React 组件与 hooks 封装 |
| `@bsg-export/vue` | Vue 封装 |
| `@bsg-export/svelte` | Svelte 封装 |
| `@bsg-export/solid` | Solid.js 封装 |
| `@bsg-export/worker` | Web Worker 封装 |
| `examples/` | 15 个可直接打开的示例页面 |

> 所有子包版本与主库保持一致（当前 `1.1.9`）。

## 功能范围

**包含：**

- HTML 表格导出（CSV / XLSX），支持容器内自动查找 `<table>`
- JS 数组 / 对象数组 / 树形数据导出（不依赖 DOM）
- 多工作表 XLSX 导出
- 分批异步导出（CSV / XLSX）与流式 CSV 导出
- 仅生成字节不触发下载（Worker 场景）
- Excel（xlsx/xls）解析预览，双输出：HTML 直出 + JSON 结构化数据
- 三级样式体系：全局 → 列级 → 单元格级

**不包含：**

- 服务端导出 / Node 端运行
- Excel 公式计算（统一按文本写入，防注入）
- xlsx 真正的流式写入（受 `rust_xlsxwriter` 限制，`format=Xlsx` 时回退同步逻辑）
