# 更新日志 (CHANGELOG)

本文档记录了 belobog-stellar-grid 项目的所有重要变更。

格式遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/)，
版本号遵循 [语义化版本](https://semver.org/lang/zh-CN/)。

---

## [Unreleased]

### ✨ 新增

- 🆕 **支持排除隐藏元素**：自动检测并忽略 `display: none` 的行和列

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
