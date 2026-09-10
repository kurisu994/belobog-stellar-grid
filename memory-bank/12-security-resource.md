---
name: 12-security-resource
description: 文件名校验、CSV/Excel 注入防护与 Blob URL 生命周期管理
paths:
  - "src/validation.rs"
  - "src/resource.rs"
---

# 安全与资源 (Security & Resources)

> 改 `src/validation.rs` / `src/resource.rs`，或新增任何"下载文件"能力前先读这里。

## 不可动摇的顺序

```
校验文件名  →  创建 Blob  →  创建 Object URL  →  <a download> 点击  →  延迟 revoke
```

**先校验再建 URL**。反过来的话，非法文件名提前 return 时 URL 已经创建，必然泄漏。
所有下载都走 `resource.rs` 的两个入口，不要各自复制 DOM 锚点逻辑：

```rust
resource::trigger_bytes_download(bytes, mime, filename, default_name, ext)
resource::trigger_blob_download(parts, mime, filename, default_name, ext)
```

## 文件名规则（`src/validation.rs`）

| 规则 | 内容 |
| ---- | ---- |
| 禁止字符 | 路径分隔符、控制字符、`< > : " \| ? *` |
| Windows 保留名 | `CON` / `PRN` / `AUX` / `NUL` / `COM1-9` / `LPT1-9` 等 |
| 首尾字符 | 不允许首尾点号、空格；拒绝全角点号 |
| 长度上限 | 255 字节 |
| 扩展名 | `ensure_extension` 按导出格式补齐 |

- `validate_filename` — 纯校验，返回 `Result<(), JsValue>`
- `prepare_download_filename` — 校验 + 补默认值，下载路径的预检入口，**在任何 Blob/URL 操作之前调用**
- 非法输入的错误消息一律中文、面向最终用户

## Blob URL 生命周期（`src/resource.rs`）

- `UrlGuard` — RAII 持有 Object URL，Drop 时 revoke，异常路径也不漏
- `schedule_url_revoke` — 下载点击后延迟 **10 秒** revoke：既能避免下载竞态，又不会长期泄漏
- `withBom=true` 时 CSV 前置 UTF-8 BOM，便于 Excel 正确识别中文（校验时先剥离 BOM 再判断）

## 公式注入防护

`escape_csv_injection` 处理 CSV，XLSX 侧则**一律 `write_string`**，绝不写公式：

| 场景 | 处理 |
| ---- | ---- |
| CSV 字段以 `= + - @ \t \r` 开头 | 前缀 `'`，先剥离前导 BOM 再判断 |
| XLSX 单元格 | `write_string`，Excel 不会求值 |
| 数字格式 | 不解析公式，按文本/数值语义写入 |

## 预览 HTML 净化

`src/core/html_builder.rs` 输出到 `innerHTML`，必须：

- 文本做 HTML 实体转义
- `style` 属性做值净化（过滤 `;{}<>`）
- 字体名同样过滤 `;{}<>`

> 预览侧的实现细节见 `11-excel-preview.md`。

## 负向约束（❌ 不要这么做）

- ❌ **不要先创建 Blob URL 再校验文件名**——错误路径会泄漏 URL
- ❌ **不要在导出路径用 `panic!` / `unwrap()` / `expect()`**——公开 API 返回 `Result<T, JsValue>`
- ❌ **不要用 `write_formula` 或任何会让 Excel 求值的写入**——一律 `write_string`
- ❌ **不要按字节长度对 UTF-8 字符串分支**（`"中".len() == 3` 会误入 3 字符分支导致越界）
- ❌ **不要绕过 `UrlGuard` 自己持有 Object URL**——Drop 语义是泄漏防线
- ❌ **不要把隐藏行列跳过逻辑写在安全模块里**——`display:none` / xlsx `hidden="1"` 的处理属于提取层与预览层
