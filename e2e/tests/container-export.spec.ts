import { test, expect, type Page, type Download } from '@playwright/test';

/**
 * 容器内表格导出测试 (container-export.html)
 *
 * 测试 export_table() 在容器元素上的使用：
 * - ID 在 section 容器上（Ant Design 风格）
 * - ID 在 div 容器上
 * - 自动查找内部 <table> 元素
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

  await page.goto('/examples/container-export.html', { waitUntil: 'networkidle' });
  await page.waitForTimeout(1000);
}

test.describe('容器内表格导出', () => {
  test('页面正确显示 Ant Design 风格容器表格', async ({ page }) => {
    await setupPage(page);

    // 验证 section 容器存在
    const antdContainer = page.locator('#antd-table');
    await expect(antdContainer).toBeVisible();

    // 验证容器内有表格数据
    const rows = antdContainer.locator('tbody tr');
    await expect(rows).toHaveCount(3);
  });

  test('从 Ant Design 容器导出 CSV', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('.section').first().getByRole('button', { name: '导出为 CSV' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('用户列表.csv');
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('从 Ant Design 容器导出 CSV 完成'))).toBeTruthy();
  });

  test('从 Ant Design 容器导出 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('.section').first().getByRole('button', { name: '导出为 Excel' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
  });

  test('从 div 容器导出 CSV', async ({ page }) => {
    await setupPage(page);

    // 验证 div 容器存在
    const wrapperContainer = page.locator('#wrapper-table');
    await expect(wrapperContainer).toBeVisible();

    const downloadPromise = page.waitForEvent('download');
    await page.locator('.section').nth(1).getByRole('button', { name: '导出为 CSV' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('产品列表.csv');
    expect(pageErrors).toHaveLength(0);
  });

  test('从 div 容器导出 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('.section').nth(1).getByRole('button', { name: '导出为 Excel' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toMatch(/\.xlsx$/);
    expect(pageErrors).toHaveLength(0);
    expect(consoleLogs.some((log) => log.includes('从 div 容器导出 Excel 完成'))).toBeTruthy();
  });
});
