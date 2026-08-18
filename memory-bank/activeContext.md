# 活跃上下文 (Active Context)

> 动态工作区：每次会话开始时首选读取，会话结束前主动更新。

## 当前状态

- **最新提交**：`32c5361`（全量代码审查修复，解决 panic、合并坐标、Blob URL 泄漏、OOM 上限与重复代码重构）
- **版本状态**：`1.1.9`（`CHANGELOG.md` 中 `[Unreleased]` 已记录本次审查修复）
- **测试现状**：218 个 Rust 单元/集成测试全部通过；WASM 目标（`wasm32-unknown-unknown`）编译通过；Clippy 检查通过
- **记忆银行**：已完成标准化 6 文件初始化（`projectbrief.md`、`productContext.md`、`systemPatterns.md`、`techContext.md`、`progress.md`、`activeContext.md`）

## 最近活跃文件

```
src/
├── core/
│   ├── data_export.rs      # MAX_DATA_CELLS 上限、checked_mul、format_js_number
│   ├── excel_reader.rs     # 合并绝对坐标、共享 ZipArchive、parse_sheet_xml 合并
│   ├── excel_style.rs      # dxf patternType solid 缺省语义、数字格式缩放
│   ├── export_csv.rs       # 接入 trigger_bytes_download、create_and_download_csv_parts
│   ├── export_xlsx.rs      # write_sheet 统一写入路径、resolve_freeze_pane
│   ├── style.rs            # normalize_hex_color 字符级分支、resolve_column
│   └── table_extractor.rs  # TableRowSources 统一抽象、span 越界保护
├── batch_export.rs         # 接入 TableRowSources 与统一下载
├── batch_export_xlsx.rs    # 接入 TableRowSources 与 write_sheet_with_progress
├── streaming_export.rs     # 接入统一下载、澄清内存语义注释
├── resource.rs             # trigger_blob_download / trigger_bytes_download
├── utils.rs                # escape_csv_injection 剥离 BOM
└── validation.rs           # prepare_download_filename 预检
CHANGELOG.md                # 记录 [Unreleased] 修复项
```

## 最近已做决策

| 决策 | 理由 | 影响范围 |
| ---- | ---- | -------- |
| `normalize_hex_color` 走 `chars.len()` | 避免 3 字节 UTF-8 单字符（如 `"中"`）误入 3 长度分支导致 panic | `src/core/style.rs` |
| 预览合并边界加 `start` 偏移 | `range.start() != (0,0)` 时绝对坐标必须与绝对上限比较 | `src/core/excel_reader.rs` |
| 下载前执行文件名校验 | 杜绝非法文件名路径创建但未释放 Blob URL 的隐患 | `src/resource.rs`, 各导出模块 |
| 数据区总单元格上限 5,000,000 | 与表头 `MAX_HEADER_CELLS` 对称，拦截恶意/超大输入 | `src/core/data_export.rs` |
| `write_sheet` 单点写入抽象 | 消除单表 / 多表 / 分批 XLSX 三处写入与冻结重复逻辑 | `src/core/export_xlsx.rs` |
| 共享 `ZipArchive` 读取预览信息 | 样式表与 sheet XML 共用解包流，合并区域复用 workbook | `src/core/excel_reader.rs` |
| dxf `patternType` 缺省视为 solid | 兼容 Excel 条件格式省略 patternType 的标准写法 | `src/core/excel_style.rs` |
| 极值保留原始精度 | 避免固定 10 位小数把 `1e-15` 截为 `"0"` | `src/core/data_export.rs` |

## 下一步建议

1. **版本发布准备**：如需发版，可执行 `just check` 全面检查并推进版本号（1.1.10 / 1.2.0）
2. **E2E 验证**：在具备 Node 环境下运行 `just e2e` 验证浏览器端端到端行为
3. **框架子包构建**：运行 `just build-packages` 确认各前端框架产物正常构建

## 当前阻塞 / 注意事项

- **无关键代码阻塞**
- **注意事项**：`cargo clippy --all-targets` 中有 10 处既有的测试/基准代码告警（如近似 PI 常量），不影响生产代码 `cargo clippy -- -D warnings`；日常校验以 `just lint` / `cargo clippy -- -D warnings` 为准
