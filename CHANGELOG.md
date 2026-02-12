# 更新日志 (CHANGELOG)

本文档记录了 belobog-stellar-grid 项目的所有重要变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

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
