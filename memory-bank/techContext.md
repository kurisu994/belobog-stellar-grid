# 技术上下文 (Tech Context)

> 纯事实参考，均取自仓库实际文件。

## 工具链

| 项 | 值 | 来源 |
| -- | -- | ---- |
| Rust edition | 2024 | `Cargo.toml` |
| rust-version | 1.85.0 | `Cargo.toml` |
| crate-type | `cdylib`, `rlib` | `Cargo.toml` |
| 构建工具 | `wasm-pack`（`--target web`） | `Justfile` |
| 可选优化 | `wasm-opt -Oz`（binaryen） | `Justfile optimize` |
| 本地服务器 | `basic-http-server` | `Justfile dev` |
| E2E | Playwright + Node（CI 用 Node 18） | `e2e/`, `.github/workflows/ci.yml` |

## Rust 依赖

| crate | 版本 | 用途 |
| ----- | ---- | ---- |
| `wasm-bindgen` | 0.2.106 | JS 互操作 |
| `wasm-bindgen-futures` | 0.4 | async → Promise |
| `web-sys` | 0.3.83 | DOM / Blob / URL |
| `js-sys` | 0.3.83 | JS 内建类型 |
| `csv` | 1.4.0 | CSV 编码 |
| `rust_xlsxwriter` | 0.69.0（feature `wasm`） | XLSX 写入 |
| `calamine` | 0.34 | Excel 解析（预览） |
| `zip` | 2（`default-features = false`, `deflate`） | xlsx 解包 |
| `quick-xml` | 0.37 | OOXML 解析 |
| `serde` / `serde_json` | 1 | JSON 输出 |
| `console_error_panic_hook` | 0.1.7（默认 feature） | panic 可读化 |

**dev-dependencies**

| crate | 版本 | 备注 |
| ----- | ---- | ---- |
| `wasm-bindgen-test` | 0.3.56 | — |
| `criterion` | 0.5 | 仅 `cfg(not(target_arch = "wasm32"))`，因依赖 rayon 无法编译到 wasm32 |

## 编译配置

`.cargo/config.toml`：

```toml
[target.wasm32-unknown-unknown]
runner = "wasm-bindgen-test-runner"

[profile.release]
lto = true
opt-level = "z"
codegen-units = 1
```

> 注意：release 未开启 `overflow-checks`，因此上限判断必须用 `checked_mul` 等显式检查。

## 常用命令

| 命令 | 作用 |
| ---- | ---- |
| `just check-tools` | 检查 wasm-pack / basic-http-server / cargo-edit |
| `just build` | `wasm-pack build --target web --out-dir pkg` |
| `just dev` | 构建后启动本地静态服务器 |
| `just fmt` / `just lint` / `just check` | `cargo fmt` / `cargo clippy -- -D warnings` / 两者 |
| `just test` | `cargo test` |
| `just e2e` / `just e2e-headed` | 构建后跑 Playwright |
| `just optimize` | `wasm-opt -Oz` |
| `just build-packages` | 构建 `packages/` 下 TS 子包 |
| `cargo bench --bench export_benchmarks` | Criterion 基准 |
| `cargo check --target wasm32-unknown-unknown` | 验证 wasm 目标可编译 |

## 公开 API（`src/lib.rs`）

**导出**

- `ExportFormat`（`Csv` 默认 / `Xlsx`）
- `export_table` — DOM 表格导出
- `export_tables_xlsx` — 多表 → 多 Sheet
- `export_data` — JS 数组 / 对象 / 树形数据导出
- `generate_data_bytes` — 仅生成字节（Worker 场景）
- `export_table_to_csv_batch` — CSV 分批异步
- `export_table_to_xlsx_batch` / `export_tables_to_xlsx_batch` — XLSX 分批异步
- `export_data_streaming` — 流式 CSV

**Excel 预览**

- `get_excel_sheet_list`
- `parse_excel_to_html`
- `parse_excel_to_json`

**工具**

- `UrlGuard`、`validate_filename`、`ensure_extension`、`escape_csv_injection`、`set_panic_hook`

**内部（`#[doc(hidden)] bench_exports`）**

- `generate_csv_bytes`、`generate_xlsx_bytes`、`generate_xlsx_multi_bytes`、`MergeRange`、`TableData`

## `export_data` 选项字段

来自 `core/mod.rs::ExportDataOptions`：

`columns` / `filename` / `format` / `progressCallback` / `indentColumn` / `childrenKey` /
`withBom` / `strictProgress` / `freezeRows` / `freezeCols` / `headerStyle` / `cellStyle`

流式额外支持 `chunkSize`（默认 5000，最小 1）。

## TypeScript 子包

全部位于 `packages/`，版本与主库一致（`1.1.9`）：

| 包名 | 说明 |
| ---- | ---- |
| `@bsg-export/types` | 共享类型定义（`CellStyle`、`Column`、`MergeCellValue`、`SheetInfo` 等） |
| `@bsg-export/react` | React 封装 |
| `@bsg-export/vue` | Vue 封装 |
| `@bsg-export/svelte` | Svelte 封装 |
| `@bsg-export/solid` | Solid.js 封装 |
| `@bsg-export/worker` | Web Worker 封装 |

## 测试现状

最近一次 `cargo test` 实测：

| 目标 | 通过数 |
| ---- | ------ |
| lib 单元测试 | 98 |
| `tests/lib_tests.rs` | 41 |
| `tests/test_data_export.rs` | 34 |
| `tests/test_excel_preview.rs` | 4 |
| `tests/test_resource.rs` | 8 |
| `tests/test_security.rs` | 3 |
| `tests/test_streaming_export.rs` | 26 |
| `tests/test_unified_api.rs` | 4 |
| **合计** | **218** |

E2E：`e2e/tests/` 8 个 spec 文件，约 47 个用例（array / basic / benchmark / container / multi-sheet / style / tree / wasm-init）。

> ⚠️ 已知偏差：`cargo clippy --all-targets -- -D warnings` 在**测试与基准代码**中有 10 个历史遗留告警
> （近似 PI 常量、`useless_vec`、`needless_borrow` 等）。项目标准命令是 `cargo clippy -- -D warnings`（仅 lib），该命令通过。

## CI

`.github/workflows/`：

- `ci.yml` — 标签触发，包含 lint / test / E2E / 子包构建等 job
- `benchmark.yml` — 标签触发的基准与部署
