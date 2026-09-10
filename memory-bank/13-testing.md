---
name: 13-testing
description: 测试分层、命名规范、wasm32 测试标记与运行命令
paths:
  - "tests/**"
  - "benches/**"
  - "e2e/**"
---

# 测试 (Testing)

> 新增或修改测试前先读这里。

## 测试分层

| 层 | 位置 | 运行器 | 覆盖内容 |
| -- | ---- | ------ | -------- |
| 原生单元 / 集成 | `src/` 内 `#[cfg(test)]`、`tests/*.rs` | `cargo test` | 纯逻辑、边界、安全校验 |
| WASM 单元 | `tests/*.rs` 中的 wasm32 块 | `wasm-bindgen-test-runner` | `JsValue` / DOM 相关 |
| 浏览器 E2E | `e2e/tests/*.spec.ts` | Playwright（Chromium） | 真实下载、示例页面行为 |
| 基准 | `benches/export_benchmarks.rs` | Criterion | 导出吞吐与体积 |

当前基线：**218 个 Rust 单元/集成测试全部通过**（`cargo test`）。

## 命名与组织

- 集成测试文件：`test_<领域>.rs`（放 `tests/`）
- 测试函数：`test_<模块>_<函数>_<场景>`
- 现有文件：`test_data_export.rs` / `test_excel_preview.rs` / `test_resource.rs` /
  `test_security.rs` / `test_streaming_export.rs` / `test_unified_api.rs` / `lib_tests.rs`
- E2E 用例：`array-export` / `basic-export` / `benchmark` / `container-export` /
  `multi-sheet-export` / `style-export` / `tree-export` / `wasm-init`

## 硬性要求

- 新功能必须覆盖四类输入：**正常输入、边界值、Unicode、恶意输入**；项目维持 100% 覆盖目标
- 仅 wasm32 安全的 `JsValue` 测试必须显式标记 `#[cfg(target_arch = "wasm32")]`
- `wasm-bindgen-test` 是 wasm 侧测试依赖；原生侧用它会在编译期报错
- `criterion` 必须留在 `cfg(not(target_arch = "wasm32"))` 段：
  它依赖 rayon，放进通用 `dev-dependencies` 会让整个 dev 依赖图无法为 wasm32 解析，
  连带 `#[cfg(target_arch = "wasm32")]` 测试块永远不被检查

## 常用命令

| 命令 | 作用 |
| ---- | ---- |
| `cargo test` / `just test` | 原生单元与集成测试（默认入口） |
| `cargo test --test test_security` | 定向跑某个集成测试文件 |
| `cargo test test_flatten_tree` | 定向跑某个测试函数 |
| `cargo check --target wasm32-unknown-unknown` | 验证 wasm 目标可编译 |
| `just e2e` | 先 `just build` 再跑 Playwright（Chromium） |
| `just e2e-headed` | 同上，带浏览器界面 |
| `cargo bench --bench export_benchmarks` | Criterion 基准（native only） |

E2E 依赖：Node.js 22、pnpm、Playwright、`basic-http-server`。

> 构建、发布与 CI 流程见 `14-build-release.md`。
