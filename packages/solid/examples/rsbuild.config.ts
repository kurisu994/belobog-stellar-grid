import { defineConfig } from '@rsbuild/core';
import { pluginBabel } from '@rsbuild/plugin-babel';
import { pluginSolid } from '@rsbuild/plugin-solid';

export default defineConfig({
  plugins: [
    pluginBabel({
      include: /\.(?:jsx|tsx)$/,
    }),
    pluginSolid(),
  ],
  dev: {
    // 禁用懒编译：Worker 入口中的 WASM 异步加载与懒编译代理冲突，
    // 会触发 HMR 更新，而 Worker 环境无 window 对象导致崩溃
    lazyCompilation: false,
  },
  tools: {
    rspack: {
      experiments: {
        asyncWebAssembly: true,
      },
    },
  },
});
