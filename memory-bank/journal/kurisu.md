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

## 2026-10-19 更正 npm 未发布的原因判断

- 用户追问后复核：本次两个相关流程的 npm job 都是 `skipped`，没有实际执行 npm 发布。标签流程被上游门禁失败阻断，main 修复流程则被 `if: startsWith(github.ref, 'refs/tags/v')` 排除。
- 上条「须恢复发布权限后补发」下结论过早：本机 `pnpm whoami` 的 401 不能证明 Actions 使用的 `secrets.NPM_TOKEN` 无效；旧版 E404 也不能代替本次发布结果。应先打通明确的已有标签补发路径、用现有 Secret 实际执行，再依据错误决定是否需要用户更换凭证。

## 2026-10-19 补发入口落地与 CI 实际发布结果

- `e88df5a` 新增显式 `release_tag` 输入，解析标签为不可变 SHA 后供全部验证、构建和发布 job 使用；输入留空不发，已有版本跳过，主包已发而子包未完成时可重跑补齐。
- 手动触发的运行 ref 也能是标签，因此 Release 条件不能只检查 `refs/tags/v`，还须限定事件为 `push`，否则可能覆盖已有说明 / 附件或选错版本。
- Bash 变量紧贴中文分号时用 `${source_sha}` 明确边界；本地旧 Bash 曾把 `$source_sha；` 误当作变量名而触发 unbound variable。
- 补发 CI `39210664212` 全部测试和发布包准备通过，源码固定 `v1.1.10` 的 `7e69cbf`；实际 pnpm 发布主包于 06:41:08 UTC 收到 registry PUT E404。六子包未尝试，npm 主包 latest 仍 1.1.8。
- 这次 E404 是使用 CI Secret 的实际结果，可以确认当前凭证配置无法完成发布，但仍不能区分过期、账号或包写入范围问题。公开包维护者为 `kurisu_m`；待用户核查凭证后，可用 `gh run rerun 39210664212 --failed` 复用既有门禁和产物重试，不再重打标签。

## 2026-10-19 更新 Token 后的补发重试

- 用户确认已更新 Secret，元数据时间为 06:56:54 UTC；重跑 `39210664212 --failed`，attempt 2 的发布 job 为 `117930137643`。
- 06:57:42 UTC 主包 PUT 返回 E403：`You may not perform that action with these credentials`。没有发布任何新包，主包 `1.1.10` 查询仍为 404。
- 新 Token 不代表已有包写权限：下一步应核对 Packages and scopes 是否允许直接 publish、是否包含主包，再看账号 / IP / 包安全策略；不要把这次 E403 又简单归为 Token 过期。检查截图只需权限与包列表，不索要 Token 值。

