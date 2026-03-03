import { test, expect, type Page } from '@playwright/test';

/**
 * E2E 性能基准测试 (benchmark.html)
 *
 * 测试端到端导出性能（含 WASM 初始化 + DOM/数据解析 + 文件生成）。
 * 每个场景运行多次取中位数，并设置合理的阈值断言。
 *
 * 场景：
 * - WASM 初始化耗时
 * - DOM 表格 CSV/XLSX 导出（1000 行 × 10 列）
 * - 纯数据 CSV/XLSX 导出（1000 行对象数组）
 * - 树形数据 XLSX 导出（3 层 ~584 节点）
 */

/** benchmark 运行结果 */
interface BenchmarkResult {
  median: number;
  min: number;
  max: number;
  times: number[];
}

/** 性能阈值（毫秒），对 CI 环境适当放宽 */
const THRESHOLDS = {
  wasmInit: 3000,
  domCsv: 3000,
  domXlsx: 5000,
  dataCsv: 3000,
  dataXlsx: 5000,
  treeXlsx: 5000,
};

let pageErrors: string[];
let consoleLogs: string[];

/**
 * 设置 benchmark 页面：监听错误/日志，导航并等待 WASM 初始化完成
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

  await page.goto('/examples/benchmark.html', { waitUntil: 'networkidle' });

  // 等待 WASM 初始化完成（页面会输出 "Benchmark 页面已初始化"）
  await page.waitForFunction(
    () => typeof (window as any).benchmarkDomCsv === 'function',
    { timeout: 15000 }
  );
  // 额外等待确保 DOM 表格渲染完成
  await page.waitForTimeout(500);
}

test.describe('性能基准测试', () => {
  test.describe.configure({ mode: 'serial' });

  let sharedPage: Page;

  test.beforeAll(async ({ browser }) => {
    sharedPage = await browser.newPage();
    await setupPage(sharedPage);
    // 验证页面无 JS 错误
    expect(pageErrors, '页面不应有 JS 错误').toHaveLength(0);
  });

  test.afterAll(async () => {
    // 输出完整 benchmark 结果汇总
    const results = await sharedPage.evaluate(() => (window as any).getAllBenchmarkResults());
    console.log('\n📊 Benchmark 结果汇总:');
    console.log(JSON.stringify(results, null, 2));
    await sharedPage.close();
  });

  test('WASM 初始化耗时', async () => {
    const result = await sharedPage.evaluate(() => (window as any).getWasmInitTime());
    const elapsed = result.elapsed as number;

    console.log(`⏱️  WASM 初始化: ${elapsed.toFixed(1)}ms`);
    expect(elapsed, `WASM 初始化应在 ${THRESHOLDS.wasmInit}ms 内完成`).toBeLessThan(
      THRESHOLDS.wasmInit
    );
  });

  test('DOM 表格 CSV 导出 (1000行)', async () => {
    // DOM 导出会触发下载，需要监听 download 事件
    const downloadPromise = sharedPage.waitForEvent('download');
    const result: BenchmarkResult = await sharedPage.evaluate(() =>
      (window as any).benchmarkDomCsv()
    );
    await downloadPromise;

    console.log(
      `⏱️  DOM CSV: 中位数 ${result.median.toFixed(1)}ms (${result.min.toFixed(1)}~${result.max.toFixed(1)}ms)`
    );
    expect(result.median, `DOM CSV 导出应在 ${THRESHOLDS.domCsv}ms 内完成`).toBeLessThan(
      THRESHOLDS.domCsv
    );
  });

  test('DOM 表格 XLSX 导出 (1000行)', async () => {
    const downloadPromise = sharedPage.waitForEvent('download');
    const result: BenchmarkResult = await sharedPage.evaluate(() =>
      (window as any).benchmarkDomXlsx()
    );
    await downloadPromise;

    console.log(
      `⏱️  DOM XLSX: 中位数 ${result.median.toFixed(1)}ms (${result.min.toFixed(1)}~${result.max.toFixed(1)}ms)`
    );
    expect(result.median, `DOM XLSX 导出应在 ${THRESHOLDS.domXlsx}ms 内完成`).toBeLessThan(
      THRESHOLDS.domXlsx
    );
  });

  test('纯数据 CSV 导出 (1000行)', async () => {
    const result: BenchmarkResult = await sharedPage.evaluate(() =>
      (window as any).benchmarkDataCsv()
    );

    console.log(
      `⏱️  Data CSV: 中位数 ${result.median.toFixed(1)}ms (${result.min.toFixed(1)}~${result.max.toFixed(1)}ms)`
    );
    expect(result.median, `Data CSV 导出应在 ${THRESHOLDS.dataCsv}ms 内完成`).toBeLessThan(
      THRESHOLDS.dataCsv
    );
  });

  test('纯数据 XLSX 导出 (1000行)', async () => {
    const result: BenchmarkResult = await sharedPage.evaluate(() =>
      (window as any).benchmarkDataXlsx()
    );

    console.log(
      `⏱️  Data XLSX: 中位数 ${result.median.toFixed(1)}ms (${result.min.toFixed(1)}~${result.max.toFixed(1)}ms)`
    );
    expect(result.median, `Data XLSX 导出应在 ${THRESHOLDS.dataXlsx}ms 内完成`).toBeLessThan(
      THRESHOLDS.dataXlsx
    );
  });

  test('树形数据 XLSX 导出 (~584节点)', async () => {
    const result: BenchmarkResult = await sharedPage.evaluate(() =>
      (window as any).benchmarkTreeXlsx()
    );

    console.log(
      `⏱️  Tree XLSX: 中位数 ${result.median.toFixed(1)}ms (${result.min.toFixed(1)}~${result.max.toFixed(1)}ms)`
    );
    expect(result.median, `Tree XLSX 导出应在 ${THRESHOLDS.treeXlsx}ms 内完成`).toBeLessThan(
      THRESHOLDS.treeXlsx
    );
  });
});
