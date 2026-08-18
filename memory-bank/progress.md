# 进度记录 (Progress)

> 面向历史回溯：记录版本里程碑、重大架构演进与已解决的阻碍。

## 版本历史

| 版本 | 日期 | 核心变更 |
| ---- | ---- | -------- |
| **1.1.9** | 2026-08-14 | 条件格式 sqref 范围表示避免 OOM；多 Sheet 预览索引修复；子包补齐全局样式与三级类型；`criterion` 移入 native 段解除 wasm32 编译阻塞 |
| **1.1.8** | 2026-03-25 | 隐藏行列不渲染；条件格式渲染；自闭合 xf/cfRule 收集；列宽像素下限调整 |
| **1.1.7** | 2026-03-23 | Svelte / Solid.js 预览组件；样式计算溢出修复与表格解析优化 |
| **1.1.6** | 2026-03-20 | 合并属性读取优化（减少 JS Reflect 调用）；跨度上限钳制；CSS 注入与 CSV 注入防护增强 |
| **1.1.5** | 2026-03-20 | 树形数据拍平与层级缩进；合并单元格对象格式支持 |
| **1.1.4** | 2026-03-20 | Excel 在线预览基础能力（calamine + OOXML 样式）；HTML/JSON 双输出 |
| **1.1.3** | 2026-03-20 | 完善 TypeScript 类型与文档 |
| **1.1.2** | 2026-03-20 | 表头多行合并与样式增强 |
| **1.1.1** | 2026-03-19 | 三级样式体系（全局 → 列级 → 单元格级）；冻结窗格配置 |
| **1.1.0** | 2026-03-03 | React / Vue / Svelte / Solid / Worker / Types 框架子包体系 |
| **1.0.x** | 2025-12 ~ 2026-02 | 基础功能建立：HTML 表格导出、分批异步、流式 CSV、多 Sheet、文件名安全、RAII |

## 重大架构变更

### 1. 全量代码审查与健壮性修复（2026-03-22 / 提交 `32c5361`）

- **Hex 颜色解析修复**：`normalize_hex_color` 改为按字符数量 + hex 有效性分支，彻底消除单字符 3 字节 UTF-8 输入（如 `"中"`）命中 3 分支导致的越界 panic
- **Excel 预览合并坐标对齐**：`build_merge_map` 与 `build_skip_set` 边界比较改为绝对坐标（`data_* + start`），修复 `range.start() != (0,0)` 时后半段合并被误丢的问题
- **Blob URL 泄漏收敛**：`resource::trigger_bytes_download` / `trigger_blob_download` 统一管理下载；先 `prepare_download_filename` 校验，非法文件名不再进入 URL 创建路径
- **数据导出 OOM 上限**：引入 `MAX_DATA_CELLS = 5_000_000`；表头上限检查改用 `checked_mul`；span 按真实行列网格钳制
- **Zip 句柄复用**：`ExcelStyleSheet::from_zip_archive` + `parse_sheet_dimensions_from_archive` 共享同一 `ZipArchive`，合并区域从已打开的 calamine workbook 获取，打开次数从 4 降至 2
- **写入路径与行源收敛**：抽取 `write_sheet` 作为 XLSX 写入的唯一定义，`TableRowSources` 作为 DOM 表格行源解析；`StyleSheet::resolve_column` 支持按列缓存 `Format`
- **解析鲁棒性**：`parse_sheet_xml` 合并 Start/Empty 分支；`format_js_number` 保留极小/极大值精度；dxf 填充仅 `"none"` 不上色（缺省视为 solid）

### 2. 条件格式内存爆炸治理（1.1.9 / `1b95e39`）

- **问题**：Excel 整列条件格式 `sqref="A1:D1048576"` 原本会展开为坐标集合，极端情况生成上百亿个坐标，瞬间撑爆 WASM 内存
- **解法**：改为保留矩形范围边界 `CellRange { first_row, first_col, last_row, last_col }`，内存与命中判断均从 O(单元格数) 降为 O(范围数)
- **附带修复**：`parse_cell_ref` 在 `"A0"` 下溢及超长列名 u32 溢出的两处隐患

### 3. 多端子包与三级样式体系建立（1.1.0 ~ 1.1.1）

- 拆分 `packages/` monorepo 结构，提供 5 个框架封装包与 1 个 types 包
- 建立 全局 → 列级 → 单元格级 的三级样式合并机制，统一 Format 解析流

### 4. 预览引擎实现（1.1.4）

- 基于 `calamine` 在 WASM 侧提取单元格数据
- 基于 `zip` + `quick-xml` 手写 OOXML 样式与维度解析器，绕过全量重量级解析器以控制体积

## 已解决的关键阻碍

| 阻碍 | 表现 | 解决方案 | 对应版本 |
| ---- | ---- | -------- | -------- |
| WASM 测试阻塞 | `criterion` 依赖 rayon 导致 wasm32 目标下 dev 依赖图无法解析 | 将 criterion 移入 `cfg(not(target_arch = "wasm32"))` | 1.1.9 |
| 预览 Sheet 错位 | 含隐藏 Sheet 时 `activeSheet` 存的是可见列表位置，传给 WASM 被误当作原始索引 | 区分可见列表索引与原始 workbook 索引 | 1.1.9 |
| 全局样式子包失效 | 框架层组件未把 `headerStyle` / `cellStyle` 透传给 WASM | 补全各框架导出接口的参数映射 | 1.1.9 |
| 预览合并错位 | 隐藏列跳过导致后续单元格水平错位 | 跨度计算扣除隐藏列，仅当原点可见时保留合并 | 1.1.8 |
| 自闭合 xf 丢失 | `<xf ... />` 被当作非样式元素忽略，导致样式索引整体偏移 | Start 与 Empty 事件统一收集 xf | 1.1.8 |
| 外部 tbody 重复 | 传入的 `tbodyId` 位于 `tableId` 内部时数据翻倍 | `ensure_external_tbody` 运行时强校验拦截 | 1.0.8 |

## 待推进事项（未排期）

- [ ] 评估是否将 XLSX 分批与流式导出的公共 DOM 迭代再收窄
- [ ] 评估 `packages/` 自动化测试与发布工作流联动
- [ ] 针对大数据量导出补充端到端基准追踪
