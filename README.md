<div align="center">

  <h1><code>belobog-stellar-grid</code></h1>

  <p><strong>🦀 现代化的 WebAssembly 表格导出库</strong></p>

  <p>一个安全、高效、易用的 Rust WebAssembly 库，用于将 HTML 表格导出为 CSV 和 XLSX 文件</p>

  <p>
    <img src="https://img.shields.io/badge/version-1.0.0-blue.svg" alt="Version" />
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
- **✅ 100% 测试覆盖**：47 个单元测试确保代码质量
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

#### 🚀 性能优化

- **零拷贝操作**：直接操作 DOM，参数使用 `&str` 引用
- **分批异步处理**：支持百万级数据导出，避免页面卡死
- **wee_alloc 优化**：使用轻量级分配器减小文件体积
- **LTO 优化**：链接时优化减少最终 WASM 大小
- **实时进度反馈**：支持大型表格的进度回调

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

// CSV 格式带进度
export_table("large-table", "大数据.csv", ExportFormat.Csv, (progress) => {
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

### 核心函数

#### `export_table(table_id, filename?, format?, progress_callback?)` ✅ 推荐

统一的表格导出函数，支持 CSV 和 XLSX 格式。

**参数**：

- `table_id`: 表格元素的 ID
- `filename`: 导出文件名（可选）
- `format`: 导出格式（可选，默认 CSV）
- `progress_callback`: 进度回调函数（可选）

**示例**：

```javascript
import { export_table, ExportFormat } from "belobog-stellar-grid";

// 最简单的用法
export_table("my-table");

// 指定文件名和格式
export_table("my-table", "报表.xlsx", ExportFormat.Xlsx);

// 带进度回调
export_table("large-table", "大数据", ExportFormat.Csv, (progress) => {
  console.log(`进度: ${progress.toFixed(1)}%`);
});
```

---

#### `export_table_to_csv_batch(table_id, tbody_id?, filename?, batch_size?, callback?)` 🔧 向后兼容

分批异步导出函数，专为大数据量设计。

**适用场景**：10,000+ 行数据

**参数**：

- `table_id`: 表格元素的 ID
- `tbody_id`: 可选的数据表格体 ID
- `filename`: 导出文件名（可选）
- `batch_size`: 每批处理的行数（可选，默认 1000）
- `callback`: 进度回调函数（可选）

---

### 文件名安全验证

所有导出函数都会自动验证文件名安全性：

| ✅ 允许              | ❌ 禁止          |
| -------------------- | ---------------- |
| `report_2024-12.csv` | `../etc/passwd`  |
| `数据导出.csv`       | `file<name>.csv` |
| `sales.data.csv`     | `CON.csv`        |
| `测试-文件.xlsx`     | `.hidden`        |

---

## 🔧 开发指南

### 环境要求

| 工具      | 版本要求    |
| --------- | ----------- |
| Rust      | 1.82+       |
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

# 运行测试
cargo test

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
│   │   ├── export_csv.rs  # CSV 导出
│   │   └── export_xlsx.rs # XLSX 导出
│   ├── batch_export.rs    # 异步分批导出
│   └── utils.rs           # 调试工具
├── tests/                 # 测试目录（47 个测试）
│   ├── lib_tests.rs       # 基础功能测试（35 个）
│   ├── test_resource.rs   # RAII 资源测试（8 个）
│   └── test_unified_api.rs # 统一 API 测试（4 个）
├── examples/              # 示例目录
├── pkg/                   # WASM 包输出
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

- WASM 原始大小：约 117KB（优化后）
- Gzip 压缩后：约 40KB
- Brotli 压缩后：约 35KB

---

## 📌 TODO / 开发路线图

以下是待改进的功能点：

### 数据类型处理

- [ ] 支持数据类型自动检测（数字、日期、布尔值等）
- [ ] 导出时将数字类型写入 Excel 数值单元格而非文本
- [ ] 支持 Excel 公式导出

### 样式与格式化

- [ ] 支持基础样式设置（字体、颜色、边框、背景色）
- [ ] 支持列宽自动调整
- [ ] 支持表头行特殊样式（加粗、背景色）
- [ ] 支持条件格式（数据条、图标集等）

### 表格结构

- [ ] 导出时保留合并单元格状态
- [ ] 支持多工作表导出（多表格导出到同一 Excel 文件的不同 sheet）
- [ ] 支持检测并排除隐藏行/列（`display: none`）

### 数据选择与过滤

- [ ] 支持选择性导出特定行或列
- [ ] 支持导出前数据预处理/转换
- [ ] 支持自定义列名映射

### 其他

- [ ] 支持从 JavaScript 数组直接生成文件（不依赖 DOM）
- [ ] CSV 导出添加 BOM 头选项（兼容旧版 Excel）
- [ ] 探索 Node.js/服务端支持可能性

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
