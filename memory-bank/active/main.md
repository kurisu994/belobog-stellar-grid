# 当前任务 (main)

## 目标与结果

- 发布补丁版 `1.1.10`，并核查 gh-pages 是否需要同步。
- **GitHub Release、Pages 与 npm 发布全部完成**。公开 registry 已逐一确认主包与六子包的 `1.1.10`，七包 latest 均已同步，实际 tarball 全部可下载且包名 / 版本正确。

## 当前状态

- 发布提交：`7e69cbf`，标签 `v1.1.10`；主库与六个 `packages/*` 子包版本均为 `1.1.10`。
- GitHub Release：<https://github.com/kurisu994/belobog-stellar-grid/releases/tag/v1.1.10>，已公开、非预发布、标为 Latest，附带 WASM / JS / `.d.ts` 三个产物。
- Release 收录此前全量审查的预览、导出、资源安全、框架生命周期和示例修复；详见 `CHANGELOG.md` 的 `1.1.10`。
- README 与 CDN 示例版本同步到 `1.1.10`，README 去掉错误的 `/pkg/` CDN 路径；E2E 从构建产物读取版本，不再写死 CDN 拦截版本。
- 发布后两个 CI 修复提交：`9301983` 补 `--chrome`；`72aca6a` 用 WebDriver capabilities 显式绑定 setup-chrome 安装的浏览器路径，避免驱动 158 自动选中 runner 自带的 Chrome 155。
- `e88df5a` 增加 `workflow_dispatch.release_tag` 补发入口，所有门禁与构建统一固定到标签提交，已存在的 npm 版本跳过，同一标签串行发布；默认留空不发布，手动补发不覆盖 Release / Pages。README 已记录用法，CHANGELOG 记在 Unreleased。
- `main` 相比发布标签仅增加 CI / 发布流程、发布文档和记忆记录；业务源码、依赖锁、示例、测试与版本内容没有改动，标签未被强制移动。

## 发布验证

- 本地通过：`cargo fmt -- --check`、`cargo clippy --locked --all-targets -- -D warnings`、245 个原生测试、wasm32 测试编译。
- 本地通过：Rust 1.85 原生 / wasm32 `cargo check --locked --lib`、release WASM 构建、33 个 Node/WASM 回归测试。
- 本地通过：六子包构建、56 个框架 / 发布回归测试、65 个 Chromium E2E。
- 标签原始 CI `39207476941` 因漏 `--chrome` 失败；首次 main 重跑 `39207849570` 因浏览器 / 驱动不匹配失败，均未进入 npm 发布。
- 修复后的 main CI **`39208315802` 全部验证门禁通过**，包含真实 DOM、WASM、MSRV、框架和浏览器测试；main 的 Release / npm jobs 按标签条件跳过。
- GitHub Release 手动上传该次绿色 CI 的 `wasm-package` 产物；发布前已核对其版本、源码与标签一致性。
- 补发入口的本地校验覆盖默认不发、标签 / 版本校验、Unicode / 注入 / 换行拒绝、固定 SHA、发布门禁、已存在版本跳过、部分补发、registry 异常和实际发布失败停止；3 段 Node 脚本语法通过，独立只读审查后补上 Release 仅限标签 push 的条件。
- **补发流程 `39210664212`**（工作流提交 `e88df5a`，输入 `v1.1.10`）：日志确认源码 `7e69cbfaf5830c7ec617c7e4e94324f86dab8b59`、版本 `1.1.10`、`publish=true`。代码、MSRV、WASM/DOM、E2E、子包回归和全部发布包准备均通过。
- 该流程 npm job **实际执行后失败，而非 skipped**：`2026-10-19T06:41:08Z`，发布 `belobog-stellar-grid@1.1.10` 的 `PUT https://registry.npmjs.org/belobog-stellar-grid` 返回 `E404 Not found`，流程在主包停止，六子包尚未尝试。
- 用户随后更新 `NPM_TOKEN`（Secret 元数据时间 `2026-10-19T06:56:54Z`），已执行 `gh run rerun 39210664212 --failed`。**attempt 2 / job `117930137643`** 于 `06:57:42Z` 真实发布返回 **E403：`You may not perform that action with these credentials`**，当时仍在主包停止。
- 用户再次更新 Secret（元数据时间 `2026-10-19T07:04:44Z`）后，**attempt 3 / 发布 job `117932763345` 成功，整个流程 conclusion=success**。日志分别出现主包和六子包的 `+ <name>@1.1.10`；旧 E404 / E403 不再是当前阻塞。
- 公开 registry 的完整包元数据及实际 tarball 已逐一确认七包版本与 latest 均为 `1.1.10`，子包 types 依赖为 `^1.1.10`，没有残留 file/link/workspace 本地依赖。
- **Vue 等待已结束**：`@bsg-export/vue@1.1.10` 的 npm `published_at` 为 `2026-10-19T07:14:15.469Z`，公开元数据与实际 tarball 均通过校验。先前返回 404 是 npm 受理后的处理阶段，不应因此再次换 Token 或重复发布。
- npm 主包实际 tarball 已下载并校验 WASM SHA-256，与 GitHub Release / Pages 完全一致。Release 说明中已删除过时的「npm 包尚未同步」提示，附件和标签不变。

## gh-pages

- 标签触发的 Benchmark 流程 **`39207476954` 成功**，自动更新示例、WASM 与基准数据，无须手工修改 gh-pages。
- gh-pages 部署提交：`6b28ff4`，明确对应发布提交 `7e69cbf`；Pages 部署流程 **`39207907174` 成功**。
- 线上地址：<https://kurisu994.github.io/belobog-stellar-grid/>。
- 线上 `pkg/package.json` 已确认版本 `1.1.10`，示例目录与标签完全一致，`dev/bench/` 历史数据保留。
- Release / 绿色 CI / 线上 Pages 的 WASM SHA-256 均为 `1ef8fbe3a8ccfafded30839652231e35828f2446ce2741fb25ba16617875c82e`。
- Aside 真实浏览器验证：导航可访问，CDN 演示页在 Pages 使用本地 pkg 完成 WASM 初始化，成功下载 `CDN_示例数据.csv`（202 字节）。npm 主包已另行校验，不以 Pages 本地 pkg 成功替代 npm 发布证据。

## 当前阻塞与下一步

1. **本次版本发布无剩余阻塞**，七个 npm 包全部可安装，不需要再补发或调整 Token。
2. 后续已有标签确需补发时使用 `gh workflow run ci.yml --ref main -f release_tag=<版本标签>`；输入留空只检查，不移动标签，不因 npm 处理延迟重复 publish。
3. 不记录或索要 Secret 值。共识层 `00-project.md` / `14-build-release.md` 等仍是旧版本事实快照，本次未直接改动这些需 PR review 的文件。

## 未排期事项

- 评估进一步收窄 XLSX 分批与流式导出的公共 DOM 迭代。
- 补充大数据量端到端基准追踪。
- 如接入 Codex / OpenCode，再按平台规范补装记忆银行注入 hook。
