# 📚 belobog-stellar-grid 使用示例

这个目录包含了 belobog-stellar-grid 的完整使用示例。每个示例都是一个独立的 HTML 文件，展示了库的不同功能。

## 🚀 快速开始

### 前置要求

1. 构建 WASM 包：

```bash
wasm-pack build --target web
```

2. 启动本地服务器：

```bash
# 推荐：使用 basic-http-server（Rust 实现，快速简单）
# 如果尚未安装，使用以下命令安装：
cargo install basic-http-server

# 启动服务器（默认端口 4000）
basic-http-server .

# 或指定端口
basic-http-server . -a 127.0.0.1:8000

# 其他备选方案：
# 使用 Python
python -m http.server 8000

# 使用 Node.js http-server
npx http-server .
```

3. 在浏览器中打开示例文件

## 📋 示例清单

### 1. basic-export.html - 基本导出示例

**功能**：

- ✅ 使用默认文件名导出
- ✅ 使用自定义文件名导出
- ✅ 使用带日期的文件名导出

**适用场景**：

- 快速上手
- 简单的表格导出需求
- 学习基本 API 用法

**关键代码**：

```javascript
import init, { export_table_to_csv } from "../pkg/belobog_stellar_grid.js";

await init();
export_table_to_csv("table-id", "文件名.csv");
```

---

### 2. progress-export.html - 进度条导出示例

**功能**：

- ✅ 大数据集导出
- ✅ 实时进度显示
- ✅ CSV 分批异步导出
- ✅ **XLSX 分批异步导出** 🆕
- ✅ 导出性能统计
- ✅ 操作日志记录

**适用场景**：

- 导出大型表格（100+ 行）
- 需要用户反馈的场景
- 性能监控和优化

**关键代码**：

```javascript
import init, { export_table_to_csv_batch, export_table_to_xlsx_batch } from "../pkg/belobog_stellar_grid.js";

await init();

// 导出 CSV
await export_table_to_csv_batch(
  "table-id",
  "tbody-id", // 可选的 tbody ID
  "文件名.csv",
  1000, // 每批 1000 行
  (progress) => {
    console.log(`进度: ${progress}%`);
  },
);

// 导出 Excel
await export_table_to_xlsx_batch("table-id", "tbody-id", "文件名.xlsx", 1000, (progress) => {
  console.log(`进度: ${progress}%`);
});
```

---

### 3. advanced-features.html - 高级特性示例

**功能**：

- ✅ 自定义文件名
- ✅ 文件名安全验证演示
- ✅ 批量导出多个表格
- ✅ 错误处理演示

**适用场景**：

- 复杂的导出需求
- 需要批量处理
- 学习错误处理最佳实践

**关键代码**：

```javascript
// 批量导出
const tables = ["table-1", "table-2", "table-3"];
for (const tableId of tables) {
  export_table_to_csv(tableId, `${tableId}.csv`);
  await new Promise((resolve) => setTimeout(resolve, 100));
}
```

---

### 4. virtual-scroll-export.html - 虚拟滚动示例 🚀

**功能**：

- ✅ 支持百万级数据渲染
- ✅ 虚拟滚动技术（高性能）
- ✅ 智能选择渲染模式
- ✅ Web Worker 异步生成数据
- ✅ 实时性能统计

**适用场景**：

- 超大数据集（10000+ 行）
- 需要极致性能的场景
- 百万级数据导出
- 性能对比演示

**性能对比**：

- 传统渲染 100 万行：~60 秒，内存 ~1GB，可能崩溃
- 虚拟滚动 100 万行：<0.1 秒，内存 ~50MB，丝滑流畅

**关键代码**：

```javascript
// 虚拟滚动只渲染可见区域
class VirtualScrollRenderer {
  render() {
    const start = Math.floor(scrollTop / ROW_HEIGHT);
    const end = Math.ceil((scrollTop + containerHeight) / ROW_HEIGHT);
    // 只渲染可见行 + 缓冲区
    for (let i = start; i < end; i++) {
      // 创建行...
    }
  }
}
```

**技术亮点**：

- 📊 只渲染 20-30 个可见 DOM 节点
- 💾 内存占用减少 95%
- ⚡ 渲染速度提升 600 倍
- 🎯 自动选择最佳渲染策略（< 10000 行直接渲染，≥ 10000 行虚拟滚动）

---

### 5. container-export.html - 容器内表格导出示例 🏗️

**功能**：

- ✅ 支持传入容器 ID（如 div, section）
- ✅ 自动查找容器内的 table 元素
- ✅ 兼容 Ant Design、Element Plus 等 UI 框架结构

**适用场景**：

- 使用 UI 组件库（如 Ant Design Table）
- 无法直接获取内部 table ID 的情况
- 嵌套结构的表格导出

**关键代码**：

```javascript
// table-container 是包裹 table 的 div 的 ID
export_table("table-container", "AntDesign表格.xlsx", ExportFormat.Xlsx);
```

---

### 6. array-export.html - 数组直接导出示例 📦

**功能**：

