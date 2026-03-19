import { test, expect, type Page, type Download } from '@playwright/test';

/**
 * Excel 样式定制导出测试 (style-export.html)
 *
 * 测试 export_data() 和 export_table() 的样式导出功能：
 * - 全局样式（headerStyle / cellStyle）
 * - 列级样式（column.style / column.headerStyle / column.width）
 * - 单元格样式（{ value, style }）
 * - 三级样式叠加
 * - DOM 表格 + 样式导出
 */

let pageErrors: string[];
let consoleLogs: string[];

async function setupPage(page: Page) {
  pageErrors = [];
  consoleLogs = [];

  page.on('pageerror', (error) => {
    pageErrors.push(error.message);
  });

  page.on('console', (msg) => {
    consoleLogs.push(msg.text());
  });

  await page.goto('/examples/style-export.html', { waitUntil: 'networkidle' });
  await page.waitForTimeout(1000);
}

test.describe('Excel 样式定制导出', () => {
  test('WASM 模块加载成功', async ({ page }) => {
    await setupPage(page);

    const status = page.locator('#status');
    await expect(status).toContainText('WASM 模块已加载');
    expect(pageErrors).toHaveLength(0);
  });

  test('全局样式导出为 XLSX', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-global-style-xlsx').click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('全局样式导出成功'))).toBeTruthy();
  });

  test('列级样式导出为 XLSX', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-column-style-xlsx').click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('列级样式导出成功'))).toBeTruthy();
  });

  test('单元格样式导出为 XLSX', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-cell-style-xlsx').click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('单元格样式导出成功'))).toBeTruthy();
  });

  test('三级样式叠加导出为 XLSX', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-combined-style-xlsx').click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('三级样式叠加导出成功'))).toBeTruthy();
  });

  test('DOM 表格样式导出为 XLSX', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-dom-style-xlsx').click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('DOM 表格样式导出成功'))).toBeTruthy();
  });

  test('导出的 XLSX 文件大小合理', async ({ page }) => {
    await setupPage(page);

    // 全局样式导出 — 验证文件不为空
    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-global-style-xlsx').click();
    const download: Download = await downloadPromise;

    const path = await download.path();
    expect(path).toBeTruthy();

    // XLSX 文件至少有几百字节（ZIP 头 + 样式信息）
    const fs = require('fs');
    const stats = fs.statSync(path);
    expect(stats.size).toBeGreaterThan(500);
  });
});
