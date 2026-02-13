# 更新日志 (CHANGELOG)

本文档记录了 belobog-stellar-grid 项目的所有重要变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

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
