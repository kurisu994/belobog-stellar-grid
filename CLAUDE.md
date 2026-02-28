# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 常用命令 (Commands)

### 环境检查

- **检查开发环境**: `just check-tools` (验证所有必要工具是否安装)

### 开发与构建

- **启动开发服务器**: `just dev` (构建并运行 `basic-http-server`)
- **构建 WASM**: `just build` 或 `wasm-pack build --target web --out-dir pkg`
- **代码检查**: `cargo clippy -- -D warnings` (严格模式)
- **格式化**: `cargo fmt`
- **全面检查**: `just check` (格式化 + Lint)
- **WASM 优化**: `just optimize` (需要安装 wasm-opt)

### 测试 (Tests)

- **运行所有测试**: `cargo test` + `cd e2e && npx playwright test`
- **运行特定测试文件**:
  - `cargo test --test lib_tests`: DOM 基础功能
  - `cargo test --test test_data_export`: 纯数据/树形/合并/表头
  - `cargo test --test test_resource`: RAII 资源管理
  - `cargo test --test test_unified_api`: 统一 API
  - `cargo test --test test_security`: 安全测试（CSV 注入等）
- **按名称过滤单个测试**: `cargo test -- test_flatten_tree`
- **全面测试**: `just test`
- **修改后完整检查**: `cargo test && cargo clippy -- -D warnings && cargo fmt`

### 版本发布

- **CI 自动发布 (推荐)**: `just ci-release patch|minor|major` (自动 Tag、Push 触发 Action)
- **手动升级版本**: `just bump patch|minor|major`
- **本地完整发布流程**: `just release patch|minor|major` (包含测试和检查)
- **发布前测试**: `just dry-run` (模拟发布到 npm)
- **手动发布到 npm**: `just publish [tag]` (tag 默认为 latest)
- **查看发布信息**: `just info`

发布版本前先更新`CHANGELOG.md`文件，记录本次版本的新增功能、修复的 bug、性能优化等重要变更信息。确保每个变更都清晰描述，并按照 Keep a Changelog 的格式进行分类和排序。未发布的改动通常是在[unreleased]部分记录，发布新版本时将其移动到对应的版本标题下，并添加发布日期。

### 子包管理

