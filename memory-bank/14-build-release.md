---
name: 14-build-release
description: 工具链、依赖版本、编译配置、构建发布命令与 CI 流程
paths:
  - "Justfile"
  - "Cargo.toml"
  - ".cargo/config.toml"
  - ".github/workflows/**"
  - "packages/**"
---

# 构建与发布 (Build & Release)

> 改依赖、`Justfile`、子包或 CI 前先读这里。以下均为仓库实际文件的事实快照。

## 工具链

| 项 | 值 | 来源 |
| -- | -- | ---- |
| Rust edition | 2024 | `Cargo.toml` |
| rust-version | 1.85.0 | `Cargo.toml` |
| crate-type | `cdylib`, `rlib` | `Cargo.toml` |
| 构建工具 | `wasm-pack`（`--target web`） | `Justfile` |
| 可选优化 | `wasm-opt -Oz`（binaryen） | `Justfile optimize` |
| 本地服务器 | `basic-http-server` | `Justfile dev` |
| E2E | Playwright（本地，Node 22+ / pnpm） | `Justfile e2e`, `e2e/` |
| CI 浏览器冒烟 | Node 18 + puppeteer + http-server | `.github/workflows/ci.yml` |

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

> 注意：release **未开启** `overflow-checks`，因此上限判断必须用 `checked_mul` 等显式检查。

## Justfile 命令

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

## TypeScript 子包

全部位于 `packages/`，版本与主库保持一致（当前 `1.1.9`）：

| 包名 | 说明 |
| ---- | ---- |
| `@bsg-export/types` | 共享类型定义（`CellStyle`、`Column`、`MergeCellValue`、`SheetInfo` 等） |
| `@bsg-export/react` | React 封装 |
| `@bsg-export/vue` | Vue 封装 |
| `@bsg-export/svelte` | Svelte 封装 |
| `@bsg-export/solid` | Solid.js 封装 |
| `@bsg-export/worker` | Web Worker 封装 |

发版流程由 `Justfile` 的 `bump-core` 统一驱动，它会同步：

1. `Cargo.toml` 版本（`cargo set-version --bump`）
2. `README.md` 中的 `version-` 标记
3. 六个 `packages/*/package.json` 的 `version`
4. 子包内 `@bsg-export/types` 依赖的 `^` 版本
5. `CHANGELOG.md`：`## [Unreleased]` 下插入 `## [新版本] - 日期`

## CI

`.github/workflows/`：

- `ci.yml` — 标签触发，含 lint / test / WASM 构建 / Puppeteer 冒烟 / 发布 / Release
- `benchmark.yml` — 标签触发，含 Criterion 基准、WASM 体积追踪、示例页部署

> 测试分层与本地测试命令见 `13-testing.md`；依赖升级与安全审计规范见 `~/develop/Agent/rules/dependencies.md`。
