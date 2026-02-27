# 更新日志 (CHANGELOG)

本文档记录了 belobog-stellar-grid 项目的所有重要变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

## [Unreleased]

---

## [1.0.9] - 2026-02-27

### ✨ 新增

- ❄️ **XLSX 冻结窗格 (Freeze Panes)**: 支持冻结表头行/列，方便大数据量 Excel 浏览。
  - DOM 模式自动检测 `<thead>` 行数，导出时自动冻结表头
  - Data 模式自动根据 `columns` 表头深度冻结
  - `export_data` / `generate_data_bytes` 新增 `freezeRows` / `freezeCols` 可选配置，支持用户自定义冻结位置
  - 所有 XLSX 导出路径均支持：同步单表/多表、异步分批单表/多表
  - TypeScript 类型定义 `ExportDataOptions` 新增 `freezeRows?` / `freezeCols?` 字段

## [1.0.8] - 2026-02-26

### ✨ 新增

- 🚀 **Web Worker 导出支持**: 新增 `@bsg-export/worker` 子包，将导出计算移至 Worker 线程，彻底避免大数据量导出时主线程阻塞。
  - 新增 `generate_data_bytes` WASM 函数：生成 CSV/XLSX 文件字节（不触发下载），专为 Worker 场景设计
  - `ExportWorker` 类：管理 Worker 生命周期、消息协议和文件下载触发
  - 支持 Transferable 零拷贝字节传输和进度回调
- 🧩 **框架 Worker 集成**: `@bsg-export/react` 和 `@bsg-export/vue` 新增 `useWorkerExporter` Hook/Composable
  - 接受 Worker 工厂函数，自动管理 Worker 生命周期和 WASM 初始化
  - 提供 `exportData`（生成并下载）和 `generateBytes`（仅生成字节）方法

### ♻️ 重构

- ♻️ **导出核心拆分**: 将 CSV/XLSX 的"文件生成"与"触发下载"解耦：
  - `export_csv.rs`: 提取 `generate_csv_bytes()` 纯字节生成函数，BOM 处理内聚
  - `export_xlsx.rs`: 提取 `generate_xlsx_bytes()` / `generate_xlsx_multi_bytes()` 纯字节生成函数
  - 原 `export_as_csv()` / `export_as_xlsx()` 复用新函数后触发下载，行为不变


## [1.0.7] - 2026-02-26

### 🐛 修复

- 🐛 **React / Vue 导出生命周期修复**: 将 `exportTable` 等导出动作从 `void` 变更为返回执行结果 `boolean` / `Promise<boolean>` 状态指示，确保能在完整的尝试逻辑后，且由于并未返回错误时，准确触发 `onExportSuccess` / `emit('success')` 回调。
- 🐛 **Vue 组件事件补充**: 修复 Vue `<ExportButton>` 中并未正确对外暴露进度信息的疏漏，新增对内部 `progress` 的监听映射并派发 `progress` 事件。
- 🐛 **WASM 初始化失败不可重试**: 修复 React/Vue `initWasm()` 中 WASM 加载失败后 `wasmInitPromise` 永久持有 rejected Promise 的问题。失败时重置 Promise 缓存，允许后续调用重新触发初始化。
- 🐛 **Blob URL 下载竞态**: 移除 `UrlGuard` RAII 即时释放策略，改用 `schedule_url_revoke` 通过 `setTimeout(10s)` 延迟释放 Blob URL，避免 `anchor.click()` 后立即 revoke 导致下载失败。
- 🔧 **类型接口聚合与唯一化**: 清理重构并移除了 `@bsg-export/react` 及 `@bsg-export/vue` 中的重复入口参数选项接口（如 `ExportTableOptions`），并悉数汇总于 `@bsg-export/types` 内部，完美解决跨包定义引用的问题。
- 📝 **Rust 内部细节优化**: 追加补齐了 `export_data` 对 `strictProgressCallback` 设置项的 JSDoc 注释，在计算单元格合并坐标时优化去除了无必要的防御性溢出判断。

