import { useState, useEffect, useRef, useCallback } from 'react';
import { ExportWorker } from '@bsg-export/worker';
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
  const [initialized, setInitialized] = useState(false);
  const [loading, setLoading] = useState(false);
  const [progress, setProgress] = useState(0);
  const [message, setMessage] = useState('');
  const [error, setError] = useState<string | null>(null);
  const exporterRef = useRef<ExportWorker | null>(null);

  // 初始化 ExportWorker
  useEffect(() => {
    let cancelled = false;

    const worker = new Worker(
      new URL('./export-worker.ts', import.meta.url),
      { type: 'module' },
    );
    const exporter = new ExportWorker(worker);
    exporterRef.current = exporter;

    exporter
      .init()
      .then(() => {
        if (!cancelled) setInitialized(true);
      })
      .catch((err) => {
        if (!cancelled) setError(`初始化失败: ${err}`);
      });

    return () => {
      cancelled = true;
      exporter.terminate();
      exporterRef.current = null;
    };
  }, []);

  // Worker 导出数据
  const handleExport = useCallback(async (format: ExportFormat) => {
    const exporter = exporterRef.current;
    if (!exporter) return;

    setLoading(true);
    setProgress(0);
    setError(null);
    try {
      await exporter.exportData(
        sampleData,
        { columns, filename: 'Worker导出示例', format },
        { onProgress: (p) => setProgress(p) },
      );
      setMessage(`${format === ExportFormat.Csv ? 'CSV' : 'Excel'} 导出成功！`);
      setTimeout(() => setMessage(''), 3000);
    } catch (err) {
      setError(`导出失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  // 仅生成字节（不触发下载）
  const handleGenerateBytes = useCallback(async () => {
    const exporter = exporterRef.current;
    if (!exporter) return;

    setLoading(true);
    setProgress(0);
    setError(null);
    try {
      const bytes = await exporter.generateBytes(
        sampleData,
        { columns, filename: 'Worker导出示例', format: ExportFormat.Xlsx },
        { onProgress: (p) => setProgress(p) },
      );
      setMessage(`生成成功！文件大小: ${bytes.byteLength} 字节`);
      setTimeout(() => setMessage(''), 3000);
    } catch (err) {
      setError(`生成失败: ${err}`);
    } finally {
      setLoading(false);
    }
  }, []);

  return (
    <div className="app">
      <h1>🚀 BSG Export — Worker 示例</h1>

      {error && <div className="error">错误: {error}</div>}
      {message && <div className="success">{message}</div>}

      <section className="section">
        <h2>⚡ ExportWorker 类演示</h2>
        <p className="desc">
          直接使用 @bsg-export/worker 的 ExportWorker 类，在 Web Worker 中执行导出操作
        </p>

        <table>
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
            onClick={() => handleExport(ExportFormat.Csv)}
          >
            Worker 导出 CSV
          </button>
          <button
            disabled={!initialized || loading}
            onClick={() => handleExport(ExportFormat.Xlsx)}
          >
            Worker 导出 Excel
          </button>
          <button
            disabled={!initialized || loading}
            onClick={handleGenerateBytes}
          >
            生成字节（不下载）
          </button>
        </div>
        {loading && <div className="progress">处理中... {progress}%</div>}
      </section>

      <footer>
        <p>
          Worker {initialized ? '✅ 已就绪' : '⏳ 初始化中...'}
        </p>
      </footer>
    </div>
  );
}
