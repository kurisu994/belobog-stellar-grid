# Excel 在线预览 (Rust WASM) 开发计划

## 需求概述

通过接口获取 Excel 文件地址，在 Web 页面中渲染文件内容，要求：

- **保持 Excel 原始样式**（字体、颜色、边框、合并单元格等）
- **只渲染有数据的区域**，不展示空白单元格和行列头
- **可嵌入到普通 DOM 节点中**，与其他页面元素共存
- **只读查看**，不需要编辑能力

基于前期方案对比，确认采用 **方案：Rust WASM** 作为 `belobog-stellar-grid` 中 Excel 文件在线预览的核心实现。
该方案充分契合现有项目的基建体系，能与原有的 `src/core` 逻辑及 JS Wrap 生态完美融合。开发此功能需按照以下阶段逐步进行：

## 阶段一：技术预研与基础设施搭建

> **目标**：跑通核心链路，引入依赖并打通 Rust 到 JS 的传递闭环。

1. **依赖选型对比与引入**：
   - 在 `Cargo.toml` 中引入 Excel 解析库。
   - 优先评估 `umya-spreadsheet`（样式全，但包体偏大）与 `calamine`（轻量，仅基础格式）的体积和性能权衡，建议初期采用 `umya-spreadsheet` 保障原生样式覆盖度。
   - ⚠️ **备选路线**：若上述 crate 不兼容 WASM 目标，则采用 `zip` + `quick-xml` 手动解析 xlsx（xlsx 本质是 zip 包含 XML 文件），需额外实现样式解析逻辑。
2. **WASM 可编译性验证（关键前置步骤）**：
   - 引入依赖后，**第一时间执行** `cargo build --target wasm32-unknown-unknown` 确认依赖链无文件系统、网络等不兼容调用。
   - 若编译失败，需排查依赖中的 `std::fs`、`std::net` 等非 WASM 安全调用，评估是否需要切换备选库或提交上游 PR。
3. **核心模块创建**：
   - 在 `src/core/` 建立 `excel_preview.rs` 作为解析模块。
4. **全局 API 暴露**：
   - 在 `src/core/mod.rs` & `src/lib.rs` 暴露统一入口，例如：

     ```rust
     #[wasm_bindgen(js_name = parseExcelToHtml)]
     pub fn parse_excel_to_html(data: &[u8], options: Option<JsValue>) -> Result<String, JsValue>
     ```

   - `options` 支持以下可选配置（与 `export_data(data, options)` 的风格对齐）：
     - `sheetIndex` / `sheetName`：指定渲染哪个 Sheet（默认渲染第一个活跃 Sheet）
     - `maxRows`：最大渲染行数上限（防止海量 DOM 导致页面假死）
     - `includeStyles`：是否保留样式（默认 `true`，设为 `false` 可快速预览纯数据）

5. **编译体积优化配置**：
   - 检查并优化 `.wasm` 输出体积（配置 `lto = "fat"`, `opt-level = 'z'`），确保引入新库后包体积增量可控。

## 阶段二：核心解析逻辑与 HTML 拼装

> **目标**：解析二进制 Excel 数据，提取结构、数据和样式映射，将其转译为原生 `<table>` 字符串。

1. **基础表结构构建**：
   - 根据 `options` 参数读取指定工作表（默认第一个活跃 Sheet）。
   - 计算数据实际范围（摒弃无效空白行/列）。
   - 若启用 `maxRows` 限制，截断超出部分并在返回的 HTML 中附加截断提示。
2. **数据与格式映射**：
   - 读取单元格数据。
   - **合并单元格算法（核心难点）**：解析 `MergeCells` 逻辑，输出 `rowspan` 和 `colspan` 属性，并在后续生成时跳过被占用的网格（避免 DOM 错位）。
3. **CSS 样式映射（重点优先集萃）**：建设 `style.rs` 负责转译：
   - **背景色** (`background-color`)
   - **字体** (`font-weight`, `font-style`, `font-family`, `font-size`)
   - **文本颜色** (`color`)
   - **文本对齐** (`text-align`, `vertical-align`)
   - **边框** (`border`)