### ✨ 新增

- 🆕 **`strict_progress_callback` 参数完善**: 为以下 4 个 API 新增 `strict_progress_callback` 可选参数，与 `export_table` 保持 API 一致性：
  - `export_tables_xlsx` — 多工作表同步导出
  - `export_table_to_csv_batch` — 分批异步 CSV 导出
  - `export_table_to_xlsx_batch` — 分批异步 XLSX 导出
  - `export_tables_to_xlsx_batch` — 多工作表分批异步导出
- 🆕 **TS 类型补全**: 为 `ExportDataOptions`、`ExportTablesXlsxOptions`、`ExportCsvBatchOptions`、`ExportXlsxBatchOptions`、`ExportTablesBatchOptions` 接口和对应函数签名添加 `strictProgressCallback` 字段

---

## [1.0.6] - 2026-02-26

### ✨ 新增

- 🆕 **框架集成库**: 提供 React 和 Vue 3 官方封装组件
  - `@bsg-export/react`：`useExporter()` Hook + `<ExportButton>` 组件，自动管理 WASM 初始化、导出状态和进度
  - `@bsg-export/vue`：`useExporter()` Composable + `<ExportButton>` 组件，ref 响应式状态 + 插槽支持
  - 所有导出方法均类型安全，参数使用 Options 对象模式
- 🆕 **严格 TypeScript 类型定义**: `@bsg-export/types`
  - 替代 wasm-bindgen 自动生成的 `any` 类型
  - 提供 `Column`、`ExportDataOptions`、`SheetConfig`、`BatchSheetConfig`、`MergeCellValue` 等核心接口
  - 所有 6 个导出函数的类型安全签名重新声明

### 🔧 构建和发布流程

- 📦 **子包版本同步**: `bump-core` 升级 `Cargo.toml` 后自动同步 `packages/*/package.json` 版本
- 📦 **子包构建发布命令**: `just build-packages` / `just publish-packages`
- 📦 **CI 集成发布**: `publish-npm` job 主包发布后自动构建并发布三个子包

### 🛠️ 重构

- 🔧 **DOM 提取逻辑重构**: 消除 DOM 遍历代码的重复逻辑
  - 提取 `resolve_table()` 函数：封装 window → document → getElementById → find_table_element 流程
  - 提取 `get_table_row()` 辅助函数：封装行元素获取和类型转换
  - 提取 `process_row_cells()` 函数：封装完整的单元格迭代循环（tracker pop、隐藏检测、span 处理、colspan 填充）
  - 新增 `RowProcessResult` 结构体：统一返回行数据和单元格合并信息
  - 新增 `compute_merge_ranges()` 辅助函数：XLSX 合并区域计算
  - 新增 `count_visible_rows()` 辅助函数：rowspan 可见行计数
  - 重构 `extract_table_data`：从 ~80 行减少到 ~20 行
  - 重构 `extract_table_data_with_merge`：从 ~120 行减少到 ~30 行
  - 重构 `batch_export.rs`：移除 ~70 行重复 DOM 遍历代码
  - 重构 `batch_export_xlsx.rs`：移除 ~80 行重复代码，新增 `count_visible_rows_cross_source` 跨数据源辅助函数

### ✅ 测试

- 🧪 **E2E 测试**: 引入 Playwright 进行端到端测试
  - 新增 `e2e/` 目录，包含 Playwright 配置和测试用例
  - 基础导出测试（10 个）：WASM 初始化、CSV 基础/自动扩展名/默认文件名/BOM、XLSX 基础/合并单元格、容器支持、隐藏行排除
  - 数据导出测试（10 个）：二维数组 CSV/XLSX、对象数组 columns 配置、嵌套表头、树形数据、错误处理（5 种场景）、进度回调
  - 测试页面拦截下载，通过 DOM 捕获导出结果验证

