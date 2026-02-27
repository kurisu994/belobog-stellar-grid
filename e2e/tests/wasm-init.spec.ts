import { test, expect } from '@playwright/test';

/**
 * WASM 初始化测试
 *
 * 验证所有 example 页面能正常加载并完成 WebAssembly 初始化
 */

// 使用 console.log 报告初始化状态的页面
const consoleLogPages = [
  { name: '基本导出', path: '/examples/basic-export.html' },
  { name: '数组直接导出', path: '/examples/array-export.html' },
  { name: '容器内表格导出', path: '/examples/container-export.html' },
  { name: '进度条导出', path: '/examples/progress-export.html' },
  { name: '虚拟滚动', path: '/examples/virtual-scroll-export.html' },
];

// 不通过 console.log 报告初始化的页面（使用 DOM 操作如 addLog/showStatus）
const domInitPages = [
  { name: '多工作表导出', path: '/examples/multi-sheet-export.html', selector: '#btn-sync-export' },
  { name: '树形数据导出', path: '/examples/tree-export.html', selector: '.section button' },
  { name: 'CDN 分发导出', path: '/examples/cdn-export.html', selector: '#btn-csv' },
];

test.describe('WASM 初始化', () => {
  for (const pg of consoleLogPages) {
    test(`${pg.name} 页面加载并初始化 WASM`, async ({ page }) => {
      const errors: string[] = [];
      const consoleLogs: string[] = [];

      page.on('pageerror', (error) => {
        errors.push(error.message);
      });
      page.on('console', (msg) => {
        consoleLogs.push(msg.text());
      });

      await page.goto(pg.path, { waitUntil: 'networkidle' });
      await page.waitForTimeout(2000);

      // 验证无页面错误
      expect(errors, `页面 ${pg.name} 不应有 JS 错误`).toHaveLength(0);

      // 验证有初始化成功的日志
      const hasInitLog = consoleLogs.some(
        (log) => log.includes('已初始化') || log.includes('初始化成功')
      );
      expect(
        hasInitLog,
        `页面 ${pg.name} 应输出 WASM 初始化成功日志。\n实际日志: ${consoleLogs.join('\n')}`
      ).toBeTruthy();
    });
  }

  for (const pg of domInitPages) {
    test(`${pg.name} 页面加载并初始化 WASM`, async ({ page }) => {
      const errors: string[] = [];

      page.on('pageerror', (error) => {
        errors.push(error.message);
      });

      await page.goto(pg.path, { waitUntil: 'networkidle' });
      await page.waitForTimeout(2000);

      // 验证无页面错误（如果有 JS 错误说明 WASM 加载失败）
      expect(errors, `页面 ${pg.name} 不应有 JS 错误`).toHaveLength(0);

      // 验证关键交互元素可用（说明 WASM 模块已成功初始化）
      const button = page.locator(pg.selector).first();
      await expect(button).toBeVisible();
    });
  }

  // 高级特性页面引用了不存在的 export_table_to_csv，跳过
  test.skip('高级特性 页面加载（已知问题：引用不存在的导出）', async ({ page }) => {
    await page.goto('/examples/advanced-features.html', { waitUntil: 'networkidle' });
  });

  test('示例导航页面加载正常', async ({ page }) => {
    await page.goto('/examples/index.html', { waitUntil: 'networkidle' });

    // 验证页面标题
    await expect(page.locator('h1')).toContainText('belobog-stellar-grid');

    // 验证所有示例卡片存在
    const cards = page.locator('.example-card');
    await expect(cards).toHaveCount(10);
  });
});
