import { defineConfig } from '@playwright/test';

/**
 * Playwright E2E 测试配置
 *
 * 测试前需要先构建 WASM：wasm-pack build --target web
 */
export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  timeout: 30000,

  use: {
    // 使用本地服务器地址
    baseURL: 'http://localhost:4000',
    trace: 'on-first-retry',
  },

  projects: [
    {
      name: 'chromium',
      use: {
        browserName: 'chromium',
      },
    },
  ],

  // 自动启动 basic-http-server 作为测试服务器
  webServer: {
    command: 'basic-http-server --addr 127.0.0.1:4000 ..',
    port: 4000,
    reuseExistingServer: !process.env.CI,
    cwd: __dirname,
    timeout: 10000,
  },
});