## [1.0.5] - 2026-02-13

### 📝 文档优化

- 📚 **API 文档完善**：明确所有函数参数类型和属性
- 📋 **导出函数文档**：移除导出函数参数描述中的多余空行
- 📝 **CI 文档补充**：补充 `ci-release` 命令的用法说明，明确 `minor` 和 `major` 作为 `level` 参数的合法选项
- 🎨 **项目规则优化**：优化项目规则文件（.agent/rules/project-rules.md、.cursor/rules/projectrules.mdc、.kiro/steering/project-rules.md、.trae/rules/project_rules.md）和 CLAUDE.md 文档

### 🔧 构建和发布流程

- 🔧 **Justfile 重构**：重构并增强构建和发布工作流
  - 重构 justfile 文件结构，添加清晰的章节注释和工具检查
  - 将版本升级逻辑提取为独立配方，改进跨平台兼容性
  - 优化发布流程，分离本地发布和 CI 发布的不同路径
  - 添加代码质量检查、测试和优化步骤作为发布前置条件
  - 增强错误处理和用户确认机制，提升开发体验
- 🔧 **Check 命令完善**：完善 check 命令，包含测试
- 📦 **发布流程增强**：添加代码质量检查和版本号自动同步

### 🎯 核心功能优化

- ✨ **严格进度回调**：添加严格进度回调功能并增强错误处理
- 📝 **API 设计优化**：重构导出函数参数描述，提高文档清晰度

### 📦 发布信息

- 📝 **版本号更新**：升级到 1.0.5 版本

## [1.0.4] - 2026-02-13

### ✨ 新增

- 🆕 **CSV BOM 支持**（CODE_REVIEW #10）
  - `export_table` 和 `export_data` 新增 `with_bom` 选项
  - 为 CSV 文件添加 UTF-8 BOM 头，彻底解决 Windows Excel 打开中文 CSV 乱码问题
- 🆕 **严格进度回调模式** `strict_progress_callback` / `strictProgressCallback`
  - `export_table` 新增第 7 个参数 `strict_progress_callback`
  - `export_data` options 新增 `strictProgressCallback` 字段
  - 启用后进度回调失败将中止导出并返回错误，适用于需要精确控制导出流程的场景

### 💥 破坏性变更

- **`export_data` API 参数重构**（#10）
  - 旧签名：`export_data(data, columns?, filename?, format?, callback?, indent_column?, children_key?)`（7 个参数）
  - 新签名：`export_data(data, options?)`（2 个参数）
  - `options` 为 JS 对象，字段：`columns`、`filename`、`format`、`progressCallback`、`indentColumn`、`childrenKey`
  - 所有 JS 调用端需要更新为 options 对象模式

### 🛡️ 安全性

- 🔒 **公式注入防护 (Formula Injection Protection)**
  - CSV 导出：自动转义以 `=`, `+`, `-`, `@` 开头的单元格内容（添加 `'` 前缀），防止电子表格软件自动执行恶意代码
  - Excel 导出：强制使用文本格式写入单元格，禁用自动公式解析
- 🔒 **修复 XLSX 公式注入安全漏洞**（CODE_REVIEW #2, #3）
  - `export_as_xlsx()` 不再将 `=` 开头的内容作为公式执行，统一使用 `write_string`
  - 所有 XLSX 导出路径（同步、异步、单表、多表）行为一致
- 🔒 **文件名验证增强**（CODE_REVIEW #13）
  - 新增 ASCII 控制字符（0x00-0x1F）检测，防止恶意文件名
- 🔒 **树形/表头递归深度限制**（CODE_REVIEW #6）
  - `flatten_tree_data()` 和 `parse_columns()` 添加 `MAX_DEPTH=64` 限制
  - 超过限制时返回友好的中文错误提示
- 🔒 **数组类型验证**（CODE_REVIEW #8）
  - `parse_sheet_configs()` 和 `parse_batch_sheet_configs()` 新增 `Array::is_array()` 验证
  - 防止非数组值被静默包装为单元素数组