- ✅ 不依赖 DOM，直接从 JS 数组导出
- ✅ 支持 API 数据直接导出
- ✅ 支持多种数据类型（字符串、数字、布尔值）
- ✅ 支持嵌套表头（多级分组列）
- ✅ 支持数据合并单元格（colSpan / rowSpan）
- ✅ 带进度回调

**适用场景**：

- 后端 API 返回纯 JSON 数据
- 页面无表格展示但需要导出
- 数据经过前端处理/计算后的导出

**关键代码**：

```javascript
import init, { export_data, ExportFormat } from "../pkg/belobog_stellar_grid.js";

await init();

// 模式 1：二维数组直接导出
const data = [
  ['姓名', '年龄', '部门'],
  ['张三', 28, '研发部'],
  ['李四', 32, '市场部']
];
export_data(data, { filename: "员工名单.xlsx", format: ExportFormat.Xlsx });

// 模式 2：columns + dataSource（Ant Design 风格）
const columns = [
  { title: '姓名', key: 'name' },
  { title: '其他', children: [
    { title: '年龄', key: 'age' },
    { title: '住址', key: 'address' }
  ]}
];
const dataSource = [
  { name: '张三', age: 28, address: '杭州' },
  { name: '李四', age: 32, address: '北京' }
];
export_data(dataSource, { columns, filename: "员工信息.xlsx", format: ExportFormat.Xlsx });
```

---

### 7. tree-export.html - 树形数据导出示例 🌳

**功能**：

- ✅ 递归拍平嵌套 children 结构
- ✅ 可选层级缩进（指定列自动添加空格）
- ✅ 自定义 children 字段名
- ✅ 支持嵌套表头 + 树形数据组合
- ✅ 大数据量树形导出

**适用场景**：

- 组织架构导出
- 商品分类目录导出
- 菜单/权限树导出
- 任何包含父子关系的层级数据

**关键代码**：

```javascript
import init, { export_data, ExportFormat } from "../pkg/belobog_stellar_grid.js";

await init();

const treeData = [
  {
    name: 'CEO', title: 'CEO',
    children: [
      { name: 'CTO', title: 'CTO' },
      { name: 'CFO', title: 'CFO',
        children: [
          { name: '会计', title: '会计' }
        ]
      }
    ]
  }
];

const columns = [
  { title: '姓名', key: 'name' },
  { title: '职位', key: 'title' }
];

// 基本导出（传入 childrenKey 启用树形模式）
export_data(treeData, { columns, filename: '组织架构.xlsx', format: ExportFormat.Xlsx, childrenKey: 'children' });

// 带层级缩进导出（'name' 列会根据层级自动添加空格）
export_data(treeData, { columns, filename: '组织架构.xlsx', format: ExportFormat.Xlsx, indentColumn: 'name', childrenKey: 'children' });

// 自定义 children 字段名
export_data(data, { columns, filename: 'file.xlsx', format: ExportFormat.Xlsx, indentColumn: 'name', childrenKey: 'subCategories' });
```

## 🎯 使用指南

### 基本使用流程

1. **初始化 WASM 模块**

```javascript
import init from "../pkg/belobog_stellar_grid.js";
await init();
```

2. **导出表格**

```javascript
import { export_table_to_csv } from "../pkg/belobog_stellar_grid.js";
export_table_to_csv("your-table-id", "文件名");
```

3. **错误处理**

```javascript
try {
  export_table_to_csv("table-id", "文件名");
} catch (error) {
  console.error("导出失败:", error);
}
```

### 进阶使用

#### 带进度回调

```javascript
import { export_table_to_csv_with_progress } from "../pkg/belobog_stellar_grid.js";

export_table_to_csv_with_progress("large-table", "data.csv", (progress) => {
  // 更新 UI
  document.getElementById("progress").textContent = `${Math.round(progress)}%`;
});
```

#### 动态文件名

```javascript
const today = new Date().toISOString().split("T")[0];
const filename = `数据_${today}.csv`;
export_table_to_csv("table-id", filename);
```

#### 批量导出

```javascript
async function exportAll() {
  const tables = [
    { id: "sales", name: "销售数据" },
    { id: "products", name: "产品信息" },
    { id: "customers", name: "客户列表" },
  ];

  for (const table of tables) {
    export_table_to_csv(table.id, table.name);
    await new Promise((r) => setTimeout(r, 200)); // 避免浏览器限制
  }
}
```

## 🛡️ 文件名验证规则

库会自动验证文件名的安全性：

### ✅ 允许的字符

- 字母数字：`A-Z a-z 0-9`
- 下划线和连字符：`_ -`
- 点号：`.`（不在开头和结尾）
- 空格（不在开头和结尾）
- Unicode 字符（中文、日文、韩文等）

### ❌ 禁止的字符

- 路径分隔符：`/ \`
- 危险字符：`< > : " | ? *`
- Windows 保留名称：`CON PRN AUX NUL COM1-9 LPT1-9`

### 📏 长度限制

- 最小：1 字符
- 最大：255 字符

## 🔧 故障排除

### 问题：WASM 模块加载失败

**解决方案**：

1. 确保使用 HTTP 服务器（不是 file://）
2. 检查 pkg/ 目录是否存在
3. 确认路径正确（例如：`../pkg/belobog_stellar_grid.js`）

