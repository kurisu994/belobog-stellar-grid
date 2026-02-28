<div align="center">

  <h1><code>belobog-stellar-grid</code></h1>

  <p><strong>🦀 现代化的 WebAssembly 表格导出库</strong></p>

  <p>一个安全、高效、易用的 Rust WebAssembly 库，用于将 HTML 表格导出为 CSV 和 XLSX 文件</p>

  <p>
    <img src="https://img.shields.io/badge/version-1.0.10-blue.svg" alt="Version" />
    <img src="https://img.shields.io/badge/rust-edition%202024-orange.svg" alt="Rust Edition" />
    <img src="https://img.shields.io/badge/test_coverage-100%25-brightgreen.svg" alt="Test Coverage" />
    <img src="https://img.shields.io/badge/license-MIT%2FApache--2.0-green.svg" alt="License" />
    <a href="https://github.com/kurisu994/belobog-stellar-grid"><img src="https://img.shields.io/badge/github-belobog--stellar--grid-181717.svg?logo=github" alt="GitHub" /></a>
  </p>

<sub>Built with 🦀🕸 by <a href="https://rustwasm.github.io/">Rust and WebAssembly</a></sub>

</div>

---

## 📋 项目简介

`belobog-stellar-grid` 是一个高性能的 WebAssembly 库，让你可以轻松地在浏览器中将 HTML 表格导出为 CSV/XLSX 文件。项目采用模块化架构设计，包含完善的文件名验证、RAII 资源管理和分批异步处理机制。

### 为什么选择这个库？

- **🎯 零配置**：开箱即用，无需复杂的设置
- **🚀 极致性能**：Rust 原生速度 + WebAssembly 优化
- **🔒 企业级安全**：内置文件名验证，防止路径遍历攻击
- **📦 轻量级**：约 1.3MB 的 WASM 文件（Gzip 压缩后约 450KB）
- **✅ 100% 测试覆盖**：121 个单元测试 + 33 个 E2E 测试确保代码质量
- **🏗️ 模块化架构**：清晰的模块设计，易于维护和扩展
- **🌍 国际化支持**：完美支持中文、日文、韩文等 Unicode 字符
- **💾 多格式导出**：支持 CSV 和 XLSX (Excel) 两种格式

### ✨ 核心特性

#### 🛡️ 安全性

- **RAII 资源管理**：`UrlGuard` 自动清理 Blob URL
- **文件名安全验证**：阻止路径遍历、危险字符等 10+ 种威胁
- **全面错误处理**：所有函数返回 `Result<T, JsValue>`
- **内存安全保证**：得益于 Rust 的所有权系统
- **中文错误消息**：用户友好的错误提示
- **公式注入防护**：自动转义 CSV/Excel 中的危险字符（=, +, -, @）

#### 🚀 性能优化

- **零拷贝操作**：直接操作 DOM，参数使用 `&str` 引用
- **分批异步处理**：支持百万级数据导出，避免页面卡死
- **内存安全**：使用 Rust 默认分配器，确保安全性和性能
- **LTO 优化**：链接时优化减少最终 WASM 大小
- **实时进度反馈**：支持大型表格的进度回调

#### 🛠️ 高级功能

- **合并单元格支持**：完美支持 `rowspan` 和 `colspan`，保持表格结构
- **多工作表导出**：支持将多个表格导出到同一个 Excel 文件的不同 Sheet
- **数据导出模式**：不依赖 DOM，直接支持 JS 数组/对象导出
- **树形数据支持**：自动处理树形结构数据，实现层级缩进
- **智能过滤**：自动检测并排除隐藏的行/列 (`display: none`)
- **容器查找**：自动在容器元素中查找表格
- **Web Worker 支持**：将导出计算移至 Worker 线程，避免主线程阻塞
- **字节生成模式**：`generate_data_bytes` 支持仅生成文件字节而不触发下载，适用于 Worker 场景
- **冻结窗格**：XLSX 导出自动冻结表头行，支持自定义冻结行/列数

## 🚀 快速开始

### 30 秒上手

```html
<!DOCTYPE html>
<html>
  <head>
    <script type="module">
      import init, { export_table, ExportFormat } from "./pkg/belobog_stellar_grid.js";

      // 1. 初始化（只需一次）
      await init();

      // 2. 导出为 CSV（默认）
      document.getElementById("csv-btn").onclick = () => {
        export_table("my-table", "数据.csv");
      };

      // 3. 导出为 Excel
      document.getElementById("xlsx-btn").onclick = () => {
        export_table("my-table", "数据.xlsx", ExportFormat.Xlsx);
      };
    </script>
  </head>
  <body>
    <table id="my-table">
      <tr>
        <th>姓名</th>
        <th>年龄</th>
      </tr>
      <tr>
        <td>张三</td>
        <td>25</td>
      </tr>
    </table>
    <button id="csv-btn">导出 CSV</button>
    <button id="xlsx-btn">导出 Excel</button>
  </body>
</html>
```

