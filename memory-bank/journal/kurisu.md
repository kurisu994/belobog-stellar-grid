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

## 2026-10-19 审查修复与预检复核

- 全量审查的预览、导出、示例、CI 和框架子包问题已在工作区修好，尚未提交。
- 预检不能只数 `<c>` 标签：calamine 会展开自闭合 `<row/>`，并跳过 `DataRef::Empty`；`uniqueCount` 在读正文前就会 `Vec::reserve`。
- 日期格式里的 `m/mm` 不能在看到小时后一直当分钟，遇到后续日期代码要回到月份。
- `.xls` 仍无法在打开前按坐标拒绝稀疏表，文档里写明了这个缺口。
- 框架 `tsconfig.paths` 必须指向仓库内声明，否则 `pnpm run build` 仍会吃到旧的已发布类型。

## 2026-10-19 复核 0e29f63

- 预检状态机只在 `in_sheet_data` 上判断标签，没考虑单元格体内的嵌套标记；`<f><row/></f>` 会让行列/开启单元格状态失步，普通过滤器挡住的文件被放行，calamine 仍按 1e9 格分配（实测被 SIGKILL）。
- `preflight` 要求 `xl/workbook.xml`，把 ODS/XLSB 从「能预览」变成硬报错；calamine 自己认得这两种格式。
- `sheet_paths` 按 `<sheet>` 顺序返回路径，重复 `r:id` 时同一 XML 被完整扫描多次；19.5KB 文件 400 个重复条目让预检跑了 16.8s。
- 数字导出改成 `to_string()` 全精度，`0.1+0.2` 会写出 `0.30000000000000004`，CHANGELOG 的「修复尾随小数」说法与实际方向相反。

## 2026-10-19 修复复核发现的预检问题

- 用「正文容器 + 嵌套深度」代替只看 `in_sheet_data`：`<v>`/`<f>`/`<is>` 里的标签一律忽略，否则 `<f><row/></f>` 会把行号推走并清掉当前单元格，巨矩形直接放行。
- 预检不能假设文件一定是 xlsx；缺 `xl/workbook.xml` 就返回 `Ok(None)` 降级，ODS/XLSB 才不会从「能预览」变成报错。
- 重复 `<sheet r:id>` 会按路径重复扫描同一份 XML，去重后总工作量才受总解压体积约束。
- 数字导出保留完整精度是可辩护的选择，错的只是 CHANGELOG 的措辞；不要为了文案去改行为。

## 2026-10-19 发布 1.1.10 的流程注意事项

- `wasm-pack test --chromedriver <path>` 不等于选了 Chrome，仍必须传 `--chrome`；补参数后才能真正进入 DOM 回归。
- setup-chrome 安装了配套驱动，也不代表测试会自动用配套浏览器：本次驱动 158 选中了 runner 自带的 Chrome 155。通过 `WASM_BINDGEN_TEST_WEBDRIVER_JSON` 指向临时 capabilities，并在 `goog:chromeOptions.binary` 写入 action 的 `chrome-path` 后通过。
- 已推送标签不强制移动；本次在 main 修 CI、跑通全部门禁，核对业务源码与原标签无差异后，用绿色 CI 产物完成 Release。WASM 哈希与标签自动部署的 Pages 完全一致。
- 版本升级还要同步 CDN 示例和测试拦截地址；测试现从 `pkg/package.json` 读取版本，避免发版后请求未发布 CDN 而误失败。
- GitHub Release、Pages、npm 是三个独立结果。本次前两项已完成；本机 npm 身份查询 401，上次标签发布 404，npm 主包 latest 仍是 1.1.8，须恢复发布权限后补发，不能仅凭标签存在宣布 npm 成功。

