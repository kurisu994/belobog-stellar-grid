# 测试说明

本目录包含 belobog-stellar-grid 项目的所有集成测试，共 **100 个测试用例**。

## 目录结构

```
tests/
├── README.md              # 本文件
├── lib_tests.rs           # 核心功能测试（41 个）⭐
├── test_data_export.rs    # 纯数据导出测试（33 个）⭐
├── test_resource.rs       # RAII 资源管理测试（8 个）
├── test_security.rs       # 安全/CSV注入测试（3 个）
├── test_unified_api.rs    # 统一 API 测试（4 个）
├── BUILD_REPORT.md        # 构建报告
├── browser/               # 浏览器环境测试（wasm-pack test）
└── fixtures/              # 测试数据文件
    └── test-page.html     # 手动功能验证页面
```

> 另有 `src/core/data_export.rs` 中的 11 个内联单元测试。

## 运行测试

```bash
# 运行所有测试（100 个）
cargo test

# 按文件运行
cargo test --test lib_tests           # 核心功能（41 个）
cargo test --test test_data_export    # 纯数据导出（33 个）
cargo test --test test_resource       # 资源管理（8 个）
cargo test --test test_security       # CSV 注入（3 个）
cargo test --test test_unified_api    # ExportFormat（4 个）

# 按名称过滤
cargo test test_flatten_tree          # 单个测试
cargo test -- --nocapture             # 显示 println 输出

# 完整检查
cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

## 测试文件说明

### lib_tests.rs（41 个）

核心功能覆盖，包括：

- **文件名处理**（7 个）：扩展名补全、空文件名、自定义文件名
- **CSV 生成**（6 个）：基础写入、特殊字符转义、空值处理
- **文件名验证**（14 个）：危险字符、路径遍历、长文件名、Unicode
- **输入验证**（4 个）：空输入、非法参数
- **边界/回归测试**（10 个）：大数据、极端输入、已修复 Bug 验证

### test_data_export.rs（33 个）

纯数据导出逻辑（不依赖 DOM），包括：

- **二维数组导出**：基本数组、空数组、Unicode 内容
- **对象数组 + columns 配置**：简单表头、嵌套表头（多级 children）
- **树形数据**：递归拍平、层级缩进、自定义 `childrenKey`
- **合并单元格**：`rowSpan`/`colSpan` 处理
- **数据格式化**：数字/布尔/null 类型转换

### test_resource.rs（8 个）

`UrlGuard` RAII 资源管理验证：

- Guard 创建与 Drop 行为
- 多 Guard 并发管理
- 空 URL / 特殊 URL 处理

### test_security.rs（3 个）

CSV 注入防护测试：`=`、`+`、`-`、`@` 等危险前缀转义。

### test_unified_api.rs（4 个）

`ExportFormat` 枚举行为验证（Csv/Xlsx 判断与默认值）。

## 测试统计

| 测试文件 | 数量 | 覆盖模块 |
|----------|------|----------|
| lib_tests.rs | 41 | CSV 生成、文件名验证、输入校验 |
| test_data_export.rs | 33 | 纯数据导出、树形结构、嵌套表头 |
| data_export.rs（内联） | 11 | 内部算法（表头解析、树形拍平） |
| test_resource.rs | 8 | UrlGuard RAII |
| test_unified_api.rs | 4 | ExportFormat 枚举 |
| test_security.rs | 3 | CSV 注入防护 |
| **合计** | **100** | |

## 命名规范

测试函数命名：`test_<模块>_<函数>_<场景>`

```rust
#[test]
fn test_flatten_tree_basic_two_level()     // 模块_函数_场景
fn test_csv_special_characters()           // 模块_场景
fn test_filename_validation_path_traversal() // 模块_函数_具体场景
```

## 注意事项

- `JsValue::from_f64()` 等在非 wasm32 会 panic，须加 `#[cfg(target_arch = "wasm32")]`
- 新增功能必须添加测试，覆盖：正常输入、边界值、Unicode、恶意输入
- 每次修改后运行 `cargo test && cargo clippy -- -D warnings`