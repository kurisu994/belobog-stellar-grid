# Excel 在线预览 (Rust WASM) 开发计划

## 一、需求概述

通过接口获取 Excel 文件地址，在 Web 页面中渲染文件内容，要求：

- **保持 Excel 原始样式**（字体、颜色、边框、合并单元格等）
- **只渲染有数据的区域**，不展示空白单元格和行列头
- **可嵌入到普通 DOM 节点中**，与其他页面元素共存
- **只读查看**，不需要编辑能力

基于前期方案对比，确认采用 **Rust WASM** 作为 `belobog-stellar-grid` 中 Excel 文件在线预览的核心实现。

---

## 二、参考实现分析（fw-breport-profit/excel-demo）

现有 JS 方案基于 ExcelJS + SheetJS，核心文件 `ExcelTable.tsx`（约 735 行）。

**数据流**：

```text
fetch(url) → ArrayBuffer → 格式检测(魔数 D0CF11E0)
  ├─ .xls  → SheetJS 解析
  └─ .xlsx → ExcelJS 解析
→ ParsedSheet[] → HTML Table 渲染
```

**核心数据结构**：

```typescript
interface ParsedSheet {
  name: string;
  rows: ParsedRow[];
  colWidths: number[];
}

interface ParsedCell {
  value: string;
  style: CSSProperties;
  colSpan?: number;
  rowSpan?: number;
}
```

**已实现的样式能力**：

- 字体：fontFamily / fontSize / fontWeight / fontStyle / textDecoration / color
- 填充：backgroundColor（pattern 类型）
- 对齐：textAlign / verticalAlign / whiteSpace / wordBreak
- 边框：thin / medium / thick / dotted / dashed / double
- 主题色 + tint 色调偏移（OOXML 标准 10 色主题）
- 数字格式化（百分比、千分位、固定小数）
- 公式递归提取 result、富文本拼接、超链接提取

**合并单元格**：构建 mergeMap，主单元格记录 rowSpan/colSpan，被合并单元格标记 skip。

**列宽算法**：跳过多列合并单元格，估算文本像素宽度（CJK ≈ fontSize，Latin ≈ 55%），范围 55px~300px。

---

## 三、依赖选型

### 3.1 方案对比

| 维度             | calamine                                      | umya-spreadsheet                     | zip + quick-xml 手动        |
| ---------------- | --------------------------------------------- | ------------------------------------ | --------------------------- |
| WASM 编译        | 直接可用（xlsx-wasm-parser 已验证）            | 需 workaround（getrandom/js）        | 直接可用                    |
| 样式读取         | PR #538 即将合并 / calamine-styles fork 可用   | 完整支持                             | 需自行实现 OOXML 规范       |
| 合并单元格       | 已支持（`merged_regions()` API）               | 已支持                               | 需自行实现                  |
| .xls 格式        | 支持                                          | 不支持                               | 不支持（需额外 OLE2 解析）  |
| WASM 体积增量    | ~300-500KB（gzip ~100-200KB）                  | ~1-3MB（gzip ~400KB-1MB）            | ~100-200KB                  |
| 依赖复杂度       | 低（quick-xml / zip / encoding_rs）            | 高（加密/正则/图片/chrono）          | 最低                        |
| 开发工作量       | 低（API 现成）                                 | 低                                   | 极高（手写 OOXML 解析）     |
| 维护活跃度       | 高                                            | 中                                   | N/A                         |

### 3.2 选型结论

**首选 calamine + calamine-styles**（过渡期），待 calamine PR #538 合并后切回主线。

理由：

1. WASM 兼容性已被 xlsx-wasm-parser 项目验证，纯 Rust 无系统依赖
2. 样式支持通过 `calamine-styles` fork 可立即获得（Font / Fill / Border / Alignment / NumberFormat）
3. 体积可控（~300-500KB），与现有项目 wasm-pack 技术栈完全一致
4. 合并单元格已原生支持（`merged_regions()` API）
5. 同时支持 .xlsx 和 .xls 格式

**兜底方案**：若 calamine 生态无法满足样式精度要求，则在 calamine 基础上用 zip + quick-xml 补充 `xl/styles.xml` 解析逻辑，而非完全从零开始。

---

## 四、架构设计

### 4.1 整体架构

