# Repository Guidelines

## 项目结构与模块组织

核心 Rust/WASM 代码位于 `src/`：`lib.rs` 只负责模块声明和公开重导出；`src/core/` 包含 DOM/数据提取、CSV/XLSX 生成、Excel 解析与样式转换；顶层的 `batch_export*.rs`、`streaming_export.rs`、`validation.rs` 和 `resource.rs` 分别处理批量导出、流式导出、安全校验和 Blob URL 生命周期。集成测试放在 `tests/`，浏览器测试放在 `e2e/tests/`，基准在 `benches/`，示例页面在 `examples/`。`packages/` 保存 TypeScript 类型及 React、Vue、Svelte、Solid、Worker 封装。`pkg/` 与 `target/` 是生成产物，请勿手工编辑。

## 构建、测试与本地开发

项目要求 Rust 1.85+、`wasm-pack`；E2E 还需 Node.js 22、pnpm、Playwright 和 `basic-http-server`。

- `just check-tools`：检查本地开发工具。
- `just build`：执行 `wasm-pack build --target web --out-dir pkg`，生成浏览器可用包。
- `just dev`：构建后在仓库根目录启动本地静态服务器。
- `cargo test`（或 `just test`）：运行原生单元与集成测试。
- `just e2e`：先构建 WASM，再运行 Chromium Playwright 测试。
- `cargo fmt -- --check && cargo clippy -- -D warnings`：提交前检查格式与警告。
- `cargo bench --bench export_benchmarks`：运行 Criterion 导出基准。

## 编码与测试规范

使用 Rust 2024 和 `rustfmt` 默认格式；变量、函数使用英文 `snake_case`，注释、文档及错误消息使用中文。公开 WASM API 应返回 `Result<T, JsValue>`，避免 `panic!`/`unwrap()`；导出前调用文件名校验，Blob URL 使用 `UrlGuard` 管理。

测试文件命名为 `test_<领域>.rs`，测试函数遵循 `test_<模块>_<函数>_<场景>`。新功能须覆盖正常输入、边界值、Unicode 和恶意输入，并维持项目现有 100% 覆盖目标。仅 wasm32 安全的 `JsValue` 测试必须加 `#[cfg(target_arch = "wasm32")]`。可用 `cargo test --test test_security` 或 `cargo test test_flatten_tree` 定向运行。

## 提交与 Pull Request 规范

提交消息使用中文，格式为可选 emoji 加 `type(scope): 动词开头的简短主题`，例如 `🐛 fix(Excel解析): 修复列宽转换`；常用类型包括 `feat`、`fix`、`docs`、`refactor`、`perf`、`test`、`chore`。主题不超过 50 字，正文说明原因和破坏性变更。

PR 应说明问题、方案、影响范围与风险，关联 Issue，并列出实际执行的测试命令。涉及示例页面或浏览器行为时附截图或录屏；面向用户的变化同步写入 `CHANGELOG.md` 的 `[Unreleased]`。提交前确保格式、Clippy、相关 Rust 测试及 E2E 测试通过。

## AI 会话收尾与记忆银行

项目长期记忆在 `memory-bank/`，四层结构（共识 / 任务 / 个人 / 归档）与各层的写入规则见 `memory-bank/README.md`；开工前先读 `memory-bank/00-project.md`，后续按 README 的路径映射表定位对应规范。

每次最终回复前，AI 必须检查本轮是否产生代码变更、重要决策、阻塞或下一步计划；如有：

- 任务进展、活跃文件、下一步与阻塞 → 更新当前分支的 `memory-bank/active/<branch>.md`
- 会话流水与踩过的坑 → 追加到 `memory-bank/journal/<dev>.md`（**只追加、禁改历史、每段留空行**）
- 里程碑或架构演进 → 写入 `memory-bank/archive/YYYY-MM/`
- 共识层（`00-project.md` / `1X-*.md`）仅在项目约定确实变化时改动，且需走 PR review