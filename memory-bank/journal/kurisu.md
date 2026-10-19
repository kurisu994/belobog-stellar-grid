# 开发日志 (Kurisu)

> 只追加，禁改历史条目。每段结尾留一个空行，`merge=union` 才不会把两段黏成一坨。

---

## 2026-09-02 记忆银行迁移到四层结构

- 旧版 6 枢纽（`projectbrief.md` / `productContext.md` / `systemPatterns.md` /
  `techContext.md` / `activeContext.md` / `progress.md`）已迁移到四层结构。
- 采集到的仓库事实：HEAD `33b1655`、版本 `1.1.9`、Rust edition 2024 / MSRV 1.85.0、
  `cargo test` 218 个测试全绿、六个 `packages/*` 子包版本均为 `1.1.9`。
- 领域切分依据是代码位置而非篇幅：导出主干 / Excel 预览 / 安全与资源 / 测试 / 构建发布。
- 踩坑点：`criterion` 必须留在 `cfg(not(target_arch = "wasm32"))` 段，
  否则整个 dev 依赖图无法为 wasm32 解析，wasm32 测试块会静默不被检查。

---

## 2026-09-10 记忆银行增量校准

- 迁移已提交（`abc4cdf`），本轮只做 update 模式：不重写已有规范，只补缺口。
- README 缺「写入门槛」与「个人日志归属」两节，从 `~/develop/Agent/rules/memory-bank.md`
  回填，并补一句「与本文冲突时回查全局规范」，避免项目文档被误读成「每轮必须追加流水」。
- 实测 hook 注入：`src/core/export_*.rs` → `10-export-core`、`src/core/html_builder.rs` →
  `11-excel-preview`、`packages/**` → `14-build-release`、`benches/**` → `13-testing` 全部命中。
- 踩坑点：hook 对同一份规范每会话只注入一次。第一轮测试全部用同一个 `session_id` 时，
  `export_xlsx.rs` 与 `packages/react/index.ts` 返回空，差点误判为 glob 写错；换 `session_id`
  后立即正常。以后验证 paths 要一份规范一个 `session_id`。
- 事实复核：常量、分批默认 1000 行、15 个示例页面、8 个 E2E 用例、6 个子包 1.1.9 与文档记录一致。
- 项目只有 `.claude/` 平台目录，未装 Codex / OpenCode 的 hook，已在 `active/main.md` 待推进项记录。

