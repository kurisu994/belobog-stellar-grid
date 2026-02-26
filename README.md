<div align="center">

  <h1><code>belobog-stellar-grid</code></h1>

  <p><strong>🦀 现代化的 WebAssembly 表格导出库</strong></p>

  <p>一个安全、高效、易用的 Rust WebAssembly 库，用于将 HTML 表格导出为 CSV 和 XLSX 文件</p>

  <p>
    <img src="https://img.shields.io/badge/version-1.0.5-blue.svg" alt="Version" />
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
- **📦 轻量级**：约 117KB 的 WASM 文件（gzip 后约 40KB）
- **✅ 100% 测试覆盖**：103 个单元测试 + 20 个 E2E 测试确保代码质量
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
- **`export_tables_xlsx`**：导出多 Sheet Excel。
- **`export_table_to_csv_batch`**：CSV 异步分批导出。
- **`export_table_to_xlsx_batch`**：XLSX 异步分批导出。

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

1. 升级版本号
2. 提交更改并打标签
3. 推送到 GitHub
4. 触发 GitHub Actions 自动发布到 npm

> ⚠️ 需要在 GitHub 仓库设置 `NPM_TOKEN` secret

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

# 4. 发布到 npm
just publish latest    # 发布稳定版
just publish beta      # 发布 beta 版
```

#### 其他命令

```bash
just info       # 查看当前版本
just test       # 运行测试
just build      # 构建 WASM
just optimize   # 优化 WASM 文件大小
```

### 项目结构

```
belobog-stellar-grid/
├── src/                    # 源代码
│   ├── lib.rs             # 主入口
│   ├── validation.rs      # 文件名验证
│   ├── resource.rs        # RAII 资源管理
│   ├── core/              # 核心导出模块组
│   │   ├── mod.rs         # 统一 API 和协调
│   │   ├── table_extractor.rs  # 表格数据提取
│   │   ├── data_export.rs # 数据导出（columns + dataSource，支持嵌套表头、数据合并、树形数据）
│   │   ├── export_csv.rs  # CSV 导出
│   │   └── export_xlsx.rs # XLSX 导出
│   ├── batch_export.rs    # 异步分批导出（CSV）
│   ├── batch_export_xlsx.rs # 异步分批导出（XLSX）
│   └── utils.rs           # 调试工具
├── tests/                 # 单元测试目录（103 个测试）
│   ├── lib_tests.rs       # 基础功能测试（41 个）
│   ├── test_resource.rs   # RAII 资源测试（8 个）
│   ├── test_unified_api.rs # 统一 API 测试（4 个）
│   ├── test_data_export.rs # 数据导出测试（33 个）
│   └── test_security.rs   # 安全/CSV注入测试（3 个）
├── e2e/                   # E2E 测试目录（Playwright，20 个测试）
│   ├── playwright.config.ts # Playwright 配置
│   └── tests/             # E2E 测试用例
│       ├── e2e-test-page.html   # 测试页面
│       ├── basic-export.spec.ts # 基础导出测试（10 个）
│       └── data-export.spec.ts  # 数据导出测试（10 个）
├── examples/              # 示例目录
├── pkg/                   # WASM 包输出
├── API.md                 # API 详细文档
├── llms.txt               # LLM 上下文文档
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

以下是待改进的功能点：

### 🛡️ 安全与稳定性 (优先)

- [x] **修复 XLSX 公式注入漏洞**: 默认禁用公式自动执行，统一使用 `write_string` 写入。
- [x] **递归深度限制**: 树形数据和嵌套表头添加 `MAX_DEPTH=64` 限制，防止栈溢出。
- [x] **移除 Panic**: 将 `yield_to_browser()` 中的 `expect()` 替换为优雅的错误处理。
- [x] **控制字符检测**: 文件名验证新增 ASCII 控制字符（0x00-0x1F）检测。
- [x] **数组类型验证**: `parse_sheet_configs` 新增 `Array::is_array()` 验证。

### 📊 功能一致性

- [ ] **多表导出严格进度回调**: `export_tables_xlsx` 函数添加 `strict_progress_callback` 参数支持。
- [ ] **批量导出严格进度回调**: 三个批量导出函数添加 `strict_progress_callback` 参数支持。
- [x] **Format 参数浮点数验证**: 增强 `parse_export_data_options` 中 format 参数验证，拒绝非整数输入。

### ✨ 新特性

- [ ] **Excel 样式定制**: 支持设置字体、颜色、边框、背景色等。
- [ ] **图片导出**: 支持将图片插入到 Excel 单元格中。
- [x] **CSV BOM 支持**: 为 CSV 添加 UTF-8 BOM 头，解决 Windows Excel 乱码问题。
- [ ] **数据选择与过滤**: 支持选择性导出特定行或列，以及数据预处理。

### 💻 开发体验 (DX)

- [ ] **框架集成库**: 提供 `@belobog/react`、`@belobog/vue` 等官方封装组件。
- [ ] **类型定义优化**: 提供更严格的 TypeScript 类型定义。

### 🛠️ 工程化与重构

- [x] **DOM 提取逻辑重构**: 消除 DOM 遍历代码的重复逻辑。提取 `resolve_table`、`process_row_cells` 等共用函数，4 个调用方统一使用。
- [ ] **Node.js / 服务端支持**: 探索在非浏览器环境下的运行能力。
- [x] **E2E 测试**: 引入 Playwright 进行端到端测试，覆盖 WASM 初始化、CSV/XLSX 导出、数据导出、错误处理等 20 个测试用例。

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

- **[MIT License](LICENSE-MIT)** - 简单宽松
- **[Apache License 2.0](LICENSE-APACHE)** - 更多法律保护

---

## 🙏 致谢

感谢以下项目和社区：

- [Rust](https://www.rust-lang.org/) - 强大的系统编程语言
- [WebAssembly](https://webassembly.org/) - 革命性的 Web 技术
- [wasm-bindgen](https://github.com/rustwasm/wasm-bindgen) - Rust 与 JS 的桥梁
- [csv](https://github.com/BurntSushi/rust-csv) - 优秀的 CSV 处理库
- 所有贡献者和使用者 ❤️

---
