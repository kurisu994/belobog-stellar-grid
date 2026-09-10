---
name: 00-project
description: 项目定位、功能边界、核心用户流与产品约束（开局必读）
---

# 项目定位 (Project)

> 开局必读。项目是什么、边界在哪、为什么这么设计。
> 编码约定按代码路径分散在 `1X-*.md`，索引见 `README.md`。

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

## 产品约束

浏览器端导出是**用户可感知的同步操作**：点击按钮后必须尽快出现下载，且页面不能卡死。设计围绕三个约束展开：

1. **响应性**：大数据必须分批 / 流式，中间让出事件循环并回报进度
2. **正确性**：合并单元格、样式、冻结窗格要与页面所见一致
3. **安全性**：导出内容会被 Excel 打开，必须防公式注入；文件名来自用户输入，必须防路径遍历

## 核心用户流

### 流程一：DOM 表格导出

```
用户点击导出
    │
    ├─ export_table(tableId, filename, format?)
    │       │
    │       ├─ resolve_table: 按 ID 找 <table>，找不到则在容器内查找
    │       ├─ extract_table_data_with_merge: 遍历行列
    │       │     ├─ RowSpanTracker 处理 rowspan 占位
    │       │     └─ exclude_hidden 时跳过 display:none
    │       ├─ CSV → escape_csv_injection 逐格转义
    │       │  XLSX → write_string + 样式/合并/冻结
    │       └─ trigger_bytes_download
    │             ├─ prepare_download_filename（先校验！）
    │             ├─ 创建 Blob + Object URL
    │             ├─ <a download> 点击
    │             └─ schedule_url_revoke（10 秒后释放）
    ▼
浏览器下载文件
```

### 流程二：大数据分批导出

```
export_table_to_csv_batch / export_table_to_xlsx_batch
    │
    ├─ TableRowSources::open  统一解析 table + 可选外部 tbody
    │
    └─ while 还有行:
          ├─ 处理一批（默认 1000 行）
          ├─ CSV: 编码成 Uint8Array 片段推入 blob_parts
          │  XLSX: 累积到 TableData
          ├─ report_progress(百分比)
          └─ yield_to_browser()  ← 让出主线程
```

### 流程三：Excel 预览

```
parse_excel_to_html(bytes, options) / parse_excel_to_json(...)
    │
    ├─ calamine 打开 workbook（1 次）
    ├─ 共享 ZipArchive（1 次）解析 styles.xml + sheet XML
    │     ├─ 行高/列宽/隐藏行列
    │     ├─ 单元格样式索引
    │     └─ 条件格式（范围表示，不展开坐标）
    ├─ 合并区域从已打开的 workbook 读取
    └─ 输出 HTML <table>（内联 style）或 JSON 结构
```

## 特殊约束

| 约束 | 说明 |
| ---- | ---- |
| tbody 必须外置 | 传 `tbodyId` 时该元素**不能**在 `tableId` 表格内部，否则数据重复导出；`ensure_external_tbody` 运行时强制拦截 |
| 公式注入 | CSV 字段以 `= + - @ \t \r` 开头会加前缀 `'`；会先剥离前导 BOM 再判断 |
| XLSX 写入 | 一律 `write_string`，绝不写公式 |
| 文件名 | 禁止路径分隔符、控制字符、`< > : " \| ? *`、Windows 保留名、首尾点/空格、全角点号；上限 255 字节 |
| 隐藏行列 | 预览与导出均可跳过 `display:none` / xlsx `hidden="1"` |
| 错误消息 | 一律中文，面向最终用户 |

## 交互逻辑

- **进度回调**：接收 0–100 的百分比。默认宽松模式（回调抛错仅 `console.warn`）；`strictProgressCallback=true` 时回调失败会中断导出
- **冻结窗格**：默认按表头行数自动冻结；用户可用 `freezeRows` / `freezeCols` 覆盖；**超出数据区则回退为不冻结**
- **样式优先级**：全局 → 列级 → 单元格级，逐层 merge
- **BOM**：`withBom=true` 时 CSV 加 UTF-8 BOM，便于 Excel 正确识别中文

## 数据安全

- 导出内容不上传，全程在浏览器内存中处理
- Blob URL 延迟 10 秒 revoke，避免下载竞态又不长期泄漏
- 非法文件名在**创建 Blob URL 之前**就返回错误，杜绝 URL 泄漏
- 预览 HTML 输出对文本做实体转义，对 `style` 属性做值净化
