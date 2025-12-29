# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 核心约束（必须遵守）

### 1. 模块化原则

```
src/
├── lib.rs              # 仅做模块声明和重导出
├── validation.rs       # 文件名验证
├── resource.rs         # 资源管理（RAII）
├── core/               # 核心导出模块组
│   ├── mod.rs          # 统一 API 和协调
│   ├── table_extractor.rs  # 表格数据提取
│   ├── export_csv.rs   # CSV 导出
│   └── export_xlsx.rs  # XLSX 导出
├── batch_export.rs     # 异步分批导出
└── utils.rs            # 调试工具
```

**禁止**：❌ 在 `lib.rs` 中添加业务逻辑 | ❌ 跨模块混合职责 | ❌ 绕过模块边界访问

### 2. 安全优先

- ✅ 文件名必须通过 `validate_filename()` 验证
- ✅ DOM 操作必须检查返回值
- ✅ 使用 `Result<T, JsValue>` 而非 `panic!`
- ✅ 中文错误消息

### 3. RAII 资源管理

```rust
// ✅ 正确：自动管理
let _url_guard = UrlGuard::new(&url);

// ❌ 错误：手动管理（异常时泄漏）
let url = create_object_url();
Url::revoke_object_url(&url);
```

### 4. 零拷贝原则

```rust
// ✅ 使用引用
fn process_data(data: &str) { }

// ❌ 获取所有权
fn process_data(data: String) { }
```

---

## 代码修改规范

### 添加新功能

1. **确定模块**：导出 → `core/`，数据提取 → `core/table_extractor.rs`，验证 → `validation.rs`
2. **编写函数**：必须包含中文文档注释和 `#[wasm_bindgen]`
3. **在 `lib.rs` 重导出**：`pub use module::new_function;`
4. **添加测试**

### 修改前检查

- [ ] 是否破坏向后兼容？需要 `#[deprecated]`
- [ ] 错误处理完整（中文）
- [ ] 使用 RAII 管理资源
- [ ] 无 `panic!` / `unwrap()`
- [ ] 添加测试

---

## 测试要求

```bash
# 运行所有测试（47个，都在 tests/ 目录）
cargo test

# 运行单个测试文件
cargo test --test lib_tests
cargo test --test test_resource
cargo test --test test_unified_api

# 运行 src/ 下的测试（当前为 0 个）
cargo test --lib

# 检查和格式化
cargo clippy -- -D warnings
cargo fmt
```

**必须测试**：正常输入、边界值、非法输入、Unicode、大数据

---

## 常见错误速查

| 错误类型       | ❌ 错误                           | ✅ 正确                          |
| -------------- | --------------------------------- | -------------------------------- |
| **忘记验证**   | `pub fn export(filename: String)` | `validate_filename(&filename)?;` |
| **资源泄漏**   | 手动 `revoke_object_url()`        | 使用 `UrlGuard`                  |
| **使用 panic** | `panic!("数据为空")`              | `Err(JsValue::from_str("..."))`  |
| **英文错误**   | `"File not found"`                | `"未找到文件"`                   |

---

## 快速参考

### 技术栈

- Rust Edition 2024 + WebAssembly
- wasm-bindgen, web-sys, csv, rust_xlsxwriter

### 核心 API（推荐使用）

```rust
#[wasm_bindgen]
pub fn export_table(
    table_id: &str,
    filename: Option<String>,
    format: Option<ExportFormat>,  // Csv | Xlsx
    progress_callback: Option<js_sys::Function>,
) -> Result<(), JsValue>
```

**JavaScript 示例**：

```javascript
import init, { export_table, ExportFormat } from "./pkg/belobog_stellar_grid.js";
await init();

export_table("my-table"); // 最简单
export_table("my-table", "报表.xlsx", ExportFormat.Xlsx); // 指定格式
export_table("my-table", "数据", ExportFormat.Csv, (p) => {
  // 带进度
  console.log(`进度: ${p.toFixed(1)}%`);
});
```

### 重要路径

- 核心 API: `src/core/mod.rs`
- 数据提取: `src/core/table_extractor.rs`
- CSV/Excel 导出: `src/core/export_csv.rs`, `src/core/export_xlsx.rs`
- 异步分批: `src/batch_export.rs`
- 验证/资源: `src/validation.rs`, `src/resource.rs`
- 测试: `tests/lib_tests.rs` (35 个), `tests/test_resource.rs` (8 个), `tests/test_unified_api.rs` (4 个)

### 构建和优化

```bash
wasm-pack build --target web   # 构建
./build.sh                      # 一键构建（推荐）
wasm-opt -Oz pkg/*.wasm -o pkg/*.wasm  # 优化（可选）
```

---

## 关键设计原则

1. **简洁至上** - 最简单的解决方案
2. **安全第一** - 验证输入、优雅错误处理、中文消息
3. **模块化** - 清晰的职责分离
4. **性能优化** - 零拷贝 + 分批异步 + RAII