- 🔒 **CSV 注入防护增强**（CODE_REVIEW #4）
  - 在 `escape_csv_injection` 中新增了对制表符 `\t` 的转义支持，防止特殊格式的注入攻击
- 🔒 **XLSX 列索引溢出防护**（CODE_REVIEW #5）
  - 添加了最大列数检查（16383），防止超出 Excel 限制导致的各种问题

### 🐛 修复

- 修复 `yield_to_browser()` 中使用 `expect()` 违反 no-panic 约束的问题（CODE_REVIEW #1）
  - 将 `window` 获取移到 `Promise::new()` 闭包外部，使用 `?` 操作符优雅处理错误
- 修复 `js_value_to_string()` 冗余 fallback 导致 Symbol/BigInt 类型静默丢失的问题（CODE_REVIEW #9）
  - 改为使用 `format!("{:?}", val)` 输出未知类型的 Debug 表示
- 修复 `batch_export_xlsx` 分批导出时，隐藏行导致的合并单元格范围计算错误
  - 修正了 `MergeRange` 逻辑，现在能正确跳过隐藏行计算 `rowspan` 覆盖范围
- 修复 `table_extractor` DOM 提取时的隐藏行合并单元格范围计算错误
- 修复 `UrlGuard` 资源释放失败时的错误日志被静默丢弃的问题（改用 `console.error`）
- 修复文件名长度验证逻辑（从字节长度改为字符长度，更好支持中文文件名）
- 修复浮点数转整数时的边界精度丢失风险
- **删除 `export_data_impl` 中的死代码分支**（CODE_REVIEW #6 建议改进）
  - `parse_export_data_options` 已过滤 null/undefined 的 columns，`cols.is_null() || cols.is_undefined()` 永远不会执行
- 🔧 **修复 `Reflect::get` 异常被静默吞掉**（CODE_REVIEW2 §4.1）
  - `extract_data_rows()` 和 `flatten_tree_data()` 中 `Reflect::get` 的 `Err` 不再用 `unwrap_or(JsValue::NULL)` 丢弃
  - 新增 `get_object_property()` 辅助函数，区分"字段不存在"和"getter 异常"，后者传播为错误
- 🔧 **修复进度回调失败策略不一致**（CODE_REVIEW2 §4.2）
  - 统一使用 `report_progress()` 处理所有进度回调，默认模式下失败仅 `console.warn`
  - 新增 `strict_progress_callback` / `strictProgressCallback` 选项，启用后回调失败将中止导出
- 🔧 **修复 tbodyId 误用缺少运行时防护**（CODE_REVIEW2 §4.3）
  - 新增 `ensure_external_tbody()` 运行时校验，确保传入的 tbody 元素存在于目标 table 内部
  - `export_table_to_csv_batch` 和 `export_table_to_xlsx_batch` 在批量导出前执行验证
- 🔧 **修复 format 非法值静默回落**（CODE_REVIEW2 §4.4）
  - `export_data` 的 `format` 参数不再对非 0/1 值静默回落为 CSV
  - 非法 format 值（如 2、字符串等）现在返回明确的中文错误提示

### ⚡ 优化

- **`Format::new()` 提升到循环外部**（CODE_REVIEW #12）：多工作表导出时不再每个 sheet 重复创建 `Format` 实例
- **`dangerous_chars` 迭代优化**（#12）：将 `for` 循环改为函数式 `find()` 写法，代码更简洁
- **`table_extractor` 性能优化**（#14）：CSV 路径下 `extract_table_data()` 独立实现，跳过 `merge_ranges` 计算和内存分配，减少不必要开销
- 移除不再维护的 `wee_alloc` 分配器，改用 Rust 默认分配器（更安全、现代）
- 清理冗余代码和未使用的导入
- 消除 `test_data_export.rs` 中的编译警告
- **错误日志记录**：在进度回调执行失败时，现在会通过 `console.warn` 记录错误，而不是静默失败（CODE_REVIEW #9）
- **代码重构**（CODE_REVIEW #7, #13, #15, #16, #17）
  - 重构了 `batch_export_xlsx.rs` 消除重复代码
  - 优化了 DOM 提取逻辑
  - 移除了多余的 `unwrap_or`
  - 修复了 `validate_filename` 的可见性问题