```text
                        belobog-stellar-grid
┌─────────────────────────────────────────────────────────┐
│  src/lib.rs (WASM 入口，重导出)                          │
│  ├── src/core/mod.rs (统一 API)                          │
│  │   ├── export_table / export_data (已有导出功能)        │
│  │   └── parse_excel_* (新增：Excel 预览)                │
│  ├── src/core/excel_reader.rs (新增：Excel 解析核心)      │
│  ├── src/core/excel_style.rs  (新增：样式 → CSS 映射)    │
│  └── src/core/html_builder.rs (新增：HTML Table 拼装)    │
└─────────────────────────────────────────────────────────┘
         ↓ wasm-pack build
┌─────────────────────────────────────────────────────────┐
│  pkg/ (WASM 包输出)                                      │
│  ├── parseExcelToHtml(data, options?) → string           │
│  ├── parseExcelToJson(data, options?) → object           │
│  └── getExcelSheetList(data) → SheetInfo[]               │
└─────────────────────────────────────────────────────────┘
         ↓ npm 发布
┌─────────────────────────────────────────────────────────┐
│  packages/                                               │
│  ├── types/   → PreviewOptions 类型定义                   │
│  ├── react/   → useExcelPreview Hook + <ExcelPreview />  │
│  └── vue/     → useExcelPreview Composable + 组件         │
└─────────────────────────────────────────────────────────┘
```

### 4.2 新增模块职责

| 模块 | 职责 | 对标 JS 实现 |
| --- | --- | --- |
| `excel_reader.rs` | 调用 calamine 解析 Excel 二进制数据，提取工作表列表、单元格值、合并区域、行高列宽 | ExcelJS `workbook.xlsx.load()` + `parseSheet()` |
| `excel_style.rs` | 将 calamine 样式对象转换为 CSS 内联样式字符串，含主题色/tint/边框映射 | `cellToCss()` + `parseColor()` + `applyTint()` |
| `html_builder.rs` | 拼装 `<table>` HTML 字符串，处理 rowSpan/colSpan、列宽、行高、数据区域裁剪 | `ExcelTable` 组件的 JSX 渲染逻辑 |

### 4.3 WASM API 设计

```rust
/// 解析 Excel 文件并返回 HTML Table 字符串
#[wasm_bindgen(js_name = parseExcelToHtml)]
pub fn parse_excel_to_html(data: &[u8], options: JsValue) -> Result<JsValue, JsValue>

/// 解析 Excel 文件并返回结构化 JSON（供前端自行渲染）
#[wasm_bindgen(js_name = parseExcelToJson)]
pub fn parse_excel_to_json(data: &[u8], options: JsValue) -> Result<JsValue, JsValue>

/// 获取 Excel 文件的工作表列表（名称 + 行列数）
#[wasm_bindgen(js_name = getExcelSheetList)]
pub fn get_excel_sheet_list(data: &[u8]) -> Result<JsValue, JsValue>
```

**Options 参数**（与 `export_data(data, options)` 风格对齐）：

```typescript
interface PreviewOptions {
  sheetIndex?: number;      // 指定渲染的 Sheet 索引（默认 0）
  sheetName?: string;       // 或按名称指定 Sheet
  maxRows?: number;         // 最大渲染行数（防 DOM 假死）
  maxCols?: number;         // 最大渲染列数
  includeStyles?: boolean;  // 是否保留样式（默认 true，false 可快速预览纯数据）
  trimEmpty?: boolean;      // 是否裁剪空白区域（默认 true）
}
```

**返回值设计（parseExcelToJson）**：

```typescript
interface ParsedWorkbook {
  sheets: ParsedSheet[];
  activeSheet: number;
}

interface ParsedSheet {
  name: string;
  rows: ParsedRow[];
  colWidths: number[];
  mergedCells: MergeRange[];
  truncated?: boolean;       // 是否因 maxRows 被截断
}

interface ParsedRow {
  height?: number;
  cells: (ParsedCell | null)[];
}

interface ParsedCell {
  value: string;
  style?: string;            // 内联 CSS 字符串
  colSpan?: number;
  rowSpan?: number;
}
```

### 4.4 双输出模式

提供两种输出以适配不同场景：

1. **HTML 模式**（`parseExcelToHtml`）：Rust 侧完成全部拼装，返回完整 `<table>` HTML 字符串。前端通过 `dangerouslySetInnerHTML` / `v-html` 直接挂载。适合简单集成场景。

2. **JSON 模式**（`parseExcelToJson`）：返回结构化数据，前端自行渲染。适合需要自定义渲染逻辑（虚拟滚动、交互等）的场景。

---

## 五、样式映射方案

### 5.1 CSS 映射清单（对标 excel-demo）

