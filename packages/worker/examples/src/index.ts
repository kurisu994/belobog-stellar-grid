import { ExportWorker } from '@bsg-export/worker';
import { ExportFormat } from '@bsg-export/types';
import type { Column } from '@bsg-export/types';
import './style.css';

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

// 状态元素
const statusEl = document.getElementById('status')!;
const progressEl = document.getElementById('progress')!;
const messageEl = document.getElementById('message')!;

function updateStatus(text: string) {
  statusEl.textContent = text;
}

function updateProgress(text: string) {
  progressEl.textContent = text;
}

function showMessage(text: string, type: 'success' | 'error') {
  messageEl.textContent = text;
  messageEl.className = type;
  messageEl.style.display = 'block';
  setTimeout(() => {
    messageEl.style.display = 'none';
  }, 3000);
}

// 创建 Worker 和 ExportWorker 实例
const worker = new Worker(new URL('./export-worker.ts', import.meta.url), {
  type: 'module',
});
const exporter = new ExportWorker(worker);

// 初始化
updateStatus('⏳ Worker 初始化中...');

exporter
  .init()
  .then(() => {
    updateStatus('✅ Worker 已就绪');
    // 启用所有按钮
    document.querySelectorAll<HTMLButtonElement>('button[disabled]').forEach((btn) => {
      btn.disabled = false;
    });
  })
  .catch((err) => {
    updateStatus('❌ 初始化失败');
    showMessage(`初始化错误: ${err}`, 'error');
  });

// 绑定导出按钮事件
document.getElementById('export-csv')?.addEventListener('click', async () => {
  updateProgress('导出中...');
  try {
    await exporter.exportData(sampleData, {
      columns,
      filename: 'Worker导出示例',
      format: ExportFormat.Csv,
    }, {
      onProgress: (p) => updateProgress(`导出中... ${p}%`),
    });
    updateProgress('');
    showMessage('CSV 导出成功！', 'success');
  } catch (err) {
    updateProgress('');
    showMessage(`导出失败: ${err}`, 'error');
  }
});

document.getElementById('export-xlsx')?.addEventListener('click', async () => {
  updateProgress('导出中...');
  try {
    await exporter.exportData(sampleData, {
      columns,
      filename: 'Worker导出示例',
      format: ExportFormat.Xlsx,
    }, {
      onProgress: (p) => updateProgress(`导出中... ${p}%`),
    });
    updateProgress('');
    showMessage('Excel 导出成功！', 'success');
  } catch (err) {
    updateProgress('');
    showMessage(`导出失败: ${err}`, 'error');
  }
});

document.getElementById('generate-bytes')?.addEventListener('click', async () => {
  updateProgress('生成中...');
  try {
    const bytes = await exporter.generateBytes(sampleData, {
      columns,
      filename: 'Worker导出示例',
      format: ExportFormat.Xlsx,
    }, {
      onProgress: (p) => updateProgress(`生成中... ${p}%`),
    });
    updateProgress('');
    showMessage(`生成成功！文件大小: ${bytes.byteLength} 字节`, 'success');
  } catch (err) {
    updateProgress('');
    showMessage(`生成失败: ${err}`, 'error');
  }
});
