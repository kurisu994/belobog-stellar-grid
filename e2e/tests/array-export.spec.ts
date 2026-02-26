import { test, expect, type Page, type Download } from '@playwright/test';

/**
 * 数组直接导出测试 (array-export.html)
 *
 * 测试 export_data() 的各种数组导出场景：
 * - 二维数组 CSV / XLSX
 * - CSV 带 BOM
 * - 对象数组 + 表头配置
 * - 嵌套表头
 * - 三级嵌套表头
 * - 数据合并单元格
 * - 错误处理
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

  await page.goto('/examples/array-export.html', { waitUntil: 'networkidle' });
  await page.waitForTimeout(1000);
}

test.describe('数组直接导出 - export_data()', () => {
  test('二维数组导出为 CSV', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 1 的 "📄 导出为 CSV" 按钮
    await page.locator('.section').first().locator('button', { hasText: '导出为 CSV' }).first().click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('用户数据.csv');
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('二维数组导出 CSV 完成'))).toBeTruthy();
  });

  test('二维数组导出为 CSV（带 BOM）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('button', { hasText: '导出为 CSV (带 BOM)' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('用户数据_BOM.csv');
    expect(pageErrors).toHaveLength(0);

    // 验证 BOM 头
    const content = await (await download.createReadStream()).toArray();
    const buffer = Buffer.concat(content);
    expect(buffer[0]).toBe(0xef);
    expect(buffer[1]).toBe(0xbb);
    expect(buffer[2]).toBe(0xbf);
  });

  test('二维数组导出为 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('.section').first().locator('button', { hasText: '导出为 Excel' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('用户数据.xlsx');
    expect(pageErrors).toHaveLength(0);
  });

  test('对象数组 + 表头配置导出为 CSV', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 2 的按钮
    await page.locator('.section').nth(1).locator('button', { hasText: '导出为 CSV' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('用户详情.csv');
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('对象数组导出 CSV 完成'))).toBeTruthy();
  });

  test('对象数组 + 表头配置导出为 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('.section').nth(1).locator('button', { hasText: '导出为 Excel' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('用户详情.xlsx');
    expect(pageErrors).toHaveLength(0);
  });

  test('嵌套表头导出为 Excel（含合并单元格）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 3 的 Excel 按钮
    await page.locator('.section').nth(2).locator('button', { hasText: '导出为 Excel' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('嵌套表头.xlsx');
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('嵌套表头导出'))).toBeTruthy();
  });

  test('三级嵌套表头导出 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('button', { hasText: '导出三级表头 Excel' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('三级表头.xlsx');
    expect(pageErrors).toHaveLength(0);
  });

  test('数据合并单元格导出为 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 5 的 Excel 按钮
    await page.locator('.section').nth(4).locator('button', { hasText: '导出为 Excel' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('数据合并.xlsx');
    expect(pageErrors).toHaveLength(0);
  });

  test('错误处理：对象数组缺少 columns 配置', async ({ page }) => {
    await setupPage(page);

    // 先注册 dialog handler（在点击之前）
    page.on('dialog', async (dialog) => {
      expect(dialog.message()).toContain('成功捕获预期错误');
      await dialog.accept();
    });

    // 点击错误用法按钮
    await page.locator('button', { hasText: '测试错误用法' }).click();

    await page.waitForTimeout(1000);

    // 验证 console 中捕获了预期错误
    expect(consoleLogs.some((log) => log.includes('捕获到预期错误'))).toBeTruthy();
  });
});
