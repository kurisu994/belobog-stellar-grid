import { createExporter, createWorkerExporter } from '@bsg-export/solid';
import { ExportFormat } from '@bsg-export/types';
import type { Column } from '@bsg-export/types';
import { createSignal } from 'solid-js';
import './App.css';

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

export default function App() {
  const [message, setMessage] = createSignal('');

  // 基本导出
  const { initialized, loading, progress, error, exportTable, exportData } =
    createExporter();

  // Worker 导出
  const {
    initialized: workerReady,
    loading: workerLoading,
    progress: workerProgress,
    exportData: workerExportData,
  } = createWorkerExporter(
    () =>
      new Worker(new URL('./export-worker.ts', import.meta.url), {
        type: 'module',
      }),
  );

  const handleTableExport = (format: ExportFormat) => {
    exportTable({ tableId: 'demo-table', filename: '表格导出示例', format });
  };

  const handleDataExport = (format: ExportFormat) => {
    exportData(sampleData, { columns, filename: '数据导出示例', format });
  };

  const handleWorkerExport = async (format: ExportFormat) => {
    const success = await workerExportData(sampleData, {
      columns,
      filename: 'Worker导出示例',
      format,
    });
    if (success) {
      setMessage('Worker 导出成功！');
      setTimeout(() => setMessage(''), 3000);
    }
  };

  return (
    <div class="app">
      <h1>🚀 BSG Export — Solid 示例</h1>

      {error() && <div class="error">错误: {error()!.message}</div>}
      {message() && <div class="success">{message()}</div>}

      {/* DOM 表格导出 */}
      <section class="section">
        <h2>📋 DOM 表格导出</h2>
        <p class="desc">从 HTML 表格元素直接导出为文件</p>

        <table id="demo-table">
          <thead>
            <tr>
              <th>姓名</th>
              <th>年龄</th>
              <th>城市</th>
              <th>邮箱</th>
            </tr>
          </thead>
          <tbody>
            {sampleData.map((row) => (
              <tr>
                <td>{row.name}</td>
                <td>{row.age}</td>
                <td>{row.city}</td>
                <td>{row.email}</td>
              </tr>
            ))}
          </tbody>
        </table>

        <div class="actions">
          <button
            disabled={!initialized() || loading()}
            onClick={() => handleTableExport(ExportFormat.Csv)}
          >
            导出 CSV
          </button>
          <button
            disabled={!initialized() || loading()}
            onClick={() => handleTableExport(ExportFormat.Xlsx)}
          >
            导出 Excel
          </button>
        </div>
        {loading() && (
          <div class="progress">导出中... {progress()}%</div>
        )}
      </section>

      {/* 纯数据导出 */}
      <section class="section">
        <h2>📊 纯数据导出</h2>
        <p class="desc">从 JavaScript 数据数组导出为文件</p>

        <div class="actions">
          <button
            disabled={!initialized() || loading()}
            onClick={() => handleDataExport(ExportFormat.Csv)}
          >
            导出 CSV
          </button>
          <button
            disabled={!initialized() || loading()}
            onClick={() => handleDataExport(ExportFormat.Xlsx)}
          >
            导出 Excel
          </button>
        </div>
      </section>

      {/* Worker 导出 */}
      <section class="section">
        <h2>⚡ Worker 后台导出</h2>
        <p class="desc">
          使用 Web Worker 在后台线程处理导出，不阻塞 UI
        </p>

        <div class="actions">
          <button
            disabled={!workerReady() || workerLoading()}
            onClick={() => handleWorkerExport(ExportFormat.Csv)}
          >
            Worker 导出 CSV
          </button>
          <button
            disabled={!workerReady() || workerLoading()}
            onClick={() => handleWorkerExport(ExportFormat.Xlsx)}
          >
            Worker 导出 Excel
          </button>
        </div>
        {workerLoading() && (
          <div class="progress">
            Worker 导出中... {workerProgress()}%
          </div>
        )}
      </section>

      <footer>
        <p>
          WASM {initialized() ? '✅ 已就绪' : '⏳ 初始化中...'} | Worker{' '}
          {workerReady() ? '✅ 已就绪' : '⏳ 初始化中...'}
        </p>
      </footer>
    </div>
  );
}