### 📝 文档

- 更新 `examples/README.md`：修复过时的 API 引用，补充 `multi-sheet-export.html` 示例文档
- 更新 `README.md`：添加多工作表示例、更新测试计数（100→103）、版本徽章（1.0.3→1.0.4）
- 更新 `README.md`：Rust 版本要求从 1.82+ 更正为 1.85.0+（edition 2024 需要）
- 更新 `API.md`：修正 `export_table` 签名（新增 `with_bom`、`strict_progress_callback`）、`export_data` options（新增 `withBom`、`strictProgressCallback`、format 严格验证说明）、批量导出函数签名（补全 `tbody_id`、`exclude_hidden`、`with_bom` 参数）
- 更新 `Cargo.toml`：添加 `rust-version = "1.85.0"` 字段
- 更新 `tests/BUILD_REPORT.md`：修正过时的 API 函数名和版本号
- 更新 `tests/README.md`：测试数量 100→103，补充 `utils.rs` 和 `validation.rs` 内联测试说明
- 修正 `core/mod.rs` 和 `batch_export.rs` 文档注释中过时的包名引用（`excel_exporter.js` → `belobog_stellar_grid.js`）
- 修复 `basic-export.html` 中 `export_table` 使用字符串 `'csv'` 而非 `ExportFormat.Csv` 的问题
- 更新 `CODE_REVIEW.md`：第三轮全面代码审查，新增 17 个发现（含 2 个严重问题）

## [1.0.3] - 2026-02-12

### ✨ 新增

- 🆕 **直接数据导出 (Direct Data Export)**: `export_data()`
  - 支持不依赖 DOM，直接将 JavaScript 二维数组导出为 CSV 或 Excel 文件
  - 支持 columns + dataSource 模式（Ant Design 风格表头配置）
  - 支持嵌套表头（多级分组列，自动生成合并单元格）
  - 支持数据合并单元格（`{ value, colSpan?, rowSpan? }` 格式）
  - 支持树形数据导出（传入 `children_key` 参数启用，递归拍平 + 层级缩进）
  - 适用于纯数据导出、树形结构（组织架构、商品分类等）场景

- 🆕 **容器元素支持**
  - 导出函数现在支持传入容器元素 ID（如 `div`），会自动查找内部的 `table` 元素
  - 完美兼容 Ant Design、Element Plus 等 UI 组件库的表格结构
  - 涉及所有导出 API：`export_table`, `export_table_to_csv_batch`, `export_table_to_xlsx_batch`

### 🛡️ 优化

- **错误提示优化**：针对 `export_data` 误用场景（传入对象数组但未提供 `columns`）增加了友好的错误提示，明确区分二维数组模式和对象数组模式。

### 📝 示例

- 新增 `array-export.html`：数组直接导出示例（7 个场景：基础、columns、嵌套表头、3 级表头、数据合并、大数据、错误处理）
- 新增 `container-export.html`：容器元素导出示例（Ant Design 表格结构）
- 新增 `tree-export.html`：树形数据导出示例（组织架构、商品分类、嵌套表头+树形、大数据量）

## [1.0.2] - 2026-02-11

### ✨ 新增

- 🆕 **多工作表导出 (Multi-sheet Export)**
  - 支持将多个 HTML 表格导出到同一个 Excel 文件的不同工作表中
  - 新增同步 API `export_tables_xlsx()`
  - 新增异步分批 API `export_tables_to_xlsx_batch()`（支持大数据量）
  - 支持自定义工作表名称和排除隐藏行列配置