| Excel 属性 | CSS 属性 | 复杂度 | 优先级 |
| --- | --- | --- | --- |
| Font.name | font-family | 低 | P0 |
| Font.size | font-size (pt) | 低 | P0 |
| Font.bold | font-weight: bold | 低 | P0 |
| Font.italic | font-style: italic | 低 | P0 |
| Font.underline | text-decoration: underline | 低 | P1 |
| Font.strikethrough | text-decoration: line-through | 低 | P1 |
| Font.color | color | 中（需处理主题色） | P0 |
| Fill.fgColor | background-color | 中（pattern 类型） | P0 |
| Alignment.horizontal | text-align | 低 | P0 |
| Alignment.vertical | vertical-align | 低 | P0 |
| Alignment.wrapText | white-space: pre-wrap | 低 | P0 |
| Border.* | border-top/bottom/left/right | 中（样式映射） | P0 |
| NumberFormat | 数字格式化显示 | 高 | P1 |
| 主题色 + tint | 颜色计算 | 高 | P1 |

### 5.2 主题色处理

OOXML 标准定义 10 个主题色，tint 值控制明暗偏移：

```rust
const THEME_COLORS: [&str; 10] = [
    "#FFFFFF", "#000000", "#E7E6E6", "#44546A", "#4472C4",
    "#ED7D31", "#A5A5A5", "#FFC000", "#5B9BD5", "#70AD47",
];

fn apply_tint(hex: &str, tint: f64) -> String {
    // tint > 0: 混合白色（变亮）
    // tint < 0: 混合黑色（变暗）
}
```

### 5.3 边框样式映射

```rust
fn border_style_to_css(style: &str) -> &str {
    match style {
        "thin"   => "1px solid",
        "medium" => "2px solid",
        "thick"  => "3px solid",
        "dotted" => "1px dotted",
        "dashed" => "1px dashed",
        "double" => "3px double",
        "hair"   => "1px solid",
        _        => "1px solid",
    }
}
```

---

## 六、安全防护

与现有项目安全策略保持一致：

| 威胁 | 防护措施 | 实现位置 |
| --- | --- | --- |
| XSS 注入 | 单元格文本 HTML 实体转义（`<>&"'`） | html_builder.rs |
| CSS 注入 | 过滤 `expression()` / `url()` / `javascript:` | excel_style.rs |
| 路径遍历 | 复用现有 `validate_filename()` | validation.rs |
| 栈溢出 | 限制嵌套深度（MAX_DEPTH=64） | excel_reader.rs |
| OOM | maxRows/maxCols 上限 + 数据区域裁剪 | excel_reader.rs |
| 恶意文件 | calamine 自身的格式校验 + 错误处理 | excel_reader.rs |

---

## 七、实施路线

### 阶段一：技术预研与基础设施搭建（预计 1-2 天）

> **目标**：跑通核心链路，引入依赖并打通 Rust 到 JS 的传递闭环。

1. **依赖引入与 WASM 编译验证**：
   - 在 `Cargo.toml` 引入 `calamine`（或 `calamine-styles`）
   - **第一时间执行** `cargo build --target wasm32-unknown-unknown` 确认依赖链无 `std::fs`、`std::net` 等不兼容调用
   - 若编译失败，排查依赖链或切换备选方案

2. **最小 PoC**：
   - 编写最小可运行代码：读取 .xlsx 文件 → 提取单元格值 → 返回 JSON
   - 确认合并单元格 API（`merged_regions()`）可用

3. **核心模块创建**：
   - 在 `src/core/` 建立 `excel_reader.rs`

4. **全局 API 暴露**：
   - 在 `src/core/mod.rs` & `src/lib.rs` 暴露 `parse_excel_to_html` 入口

5. **编译体积评估**：
   - 检查 `.wasm` 输出体积增量，配置 `lto = "fat"`, `opt-level = 'z'`

### 阶段二：核心解析逻辑与 HTML 拼装（预计 3-5 天）

> **目标**：解析二进制 Excel 数据，提取结构、数据和样式映射，转译为原生 `<table>` 字符串。

1. **excel_reader.rs — 数据提取**：
   - 根据 `options` 参数读取指定工作表（默认第一个活跃 Sheet）
   - 计算数据实际范围（摒弃无效空白行/列）
   - 若启用 `maxRows` 限制，截断超出部分并附加截断标记
   - **合并单元格算法**：解析 `MergeCells`，输出 `rowspan`/`colspan`，跳过被占用网格

2. **excel_style.rs — 样式映射引擎**：
   - P0 样式：背景色、字体（family/size/bold/italic/color）、对齐、边框
   - P1 样式：下划线、删除线、数字格式化、主题色 + tint
   - CSS 注入防护：过滤 `expression()` / `url()` / `javascript:`

