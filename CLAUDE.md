# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 常用命令 (Commands)

### 开发与构建

- **启动开发服务器**: `just dev` (构建并运行 `basic-http-server`)
- **构建 WASM**: `wasm-pack build --target web` 或 `just build`
- **代码检查**: `cargo clippy -- -D warnings`
- **格式化**: `cargo fmt`
- **WASM 优化**: `wasm-opt -Oz pkg/*.wasm -o pkg/*.wasm` 或 `just optimize`

### 测试 (Tests)

- **运行所有测试**: `cargo test` (位于 `tests/` 目录)
- **运行特定测试**:
  - `cargo test --test lib_tests`: 基础 DOM 导出功能
  - `cargo test --test test_data_export`: **[核心]** 纯数据/数组导出、树形结构、复杂表头
  - `cargo test --test test_resource`: RAII 资源管理
  - `cargo test --test test_unified_api`: 统一入口 API

### 版本发布 (使用 Just)

- **CI 自动发布 (推荐)**: `just ci-release [patch|minor|major]` (自动 Tag、Push 触发 Action)
- **手动升级版本**: `just bump [patch|minor|major]`
- **发布前检查**: `just dry-run`
- **手动发布到 npm**: `just publish [tag]`

## 项目架构 (Architecture)

### 核心设计

这是一个 Rust 编写的 WebAssembly 库，用于在浏览器端导出 Excel/CSV。设计遵循 **RAII 资源管理**、**零拷贝** 和 **模块化** 原则。

### 目录结构

```
src/
├── lib.rs              # WASM 入口，仅做模块声明和重导出 (Re-exports only)
├── core/               # 核心业务逻辑
│   ├── mod.rs          # 统一 API (export_table, export_data) 和协调层
│   ├── data_export.rs  # [核心] 纯数据导出逻辑 (处理嵌套表头、树形数据、合并单元格)
│   ├── table_extractor.rs  # DOM 解析与数据提取 (支持合并单元格、隐藏行列检测)
│   ├── export_csv.rs   # CSV 格式生成
│   ├── export_xlsx.rs  # XLSX 格式生成 (支持合并单元格、公式导出、多 Sheet)
├── batch_export.rs     # CSV 异步分批处理 (针对大数据量，防止 UI 阻塞)
├── batch_export_xlsx.rs # XLSX 异步分批处理
├── validation.rs       # 安全模块：文件名与输入验证 (防止路径遍历等攻击)
├── resource.rs         # RAII 模式：UrlGuard 自动管理 Blob URL 生命周期
└── utils.rs            # 调试与辅助工具
```

### 关键数据流

1. **输入**:
   - **DOM 模式**: 用户传入 `table_id`。
   - **Data 模式**: 用户传入 JS 数据数组 (`data`) 和列配置 (`columns`)。
2. **验证**: `validation.rs` 检查文件名安全性。
3. **提取/处理**:
   - **DOM**: `table_extractor.rs` 遍历 DOM 提取数据。
   - **Data**: `data_export.rs` 处理树形结构拍平、嵌套表头解析。
4. **生成**: 根据格式调用 `export_csv` 或 `export_xlsx` 生成二进制数据 (`Vec<u8>`)。
5. **导出**: 创建 `Blob` 和 `ObjectUrl`，触发浏览器下载。
6. **清理**: `UrlGuard` 在作用域结束时自动 revoke URL。

## 核心功能模块

### 统一导出 API (src/core/mod.rs)

- **`export_table`**: DOM 导出。支持 CSV/XLSX、进度回调、隐藏行列排除。
- **`export_data`**: 纯数据导出。
  - 支持二维数组或对象数组。
  - **树形数据**: 通过 `children_key` 自动递归拍平，支持 `indent_column` 缩进。
  - **复杂表头**: 支持 `columns` 配置嵌套 children 实现多级表头。
  - **合并单元格**: 支持数据中定义 `rowSpan`/`colSpan`。
- **`export_tables_xlsx`**: 多工作表导出。将多个表格（DOM 或 Data）导出到同一个 Excel 文件的不同 Sheet。

### 表格数据提取 (src/core/table_extractor.rs)

- 支持 `display: none` 隐藏行列检测。
- 自动识别 HTML `rowspan` 和 `colspan`。
- 支持容器查找：如果 ID 是 `div`，自动查找内部的 `table`。

### 格式导出器

- **CSV**: 使用 `csv` crate，高性能，不支持合并单元格。
- **XLSX**: 使用 `rust_xlsxwriter`。
  - 支持多 Sheet。
  - 支持样式、宽高等基础配置。

## 编码规范 (Coding Guidelines)

### 语言与交流

- **中文优先**: 所有注释、文档、错误消息必须使用中文。
- **命名**: 变量/函数使用英文，注释说明用途。

### 核心约束 (Critical Constraints)

1. **模块隔离**: `lib.rs` 不含业务逻辑；核心逻辑必须在 `src/core/` 中。
2. **安全优先**:
    - 导出前必须调用 `validate_filename()`。
    - 必须使用 `Result<T, JsValue>` 处理错误，**严禁** `panic!`。
3. **RAII 资源管理**:
    - **必须**使用 `UrlGuard::new(&url)` 管理 Blob URL。
    - 禁止手动调用 `revoke_object_url`。
4. **WASM 兼容性**:
    - 导出函数必须标记 `#[wasm_bindgen]`。
    - 尽量使用引用 `&str` 传递字符串以减少拷贝。

### 常见错误速查

| 错误类型   | ❌ 错误写法                    | ✅ 正确写法                              |
|--------|---------------------------|-------------------------------------|
| **验证** | `fn export(name: String)` | `validate_filename(&name)?;`        |
| **资源** | 手动 revoke URL             | `let _guard = UrlGuard::new(&url);` |
| **错误** | `panic!("error")`         | `Err(JsValue::from_str("错误说明"))`    |
| **引用** | `String` 参数传递            | `&str` 参数传递 (WASM 边界除外)          |

## 测试指南

- **新增功能必须添加测试**：
  - DOM 相关功能添加到 `lib_tests.rs` 或 `test_unified_api.rs`。
  - 纯数据逻辑（树形、合并、表头）必须添加到 `test_data_export.rs`。
- **测试覆盖重点**：
  - 正常输入 vs 边界值 (空数据)。
  - Unicode 字符 (中文文件名、内容)。
  - 恶意文件名 (路径遍历)。
  - 树形数据的递归层级和缩进逻辑。
