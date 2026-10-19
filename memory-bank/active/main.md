# 当前任务 (main)

## 目标与结果

- 发布补丁版 `1.1.10`，并核查 gh-pages 是否需要同步。
- **GitHub Release 与 Pages 已完成，npm 尚未发布**；不要把版本标签或 Release 成功理解为 npm 成功。

## 当前状态

- 发布提交：`7e69cbf`，标签 `v1.1.10`；主库与六个 `packages/*` 子包版本均为 `1.1.10`。
- GitHub Release：<https://github.com/kurisu994/belobog-stellar-grid/releases/tag/v1.1.10>，已公开、非预发布、标为 Latest，附带 WASM / JS / `.d.ts` 三个产物。
- Release 收录此前全量审查的预览、导出、资源安全、框架生命周期和示例修复；详见 `CHANGELOG.md` 的 `1.1.10`。
- README 与 CDN 示例版本同步到 `1.1.10`，README 去掉错误的 `/pkg/` CDN 路径；E2E 从构建产物读取版本，不再写死 CDN 拦截版本。
- 发布后两个 CI 修复提交：`9301983` 补 `--chrome`；`72aca6a` 用 WebDriver capabilities 显式绑定 setup-chrome 安装的浏览器路径，避免驱动 158 自动选中 runner 自带的 Chrome 155。
- `main` 与发布标签的差异仅为上述 CI 修复（另有本次记忆记录）；业务源码、依赖锁、示例、测试与版本内容没有改动，标签未被强制移动。

## 发布验证

- 本地通过：`cargo fmt -- --check`、`cargo clippy --locked --all-targets -- -D warnings`、245 个原生测试、wasm32 测试编译。
- 本地通过：Rust 1.85 原生 / wasm32 `cargo check --locked --lib`、release WASM 构建、33 个 Node/WASM 回归测试。
- 本地通过：六子包构建、56 个框架 / 发布回归测试、65 个 Chromium E2E。
- 标签原始 CI `39207476941` 因漏 `--chrome` 失败；首次 main 重跑 `39207849570` 因浏览器 / 驱动不匹配失败，均未进入 npm 发布。
- 修复后的 main CI **`39208315802` 全部验证门禁通过**，包含真实 DOM、WASM、MSRV、框架和浏览器测试；main 的 Release / npm jobs 按标签条件跳过。
- GitHub Release 手动上传该次绿色 CI 的 `wasm-package` 产物；发布前已核对其版本、源码与标签一致性。

## gh-pages

- 标签触发的 Benchmark 流程 **`39207476954` 成功**，自动更新示例、WASM 与基准数据，无须手工修改 gh-pages。
- gh-pages 部署提交：`6b28ff4`，明确对应发布提交 `7e69cbf`；Pages 部署流程 **`39207907174` 成功**。
- 线上地址：<https://kurisu994.github.io/belobog-stellar-grid/>。
- 线上 `pkg/package.json` 已确认版本 `1.1.10`，示例目录与标签完全一致，`dev/bench/` 历史数据保留。
- Release / 绿色 CI / 线上 Pages 的 WASM SHA-256 均为 `1ef8fbe3a8ccfafded30839652231e35828f2446ce2741fb25ba16617875c82e`。
- Aside 真实浏览器验证：导航可访问，CDN 演示页在 Pages 使用本地 pkg 完成 WASM 初始化，成功下载 `CDN_示例数据.csv`（202 字节）。这不代表 npm/unpkg 已有 `1.1.10`。

## 当前阻塞与下一步

1. **已确认的直接原因是 npm 发布 job 未执行，而非本次发布报鉴权错误**：标签 CI 在 WASM/DOM 门禁失败，发布 job 被跳过；修复后的 main 手动 CI 虽全绿，但 `publish-npm.if` 仅允许 `refs/tags/v*`，因此仍跳过。仓库已配置 `NPM_TOKEN`。
2. 主包 npm latest 核查仍为 `1.1.8`；`1.1.10` 尚未同步。应补齐显式的已有标签补发路径，保留版本、产物来源和测试门禁，再用现有 CI Secret 实际执行发布。不能因本机 `pnpm whoami` 返回 401 就要求用户先更换 CI Token：本机登录与 Actions Secret 是独立认证环境；`1.1.9` 的 E404 也只属历史证据，当前 Secret 有效性尚未验证。
3. 原标签绑定旧 CI 文件，直接重跑原标签 CI 仍会遇到旧脚本问题；不要盲目重跑或强制移动标签，也不要把 main 的所有手动检查都改成自动发布。新增补发入口时需明确发布版本与对应产物；子包继续由 `packages/scripts/prepare-publish.mjs` 生成发布副本。不要把 token 写入仓库、日志或聊天。
4. 共识层 `00-project.md` / `14-build-release.md` 等仍是旧版本事实快照，本次未直接改动这些需 PR review 的文件。

## 未排期事项

- 评估进一步收窄 XLSX 分批与流式导出的公共 DOM 迭代。
- 补充大数据量端到端基准追踪。
- 如接入 Codex / OpenCode，再按平台规范补装记忆银行注入 hook。
