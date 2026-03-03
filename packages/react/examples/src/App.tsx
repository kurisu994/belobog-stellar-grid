import { useState } from 'react';
import { useExporter, useWorkerExporter } from '@bsg-export/react';
import { ExportFormat } from '@bsg-export/types';
import type { Column } from '@bsg-export/types';
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
  const [message, setMessage] = useState('');

  // 基本导出 Hook
  const { initialized, loading, progress, error, exportTable, exportData } =
    useExporter();

  // Worker 导出 Hook
  const {
    initialized: workerReady,
    loading: workerLoading,
    progress: workerProgress,
    exportData: workerExportData,
  } = useWorkerExporter(
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
    <div className="app">
      <h1>🚀 BSG Export — React 示例</h1>

      {error && <div className="error">错误: {error.message}</div>}
      {message && <div className="success">{message}</div>}

      {/* DOM 表格导出 */}
      <section className="section">
        <h2>📋 DOM 表格导出</h2>
        <p className="desc">从 HTML 表格元素直接导出为文件</p>

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
              <tr key={row.name}>
                <td>{row.name}</td>
                <td>{row.age}</td>
                <td>{row.city}</td>
                <td>{row.email}</td>
              </tr>
            ))}
          </tbody>
        </table>

        <div className="actions">
          <button
            disabled={!initialized || loading}
            onClick={() => handleTableExport(ExportFormat.Csv)}
          >
            导出 CSV
          </button>
          <button
            disabled={!initialized || loading}
            onClick={() => handleTableExport(ExportFormat.Xlsx)}
          >
            导出 Excel
          </button>
        </div>
        {loading && <div className="progress">导出中... {progress}%</div>}
      </section>

      {/* 纯数据导出 */}
      <section className="section">
        <h2>📊 纯数据导出</h2>
        <p className="desc">从 JavaScript 数据数组导出为文件</p>

        <div className="actions">
          <button
            disabled={!initialized || loading}
            onClick={() => handleDataExport(ExportFormat.Csv)}
          >
            导出 CSV
          </button>
          <button
            disabled={!initialized || loading}
            onClick={() => handleDataExport(ExportFormat.Xlsx)}
          >
            导出 Excel
          </button>
        </div>
      </section>

      {/* Worker 导出 */}
      <section className="section">
        <h2>⚡ Worker 后台导出</h2>
        <p className="desc">
          使用 Web Worker 在后台线程处理导出，不阻塞 UI
        </p>

        <div className="actions">
          <button
            disabled={!workerReady || workerLoading}
            onClick={() => handleWorkerExport(ExportFormat.Csv)}
          >
            Worker 导出 CSV
          </button>
          <button
            disabled={!workerReady || workerLoading}
            onClick={() => handleWorkerExport(ExportFormat.Xlsx)}
          >
            Worker 导出 Excel
          </button>
        </div>
        {workerLoading && (
          <div className="progress">Worker 导出中... {workerProgress}%</div>
        )}
      </section>

      <footer>
        <p>
          WASM {initialized ? '✅ 已就绪' : '⏳ 初始化中...'} | Worker{' '}
          {workerReady ? '✅ 已就绪' : '⏳ 初始化中...'}
        </p>
      </footer>
    </div>
  );
}