- 🆕 **支持排除隐藏元素**：自动检测并忽略 `display: none` 的行和列

### 🐛 修复

- 修复合并单元格（colspan/rowspan）导出到 Excel 后内容为空的问题
  - `merge_range()` 会覆盖首单元格已写入的内容，改为传入实际文本值
  - 同时影响单表导出 `export_as_xlsx` 和多表导出 `export_as_xlsx_multi`

### 📝 示例

- 多工作表导出示例页面新增表头合并（colspan + rowspan 两级表头）和表体 colspan 测试数据

## [1.0.1] - 2026-02-02

### ✨ 新增

- 🆕 **XLSX 分批异步导出**：`export_table_to_xlsx_batch()`
  - 采用两阶段策略：分批读取 DOM (80%) + 同步生成 XLSX (20%)
  - 支持百万级数据 Excel 导出，保持页面响应性
  - 与 CSV 批量导出保持一致的 API 设计

### 📝 文档改进

- 📋 **README 添加 TODO 路线图**：规划了 16 项待改进功能
  - 数据类型处理（自动检测、数值单元格、公式支持）
  - 样式与格式化（字体颜色、列宽调整、表头样式）
  - 表格结构（合并单元格、多工作表、隐藏行列检测）
  - 数据选择与过滤（选择性导出、数据预处理、列名映射）
  - 其他（JS 数组生成、BOM 头选项、服务端探索）

- 📖 **examples/README.md 格式优化**
  - 修复 Markdown 格式问题
  - 统一代码风格（引号、缩进）
  - 补充代码块前后的空行

### 🔧 内部改进

- 新增 `batch_export_xlsx.rs` 模块
- 测试用例增加至 47 个

---

## [1.0.0] - 2025-12-16

### 🎉 首次正式发布

#### 核心功能

- ✅ **统一导出 API**：`export_table(table_id, filename, format, progress_callback)`
  - 支持 CSV 和 XLSX 格式导出
  - 可选的实时进度反馈（0-100）
  - 类型安全的 `ExportFormat` 枚举

- 🔒 **安全优先**
  - 完整的文件名验证（路径分隔符、危险字符、Windows 保留名称、长度限制）
  - RAII 资源管理，自动清理内存
  - 全面的错误处理，中文错误消息

- ⚡ **高性能**
  - 分批异步处理，支持百万级数据导出
  - 进度回调每10行更新，避免性能开销
  - 浏览器控制权让出机制，保持页面响应

#### 架构设计

- 📦 **模块化结构**
  - `core/mod.rs` - 核心协调模块
  - `core/export_csv.rs` - CSV 导出模块
  - `core/export_xlsx.rs` - Excel 导出模块
  - `core/table_extractor.rs` - 表格数据提取模块
  - `validation.rs` - 文件名验证模块
  - `resource.rs` - RAII 资源管理模块
  - `batch_export.rs` - 异步批量导出模块

- ✨ **职责分离**：每个模块单一职责，便于维护和扩展

#### 示例文件

- ✅ `basic-export.html` - 基础导出示例
- ✅ `advanced-features.html` - 高级功能示例
- ✅ `progress-export.html` - 进度导出示例（带进度条和实时反馈）
- ✅ `virtual-scroll-export.html` - 虚拟滚动示例

#### 测试覆盖

- 🧪 33 个全面的单元测试
- 📊 测试覆盖率 100%
- 📝 完整的测试文档

#### 技术栈

- Rust Edition 2024
- wasm-bindgen 0.2.106
- web-sys 0.3.83
- js-sys 0.3.83
- csv 1.4.0
- rust_xlsxwriter 0.69.0

#### 性能数据

- **小数据**（1,000 行）：~0.1s
- **中等数据**（10,000 行）：~1.2s（页面保持响应）
- **大数据**（100,000 行）：~12s（流畅导出）
- **超大数据**（1,000,000 行）：~120s（完全可用）
