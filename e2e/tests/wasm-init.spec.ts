import { test, expect } from '@playwright/test';
import { resolve } from 'node:path';

/** 验证示例真正完成 WASM 初始化，而不是只检查静态按钮可见。 */
const consoleLogPages = [
  { name: '基本导出', path: '/examples/basic-export.html' },
  { name: '数组直接导出', path: '/examples/array-export.html' },
  { name: '容器内表格导出', path: '/examples/container-export.html' },
  { name: '进度条导出', path: '/examples/progress-export.html' },
  { name: '虚拟滚动', path: '/examples/virtual-scroll-export.html' },
  { name: '高级特性', path: '/examples/advanced-features.html' },
  { name: 'Excel 预览', path: '/examples/excel-preview.html' },
  { name: '样式导出', path: '/examples/style-export.html' },
  { name: 'Worker 导出', path: '/examples/worker-export.html' },
  { name: '性能基准', path: '/examples/benchmark.html' },
];

const downloadInitPages = [
  {
    name: '多工作表导出',
    path: '/examples/multi-sheet-export.html',
    selector: '#btn-sync-export',
    status: '#log',
    success: 'WASM 模块初始化成功',
    filename: '多工作表报表.xlsx',
  },
  {
    name: '树形数据导出',
    path: '/examples/tree-export.html',
    selector: '.section button',
    status: '#status1',
    success: '导出成功',
    filename: '组织架构.csv',
  },
];

test.describe('WASM 初始化', () => {
  for (const pg of consoleLogPages) {
    test(`${pg.name} 页面加载并初始化 WASM`, async ({ page }) => {
      const errors: string[] = [];
      const consoleLogs: string[] = [];
      page.on('pageerror', (error) => errors.push(error.message));
      page.on('console', (msg) => {
        if (msg.type() === 'log') consoleLogs.push(msg.text());
      });

      await page.goto(pg.path, { waitUntil: 'domcontentloaded' });
      await expect.poll(
        () => consoleLogs.some((log) => log.includes('已初始化') || log.includes('初始化成功')),
        { message: `页面 ${pg.name} 应输出 WASM 初始化成功日志` },
      ).toBe(true);
      expect(errors, `页面 ${pg.name} 不应有 JS 错误`).toHaveLength(0);
    });
  }

  for (const pg of downloadInitPages) {
    test(`${pg.name} 页面加载并初始化 WASM`, async ({ page }) => {
      const errors: string[] = [];
      page.on('pageerror', (error) => errors.push(error.message));
      await page.goto(pg.path, { waitUntil: 'domcontentloaded' });

      // 树形页面在 await init() 后才注册处理函数；多表页面在第一次导出时初始化。
      const button = page.locator(pg.selector).first();
      await expect.poll(() => button.evaluate((element) => {
        const handler = element.getAttribute('onclick')?.match(/^(\w+)\(/)?.[1];
        return !!handler && typeof Reflect.get(window, handler) === 'function';
      })).toBe(true);
      const downloadPromise = page.waitForEvent('download');
      await button.click();
      const download = await downloadPromise;
      expect(download.suggestedFilename()).toBe(pg.filename);
      expect(await download.failure()).toBeNull();
      await expect(page.locator(pg.status)).toContainText(pg.success);
      expect(errors, `页面 ${pg.name} 不应有 JS 错误`).toHaveLength(0);
    });
  }

  test('CDN 分发导出使用包根路径并初始化 WASM', async ({ page }) => {
    const errors: string[] = [];
    const requestedFiles: string[] = [];
    page.on('pageerror', (error) => errors.push(error.message));
    // 保留真实 CDN URL 及相对 WASM 路径解析，只将传输替换为本地构建产物，避免外网波动。
    await page.route('https://unpkg.com/belobog-stellar-grid@1.1.9/**', async (route) => {
      const filename = new URL(route.request().url()).pathname.replace('/belobog-stellar-grid@1.1.9/', '');
      requestedFiles.push(filename);
      if (!['belobog_stellar_grid.js', 'belobog_stellar_grid_bg.wasm'].includes(filename)) {
        await route.abort();
        return;
      }
      await route.fulfill({
        path: resolve(__dirname, '../../pkg', filename),
        contentType: filename.endsWith('.wasm') ? 'application/wasm' : 'application/javascript',
        headers: { 'Access-Control-Allow-Origin': '*' },
      });
    });
    await page.goto('/examples/cdn-export.html', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('#status-msg .success')).toContainText('WASM 初始化成功');
    await expect(page.locator('#btn-csv')).toBeEnabled();
    await expect(page.locator('#btn-xlsx')).toBeEnabled();
    const downloadPromise = page.waitForEvent('download');
    await page.locator('#btn-csv').click();
    const download = await downloadPromise;
    expect(download.suggestedFilename()).toBe('CDN_示例数据.csv');
    expect(await download.failure()).toBeNull();
    expect(requestedFiles).toEqual(['belobog_stellar_grid.js', 'belobog_stellar_grid_bg.wasm']);
    expect(errors).toHaveLength(0);
  });

  test('流式导出页面初始化 WASM', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (error) => errors.push(error.message));
    await page.goto('/examples/streaming-export.html', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('#log')).toContainText('WASM 模块初始化成功');
    expect(errors).toHaveLength(0);
  });

  test('示例导航页面加载正常', async ({ page }) => {
    await page.goto('/examples/index.html', { waitUntil: 'domcontentloaded' });
    await expect(page.locator('h1')).toContainText('belobog-stellar-grid');

    // 不绑定卡片数量；检查核心入口、链接唯一性和每个目标确实存在。
    const cards = page.locator('a.example-card');
    const links = await cards.evaluateAll((elements) => elements.map((element) => element.getAttribute('href')));
    expect(links).toEqual(expect.arrayContaining(['basic-export.html', 'advanced-features.html', 'excel-preview.html']));
    expect(new Set(links).size).toBe(links.length);
    for (const link of links) {
      expect(link).toMatch(/^[\w-]+\.html$/);
      const response = await page.request.get(`/examples/${link}`);
      expect(response.ok(), `示例入口 ${link} 应可访问`).toBe(true);
    }
  });
});
