# AI 助手规则 - belobog-stellar-grid

## 模块结构
```
src/
├── lib.rs              # 仅模块声明和重导出，无业务逻辑
├── validation.rs       # 文件名验证
├── resource.rs         # RAII 资源管理
├── core/
│   ├── mod.rs          # 统一 API (export_table, export_data)
│   ├── data_export.rs  # [核心] 纯数据导出 (树形/合并/表头)
│   ├── table_extractor.rs  # DOM 数据提取
│   ├── export_csv.rs   # CSV 导出
│   └── export_xlsx.rs  # XLSX 导出 (多 Sheet)
├── batch_export.rs     # CSV 异步分批导出
├── batch_export_xlsx.rs # XLSX 异步分批导出
└── utils.rs
```

## 核心约束
- **输入验证**: 文件名必须通过 `validate_filename()` 验证
- **错误处理**: 使用 `Result<T, JsValue>` 而非 `panic!`
- **本地化**: 错误消息必须使用中文
- **资源管理**: Blob URL 必须用 `UrlGuard` 管理
- **引用传递**: 参数优先使用 `&str` 引用
- **数据流**: DOM 模式走 `table_extractor`，Data 模式走 `data_export`

## 发布流程
```bash
just ci-release patch/minor/major  # CI 自动发布 (推荐)
just bump patch/minor/major        # 手动升级版本
just publish latest/beta/next      # 手动发布到 npm
```

## 测试
```bash
cargo test              # 运行所有测试
cargo clippy -- -D warnings
cargo fmt
```

## 重要路径
- 核心 API: `src/core/mod.rs`
- 纯数据逻辑: `src/core/data_export.rs`
- 数据提取: `src/core/table_extractor.rs`
- 验证: `src/validation.rs`
- 测试:
    - `tests/test_data_export.rs` (纯数据/树形/合并)
    - `tests/lib_tests.rs` (DOM 基础)
    - `tests/test_resource.rs` (RAII)
    - `tests/test_unified_api.rs` (统一 API)

---

## 关键设计原则

1. **简洁至上** - 最简单的解决方案
2. **安全第一** - 验证输入、优雅错误处理、中文消息
3. **模块化** - 清晰的职责分离 (DOM vs Data)
4. **性能优化** - 零拷贝 + 分批异步 + RAII
