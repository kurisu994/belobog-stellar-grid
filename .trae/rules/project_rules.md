# AI 助手规则 - belobog-stellar-grid

## 模块结构

```
src/
├── lib.rs              # 仅模块声明和重导出，无业务逻辑
├── validation.rs       # 文件名验证 (防止路径遍历等攻击)
├── resource.rs         # RAII 资源管理 (UrlGuard)
├── core/
│   ├── mod.rs          # 统一 API (export_table, export_data, export_tables_xlsx)
│   ├── data_export.rs  # [核心] 纯数据导出 (树形/合并/表头)
│   ├── table_extractor.rs  # DOM 数据提取
│   ├── export_csv.rs   # CSV 格式生成
│   └── export_xlsx.rs  # XLSX 格式生成 (多 Sheet)
├── batch_export.rs     # CSV 异步分批处理
├── batch_export_xlsx.rs # XLSX 异步分批处理
└── utils.rs            # 辅助工具函数

packages/
├── types/              # @bsg-export/types — 严格 TypeScript 类型定义
├── react/              # @bsg-export/react — React Hook + 组件
└── vue/                # @bsg-export/vue  — Vue 3 Composable + 组件
```

## 核心约束

- **输入验证**: 文件名必须通过 `validate_filename()` 验证
- **错误处理**: 使用 `Result<T, JsValue>` 而非 `panic!` 或 `unwrap()`
- **本地化**: 错误消息必须使用中文
- **资源管理**: Blob URL 必须用 `UrlGuard` 管理
- **引用传递**: 参数优先使用 `&str` 引用 (WASM 边界除外)
- **数据流**: DOM 模式走 `table_extractor`，Data 模式走 `data_export`
- **安全**: 禁止路径遍历、危险字符，防止公式注入

## 发布流程

```bash
just ci-release patch/minor/major  # CI 自动发布 (推荐，含子包)
just bump patch/minor/major        # 手动升级版本
just dry-run                        # 发布前测试
just publish latest/beta/next      # 手动发布到 npm
just build-packages                 # 构建子包 (types/react/vue)
just publish-packages               # 发布子包到 npm
```

## 测试

```bash
cargo test                          # 运行所有测试 (103 个)
cargo test --test lib_tests         # DOM 基础功能测试
cargo test --test test_data_export  # 纯数据/树形/合并/表头测试
cargo test --test test_resource     # RAII 资源管理测试
cargo test --test test_unified_api  # 统一 API 测试
cargo test -- test_flatten_tree     # 按名称过滤单个测试
```

## 重要路径

- **核心 API**: `src/core/mod.rs` - 统一入口
- **纯数据逻辑**: `src/core/data_export.rs` - 核心算法
- **DOM 提取**: `src/core/table_extractor.rs` - DOM 解析
- **验证**: `src/validation.rs` - 安全检查
- **CSV 导出**: `src/core/export_csv.rs` - 格式生成
- **XLSX 导出**: `src/core/export_xlsx.rs` - Excel 生成
- **分批处理**: `src/batch_export.rs` 和 `src/batch_export_xlsx.rs`
- **辅助工具**: `src/utils.rs` - 调试和辅助
- **类型定义**: `packages/types/src/index.ts` - 严格 TS 类型
- **React 封装**: `packages/react/src/` - useExporter + ExportButton
- **Vue 封装**: `packages/vue/src/` - useExporter + ExportButton

## 测试文件分布

| 测试文件               | 测试内容                     | 数量  |
|------------------------|-----------------------------|------|
| lib_tests.rs           | DOM 基础功能                 | 41   |
| test_resource.rs       | RAII 资源管理                | 8    |
| test_unified_api.rs    | 统一 API 接口                | 4    |
| test_data_export.rs    | 纯数据/树形/合并/表头         | 33   |
| test_security.rs       | 安全测试 (CSV 注入等)         | 3    |
| **总计**               | **103 个单元测试**           | **103**|

## 关键设计原则

1. **简洁至上** - 最简单的解决方案
2. **安全第一** - 验证输入、优雅错误处理、中文消息
3. **模块化** - 清晰的职责分离 (DOM vs Data)
4. **性能优化** - 零拷贝 + 分批异步 + RAII
5. **全面测试** - 103 个单元测试覆盖所有核心功能
6. **易用性强** - 中文错误提示和简单的 API
