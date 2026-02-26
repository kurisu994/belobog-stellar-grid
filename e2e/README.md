# E2E 测试

基于 [Playwright](https://playwright.dev/) 的端到端测试，验证 WebAssembly 导出功能在真实浏览器中的运行情况。

## 环境要求

- **Node.js 22+**（项目使用 [fnm](https://github.com/Schniz/fnm) 管理，`.node-version` 已配置）
- **WASM 包**：运行前需先构建 `wasm-pack build --target web`
- **basic-http-server**：测试自动启动本地服务器（端口 4000）

## 快速开始

```bash
# 从项目根目录运行（自动构建 WASM + 执行测试）
just e2e

# 带浏览器界面运行
just e2e-headed
```

### 手动运行

```bash
# 1. 构建 WASM
wasm-pack build --target web --out-dir pkg

# 2. 安装依赖
cd e2e && pnpm install

# 3. 安装浏览器
npx playwright install chromium

# 4. 运行测试
npx playwright test

# 5. 查看报告
npx playwright show-report
```

## 测试文件

| 文件 | 测试内容 | 数量 |
|------|---------|------|
| `wasm-init.spec.ts` | WASM 初始化 & 页面加载 | 8 |
| `basic-export.spec.ts` | DOM 表格导出 (`export_table`) | 5 |
| `array-export.spec.ts` | 数组直接导出 (`export_data`) | 9 |
| `container-export.spec.ts` | 容器内表格导出 | 5 |
| `multi-sheet-export.spec.ts` | 多工作表导出 (`export_tables_xlsx`) | 3 |
| `tree-export.spec.ts` | 树形数据导出 | 7 |
| **总计** | | **37 + 1 skip** |

## 测试策略

- ✅ 验证页面加载和 WASM 初始化无错误
- ✅ 验证导出按钮点击后触发文件下载
- ✅ 验证下载文件名正确
- ✅ 验证 CSV BOM 头（`EF BB BF`）
- ✅ 验证 console 日志输出
- ✅ 验证错误处理（如缺少 `columns` 配置）
- ⏭️ `advanced-features.html` 因引用不存在的导出而跳过

## 配置

- **`playwright.config.ts`** — Playwright 配置（Chromium、端口 4000、重试策略）
- **`.node-version`** — Node 22（fnm 自动切换）
- **`tsconfig.json`** — TypeScript 配置
