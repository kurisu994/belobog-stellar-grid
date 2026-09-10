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

2026-09 由旧版 6 枢纽结构迁移而来。旧文件 `memory-bank/projectbrief.md` 与
`memory-bank/productContext.md` 合并为 `00-project.md`；`memory-bank/systemPatterns.md` 与
`memory-bank/techContext.md` 按领域拆成 `10`~`14`；`memory-bank/activeContext.md` 整理为
`active/main.md`；`memory-bank/progress.md` 归档为 `archive/2026-09/progress-legacy.md`。
