<div align="center">

  <h1><code>belobog-stellar-grid</code></h1>

  <p><strong>🦀 现代化的 WebAssembly 表格导出库</strong></p>

  <p>一个安全、高效、易用的 Rust WebAssembly 库，用于将 HTML 表格导出为 CSV 和 XLSX 文件</p>

  <p>
    <img src="https://img.shields.io/badge/version-1.0.3-blue.svg" alt="Version" />
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
- **✅ 100% 测试覆盖**：100 个单元测试确保代码质量
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

### 核心函数

#### `export_table(table_id, filename?, format?, exclude_hidden?, progress_callback?)` ✅ 推荐

统一的表格导出函数，支持 CSV 和 XLSX 格式。

**参数**：

- `table_id`: 表格元素的 ID
- `filename`: 导出文件名（可选）
- `format`: 导出格式（可选，默认 CSV）
- `exclude_hidden`: 是否排除隐藏的行和列（可选，默认 false）
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

#### `export_table_to_xlsx_batch(table_id, tbody_id?, filename?, batch_size?, callback?)` 🆕

XLSX 格式的分批异步导出函数，解决大数据量 Excel 导出卡死问题。

**适用场景**：10,000+ 行数据的 Excel 导出

**参数**：

- `table_id`: 表格元素的 ID
- `tbody_id`: 可选的数据表格体 ID
- `filename`: 导出文件名（可选）
- `batch_size`: 每批处理的行数（可选，默认 1000）
- `callback`: 进度回调函数（可选）

**特性**：
- 采用两阶段策略：分批读取 DOM (80%) + 同步生成 XLSX (20%)
- 每批处理后让出控制权给浏览器，保持页面响应性

---

#### `export_tables_xlsx(sheets, filename?, progress_callback?)` 🆕 多工作表

将多个 HTML 表格导出到同一个 Excel 文件的不同工作表中。

**参数**：

- `sheets`: 配置数组 `Array<{ tableId: string, sheetName?: string, excludeHidden?: boolean }>`
- `filename`: 导出文件名（可选）
- `progress_callback`: 进度回调函数（可选）

**示例**：

```javascript
import { export_tables_xlsx } from "belobog-stellar-grid";

export_tables_xlsx(
  [
    { tableId: "summary-table", sheetName: "汇总", excludeHidden: true },
    { tableId: "details-table", sheetName: "详情" }
  ],
  "年度报表.xlsx"
);
```

---

#### `export_tables_to_xlsx_batch(sheets, filename?, batch_size?, callback?)` 🆕 多工作表(异步)

多工作表导出的异步分批版本，适用于包含大数据量的多个表格。

**参数**：

- `sheets`: 配置数组 `Array<{ tableId: string, tbodyId?: string, sheetName?: string, excludeHidden?: boolean }>`
- `filename`: 导出文件名（可选）
- `batch_size`: 每批处理的行数（可选，默认 1000）
- `callback`: 进度回调函数（可选）

---

#### `export_data(data, options?)` 🆕 直接数据导出

不依赖 DOM，直接将 JavaScript 二维数组或对象数组导出为 CSV 或 XLSX 文件。支持嵌套表头、数据合并和树形数据导出。

**参数**：

- `data`: JS 数组（二维数组 `Array<Array<any>>` 或对象数组 `Array<Object>`）
- `options`: 可选配置对象，包含以下字段：
  - `columns`: 表头配置数组（导出对象数组时必填），支持嵌套 `children` 实现多级表头
  - `filename`: 导出文件名
  - `format`: 导出格式（默认 CSV）
  - `progressCallback`: 进度回调函数
  - `indentColumn`: 树形模式下，需要缩进的列的 key（如 `"name"`）
  - `childrenKey`: 传入此参数启用树形数据模式，指定子节点字段名（如 `"children"`）

**示例**：

