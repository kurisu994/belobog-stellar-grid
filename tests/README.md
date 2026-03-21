# 测试说明

本目录包含 belobog-stellar-grid 项目的所有集成测试，共 **190 个测试用例**（`cargo test` 环境下）。

## 目录结构

```
tests/
├── README.md                  # 本文件
├── lib_tests.rs               # 核心功能测试（70 个）⭐
├── test_data_export.rs        # 纯数据导出测试（41 个）⭐
├── test_streaming_export.rs   # 流式导出测试（34 个）
├── test_excel_preview.rs      # Excel 预览解析测试（26 个）
├── test_resource.rs           # RAII 资源管理测试（8 个）
├── test_security.rs           # 安全/CSV注入测试（3 个）
└── test_unified_api.rs        # 统一 API 测试（4 个）
```

> 另有 `src/core/data_export.rs` 中的内联单元测试（14 个在 native 环境运行，19 个仅在 wasm32 环境运行）、`src/core/excel_reader.rs`（4 个内联测试）、`src/utils.rs`（2 个）和 `src/validation.rs`（1 个）中的内联测试。

## 运行测试

```bash
# 运行所有测试（190 个）
cargo test

# 按文件运行
cargo test --test lib_tests               # 核心功能（70 个）
cargo test --test test_data_export        # 纯数据导出（41 个）
cargo test --test test_streaming_export   # 流式导出（34 个）
cargo test --test test_excel_preview      # Excel 预览（26 个）
cargo test --test test_resource           # 资源管理（8 个）
cargo test --test test_security           # CSV 注入（3 个）
cargo test --test test_unified_api        # ExportFormat（4 个）

# 按名称过滤
cargo test test_flatten_tree          # 单个测试
cargo test -- --nocapture             # 显示 println 输出

# 性能基准测试
cargo bench --bench export_benchmarks

# 完整检查
cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

## 测试文件说明

### lib_tests.rs（70 个）

核心功能覆盖，包括：

- **文件名处理**（7 个）：扩展名补全、空文件名、自定义文件名
- **CSV 生成**（6 个）：基础写入、特殊字符转义、空值处理
- **文件名验证**（14 个）：危险字符、路径遍历、长文件名、Unicode
- **输入验证**（4 个）：空输入、非法参数
- **样式系统**（14 个）：三级样式合并、颜色解析、边框配置
- **边界/回归测试**（10 个）：大数据、极端输入、已修复 Bug 验证
- **其他**（15 个）：BOM 处理、进度回调、冻结窗格等

### test_data_export.rs（41 个）

纯数据导出逻辑（不依赖 DOM），包括：

- **二维数组导出**：基本数组、空数组、Unicode 内容
- **对象数组 + columns 配置**：简单表头、嵌套表头（多级 children）
- **树形数据**：递归拍平、层级缩进、自定义 `childrenKey`
- **合并单元格**：`rowSpan`/`colSpan` 处理
- **数据格式化**：数字/布尔/null 类型转换
- **列级样式**：style/headerStyle/width 配置

### test_streaming_export.rs（34 个）

流式 CSV 导出逻辑（不依赖 DOM），包括：

- **分块策略**：默认分块、自定义 chunkSize、边界情况
- **进度回调**：进度递增、严格模式
- **BOM 处理**：UTF-8 BOM 头写入
- **XLSX 回退**：格式检测与同步回退

### test_excel_preview.rs（26 个）

Excel 预览解析（calamine + OOXML 样式），包括：

- **基本解析**：单元格数据类型、空工作表、多工作表
- **合并单元格**：合并区域映射、跳过集合
- **数字格式**：整数/浮点/百分比/日期格式化
- **样式解析**：字体/颜色/边框/填充 CSS 转换
- **工作表选择**：索引/名称定位、隐藏 Sheet 跳过

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

| 测试文件                 | 数量    | 覆盖模块                             |
| ------------------------ | ------- | ------------------------------------ |
| lib_tests.rs             | 70      | CSV 生成、文件名验证、输入校验、样式 |
| test_data_export.rs      | 41      | 纯数据导出、树形结构、嵌套表头       |
| test_streaming_export.rs | 34      | 流式导出、分块策略、进度回调         |
| test_excel_preview.rs    | 26      | Excel 预览、样式解析、数字格式       |
| data_export.rs（内联）   | 14      | 内部算法（表头解析、树形拍平）       |
| test_resource.rs         | 8       | UrlGuard RAII                        |
| excel_reader.rs（内联）  | 4       | Excel 解析（合并、数据类型）         |
| test_unified_api.rs      | 4       | ExportFormat 枚举                    |
| test_security.rs         | 3       | CSV 注入防护                         |
| utils.rs（内联）         | 2       | 工具函数                             |
| validation.rs（内联）    | 1       | 文件名验证                           |
| **合计**                 | **190** |                                      |

> 注：`data_export.rs` 中另有 19 个 `#[cfg(target_arch = "wasm32")]` 测试仅在 wasm32 环境运行，不计入上表。

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
