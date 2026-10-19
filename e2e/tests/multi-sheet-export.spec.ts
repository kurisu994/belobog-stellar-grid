import { test, expect, type Page, type Download } from '@playwright/test';
import { readDownload, readXlsxEntry } from './helpers/download';

/**
 * 多工作表导出测试 (multi-sheet-export.html)
 *
 * 测试 export_tables_xlsx() 的多表合并导出：
 * - 同步多表导出（含隐藏行排除）
 * - 内部 tbody 的异步分批导出
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

  await page.goto('/examples/multi-sheet-export.html', { waitUntil: 'networkidle' });
  await page.waitForFunction(() => typeof Reflect.get(window, 'exportSync') === 'function');
}

test.describe('多工作表导出 - export_tables_xlsx()', () => {
  test('页面显示两个数据表格', async ({ page }) => {
    await setupPage(page);

    // 验证订单表格
    const ordersTable = page.locator('#orders-table');
    await expect(ordersTable).toBeVisible();

    // 验证库存表格
    const inventoryTable = page.locator('#inventory-table');
    await expect(inventoryTable).toBeVisible();
  });

  test('同步多表导出为 Excel', async ({ page }) => {
    await setupPage(page);

    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-sync-export').click();
    const download: Download = await downloadPromise;

    expect(download.suggestedFilename()).toBe('多工作表报表.xlsx');
    expect(pageErrors).toHaveLength(0);

    // 验证日志区域显示导出成功
    const logArea = page.locator('#log');
    await expect(logArea).toContainText('导出成功');
  });

  test('异步分批导出完整内部 tbody，不重复或丢失数据', async ({ page }) => {
    await setupPage(page);
    await page.locator('#btn-generate-data').click();
    await expect(page.locator('#btn-async-export')).toBeEnabled();

    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-async-export').click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toBe('大数据多表导出.xlsx');
    const archive = await readDownload(download);
    for (const sheet of ['sheet1.xml', 'sheet2.xml']) {
      const worksheet = readXlsxEntry(archive, `xl/worksheets/${sheet}`);
      // 每个工作表应有 1 行表头 + 5000 行数据。
      expect(worksheet.match(/<row\b/g)).toHaveLength(5001);
      expect(worksheet).toContain('ref="A1:D5001"');
    }
    await expect(page.locator('#log')).toContainText('异步导出完成');
    expect(pageErrors).toHaveLength(0);
  });

  test('订单表格包含隐藏行', async ({ page }) => {
    await setupPage(page);

    // 验证隐藏行存在但不可见
    const hiddenRow = page.locator('#orders-table tbody tr[style*="display: none"]');
    await expect(hiddenRow).toHaveCount(1);

    // 验证可见行数（表头 2 行 + 数据 3 行可见 + 1 行隐藏）
    const visibleDataRows = page.locator('#orders-table tbody tr:visible');
    await expect(visibleDataRows).toHaveCount(3);
  });
});
