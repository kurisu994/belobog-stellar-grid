# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 常用命令 (Commands)

### 构建与测试
- **构建 WASM**: `wasm-pack build --target web` 或 `just build`
- **运行所有测试**: `cargo test` (位于 `tests/` 目录)
- **运行特定测试**: `cargo test --test lib_tests`
- **代码检查**: `cargo clippy -- -D warnings`
- **格式化**: `cargo fmt`
- **WASM 优化**: `wasm-opt -Oz pkg/*.wasm -o pkg/*.wasm` 或 `just optimize`

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
│   ├── table_extractor.rs  # DOM 解析与数据提取 (零拷贝设计)
│   ├── export_csv.rs   # CSV 格式生成
│   └── export_xlsx.rs  # XLSX 格式生成
├── batch_export.rs     # 异步分批处理 (针对大数据量，防止 UI 阻塞)
├── validation.rs       # 安全模块：文件名与输入验证
├── resource.rs         # RAII 模式：UrlGuard 自动管理 Blob URL 生命周期
└── utils.rs            # 调试与辅助工具
```

### 关键数据流
1.  **输入**: 用户传入 HTML `table_id` 或数据。
2.  **验证**: `validation.rs` 检查文件名安全性 (防止路径遍历)。
3.  **提取**: `table_extractor.rs` 遍历 DOM 提取数据 (使用 `web-sys`)。
4.  **生成**: 根据格式调用 `export_csv` 或 `export_xlsx` 生成二进制数据 (`Vec<u8>`)。
5.  **导出**: 创建 `Blob` 和 `ObjectUrl`，触发浏览器下载。
6.  **清理**: `UrlGuard` 在作用域结束时自动 revoke URL，防止内存泄漏。

## 编码规范 (Coding Guidelines)

### 语言与交流
- **中文优先**: 所有注释、文档、错误消息必须使用中文。
- **命名**: 变量/函数使用英文，注释说明用途。

### 核心约束 (Critical Constraints)
1.  **模块隔离**: `lib.rs` 不含业务逻辑；核心逻辑必须在 `src/core/` 中。
2.  **安全优先**:
    - 导出前必须调用 `validate_filename()`。
    - 必须使用 `Result<T, JsValue>` 处理错误，**严禁** `panic!`。
3.  **RAII 资源管理**:
    - **必须**使用 `UrlGuard::new(&url)` 管理 Blob URL。
    - 禁止手动调用 `revoke_object_url`。
4.  **WASM 兼容性**:
    - 导出函数必须标记 `#[wasm_bindgen]`。
    - 尽量使用引用 `&str` 传递字符串以减少拷贝。

### 常见错误速查
| 错误类型 | ❌ 错误写法 | ✅ 正确写法 |
|---------|------------|------------|
| **验证** | `fn export(name: String)` | `validate_filename(&name)?;` |
| **资源** | 手动 revoke URL | `let _guard = UrlGuard::new(&url);` |
| **错误** | `panic!("error")` | `Err(JsValue::from_str("错误说明"))` |
| **文本** | `"File not found"` | `"未找到文件"` |

## 测试指南
- 所有测试位于 `tests/` 目录，涵盖正常输入、边界值、Unicode 字符和大数据量。
- 新功能必须添加对应的集成测试。
