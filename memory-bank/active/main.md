# 当前任务 (main)

> 分支 `main` 的活跃工作区。会话开始先读，结束前更新。
> 每条写完留一个空行（多人/多会话协作时便于合并）。

## 目标

- 全量审查发现的预览、导出、示例、CI 和框架子包问题已纳入当前代码提交，尚未发版
- 记忆银行四层迁移已在 `abc4cdf` 完成并提交；本文件前半仍保留 2026-09-10 的校准记录

## 验收标准

- 旧的 6 个枢纽内容零丢失地落到新结构（`--verify` 通过）✅
- `1X-*.md` 每份都带 `paths` 并能被 hook 实测命中 ✅（`src/core/export_*.rs` →
  `10-export-core`、`src/core/html_builder.rs` → `11-excel-preview`、`packages/**` →
  `14-build-release`、`benches/**` → `13-testing` 均实测命中）
- `AGENTS.md` 指向新的 `memory-bank/README.md` ✅
- README 含写入门槛与个人日志归属，且不与全局规范冲突 ✅

## 当前状态

- **当前 HEAD**：审查修复已提交到 `main`，尚未发版；不要再按工作区未提交理解
- **上一个代码提交**：`32c5361`（全量代码审查修复：panic、合并坐标、Blob URL 泄漏、
  OOM 上限与重复代码重构）；其后所有提交均为文档/记忆银行改动，无代码变更
- **版本状态**：`1.1.9`，六个 `packages/*` 子包版本一致；`CHANGELOG.md` 的 `[Unreleased]`
  已记录审查修复，尚未发版
- **事实复核结果**：常量（`MAX_DATA_CELLS=5_000_000`、`MAX_HEADER_CELLS=100_000`、
  `MAX_DEPTH=64`、`DEFAULT_CHUNK_SIZE=5000`、`EXCEL_MAX_ROW/COL=1_048_575/16_383`、
  预览侧 `100_000/16_384/1_000_000/50MB`）、分批默认 1000 行、15 个 HTML 示例、
  8 个 E2E 用例、6 个子包版本，与 `memory-bank/` 记载全部一致
- **测试基线**：218 个 Rust 单元/集成测试通过；wasm32 目标编译通过；`cargo clippy -- -D warnings` 通过
- **注入机制**：`.claude/` 已装 hook（PostToolUse + SessionStart），`.gitattributes` 已配
  `memory-bank/journal/*.md merge=union`；项目内无 `.codex/` / `.opencode/` 目录，故未装

## 活跃文件

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
memory-bank/                # 本轮唯一改动区域（README / active / journal）
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
| README 的写入门槛/日志归属从全局规范回填 | 项目文档不得用「每轮追加流水」或「按日志文件名猜用户」覆盖底线 | `memory-bank/README.md` |

## 下一步

1. **版本发布准备**：如需发版，可执行 `just check` 全面检查并推进版本号（1.1.10 / 1.2.0）
2. **E2E 验证**：在具备 Node 环境下运行 `just e2e` 验证浏览器端端到端行为
3. **框架子包构建**：运行 `just build-packages` 确认各前端框架产物正常构建

## 待推进事项（未排期）

- [ ] 评估是否将 XLSX 分批与流式导出的公共 DOM 迭代再收窄
- [ ] 评估 `packages/` 自动化测试与发布工作流联动
- [ ] 针对大数据量导出补充端到端基准追踪
- [ ] 如后续接入 Codex / OpenCode，为其补装对应 hook（当前项目只有 `.claude/`）

## 当前阻塞 / 注意事项

- **无关键代码阻塞**
- **注意事项**：`cargo clippy --all-targets` 中有 10 处既有的测试/基准代码告警（如近似 PI 常量），
  不影响生产代码 `cargo clippy -- -D warnings`；日常校验以 `just lint` / `cargo clippy -- -D warnings` 为准
- **注意事项**：hook 对同一份规范每会话只注入一次，一次验证多份规范时要换 `session_id`，
  否则会误判为「paths 没命中」

## 0e29f63 复核发现（待修复）

- 预检可被单元格体内的嵌套标签绕过（如 `<f><row/></f>`），需在 `<v>`/`<f>`/`<is>` 内忽略其它标签
- ODS/XLSB 被 `preflight` 硬拒，需按「无坐标预检」降级或明确支持范围
- 重复 `<sheet r:id>` 会让同一 worksheet 被反复完整扫描，应按去重后的路径扫描
- 数字导出改为全精度 `to_string()`，与 CHANGELOG「修复尾随小数」描述相反

## 全量审查修复（2026-10-19，已提交未发版）

> 原审查项已落地。预检的后续复核又补了三处行为对齐，别再按「未修复」理解。

- XLSX 在 calamine 打开前预检输入、解压量和真实非空矩形；自闭合 `<row/>` 推进隐式行号，`uniqueCount` 超过 100 万直接拒绝
- 纯样式空单元格不占矩形；`hh:mm dd/mm/yyyy` 的月份不再渲染成分钟
- `.xls` 仍只有 50 MiB 原始大小上限，不能说稀疏 BIFF 已被坐标预检拦住
- 示例、DOM rowspan、CSV 等宽、数据导出、四框架预览和 CI/MSRV 门禁已按审查项修改
- 五个框架 `tsconfig.paths` 指向 `pkg/belobog_stellar_grid.d.ts` 和 `packages/types/dist`