```javascript
import { export_data, ExportFormat } from "belobog-stellar-grid";

// 1. 二维数组导出
const data = [
  ["姓名", "年龄", "城市"],
  ["张三", 28, "北京"],
  ["李四", 35, "上海"]
];
export_data(data, { filename: "用户列表.csv" });

// 2. 对象数组 + 表头配置
const columns = [
  { title: "姓名", key: "name" },
  { title: "年龄", key: "age" }
];
const objData = [
  { name: "张三", age: 28 },
  { name: "李四", age: 35 }
];
export_data(objData, { columns, filename: "用户.xlsx", format: ExportFormat.Xlsx });

// 3. 嵌套表头（多行表头 + 合并单元格）
const nestedColumns = [
  { title: "姓名", key: "name" },
  { title: "其他", children: [
    { title: "年龄", key: "age" },
    { title: "住址", key: "address" }
  ]}
];
export_data(objData, { columns: nestedColumns, filename: "报表.xlsx", format: ExportFormat.Xlsx });

// 4. 数据合并单元格（colSpan / rowSpan）
const mergeData = [
  { name: { value: "张三", rowSpan: 2 }, subject: "数学", score: 90 },
  { name: { value: "", rowSpan: 0 }, subject: "英语", score: 85 },
  { name: "李四", subject: "数学", score: 95 },
];
export_data(mergeData, { columns, filename: "合并.xlsx", format: ExportFormat.Xlsx });

// 5. 树形数据导出（传入 childrenKey 启用树形模式）
const treeData = [
  {
    name: 'CEO', title: 'CEO',
    children: [
      { name: 'CTO', title: 'CTO' },
      { name: 'CFO', title: 'CFO',
        children: [{ name: '会计', title: '会计' }]
      }
    ]
  }
];
// 带层级缩进（name 列根据层级自动添加空格）
export_data(treeData, { columns, filename: '组织架构.xlsx', format: ExportFormat.Xlsx, indentColumn: 'name', childrenKey: 'children' });

// 自定义 children 字段名
export_data(data, { columns, filename: 'file.xlsx', format: ExportFormat.Xlsx, indentColumn: 'name', childrenKey: 'subCategories' });
```

**数据合并单元格说明**：

当数据对象中的值为 `{ value, colSpan?, rowSpan? }` 格式时，自动处理合并：

| 属性 | 说明 |
|------|------|
| `value` | 单元格显示的值 |
| `colSpan` | 横向合并列数（默认 1，设为 0 表示被左侧合并覆盖） |
| `rowSpan` | 纵向合并行数（默认 1，设为 0 表示被上方合并覆盖） |

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
│   │   ├── data_export.rs # 数据导出（columns + dataSource，支持嵌套表头、数据合并、树形数据）
│   │   ├── export_csv.rs  # CSV 导出
│   │   └── export_xlsx.rs # XLSX 导出
│   ├── batch_export.rs    # 异步分批导出
│   └── utils.rs           # 调试工具
├── tests/                 # 测试目录（97 个测试）
│   ├── lib_tests.rs       # 基础功能测试（41 个）
│   ├── test_resource.rs   # RAII 资源测试（8 个）
│   ├── test_unified_api.rs # 统一 API 测试（4 个）
│   └── test_data_export.rs # 数据导出测试（33 个）
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

### 表格结构

- [x] 导出时保留合并单元格状态（支持 colspan 和 rowspan）
- [x] 支持多工作表导出（多表格导出到同一 Excel 文件的不同 sheet）
- [x] 支持检测并排除隐藏行/列（`display: none`）
- [x] 支持容器元素查找（自动在 `div` 等容器中查找 `table`）

### 数据选择与过滤

- [ ] 支持选择性导出特定行或列
- [ ] 支持导出前数据预处理/转换
- [ ] 支持自定义列名映射

### 其他

- [x] 支持从 JavaScript 数组直接生成文件（不依赖 DOM）
- [x] 支持数据区域合并单元格（colSpan / rowSpan）
- [x] 支持树形数据导出（递归拍平 children + 层级缩进）
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
