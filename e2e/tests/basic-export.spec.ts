import { test, expect, type Page, type Download } from '@playwright/test';

/**
 * 基础导出测试 (basic-export.html)
 *
 * 测试 export_table() 的各种导出场景：
 * - CSV 默认导出
 * - CSV 自定义文件名
 * - CSV 带 BOM
 * - Excel 导出
 * - 排除隐藏行导出
 */

let pageErrors: string[];
let consoleLogs: string[];

/**
 * 设置页面监听器并导航到基础导出页面
 */
async function setupPage(page: Page) {
  pageErrors = [];
  consoleLogs = [];

  page.on('pageerror', (error) => {
    pageErrors.push(error.message);
  });

  page.on('console', (msg) => {
    consoleLogs.push(msg.text());
  });

  await page.goto('/examples/basic-export.html', { waitUntil: 'networkidle' });

  // 等待 WASM 初始化
  await page.waitForFunction(() => {
    return document.querySelector('#employees-table') !== null;
  });
  await page.waitForTimeout(1000);
}

test.describe('基础导出 - export_table()', () => {
  test('表格数据正确渲染', async ({ page }) => {
    await setupPage(page);

    // 验证表格存在且有数据
    const rows = page.locator('#employees-table tbody tr');
    await expect(rows).toHaveCount(5);

    // 验证第一行数据
    const firstRow = rows.first();
    await expect(firstRow.locator('td').nth(0)).toHaveText('张三');
    await expect(firstRow.locator('td').nth(1)).toHaveText('技术部');
  });

  test('导出为 CSV（默认文件名）', async ({ page }) => {
    await setupPage(page);

    // 监听下载事件
    const downloadPromise = page.waitForEvent('download');
    await page.getByRole('button', { name: '导出为 CSV（默认）' }).click();
    const download: Download = await downloadPromise;

    // 验证下载文件名
    expect(download.suggestedFilename()).toMatch(/\.csv$/);

    // 验证无错误
    expect(pageErrors).toHaveLength(0);

    // 验证 console 日志
    expect(consoleLogs.some((log) => log.includes('导出为 CSV 完成'))).toBeTruthy();
  });

  test('导出为 CSV（自定义文件名）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.getByRole('button', { name: '导出为 CSV（自定义文件名）' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('员工信息表.csv');
    expect(pageErrors).toHaveLength(0);
  });

  test('导出为 CSV（带 BOM）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.getByRole('button', { name: '导出为 CSV（带 BOM）' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('员工信息表_BOM.csv');
    expect(pageErrors).toHaveLength(0);

    // 验证文件内容以 BOM 开头
    const content = await (await download.createReadStream()).toArray();
    const buffer = Buffer.concat(content);
    // UTF-8 BOM: EF BB BF
    expect(buffer[0]).toBe(0xef);
    expect(buffer[1]).toBe(0xbb);
    expect(buffer[2]).toBe(0xbf);
  });

  test('导出为 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 使用精确匹配，避免匹配到新增的带配置的 "导出为 Excel (...)" 按钮
    await page.getByRole('button', { name: '📗 导出为 Excel', exact: true }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
  });

  test('导出可见数据（排除隐藏行）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.getByRole('button', { name: '导出可见数据' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('已隐藏第3行数据'))).toBeTruthy();
  });

  test('导出为 Excel（带冻结配置）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 使用精准的名称匹配
    await page.getByRole('button', { name: '导出为 Excel (冻结前2行和首列)' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('员工信息表(冻结).xlsx');
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('导出为 Excel (带冻结配置) 完成'))).toBeTruthy();
  });
});

