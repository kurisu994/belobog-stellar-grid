# Memory Bank

> 项目长期记忆。**会话开始时先读 `00-project.md`**；读写源码时，下方映射表里命中的
> `1X-*.md` 会被 hook 自动注入（不支持的平台请按映射表人工查阅）。

## 四层结构与写入规则

| 层 | 文件 | 内容 | 频率 | 归属 | 写法 |
| -- | ---- | ---- | ---- | ---- | ---- |
| 共识 | `00-project.md`、`1X-*.md` | 项目定位、编码约定 | 极低 | 团队 | **改写**，走 PR review |
| 任务 | `active/<branch>.md` | 本次做什么、验收标准 | 任务内高频 | 单分支 | 随便改 |
| 个人 | `journal/<dev>.md` | 会话日志、踩过的坑 | 每分钟 | 个人 | **只追加，禁改历史**，每段留空行 |
| 归档 | `archive/YYYY-MM/` | 已完成任务的决策记录 | 一次性 | 团队 | 写完不再动 |

三条红线：

1. `journal/` 只能 append，每段结尾留空行（`merge=union` 靠空行分隔两个人的条目）
2. **不要往仓库里放自动生成的汇总表**——派生数据必冲突且冲突无信息量
3. 共识层改动走 PR，冲突不要 `--ours` 硬合

## 写入门槛

- 仅记录值得跨会话保留的实质进展、重要决策、可复用结论和未解决阻塞。
- 纯查询、无变化的例行检查，以及没有新结论的重复验证，不修改任何记忆文件。
- 有有效增量时只更新内容所属的必要层；不要为完成收尾而追加流水，也不把同一内容重复抄到各层。
- 不因采用本门槛而删除已有历史记录。

## 个人日志归属

写入 `journal/<dev>.md` 前，先确认本次任务的开发者身份；同一任务内已核实且用户、仓库或身份配置未变时可以复用结果。现有个人日志以各文件开头记载的归属为准，不互相复用。

1. 优先采用用户明确指定的日志身份或项目已有的身份映射。没有明确归属时，在**当前仓库**执行以下只读命令，核对有效作者身份及配置来源，不只读取全局配置：
   ```bash
   git var GIT_AUTHOR_IDENT
   git config --show-origin --get-regexp '^user\.(name|email)$'
   ```
2. Git 身份是开发者归属的证据，不等于登录者身份。若与用户说明、项目映射冲突，核对仓库/工作树配置及作者环境变量覆盖；不能用工作目录名、系统用户名、已有日志文件名或最近提交作者单独认定当前用户，也不要修改 Git 配置来迎合某个日志文件。
3. 将已确认身份与项目映射或现有日志中明确记载的归属对应，匹配后才追加。没有对应文件时创建本人的新日志，不复用他人的文件；无命名约定时用已核实的开发者名转小写、空白替换为 `-`，文件名不得包含路径分隔符。同名文件归属不明或冲突时先确认，不覆盖。
4. 新建日志注明开发者名、核实日期和身份来源即可，不必复制完整邮箱等个人资料。无法查询 Git、配置为共享/机器人身份或检查后仍有歧义时，先完成不依赖个人归属的工作，再向用户确认日志身份；确认前不写任何个人日志。
5. 个人日志只追加且段末留空行，不搬运他人历史。若误写发生在本任务内，只撤销可确认由自己加入的内容，保留原有记录；不要为纠正身份而重写整份日志。

本项目现有归属：`journal/kurisu.md` 记载为 Kurisu，核实来源为当前仓库 `git var GIT_AUTHOR_IDENT`；新增协作者按上述流程新建自己的文件。

> 本文缺失相关条款，或与仓库根 `AGENTS.md` 的收尾要求冲突时，回查 `~/develop/Agent/rules/memory-bank.md`——那里是写入门槛与个人日志归属的底线，项目文档不能用「每轮必须追加流水」或「按已有文件名猜用户」覆盖它。

## 索引

| 文件 | 用途 |
| ---- | ---- |
| `00-project.md` | 开局必读：是什么、功能边界、核心用户流、产品约束 |
| `10-export-core.md` | 导出主干：架构、写入/下载统一路径、上限常量、公开 API |
| `11-excel-preview.md` | Excel 预览：OOXML 解析、合并坐标、条件格式 |
| `12-security-resource.md` | 文件名校验、注入防护、Blob URL 生命周期 |
| `13-testing.md` | 测试分层、命名规范、运行命令 |
| `14-build-release.md` | 工具链、依赖版本、构建发布、CI |
| `active/main.md` | 当前分支在做什么 |
| `journal/kurisu.md` | 个人开发日志 |
| `archive/2026-09/progress-legacy.md` | 1.0.x ~ 1.1.9 的版本里程碑与已解决阻碍 |

## 路径映射表

> 由各文件 frontmatter 的 `paths:` 实际抽取生成，改动 frontmatter 后请重新生成本表。
> 已实测：`src/core/export_*.rs`、`src/core/html_builder.rs`、`packages/**`、`benches/**`
> 等路径均能被 hook 正确命中（同一份规范每会话只注入一次）。

| 规范文件 | 描述 | 触发路径 |
| -------- | ---- | -------- |
| `00-project.md` | 项目定位、功能边界、核心用户流与产品约束（开局必读） | —（开局必读，不自动注入） |
| `10-export-core.md` | 导出主干架构、统一写入/下载路径、安全上限常量与公开 WASM API | `src/lib.rs` `src/utils.rs` `src/core/mod.rs` `src/core/export_*.rs` `src/core/table_extractor.rs` `src/core/data_export.rs` `src/core/style.rs` `src/batch_export*.rs` `src/streaming_export.rs` |
| `11-excel-preview.md` | Excel 在线预览的 OOXML 解析、合并坐标、条件格式与安全上限 | `src/core/excel_reader.rs` `src/core/excel_style.rs` `src/core/html_builder.rs` |
| `12-security-resource.md` | 文件名校验、CSV/Excel 注入防护与 Blob URL 生命周期管理 | `src/validation.rs` `src/resource.rs` |
| `13-testing.md` | 测试分层、命名规范、wasm32 测试标记与运行命令 | `tests/**` `benches/**` `e2e/**` |
| `14-build-release.md` | 工具链、依赖版本、编译配置、构建发布命令与 CI 流程 | `Justfile` `Cargo.toml` `.cargo/config.toml` `.github/workflows/**` `packages/**` |

## 平台支持

| 平台 | 路径触发注入 | 说明 |
| ---- | ------------ | ---- |
| Claude Code | ✅ | `.claude/settings.json` + PostToolUse 事后注入 |
| Codex | ✅ | `.codex/hooks.json` + PreToolUse 先拒后放行 |
| OpenCode | ✅ | `.opencode/plugins/*.js` 调同一 Python 引擎 |
| Pi | ❌ | 扩展只能会话级注入，工具事件无法回传上下文 |
| Grok | ❌ | 无 hook 机制，纯 pull-based |
| Cursor | ❌ | 无路径触发注入实现 |

不支持 hook 的平台，按上方映射表人工查阅对应规范。

## 迁移说明

2026-09 由旧版 6 枢纽结构迁移而来（以下旧文件已删除，仅作追溯记录）。旧文件 `memory-bank/projectbrief.md` 与
`memory-bank/productContext.md` 合并为 `00-project.md`；`memory-bank/systemPatterns.md` 与
`memory-bank/techContext.md` 按领域拆成 `10`~`14`；`memory-bank/activeContext.md` 整理为
`active/main.md`；`memory-bank/progress.md` 归档为 `archive/2026-09/progress-legacy.md`。