### 问题：找不到表格元素

**解决方案**：

1. 检查表格 ID 是否正确
2. 确保 DOM 已加载完成
3. 使用浏览器开发工具检查元素

### 问题：文件名验证失败

**解决方案**：

1. 检查文件名是否包含非法字符
2. 确认文件名长度不超过 255 字符
3. 避免使用 Windows 保留名称

### 问题：下载被浏览器阻止

**解决方案**：

1. 检查浏览器的下载设置
2. 允许该网站的弹出窗口和下载
3. 批量下载时添加延迟

## 📖 API 参考

### export_table_to_csv(table_id, filename?)

导出 HTML 表格为 CSV 文件。

**参数**：

- `table_id` (string): 表格元素的 ID
- `filename` (string, 可选): 导出文件名，默认 "table_export.csv"

**返回**：无（成功）或抛出异常（失败）

---

### export_table_to_csv_with_progress(table_id, filename?, callback?)

带进度回调的表格导出。

**参数**：

- `table_id` (string): 表格元素的 ID
- `filename` (string, 可选): 导出文件名
- `callback` (function, 可选): 进度回调函数，接收 0-100 的进度值

**返回**：无（成功）或抛出异常（失败）

---

### export_table_to_csv_batch(table_id, tbody_id?, filename?, batch_size?, callback?)

分批异步导出表格为 CSV，适用于大数据量导出。

**参数**：

- `table_id` (string): 表格元素的 ID
- `tbody_id` (string, 可选): 数据 tbody 的 ID，用于分离表头和数据
- `filename` (string, 可选): 导出文件名
- `batch_size` (number, 可选): 每批处理的行数，默认 1000
- `callback` (function, 可选): 进度回调函数

**返回**：Promise<void>

---

### export_table_to_xlsx_batch(table_id, tbody_id?, filename?, batch_size?, callback?) 🆕

分批异步导出表格为 Excel，解决大数据量 Excel 导出卡死问题。

**参数**：

- `table_id` (string): 表格元素的 ID
- `tbody_id` (string, 可选): 数据 tbody 的 ID
- `filename` (string, 可选): 导出文件名，默认 "table_export.xlsx"
- `batch_size` (number, 可选): 每批处理的行数，默认 1000
- `callback` (function, 可选): 进度回调函数，接收 0-100 的进度值

**特性**：

- 采用两阶段策略：分批读取 DOM (80%) + 同步生成 XLSX (20%)
- 每批处理后让出控制权给浏览器，保持页面响应性

**返回**：Promise<void>

---

### export_data(data, options?)

从 JavaScript 数组直接生成文件，不依赖 DOM。支持二维数组、对象数组、嵌套表头、数据合并和树形数据导出。

**参数**：

- `data` (Array): 数据源。无 columns 时为二维数组；有 columns 时为对象数组
- `options` (Object, 可选): 导出配置对象
  - `columns` (Array, 可选): 表头配置数组，支持嵌套 children
  - `filename` (string, 可选): 导出文件名
  - `format` (ExportFormat, 可选): 导出格式（Csv / Xlsx）
  - `progressCallback` (function, 可选): 进度回调函数
  - `indentColumn` (string, 可选): 树形数据模式下，指定需要缩进的列的 key
  - `childrenKey` (string, 可选): 传入此参数启用树形数据模式，指定子节点字段名（如 `"children"`）

**返回**：无（成功）或抛出异常（失败）

## 🌐 浏览器兼容性

| 浏览器  | 最低版本 | 说明        |
| ------- | -------- | ----------- |
| Chrome  | 90+      | ✅ 完全支持 |
| Firefox | 88+      | ✅ 完全支持 |
| Safari  | 14+      | ✅ 完全支持 |
| Edge    | 90+      | ✅ 完全支持 |

## 💡 最佳实践

1. **始终进行错误处理**

```javascript
try {
  export_table_to_csv("table-id", "filename");
} catch (error) {
  // 向用户显示友好的错误信息
  alert("导出失败，请重试");
}
```

2. **大表格使用进度回调**

```javascript
// 对于 > 100 行的表格，使用进度版本
if (rowCount > 100) {
  export_table_to_csv_with_progress(id, name, updateProgress);
} else {
  export_table_to_csv(id, name);
}
```

3. **批量导出添加延迟**

```javascript
// 避免浏览器下载限制
for (const table of tables) {
  export_table_to_csv(table.id, table.name);
  await new Promise((r) => setTimeout(r, 200));
}
```

4. **使用有意义的文件名**

```javascript
const date = new Date().toISOString().split("T")[0];
const filename = `${reportType}_${date}.csv`;
```

## 🤝 贡献

发现示例中的问题或有改进建议？欢迎提交 Issue 或 Pull Request！

## 📄 许可证

与主项目相同：MIT OR Apache-2.0

---

**需要帮助？**

- 查看主项目 [README](../README.md)
- 提交 [Issue](https://github.com/kurisu994/belobog-stellar-grid/issues)
- 阅读 [API 文档](../EXAMPLES.md)
