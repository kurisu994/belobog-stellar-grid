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

