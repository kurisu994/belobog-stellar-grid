# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 常用命令 (Commands)

### 构建与测试

- **构建 WASM**: `wasm-pack build --target web` 或 `just build`
- **运行所有测试**: `cargo test` (位于 `tests/` 目录)
- **运行特定测试**: `cargo test --test lib_tests` (基础功能测试), `cargo test --test test_resource` (RAII 资源测试),
  `cargo test --test test_unified_api` (统一 API 测试)
- **代码检查**: `cargo clippy -- -D warnings`
- **格式化**: `cargo fmt`
- **WASM 优化**: `wasm-opt -Oz pkg/*.wasm -o pkg/*.wasm` 或 `just optimize`
- **运行示例**: 构建后使用 `basic-http-server .` 并访问 http://localhost:4000/examples/

### 版本发布 (使用 Just)

- **CI 自动发布 (推荐)**: `just ci-release [patch|minor|major]`
- **手动升级版本**: `just bump [patch|minor|major]`
- **发布前检查**: `just dry-run`
- **手动发布到 npm**: `just publish [latest|beta|next]`

## 项目架构 (Architecture)

### 核心设计

这是一个 Rust 编写的 WebAssembly 库，用于在浏览器端导出 Excel/CSV。设计遵循 **RAII 资源管理**、**零拷贝** 和 **模块化** 原则。

### 目录结构

```
src/
├── lib.rs              # WASM 入口，仅做模块声明和重导出 (Re-exports only)
├── core/               # 核心业务逻辑
│   ├── mod.rs          # 统一 API (export_table) 和协调层
│   ├── table_extractor.rs  # DOM 解析与数据提取 (支持合并单元格、隐藏行列检测)
│   ├── export_csv.rs   # CSV 格式生成
│   └── export_xlsx.rs  # XLSX 格式生成 (支持合并单元格、公式导出)
├── batch_export.rs     # CSV 异步分批处理 (针对大数据量，防止 UI 阻塞)
├── batch_export_xlsx.rs # XLSX 异步分批处理
├── validation.rs       # 安全模块：文件名与输入验证 (防止路径遍历等攻击)
├── resource.rs         # RAII 模式：UrlGuard 自动管理 Blob URL 生命周期
└── utils.rs            # 调试与辅助工具
```

### 关键数据流

1. **输入**: 用户传入 HTML `table_id`、文件名、格式和回调函数。
2. **验证**: `validation.rs` 检查文件名安全性 (防止路径遍历)。
3. **提取**: `table_extractor.rs` 遍历 DOM 提取数据 (使用 `web-sys`，支持合并单元格和隐藏行列检测)。
4. **生成**: 根据格式调用 `export_csv` 或 `export_xlsx` 生成二进制数据 (`Vec<u8>`)。
5. **导出**: 创建 `Blob` 和 `ObjectUrl`，触发浏览器下载。
6. **清理**: `UrlGuard` 在作用域结束时自动 revoke URL，防止内存泄漏。

## 核心功能模块

### 统一导出 API (src/core/mod.rs)

- **函数**: `export_table`, `export_data`
- **功能**:
  - `export_table`: 从 HTML 表格导出，支持 CSV/XLSX、进度回调、隐藏行列排除、合并单元格
  - `export_data`: 从 JS 数据导出，支持二维数组或对象数组（需 columns），不依赖 DOM
- **参数**:
  - `export_table`: table_id, filename, format, exclude_hidden, progress_callback
  - `export_data`: data, columns, filename, format, progress_callback
- **特点**: 智能格式检测，自动添加文件扩展名，强类型输入检查

### 表格数据提取 (src/core/table_extractor.rs)

- **extract_table_data**: 基础数据提取 (不支持合并单元格，用于 CSV)
- **extract_table_data_with_merge**: 完整数据提取 (支持合并单元格，用于 XLSX)
- **特性**:
    - 支持 `display: none` 隐藏行列检测
    - 合并单元格识别与保留
    - 零拷贝 DOM 访问

### 格式导出器

#### CSV 导出 (src/core/export_csv.rs)

- 使用 `csv` crate 生成标准 CSV 文件
- 不支持合并单元格 (CSV 格式限制)
- 高性能，适合大数据量

#### XLSX 导出 (src/core/export_xlsx.rs)

- 使用 `rust_xlsxwriter` 库
- 支持合并单元格
- 支持 Excel 公式导出
- 支持自定义样式 (当前版本基础支持)

### 分批异步导出

- **batch_export.rs**: CSV 分批导出
- **batch_export_xlsx.rs**: XLSX 分批导出
- **适用场景**: 10,000+ 行数据，避免页面卡死
- **特性**: 实时进度反馈，可配置批次大小

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
| **文本** | `"File not found"`        | `"未找到文件"`                           |

## 测试指南

- 所有测试位于 `tests/` 目录，共 47 个测试：
    - `lib_tests.rs`: 35 个基础功能测试
    - `test_resource.rs`: 8 个 RAII 资源管理测试
    - `test_unified_api.rs`: 4 个统一 API 测试
- 新功能必须添加对应的集成测试。
- 测试覆盖：正常输入、边界值、Unicode 字符、大数据量、隐藏行列、合并单元格等场景。