就是这么简单！ 🎉

---

### 📦 安装方式

#### NPM 安装（推荐）

```bash
# pnpm（推荐）
pnpm add belobog-stellar-grid

# NPM
npm install belobog-stellar-grid

# Yarn
yarn add belobog-stellar-grid
```

#### 框架集成包（可选）

```bash
# React 封装
pnpm add @bsg-export/react

# Vue 3 封装
pnpm add @bsg-export/vue

# Svelte 封装（兼容 Svelte 4/5）
pnpm add @bsg-export/svelte

# Solid.js 封装
pnpm add @bsg-export/solid

# Web Worker 支持（大数据量场景推荐）
pnpm add @bsg-export/worker

# 仅类型定义
pnpm add -D @bsg-export/types
```

#### CDN 引入（无构建工具场景）

你可以直接通过 `unpkg` 或 `jsDelivr` 等 CDN 服务，在 HTML 中以原生 ES Modules 方式引入，无需安装任何依赖或使用打包工具：

```html
<script type="module">
  // 引入后，模块会自动从同一 CDN 路径拉取 .wasm 文件
  import init, { export_table } from "https://unpkg.com/belobog-stellar-grid@1.0.8/pkg/belobog_stellar_grid.js";
  
  await init();
  export_table("my-table", "导出数据.csv");
</script>
```

#### 从源码构建

```bash
git clone https://github.com/kurisu994/belobog-stellar-grid.git
cd belobog-stellar-grid
wasm-pack build --target web
```

---

### 💻 基本用法

#### 统一导出 API（推荐）

```javascript
import init, { export_table, ExportFormat } from "belobog-stellar-grid";

// 初始化模块（只需执行一次）
await init();

// 导出为 CSV（默认）
export_table("table-id");
export_table("table-id", "销售报表_2024.csv");

// 导出为 Excel
export_table("table-id", "销售报表_2024.xlsx", ExportFormat.Xlsx);
export_table("table-id", "销售报表_2024", ExportFormat.Xlsx); // 自动添加扩展名
```

#### 带进度条的导出

```javascript
import { export_table, ExportFormat } from "belobog-stellar-grid";

// CSV 格式带进度（不排除隐藏行）
export_table("large-table", "大数据.csv", ExportFormat.Csv, false, (progress) => {
  console.log(`进度: ${Math.round(progress)}%`);
  progressBar.style.width = `${progress}%`;
});

// Excel 格式带进度（排除隐藏行）
export_table("large-table", "报表.xlsx", ExportFormat.Xlsx, true, (progress) => {
  console.log(`进度: ${Math.round(progress)}%`);
  progressBar.style.width = `${progress}%`;
});
```

#### 分批异步导出（大数据量）

```javascript
import { export_table_to_csv_batch } from "belobog-stellar-grid";

// 基本用法 - 处理 10,000+ 行数据
await export_table_to_csv_batch("huge-table", "大数据.csv");

// 高级用法 - 自定义配置
await export_table_to_csv_batch(
  "huge-table",
  "百万数据.csv",
  1000, // 每批处理 1000 行
  (progress) => {
    console.log(`进度: ${Math.round(progress)}%`);
  },
);
```

#### 错误处理

```javascript
import { export_table, ExportFormat } from "belobog-stellar-grid";

try {
  export_table("table-id", "报表", ExportFormat.Xlsx);
  alert("✅ 导出成功！");
} catch (error) {
  console.error("导出失败:", error);
  alert("❌ 导出失败: " + error);
}
```

---

### 🎨 完整示例

查看 [examples/](./examples/) 目录获取完整示例：

