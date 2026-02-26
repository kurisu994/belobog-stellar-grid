import { test, expect, type Page, type Download } from '@playwright/test';

/**
 * 树形数据导出测试 (tree-export.html)
 *
 * 测试 export_data() 的树形数据导出：
 * - 组织架构 CSV / XLSX 导出
 * - 带缩进导出 (indentColumn + childrenKey)
 * - 自定义 childrenKey 导出
 * - 嵌套表头 + 树形数据组合导出
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

  await page.goto('/examples/tree-export.html', { waitUntil: 'networkidle' });
  await page.waitForTimeout(1000);
}

test.describe('树形数据导出 - export_data() 树形模式', () => {
  test('组织架构导出为 CSV', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 1 的 "📄 导出 CSV" 按钮
    await page.locator('.section').first().locator('button', { hasText: '导出 CSV' }).first().click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('组织架构.csv');
    expect(pageErrors).toHaveLength(0);

    // 验证状态提示
    const status = page.locator('#status1');
    await expect(status).toContainText('导出成功');
  });

  test('组织架构导出为 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 1 的 "📗 导出 Excel" 按钮
    await page.locator('.section').first().locator('button', { hasText: '导出 Excel' }).first().click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('组织架构.xlsx');
    expect(pageErrors).toHaveLength(0);
  });

  test('组织架构导出为 CSV（带缩进）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('button', { hasText: 'CSV (带缩进)' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('组织架构_缩进.csv');
    expect(pageErrors).toHaveLength(0);

    // 验证 CSV 内容包含 CEO 数据
    const content = await (await download.createReadStream()).toArray();
    const text = Buffer.concat(content).toString('utf-8');
    expect(text).toContain('CEO');
  });

  test('组织架构导出为 Excel（带缩进）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('.section').first().locator('button', { hasText: 'Excel (带缩进)' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('组织架构_缩进.xlsx');
    expect(pageErrors).toHaveLength(0);
  });

  test('商品分类导出（自定义 childrenKey: subCategories）', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 2 的 "📗 导出 Excel（带缩进）" 按钮
    await page.locator('.section').nth(1).locator('button', { hasText: '导出 Excel（带缩进）' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('商品分类.xlsx');
    expect(pageErrors).toHaveLength(0);

    const status = page.locator('#status2');
    await expect(status).toContainText('导出成功');
  });

  test('嵌套表头 + 树形数据组合导出', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 3 按钮
    await page.locator('button', { hasText: '导出 Excel（嵌套表头 + 树形数据）' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('部门架构.xlsx');
    expect(pageErrors).toHaveLength(0);

    const status = page.locator('#status3');
    await expect(status).toContainText('导出成功');
  });

  test('大数据量树形结构导出', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    // 场景 4 按钮
    await page.locator('button', { hasText: '导出 Excel（500+ 节点）' }).click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('大数据量树.xlsx');
    expect(pageErrors).toHaveLength(0);

    const status = page.locator('#status4');
    await expect(status).toContainText('导出成功');
  });
});