3. **html_builder.rs — HTML 拼装**：
   - 使用 `String::with_capacity` 预留内存提升拼接性能
   - HTML 实体转义（`<>&"'`）防止 XSS
   - 生成 `<colgroup>` 设置列宽、`<tr>` 设置行高
   - 处理 rowSpan/colSpan 的 `<td>` 属性

4. **统一 API 暴露**：
   - 在 `src/core/mod.rs` 暴露 `parse_excel_to_html` / `parse_excel_to_json` / `get_excel_sheet_list`
   - 在 `src/lib.rs` 重导出
   - 保障中文报错（`Result<T, JsValue>`）

### 阶段三：前端包生态接入（预计 2-3 天）

> **目标**：在 React / Vue 侧提供开箱即用的封装。
> **本期范围**：`types`、`react`、`vue` 三个子包。`solid`、`svelte`、`worker` 暂不适配。

1. **统一类型定义（`packages/types`）**：
   - 声明 `parseExcelToHtml` / `parseExcelToJson` / `getExcelSheetList` 函数签名
   - 定义 `PreviewOptions` / `ParsedWorkbook` / `ParsedSheet` 等接口

2. **React 封装（`packages/react`）**：
   - 新增 `useExcelPreview` Hook，管理 Web Worker（可选，防假死）和 Loading 状态
   - 新增 `<ExcelPreview url={string} />` 组件：fetch → ArrayBuffer → WASM → `dangerouslySetInnerHTML` 挂载

3. **Vue 封装（`packages/vue`）**：
   - 对等的 `useExcelPreview` Composable
   - `<ExcelPreview />` 组件（基于 `v-html` 或 ref 挂载）

4. **示例页面**：
   - 新增 `examples/excel-preview.html` 演示页面

### 阶段四：测试与性能攻坚（预计 2-3 天）

> **目标**：保证大文件不会引发页面崩溃，以及复杂合并场景下的鲁棒性。

1. **测试用例**：

   新建 `tests/test_excel_preview.rs`，在 `tests/fixtures/` 准备测试用 .xlsx 文件：

   | 用例类别 | 输入 | 断言要点 |
   | --- | --- | --- |
   | 基础解析 | 普通 3×3 数据 xlsx | 输出 HTML 包含 `<table>`、正确的 `<td>` 内容 |
   | 合并单元格 | 含横向/纵向合并 xlsx | HTML 包含正确的 `rowspan`/`colspan` |
   | 样式还原 | 含背景色/字体/边框 xlsx | 内联 `style` 属性包含对应 CSS |
   | 空文件 | 0 字节或纯空 Sheet | 返回中文 Error 而非 panic |
   | 损坏文件 | 非法二进制数据 | 返回中文 Error 而非 panic |
   | XSS 防护 | 单元格含 `<script>` | HTML 输出中已转义为 `&lt;script&gt;` |
   | Sheet 选择 | 多 Sheet xlsx + `sheetIndex` | 仅渲染指定 Sheet 的数据 |
   | 行数上限 | 1000 行 xlsx + `maxRows: 50` | 输出仅含 50 行 `<tr>` |

2. **安全测试**：
   - 补充 `test_security.rs` 中 Excel 预览相关的安全用例

3. **性能优化**：
   - 大文件性能测试（1000+ 行）
   - 海量 DOM 假死预防：行数/列数上界告警，或分块生成 HTML + `IntersectionObserver` 虚拟列表
   - 评估是否需要 Web Worker 异步解析
   - WASM 体积优化（`wasm-opt`）

4. **CI 集成**：
   - 确保 `cargo test` 全部通过（含新增用例）
   - `cargo clippy -- -D warnings` && `cargo fmt`

---

## 八、风险评估

| 风险 | 影响 | 概率 | 缓解措施 |
| --- | --- | --- | --- |
| calamine 样式 PR 未合并 | 需用 fork 或手动补充样式解析 | 中 | 使用 calamine-styles fork 过渡 |
| 主题色还原精度不足 | 颜色与原始 Excel 有偏差 | 中 | 参考 excel-demo 的 applyTint 算法，逐步迭代 |
| WASM 包体积膨胀 | 影响首屏加载 | 低 | 按需加载（动态 import），wasm-opt 压缩 |
| .xls 格式样式缺失 | 旧格式预览效果差 | 低 | 参考 excel-demo 的 .xls 补充样式策略 |
| 复杂条件格式不支持 | 部分样式丢失 | 中 | 明确为 MVP 不支持，后续迭代 |

---

## 九、资源评估

**精力分配**：

- **阶段一 + 阶段二**（60%）：底层解析 + 样式映射，是核心工作量
- **阶段三**（20%）：前端包集成
- **阶段四**（20%）：测试 + 性能优化

**总预估**：8-13 天
