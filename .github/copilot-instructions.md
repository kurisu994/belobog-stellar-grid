# Copilot Instructions — belobog-stellar-grid

Rust WebAssembly 库，在浏览器端将 HTML 表格或 JS 数据导出为 CSV/XLSX 文件。

**所有注释、文档、错误消息必须使用中文。** 变量/函数名使用英文。

## 命令

```bash
# 构建
wasm-pack build --target web          # 或 just build

# 测试
cargo test                            # 全部（97 个）
cargo test --test lib_tests           # DOM 基础功能
cargo test --test test_data_export    # 纯数据/树形/合并/表头
cargo test --test test_resource       # RAII 资源管理
cargo test --test test_unified_api    # 统一 API
cargo test -- test_flatten_tree       # 按名称过滤单个测试

# 检查
cargo clippy -- -D warnings
cargo fmt

# 发布
just ci-release patch|minor|major     # CI 自动发布（推荐）
```

## 架构

`lib.rs` 仅做模块声明和重导出，不含业务逻辑。

两条数据通路：
- **DOM 模式** (`export_table`): `table_extractor.rs` 解析 HTML → `export_csv/xlsx` 生成文件
- **Data 模式** (`export_data`): `data_export.rs` 处理 JS 数据（二维数组 / 对象+列配置 / 树形结构 / 合并单元格）→ 同上

关键模块：
- `src/core/mod.rs` — 统一入口：`export_table`、`export_data`、`export_tables_xlsx`
- `src/core/data_export.rs` — 核心算法：嵌套表头解析、树形拍平、合并单元格处理
- `src/core/table_extractor.rs` — DOM 提取，含 `find_table_element()` 容器查找
- `src/core/export_csv.rs` / `export_xlsx.rs` — 格式生成器
- `src/batch_export.rs` / `batch_export_xlsx.rs` — 大数据量异步分批导出
- `src/validation.rs` — 文件名安全验证
- `src/resource.rs` — `UrlGuard` RAII 管理 Blob URL

## 约束

1. **错误处理**：必须用 `Result<T, JsValue>`，严禁 `panic!` / `unwrap()`
2. **文件名验证**：导出前必须调用 `validate_filename()`
3. **资源管理**：Blob URL 必须用 `let _guard = UrlGuard::new(&url);`，禁止手动 revoke
4. **零拷贝**：参数优先 `&str`，WASM 边界除外
5. **中文错误**：`Err(JsValue::from_str("未找到文件"))` 而非英文
6. **新函数**：标记 `#[wasm_bindgen]`，在 `lib.rs` 中重导出，附中文文档注释
7. **wasm32 测试**：`JsValue::from_f64()` 等在非 wasm32 会 panic，须加 `#[cfg(target_arch = "wasm32")]`

## 测试

测试文件在 `tests/` 目录。命名规范：`test_<模块>_<函数>_<场景>`。

新功能必须添加测试，覆盖：正常输入、边界值、Unicode、恶意输入。每次修改后运行 `cargo test && cargo clippy -- -D warnings && cargo fmt`。
