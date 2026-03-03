<script lang="ts">
  import { createExporter, createWorkerExporter } from '@bsg-export/svelte';
  import { ExportFormat } from '@bsg-export/types';
  import type { Column } from '@bsg-export/types';
  import { onDestroy } from 'svelte';

  // 示例数据
  const sampleData = [
    { name: '张三', age: 28, city: '北京', email: 'zhang@example.com' },
    { name: '李四', age: 32, city: '上海', email: 'li@example.com' },
    { name: '王五', age: 25, city: '广州', email: 'wang@example.com' },
    { name: '赵六', age: 30, city: '深圳', email: 'zhao@example.com' },
    { name: '钱七', age: 27, city: '杭州', email: 'qian@example.com' },
  ];

  // 列配置
  const columns: Column[] = [
    { title: '姓名', key: 'name' },
    { title: '年龄', key: 'age' },
    { title: '城市', key: 'city' },
    { title: '邮箱', key: 'email' },
  ];

  let message = $state('');

  // 基本导出
  const {
    initialized,
    loading,
    progress,
    error,
    exportTable,
    exportData,
    destroy,
  } = createExporter();

  // Worker 导出
  const {
    initialized: workerReady,
    loading: workerLoading,
    progress: workerProgress,
    exportData: workerExportData,
    destroy: destroyWorker,
  } = createWorkerExporter(
    () =>
      new Worker(new URL('./export-worker.ts', import.meta.url), {
        type: 'module',
      }),
  );

  onDestroy(() => {
    destroy();
    destroyWorker();
  });

  function handleTableExport(format: ExportFormat) {
    exportTable({ tableId: 'demo-table', filename: '表格导出示例', format });
  }

  function handleDataExport(format: ExportFormat) {
    exportData(sampleData, { columns, filename: '数据导出示例', format });
  }

  async function handleWorkerExport(format: ExportFormat) {
    const success = await workerExportData(sampleData, {
      columns,
      filename: 'Worker导出示例',
      format,
    });
    if (success) {
      message = 'Worker 导出成功！';
      setTimeout(() => (message = ''), 3000);
    }
  }
</script>

<div class="app">
  <h1>🚀 BSG Export — Svelte 示例</h1>

  {#if $error}
    <div class="error">错误: {$error.message}</div>
  {/if}
  {#if message}
    <div class="success">{message}</div>
  {/if}

  <!-- DOM 表格导出 -->
  <section class="section">
    <h2>📋 DOM 表格导出</h2>
    <p class="desc">从 HTML 表格元素直接导出为文件</p>

    <table id="demo-table">
      <thead>
        <tr><th>姓名</th><th>年龄</th><th>城市</th><th>邮箱</th></tr>
      </thead>
      <tbody>
        {#each sampleData as row (row.name)}
          <tr>
            <td>{row.name}</td>
            <td>{row.age}</td>
            <td>{row.city}</td>
            <td>{row.email}</td>
          </tr>
        {/each}
      </tbody>
    </table>

    <div class="actions">
      <button
        disabled={!$initialized || $loading}
        onclick={() => handleTableExport(ExportFormat.Csv)}
      >
        导出 CSV
      </button>
      <button
        disabled={!$initialized || $loading}
        onclick={() => handleTableExport(ExportFormat.Xlsx)}
      >
        导出 Excel
      </button>
    </div>
    {#if $loading}
      <div class="progress">导出中... {$progress}%</div>
    {/if}
  </section>

  <!-- 纯数据导出 -->
  <section class="section">
    <h2>📊 纯数据导出</h2>
    <p class="desc">从 JavaScript 数据数组导出为文件</p>

    <div class="actions">
      <button
        disabled={!$initialized || $loading}
        onclick={() => handleDataExport(ExportFormat.Csv)}
      >
        导出 CSV
      </button>
      <button
        disabled={!$initialized || $loading}
        onclick={() => handleDataExport(ExportFormat.Xlsx)}
      >
        导出 Excel
      </button>
    </div>
  </section>

  <!-- Worker 导出 -->
  <section class="section">
    <h2>⚡ Worker 后台导出</h2>
    <p class="desc">使用 Web Worker 在后台线程处理导出，不阻塞 UI</p>

    <div class="actions">
      <button
        disabled={!$workerReady || $workerLoading}
        onclick={() => handleWorkerExport(ExportFormat.Csv)}
      >
        Worker 导出 CSV
      </button>
      <button
        disabled={!$workerReady || $workerLoading}
        onclick={() => handleWorkerExport(ExportFormat.Xlsx)}
      >
        Worker 导出 Excel
      </button>
    </div>
    {#if $workerLoading}
      <div class="progress">Worker 导出中... {$workerProgress}%</div>
    {/if}
  </section>

  <footer>
    <p>
      WASM {$initialized ? '✅ 已就绪' : '⏳ 初始化中...'} | Worker
      {$workerReady ? '✅ 已就绪' : '⏳ 初始化中...'}
    </p>
  </footer>
</div>

<style>
  .app {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }
  h1 { margin-bottom: 1.5rem; }
  h2 { margin-bottom: 0.5rem; color: #333; }
  .desc { color: #666; margin-bottom: 1rem; font-size: 0.9rem; }
  .section {
    margin-bottom: 2rem;
    padding: 1.5rem;
    border: 1px solid #e0e0e0;
    border-radius: 8px;
  }
  table { width: 100%; border-collapse: collapse; margin-bottom: 1rem; }
  th, td { border: 1px solid #ddd; padding: 8px 12px; text-align: left; }
  th { background: #f5f5f5; font-weight: 600; }
  tr:hover { background: #fafafa; }
  .actions { display: flex; gap: 0.5rem; flex-wrap: wrap; }
  button {
    padding: 8px 16px;
    border: none;
    border-radius: 4px;
    background: #4f46e5;
    color: white;
    cursor: pointer;
    font-size: 0.9rem;
  }
  button:hover:not(:disabled) { background: #4338ca; }
  button:disabled { opacity: 0.5; cursor: not-allowed; }
  .progress { margin-top: 0.5rem; color: #4f46e5; font-size: 0.9rem; }
  .error { padding: 0.75rem; background: #fef2f2; color: #dc2626; border-radius: 4px; margin-bottom: 1rem; }
  .success { padding: 0.75rem; background: #f0fdf4; color: #16a34a; border-radius: 4px; margin-bottom: 1rem; }
  footer { text-align: center; color: #999; font-size: 0.85rem; margin-top: 2rem; }
</style>