4. **安全防护（关键）**：
   - **HTML 实体转义**：单元格文本内容必须对 `<`, `>`, `&`, `"`, `'` 进行转义，防止 XSS 攻击（如恶意嵌入 `<script>` 标签的 Excel 文件）。
   - **CSS 注入防护**：内联样式值需过滤 `expression()`、`url()`、`javascript:` 等危险模式。
   - 在 `tests/test_security.rs` 中补充 Excel 预览场景的安全测试用例。
5. **内存管理**：
   - 使用 Rust 原生 `String::with_capacity` 预留足够内存提升拼接性能。
   - 保障中文报错 (`Result<T, JsValue>`)。

## 阶段三：前端包 (Packages) 生态接入

> **目标**：在 React / Vue 侧对功能进行开箱即用的封装。
> **本期覆盖范围**：`types`、`react`、`vue` 三个子包。`solid`、`svelte`、`worker` 暂不适配，后续按需扩展。

1. **统一类型定义 (`packages/types`)**：
   - 声明 `parseExcelToHtml(data: Uint8Array, options?: PreviewOptions): string;`
   - 定义 `PreviewOptions` 接口：
     ```typescript
     interface PreviewOptions {
       sheetIndex?: number;
       sheetName?: string;
       maxRows?: number;
       includeStyles?: boolean;
     }
     ```
2. **React 封装 (`packages/react`)**：
   - 新增 `useExcelPreview` Hook，管理 `Web Worker`（可选，防假死）和 Loading 状态。
   - 新增 `<ExcelPreview url={string} />` UI 组件：负责 URL `fetch` → 提供 `ArrayBuffer` → 调用 WASM → 将返回结果挂载至内部 DOM (通过 `dangerouslySetInnerHTML`)。
3. **Vue 封装 (`packages/vue`)**：
   - 增加对等的 `useExcelPreview` Composable。
   - 增加 `<ExcelPreview />` 组件（基于 `v-html` 或 ref 挂载）。

## 阶段四：性能攻坚与边界测试

> **目标**：保证大文件不会引发页面崩溃，以及复杂合并场景下的鲁棒性。

1. **海量 DOM 假死预防**：
   - 对行数/列数设定合理上界告警，或者探讨分块生成 HTML、前端使用 `IntersectionObserver` 配合虚拟列表渲染。
2. **测试文件与用例 (`tests/`)** ：
   - 新建 `tests/test_excel_preview.rs`（对标已有的 `test_data_export.rs` 等测试文件）。
   - 在 `tests/fixtures/` 下准备小型 `.xlsx` 测试文件（含各类场景）。
   - 具体用例：

     | 用例类别 | 输入 | 断言要点 |
     |---|---|---|
     | **基础解析** | 普通 3×3 数据 xlsx | 输出 HTML 包含 `<table>`、正确的 `<td>` 内容 |
     | **合并单元格** | 含横向/纵向合并 xlsx | HTML 包含正确的 `rowspan`/`colspan` |
     | **样式还原** | 含背景色/字体/边框 xlsx | 内联 `style` 属性包含对应 CSS |
     | **空文件** | 0 字节或纯空 Sheet | 返回中文 Error 而非 panic |
     | **损坏文件** | 非法二进制数据 | 返回中文 Error 而非 panic |
     | **XSS 防护** | 单元格含 `<script>` | HTML 输出中已转义为 `&lt;script&gt;` |
     | **Sheet 选择** | 多 Sheet xlsx + `sheetIndex` | 仅渲染指定 Sheet 的数据 |
     | **行数上限** | 1000 行 xlsx + `maxRows: 50` | 输出仅含 50 行 `<tr>` |

3. **CI 集成**：
   - 确保 `cargo test` 及全部前置检查 (100+ 单测用例) 及新用例均可通过。

## 资源评估与下一步操作

**耗时预估**：

- **阶段一 + 阶段二**：主要底层和解析逻辑（约占 60% 精力，主要是样式的解析映射较为繁琐）。
- **阶段三**：前端集成逻辑（约占 20% 精力）。
- **阶段四**：性能与测试用例联调（约占 20% 精力）。

**下一步**：
确认此计划无误后，优先从 **阶段一** 开始，并在 `Cargo.toml` 探明并集成 Excel 依赖解析库。