- **构建子包**: `just build-packages` (构建所有 @bsg-export/* 子包)
- **发布子包**: `just publish-packages [tag]` (发布到 npm)
- **版本同步**: `just bump-core` 自动同步子包版本
- 子包使用 **pnpm** 管理依赖

## 项目架构 (Architecture)

### 工具链要求

- Rust edition 2024, 最低版本 1.85.0
- wasm-pack, basic-http-server, cargo-edit (cargo-set-version)
- 可选: wasm-opt (binaryen) 用于 WASM 体积优化

### 核心设计原则

- **RAII 资源管理**: 自动管理 Blob URL 生命周期，防止内存泄漏
- **零拷贝操作**: 参数优先使用 `&str` 引用传递，减少内存开销
- **安全性优先**: 全面的文件名验证和公式注入防护
- **性能优化**: 支持百万级数据的异步分批处理

### 两条数据通路

#### DOM 模式 (export_table)
```
table_id → table_extractor.rs (DOM 解析) → export_csv/xlsx.rs (文件生成)
```

#### Data 模式 (export_data)
```
data + columns → data_export.rs (数据处理) → export_csv/xlsx.rs (文件生成)
```

### 目录结构

```
src/
├── lib.rs              # WASM 入口，仅做模块声明和重导出 (Re-exports only)
├── validation.rs       # 安全模块：文件名与输入验证 (防止路径遍历等攻击)
├── resource.rs         # RAII 模式：UrlGuard 自动管理 Blob URL 生命周期
├── core/               # 核心业务逻辑
│   ├── mod.rs          # 统一 API (export_table, export_data, export_tables_xlsx, generate_data_bytes)
│   ├── data_export.rs  # [核心] 纯数据导出逻辑 (处理嵌套表头、树形数据、合并单元格)
│   ├── table_extractor.rs  # DOM 解析与数据提取 (支持合并单元格、隐藏行列检测)
│   ├── export_csv.rs   # CSV 格式生成
│   └── export_xlsx.rs  # XLSX 格式生成 (支持合并单元格、公式导出、多 Sheet)
├── batch_export.rs     # CSV 异步分批处理 (针对大数据量，防止 UI 阻塞)
├── batch_export_xlsx.rs # XLSX 异步分批处理
└── utils.rs            # 调试与辅助工具

packages/                # 框架封装子包 (均为 @bsg-export/ 命名空间)
├── types/              # 严格 TypeScript 类型定义（零运行时）
├── react/              # React Hook + 组件
├── vue/                # Vue 3 Composable + 组件
├── svelte/             # Svelte Store 封装（兼容 Svelte 4/5）
├── solid/              # Solid.js Primitive + 组件
└── worker/             # Web Worker 导出封装
```

### 关键模块职责

#### 统一入口 (src/core/mod.rs)
- **export_table**: DOM 导出，支持 CSV/XLSX、进度回调、隐藏行列排除
- **export_data**: 纯数据导出，支持二维数组、对象数组、树形数据、复杂表头
- **export_tables_xlsx**: 多工作表导出，将多个表格导出到同一个 Excel 文件的不同 Sheet
- **generate_data_bytes**: 与 export_data 相同，但返回文件字节（Uint8Array）而不触发下载，专为 Worker 场景设计

#### 核心算法 (src/core/data_export.rs)
- **嵌套表头解析**: 支持配置嵌套的 children 实现多级表头
- **树形数据处理**: 通过 children_key 自动递归拍平，支持 indent_column 缩进
- **合并单元格**: 支持数据中定义 rowSpan/colSpan 属性
- **安全限制**: MAX_DEPTH=64 防止深层嵌套导致栈溢出，MAX_HEADER_CELLS=100_000 防止 OOM

#### DOM 提取 (src/core/table_extractor.rs)
- **隐藏行列检测**: 支持 display: none 的隐藏检测
- **合并单元格识别**: 自动识别 HTML rowspan 和 colspan
- **容器查找**: 如果 ID 是 div，自动查找内部的 table

#### 格式导出器
- **CSV**: 使用 csv crate，高性能，不支持合并单元格，自动转义公式注入字符
- **XLSX**: 使用 rust_xlsxwriter，支持多 Sheet、合并单元格、公式防护（统一使用 write_string）

### 资源管理

```rust
// RAII 模式确保资源正确释放
let url = Url::create_object_url_with_blob(&blob)?;
let _guard = UrlGuard::new(&url); // 作用域结束自动 revoke
```

## 编码规范 (Coding Guidelines)

### 核心约束

1. **模块隔离**: `lib.rs` 仅做模块声明和重导出，不含业务逻辑；核心逻辑必须在 `src/core/` 中。
2. **安全优先**:
    - 导出前必须调用 `validate_filename()`。
    - 必须使用 `Result<T, JsValue>` 处理错误，**严禁** `panic!` 或 `unwrap()`。
3. **RAII 资源管理**:
    - **必须**使用 `UrlGuard::new(&url)` 管理 Blob URL。
    - 禁止手动调用 `revoke_object_url`。
4. **WASM 兼容性**:
    - 导出函数必须标记 `#[wasm_bindgen]`。
    - 尽量使用引用 `&str` 传递字符串以减少拷贝。
5. **错误处理**: 错误信息必须为中文，使用 `Err(JsValue::from_str("错误说明"))`。
6. **测试约束**: 使用 JsValue 的代码需要加 `#[cfg(target_arch = "wasm32")]` 标记。

### 常见错误速查

| 错误类型   | 错误写法                    | 正确写法                              |
|--------|---------------------------|-------------------------------------|
| **验证** | `fn export(name: String)` | `validate_filename(&name)?;`        |
| **资源** | 手动 revoke URL             | `let _guard = UrlGuard::new(&url);` |
| **错误** | `panic!("error")`         | `Err(JsValue::from_str("错误说明"))`    |
| **引用** | `String` 参数传递            | `&str` 参数传递 (WASM 边界除外)          |
| **测试** | 无 cfg 属性                  | `#[cfg(target_arch = "wasm32")]`      |

### 函数规范

```rust
#[wasm_bindgen]
pub fn example_function(param: &str) -> Result<(), JsValue> {
    // 输入验证
    if param.is_empty() {
        return Err(JsValue::from_str("参数不能为空"));
    }

    // 业务逻辑

    Ok(())
}
```

### 提交规范

**所有 commit message 必须使用中文**，格式如下：

```
[表情] [类型](范围): 主题描述

示例：
🚀 feat(导出): 添加冻结窗格功能
♻️ refactor(export): 简化冻结窗格配置逻辑
🐛 fix(csv): 修复公式注入转义问题
```

**类型**: feat / fix / docs / style / refactor / perf / test / chore

## 测试指南

### 测试文件对应关系

| 测试文件               | 测试内容                     |
|------------------------|-----------------------------|
| lib_tests.rs           | DOM 基础功能                 |
| test_resource.rs       | RAII 资源管理                |
| test_unified_api.rs    | 统一 API 接口                |
| test_data_export.rs    | 纯数据/树形/合并/表头         |
| test_security.rs       | 安全测试 (CSV 注入等)         |

### 新增功能测试要求

- **DOM 相关功能**: 添加到 `lib_tests.rs` 或 `test_unified_api.rs`
- **纯数据逻辑**: 必须添加到 `test_data_export.rs`
- **安全功能**: 添加到 `test_security.rs`

### 测试命名规范

```rust
#[test]
fn test_<模块>_<函数>_<场景>() {
    // 测试代码
}
```
