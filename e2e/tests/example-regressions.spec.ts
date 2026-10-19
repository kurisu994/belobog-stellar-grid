import { test, expect } from '@playwright/test';
import { readDownload } from './helpers/download';

test.describe('示例导出回归', () => {
  for (const rowCount of [10, 50000]) {
    test(`虚拟滚动页面导出完整 ${rowCount} 行 CSV`, async ({ page }) => {
      const errors: string[] = [];
      page.on('pageerror', (error) => errors.push(error.message));
      await page.goto('/examples/virtual-scroll-export.html', { waitUntil: 'domcontentloaded' });
      await expect(page.locator('#render-mode')).toHaveText('虚拟');
      await expect(page.locator('#loading-overlay')).not.toHaveClass(/show/);

      if (rowCount === 10) {
        await page.locator('#row-input').fill(String(rowCount));
        await page.getByRole('button', { name: '重新生成' }).click();
        await expect(page.locator('#render-mode')).toHaveText('直接');
        await expect(page.locator('#table-body tr')).toHaveCount(rowCount);
      } else {
        // 滚动到末尾，确保导出的不是当前可见 DOM 片段。
        expect(await page.locator('#table-body tr').count()).toBeLessThan(rowCount);
        await page.locator('#tbody-container').evaluate((element) => {
          element.scrollTop = element.scrollHeight;
        });
        await expect(page.locator('#table-body tr').last().locator('td').first()).toHaveText('P50000');
      }

      const downloadPromise = page.waitForEvent('download');
      await page.locator('#export-btn').click();
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toMatch(/^虚拟滚动数据_\d{4}-\d{2}-\d{2}\.csv$/);
      const lines = (await readDownload(download)).toString('utf8').trimEnd().split(/\r?\n/);
      expect(lines[0]).toBe('ID,产品名称,类别,价格,库存,销量');
      expect(lines).toHaveLength(rowCount + 1);
      // 数据生成器的 ID 连续，逐行验证可发现缺行、重复行和虚拟窗口截断。
      expect(lines.slice(1).every((line, index) => {
        const id = `P${String(index + 1).padStart(4, '0')}`;
        return line.startsWith(`${id},`) && line.split(',').length === 6;
      })).toBe(true);
      expect(errors).toHaveLength(0);
    });
  }

  test('高级特性使用公开 API 导出真实 CSV 内容', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.goto('/examples/advanced-features.html', { waitUntil: 'domcontentloaded' });
    await page.locator('#custom-filename').fill('销售报表_风堇');
    const downloadPromise = page.waitForEvent('download');
    await page.getByRole('button', { name: '导出', exact: true }).click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toBe('销售报表_风堇.csv');
    const lines = (await readDownload(download)).toString('utf8').trimEnd().split(/\r?\n/);
    expect(lines).toEqual([
      '月份,销售额,增长率',
      '1月,"¥120,000",\'+15%',
      '2月,"¥135,000",\'+12.5%',
      '3月,"¥142,000",\'+5.2%',
    ]);
    await expect(page.locator('#message-1')).toContainText('导出成功');
    expect(errors).toHaveLength(0);
  });

  test('文件名验证将恶意输入作为纯文本展示', async ({ page }) => {
    const dialogs: string[] = [];
    page.on('dialog', async (dialog) => {
      dialogs.push(dialog.message());
      await dialog.dismiss();
    });
    await page.goto('/examples/advanced-features.html', { waitUntil: 'domcontentloaded' });
    const filename = '<img src=x onerror=alert(1)>风堇';
    await page.locator('#test-filename').fill(filename);
    await page.getByRole('button', { name: '验证文件名' }).click();
    await expect(page.locator('#validation-result strong').first()).toHaveText(`文件名: "${filename}"`);
    await expect(page.locator('#validation-result img')).toHaveCount(0);
    await expect(page.locator('#validation-result')).toContainText('验证失败');
    expect(dialogs).toHaveLength(0);
  });

  test('Excel 工作表名作为文本渲染，保留独立行列数 span', async ({ page }) => {
    const errors: string[] = [];
    const dialogs: string[] = [];
    page.on('pageerror', (error) => errors.push(error.message));
    page.on('dialog', async (dialog) => {
      dialogs.push(dialog.message());
      await dialog.dismiss();
    });
    await page.goto('/examples/basic-export.html', { waitUntil: 'domcontentloaded' });
    await page.waitForFunction(() => typeof Reflect.get(window, 'exportDefault') === 'function');

    // 使用真实 WASM 导出含恶意工作表名的 XLSX，再走文件上传与解析的完整路径。
    const sheetNames = ['普通工作表_风堇', '<img src=x onerror=alert(1)>'];
    const downloadPromise = page.waitForEvent('download');
    await page.evaluate(async (names) => {
      const moduleUrl = '/pkg/belobog_stellar_grid.js';
      const { export_tables_xlsx } = await import(moduleUrl);
      export_tables_xlsx(names.map((sheetName) => ({ tableId: 'employees-table', sheetName })), '工作表名回归.xlsx');
    }, sheetNames);
    const archive = await readDownload(await downloadPromise);

    await page.goto('/examples/excel-preview.html', { waitUntil: 'domcontentloaded' });
    await page.locator('#file-input').setInputFiles({
      name: '工作表名回归.xlsx',
      mimeType: 'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
      buffer: archive,
    });
    const tabs = page.locator('#sheet-tabs .sheet-tab');
    await expect(tabs).toHaveCount(sheetNames.length);
    for (const [index, name] of sheetNames.entries()) {
      await expect(tabs.nth(index)).toHaveText(`${name} (6行×5列)`);
      await expect(tabs.nth(index).locator('span.row-count')).toHaveText('(6行×5列)');
    }
    await expect(page.locator('#sheet-tabs img, #sheet-tabs script')).toHaveCount(0);
    await tabs.nth(1).click();
    await expect(page.locator('#html-preview')).toContainText('孙七');
    await expect(page.locator('#error-box')).not.toHaveClass(/show/);
    expect(dialogs).toHaveLength(0);
    expect(errors).toHaveLength(0);
  });
});