| 示例                   | 难度                                                   | 描述         |
| ---------------------- | ------------------------------------------------------ | ------------ |
| basic-export.html      | ![简单](https://img.shields.io/badge/难度-简单-green)  | 基础导出示例 |
| progress-export.html   | ![中等](https://img.shields.io/badge/难度-中等-yellow) | 进度显示示例 |
| advanced-features.html | ![进阶](https://img.shields.io/badge/难度-进阶-orange) | 高级特性示例 |
| container-export.html  | ![中等](https://img.shields.io/badge/难度-中等-yellow) | 容器元素导出示例 |
| array-export.html      | ![进阶](https://img.shields.io/badge/难度-进阶-orange) | 数组导出（嵌套表头 + 数据合并）示例 |
| tree-export.html       | ![进阶](https://img.shields.io/badge/难度-进阶-orange) | 树形数据导出（递归拍平 + 层级缩进）示例 |
| multi-sheet-export.html | ![进阶](https://img.shields.io/badge/难度-进阶-orange) | 多工作表导出示例 |
| worker-export.html     | ![高级](https://img.shields.io/badge/难度-高级-red)   | Web Worker 导出（避免主线程阻塞）示例 |
| virtual-scroll-export.html | ![高级](https://img.shields.io/badge/难度-高级-red) | 虚拟滚动导出（百万级数据）示例 |

**运行示例**：

```bash
# 1. 构建项目
wasm-pack build --target web

# 2. 启动本地服务器
cargo install basic-http-server
basic-http-server .

# 3. 访问 http://localhost:4000/examples/
```

---

## 📚 API 参考

> **提示**：该项目包含详细的 API 文档 [API.md](./API.md)。AI 辅助开发请参考 [llms.txt](./llms.txt)。

### 核心函数速览

- **`export_table`**：统一导出函数（推荐）。
- **`export_data`**：JS 数据直接导出，支持树形和复杂表头。
- **`generate_data_bytes`**：生成 CSV/XLSX 文件字节（不触发下载），专为 Web Worker 场景设计。
- **`export_tables_xlsx`**：导出多 Sheet Excel。
- **`export_table_to_csv_batch`**：CSV 异步分批导出。
- **`export_table_to_xlsx_batch`**：XLSX 异步分批导出。
- **`export_tables_to_xlsx_batch`**：多工作表分批异步 XLSX 导出。

更多细节请查阅 [API.md](./API.md)。

---

## 🤖 AI 辅助支持

本项目提供 `llms.txt` 文件，专为 LLM（大语言模型）设计。您可以将其内容作为上下文提供给 AI 助手，以便快速了解库的使用方法和 API。

---

## 🔧 开发指南

### 环境要求

| 工具      | 版本要求    |
| --------- | ----------- |
| Rust      | 1.85.0+     |
| wasm-pack | latest      |
| Node.js   | 16+（可选） |

**安装工具**：

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 安装 wasm-pack
cargo install wasm-pack
```

---

### 常用命令

```bash
# 构建项目
wasm-pack build --target web

# 运行单元测试
cargo test

# 运行 E2E 测试（需先构建 WASM）
cd e2e && npm install && npx playwright install chromium && npx playwright test

# 代码检查
cargo check
cargo fmt
cargo clippy

# 运行示例
basic-http-server .
# 访问: http://localhost:4000/examples/
```

### 发布到 npm

项目支持两种发布方式：**GitHub Actions 自动发布**（推荐）和**本地手动发布**。

#### 前置要求

```bash
# 安装 just
cargo install just

# 安装 cargo-edit（用于版本管理）
cargo install cargo-edit
```

#### 🚀 自动发布（推荐）

使用 GitHub Actions 自动构建和发布，只需一行命令：

```bash
just ci-release patch   # 补丁版本 1.0.0 -> 1.0.1
just ci-release minor   # 次要版本 1.0.0 -> 1.1.0
just ci-release major   # 主要版本 1.0.0 -> 2.0.0
```

该命令会自动：

1. 升级 `Cargo.toml`、`README.md`、`packages/*/package.json` 版本号
2. 更新 `CHANGELOG.md`（将 `[Unreleased]` 替换为版本号 + 日期）
3. 提交更改并打 Git 标签
4. 推送到 GitHub，触发 CI 流水线

**CI 流水线**（`.github/workflows/ci.yml`）触发后执行：

1. **代码质量检查** — `cargo fmt`、`cargo clippy`、`cargo test`
2. **WebAssembly 构建** — `wasm-pack build --target web --release`
3. **浏览器测试** — Puppeteer 自动化验证 WASM 加载和基本功能
4. **发布主包** — 将 `pkg/` 发布到 npm（`belobog-stellar-grid`）
5. **发布子包** — 构建并发布 `@bsg-export/types`、`@bsg-export/react`、`@bsg-export/vue`、`@bsg-export/svelte`、`@bsg-export/solid`、`@bsg-export/worker`
6. **创建 GitHub Release** — 自动提取 CHANGELOG 生成发布说明，附带 WASM 构建产物

> ⚠️ 需要在 GitHub 仓库 Settings → Secrets 中配置 `NPM_TOKEN`

#### 🔧 手动发布

如需本地手动发布：

```bash
# 1. 升级版本
just bump patch

# 2. 构建和优化
just build
just optimize

# 3. 发布前测试
just dry-run

# 4. 发布主包到 npm
just publish latest    # 发布稳定版
just publish beta      # 发布 beta 版

# 5. 构建并发布子包
just build-packages      # 构建 types/react/vue
just publish-packages    # 发布到 npm（默认 latest tag）
```

#### 子包说明

| 子包 | 说明 | 依赖 |
| --- | --- | --- |
| `@bsg-export/types` | 严格 TypeScript 类型定义 | 无 |
| `@bsg-export/react` | React Hook + 组件封装 | `@bsg-export/types` |
| `@bsg-export/vue` | Vue 3 Composable + 组件封装 | `@bsg-export/types` |
| `@bsg-export/svelte` | Svelte Store 封装 + 组件 | `@bsg-export/types` |
| `@bsg-export/solid` | Solid.js Primitive + 组件封装 | `@bsg-export/types` |
| `@bsg-export/worker` | Web Worker 导出封装 | `@bsg-export/types` |

> 子包版本号在 `just bump` 时会自动与主包同步。

#### 其他命令

```bash
just info       # 查看当前版本
just test       # 运行测试
just build      # 构建 WASM
just optimize   # 优化 WASM 文件大小
just e2e        # 运行 E2E 测试（Playwright）
```

### 项目结构

```
belobog-stellar-grid/
├── src/                    # 源代码
│   ├── lib.rs             # 主入口
│   ├── validation.rs      # 文件名验证（含 1 个内联测试）
│   ├── resource.rs        # RAII 资源管理
│   ├── core/              # 核心导出模块组
│   │   ├── mod.rs         # 统一 API 和协调
│   │   ├── table_extractor.rs  # 表格数据提取
│   │   ├── data_export.rs # 数据导出（columns + dataSource，支持嵌套表头、数据合并、树形数据，含 29 个内联测试）
│   │   ├── export_csv.rs  # CSV 导出
│   │   └── export_xlsx.rs # XLSX 导出
│   ├── batch_export.rs    # 异步分批导出（CSV）
│   ├── batch_export_xlsx.rs # 异步分批导出（XLSX）
│   └── utils.rs           # 调试工具（含 2 个内联测试）
├── tests/                 # 单元测试目录（89 个测试）
│   ├── lib_tests.rs       # 基础功能测试（41 个）
│   ├── test_resource.rs   # RAII 资源测试（8 个）
│   ├── test_unified_api.rs # 统一 API 测试（4 个）
│   ├── test_data_export.rs # 数据导出测试（33 个）
│   ├── test_security.rs   # 安全/CSV注入测试（3 个）
│   ├── fixtures/          # 测试夹具
│   └── browser/           # 浏览器测试辅助
├── e2e/                   # E2E 测试目录（Playwright，33 个测试）
│   ├── playwright.config.ts # Playwright 配置
│   └── tests/             # E2E 测试用例
│       ├── basic-export.spec.ts      # 基础导出测试（6 个）
│       ├── array-export.spec.ts      # 数组导出测试（9 个）
│       ├── container-export.spec.ts  # 容器导出测试（5 个）
│       ├── multi-sheet-export.spec.ts # 多工作表测试（3 个）
│       ├── tree-export.spec.ts       # 树形数据测试（7 个）
│       └── wasm-init.spec.ts         # WASM 初始化测试（3 个）
├── examples/              # 示例目录
├── packages/              # 框架集成子包
│   ├── types/             # @bsg-export/types - 严格 TypeScript 类型定义
│   ├── react/             # @bsg-export/react - React Hook + 组件封装
│   ├── vue/               # @bsg-export/vue - Vue 3 Composable + 组件封装
│   ├── svelte/            # @bsg-export/svelte - Svelte Store 封装 + 组件
│   ├── solid/             # @bsg-export/solid - Solid.js Primitive + 组件
│   └── worker/            # @bsg-export/worker - Web Worker 导出封装
├── pkg/                   # WASM 包输出
├── API.md                 # API 详细文档
├── CHANGELOG.md           # 更新日志
├── CODE_REVIEW_REPORT.md  # 代码审查报告
├── CLAUDE.md              # AI 辅助开发指南
├── llms.txt               # LLM 上下文文档
├── justfile               # Just 命令配置
└── README.md             # 项目文档
```

---

## 🚀 性能指标

### 运行时性能

| 数据量       | 同步导出     | 分批异步导出  | 页面响应性       |
| ------------ | ------------ | ------------- | ---------------- |
| 1,000 行     | <10ms        | <10ms         | 无明显差异       |
| 10,000 行    | ~1s（卡顿）  | ~1.2s（流畅） | **大幅改善**     |
| 100,000 行   | ~10s（卡死） | ~12s（流畅）  | **从卡死到可用** |
| 1,000,000 行 | 崩溃         | ~120s（流畅） | **完全解决**     |

### 文件大小

- WASM 原始大小：约 1.3MB（优化后）
- Gzip 压缩后：约 450KB
- Brotli 压缩后：约 400KB

---

## 📌 TODO / 开发路线图

### ✨ 功能增强

- [ ] **Excel 样式定制**: 支持设置字体、颜色、边框、背景色等单元格样式。
- [ ] **条件格式**: 支持根据数据值自动应用颜色、图标集等条件格式规则。
- [x] **冻结窗格**: 支持冻结表头行/列，方便大数据量 Excel 浏览。 ✅
- [ ] **数据验证**: 支持 Excel 下拉列表、数值范围等数据验证规则。
- [ ] **图片导出**: 支持将图片插入到 Excel 单元格中。
- [ ] **数据选择与过滤**: 支持选择性导出特定行或列，以及数据预处理/转换。

### ⚡ 性能优化

- [x] **Web Worker 支持**: 将导出计算移至 Worker 线程，彻底避免主线程阻塞。 ✅
- [x] **E2E 测试体系**: 引入 Playwright 进行端到端测试，覆盖 33 个测试场景。 ✅
- [x] **框架集成库**: 提供 React (`@bsg-export/react`)、Vue 3 (`@bsg-export/vue`)、Svelte (`@bsg-export/svelte`)、Solid.js (`@bsg-export/solid`) 官方封装组件。 ✅
- [x] **严格 TypeScript 类型**: `@bsg-export/types` 提供完整类型安全定义。 ✅
- [ ] **Streaming 导出**: 对超大文件采用流式写入，降低内存峰值占用。
- [ ] **WASM 体积优化**: 探索 `wasm-opt` 更激进的优化策略或按功能拆分 WASM 模块。
- [ ] **性能基准测试**: 建立自动化 Benchmark，持续追踪导出性能回归。

### 🌍 生态扩展

- [ ] **Node.js / 服务端支持**: 探索 `wasm32-wasip1` 或 `wasm32-unknown-unknown` 在非浏览器环境的运行能力。
- [x] **更多框架集成**: 提供 Svelte、Solid.js 等框架的官方封装。✅
- [x] **CDN 分发**: 提供 unpkg / jsDelivr 等 CDN 直接引用方式，简化非构建工具场景的接入。 ✅

---

## 🤝 社区与支持

### 获取帮助

1. 📖 查看文档和示例
2. 🔍 搜索 [Issues](https://github.com/kurisu994/belobog-stellar-grid/issues)
3. 💬 加入 [讨论区](https://github.com/kurisu994/belobog-stellar-grid/discussions)
4. 🐛 报告 [Bug](https://github.com/kurisu994/belobog-stellar-grid/issues/new)

### 贡献方式

我们欢迎各种形式的贡献：

- 🐛 报告 Bug
- 💡 提出新功能
- 📖 改进文档
- 🔧 提交代码
- ⭐ Star 项目

**代码规范**：

- 遵循 Rust 编码规范（`cargo fmt`）
- 通过 Clippy 检查（`cargo clippy`）
- 为新功能添加测试

---

## 📄 许可证

本项目采用双重许可证：

- **[MIT License](LICENSE_MIT)** - 简单宽松
- **[Apache License 2.0](LICENSE_APACHE)** - 更多法律保护

---

## 🙏 致谢

感谢以下项目和社区：

- [Rust](https://www.rust-lang.org/) - 强大的系统编程语言
- [WebAssembly](https://webassembly.org/) - 革命性的 Web 技术
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - Rust 与 JS 的桥梁
- [csv](https://github.com/BurntSushi/rust-csv) - 优秀的 CSV 处理库
- [rust_xlsxwriter](https://github.com/jmcnamara/rust_xlsxwriter) - 高性能的 Excel 写入库
- [Playwright](https://playwright.dev/) - 可靠的端到端测试框架
- 所有贡献者和使用者 ❤️

---
