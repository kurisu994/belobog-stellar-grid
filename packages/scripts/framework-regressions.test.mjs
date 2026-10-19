import assert from 'node:assert/strict';
import { readFileSync, writeFileSync, mkdtempSync, rmSync, readdirSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { createRequire } from 'node:module';
import { spawnSync } from 'node:child_process';
import test from 'node:test';

const packages = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const require = createRequire(path.join(packages, 'types/package.json'));
const ts = require('typescript');
const frameworks = ['react', 'vue', 'svelte', 'solid'];
const keys = ['loading', 'error', 'html', 'data', 'sheets', 'activeSheet', 'initialized', 'progress'];
const deferred = () => {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
};
const tick = async () => { for (let i = 0; i < 6; i++) await Promise.resolve(); };

/** 内存编译 authored TS，仅替换框架运行时，不复制被测业务逻辑。 */
function compile(relative, modules = {}, globals = {}) {
  const source = readFileSync(path.join(packages, relative), 'utf8');
  const { outputText } = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 },
  });
  const module = { exports: {} };
  new Function('require', 'module', 'exports', ...Object.keys(globals), outputText)(
    name => { if (!(name in modules)) throw new Error(`未提供运行时: ${name}`); return modules[name]; },
    module, module.exports, ...Object.values(globals),
  );
  return module.exports;
}

/** 最小框架状态适配器，React 通过重复 render 验证 hook ref 与闭包行为。 */
function harness(name, config, worker = false) {
  const cells = [], mounts = [], cleanups = [], maps = [];
  let cursor = 0, effectIndex = 0, effectCount = 0, api;
  const cell = init => { const i = cursor++; return [i, cells[i] ??= init()]; };
  const state = initial => {
    const [i, value] = cell(() => initial);
    return [value, next => { cells[i] = typeof next === 'function' ? next(cells[i]) : next; }];
  };
  const signal = initial => {
    let value = initial;
    return [() => value, next => { value = typeof next === 'function' ? next(value) : next; }];
  };
  const writable = initial => {
    let value = initial;
    const subscribers = new Set();
    return {
      set(next) { value = next; for (const callback of subscribers) callback(value); },
      subscribe(callback) { subscribers.add(callback); callback(value); return () => subscribers.delete(callback); },
    };
  };
  const get = store => { let value; store.subscribe(next => { value = next; })(); return value; };
  const modules = {
    react: {
      useState: state,
      useRef: initial => cell(() => ({ current: initial }))[1],
      useCallback: callback => callback,
      useEffect(callback) { if (effectIndex++ >= effectCount) { effectCount++; mounts.push(callback); } },
    },
    vue: { ref: value => ({ value }), onMounted: callback => mounts.push(callback), onUnmounted: callback => cleanups.push(callback) },
    'svelte/store': { writable, readonly: store => ({ subscribe: store.subscribe }), get },
    'solid-js': { createSignal: signal, onMount: callback => mounts.push(callback), onCleanup: callback => cleanups.push(callback) },
  };
  class PendingMap extends Map { constructor(...args) { super(...args); maps.push(this); } }
  const file = worker
    ? (['react', 'vue'].includes(name) ? 'use-worker-exporter.ts' : 'create-worker-exporter.ts')
    : (['react', 'vue'].includes(name) ? 'useExcelPreview.ts' : 'create-excel-preview.ts');
  const exports = compile(`${name}/src/${file}`, modules, { Map: PendingMap, fetch: (...args) => globalThis.fetch(...args) });
  const factory = Object.values(exports)[0];
  const render = () => { cursor = 0; effectIndex = 0; api = factory(config); return api; };
  render();
  for (const mount of mounts) { const cleanup = mount(); if (cleanup) cleanups.push(cleanup); }
  const read = key => {
    if (name === 'react') return render()[key];
    const value = api[key];
    return name === 'vue' ? value.value : name === 'solid' ? value() : get(value);
  };
  return {
    call(method, ...args) { if (name === 'react') render(); return api[method](...args); },
    read,
    state() { return Object.fromEntries(keys.filter(key => key in api).map(key => [key, read(key)])); },
    maps,
    cleanup() { for (const cleanup of cleanups) cleanup(); },
  };
}

const sheets = [
  { name: '隐藏', index: 0, hidden: true, rows: 1, cols: 1 },
  { name: '中文表', index: 1, hidden: false, rows: 1, cols: 1 },
  { name: '<恶意&表>', index: 2, hidden: false, rows: 1, cols: 1 },
];
function config(overrides = {}) {
  const calls = [];
  return {
    calls,
    init: async () => {},
    getExcelSheetList: () => sheets,
    parseExcelToHtml(bytes, options) { calls.push({ bytes, options }); return `${bytes[0]}:${options.sheetName ?? options.sheetIndex}`; },
    parseExcelToJson(bytes, options) { calls.push({ bytes, options }); return { sheets: [{ name: String(options.sheetName ?? options.sheetIndex), rows: [] }], activeSheet: 0 }; },
    ...overrides,
  };
}

for (const name of frameworks) {
  test(`${name}: 名称优先、真实索引、切表清名称并保留加载选项`, async () => {
    const options = { sheetName: '<恶意&表>', sheetIndex: 0, maxRows: 7, includeStyles: false };
    const cfg = config({ defaultOptions: { maxCols: 9, sheetName: '中文表' } });
    const h = harness(name, cfg);
    await h.call('loadData', new Uint8Array([1]), options);
    assert.equal(h.read('activeSheet'), 1);
    assert.equal(h.read('html'), '1:2');
    options.maxRows = 99;
    await h.call('switchSheet', 0);
    assert.equal(h.read('html'), '1:1');
    assert.equal(h.read('activeSheet'), 0);
    assert.equal((await h.call('getJsonData')).sheets[0].name, '1');
    assert.deepEqual(cfg.calls.at(-1).options, {
      sheetName: undefined, sheetIndex: 1, maxRows: 7, maxCols: 9, includeStyles: false,
    });
    assert.equal((await h.call('getJsonData', { sheetIndex: 2 })).sheets[0].name, '2');
    assert.equal((await h.call('getJsonData', { sheetName: '<恶意&表>', sheetIndex: 0 })).sheets[0].name, '<恶意&表>');
    await h.call('switchSheet', 999);
    assert.equal(h.read('activeSheet'), 0);
  });

  test(`${name}: hidden 筛选与显式选择对齐，false 展示全部`, async () => {
    const h = harness(name, config());
    await h.call('loadData', new Uint8Array([1]));
    assert.equal(h.read('html'), '1:1');
    assert.equal(h.read('activeSheet'), 0);
    assert.deepEqual(h.read('sheets').map(s => s.index), [1, 2]);
    await h.call('loadData', new Uint8Array([2]), { skipHidden: false, sheetName: '中文表' });
    assert.deepEqual(h.read('sheets').map(s => s.index), [0, 1, 2]);
    assert.equal(h.read('activeSheet'), 1);
    await h.call('switchSheet', 0);
    assert.equal((await h.call('getJsonData')).sheets[0].name, '0');
    await h.call('loadData', new Uint8Array([3]), { sheetName: '隐藏' });
    assert.equal(h.read('html'), '3:0');
    assert.equal(h.read('sheets')[h.read('activeSheet')].index, 0);
    const allHidden = harness(name, config({ getExcelSheetList: () => sheets.map(s => ({ ...s, hidden: true })) }));
    await allHidden.call('loadData', new Uint8Array([4]));
    assert.equal(allHidden.read('sheets')[allHidden.read('activeSheet')].index, 0);
    assert.equal(allHidden.read('html'), '4:0');
  });

  test(`${name}: 失败加载不提交新文件或部分 Sheet 状态`, async () => {
    const cfg = config();
    const h = harness(name, cfg);
    await h.call('loadData', new Uint8Array([1]), { maxRows: 4 });
    cfg.parseExcelToHtml = () => { throw new Error('解析失败'); };
    await h.call('loadData', new Uint8Array([2]), { sheetIndex: 2, maxRows: 99 });
    assert.equal(h.read('html'), '1:1');
    assert.equal(h.read('activeSheet'), 0);
    await h.call('getJsonData');
    assert.equal(cfg.calls.at(-1).bytes[0], 1);
    assert.equal(cfg.calls.at(-1).options.maxRows, 4);
    assert.equal(h.read('error'), '解析失败');
  });

  test(`${name}: File/URL 使用本次选项快照，旧成功/错误不覆盖新请求`, async () => {
    const cfg = config();
    const h = harness(name, cfg);
    const pending = deferred();
    const options = { maxRows: 8, sheetName: '<恶意&表>' };
    const old = h.call('loadFile', { arrayBuffer: () => pending.promise }, options);
    await tick();
    options.maxRows = 99;
    await h.call('loadData', new Uint8Array([2]), { maxRows: 6 });
    pending.resolve(new Uint8Array([1]).buffer);
    await old;
    assert.equal(h.read('html'), '2:1');
    const failure = deferred();
    const stale = h.call('loadFile', { arrayBuffer: () => failure.promise });
    await tick();
    await h.call('loadData', new Uint8Array([3]));
    failure.reject(new Error('过期错误'));
    await stale;
    assert.equal(h.read('error'), null);
    assert.equal(h.read('html'), '3:1');
    const current = deferred();
    const loading = h.call('loadFile', { arrayBuffer: () => current.promise }, options);
    options.maxRows = 100;
    await tick();
    current.resolve(new Uint8Array([4]).buffer);
    await loading;
    await h.call('getJsonData');
    assert.equal(cfg.calls.at(-1).options.maxRows, 99);
    assert.equal(h.read('activeSheet'), 1);
    // 文件读取先完成、JSON 后恢复时，即使同一加载代际也不能提交旧文件的数据。
    const replacement = deferred();
    const replace = h.call('loadFile', { arrayBuffer: () => replacement.promise });
    await tick();
    replacement.resolve(new Uint8Array([6]).buffer);
    const staleJson = h.call('getJsonData');
    await replace;
    assert.equal(await staleJson, null);
    assert.equal(h.read('data'), null);
    assert.equal(h.read('html'), '6:1');
    const originalFetch = globalThis.fetch;
    try {
      globalThis.fetch = async () => ({ ok: true, arrayBuffer: async () => new Uint8Array([5]).buffer });
      await h.call('loadUrl', '/test.xlsx', { sheetIndex: 2, maxCols: 3 });
      await h.call('switchSheet', 0);
      await h.call('getJsonData');
      assert.equal(cfg.calls.at(-1).options.maxCols, 3);
    } finally { globalThis.fetch = originalFetch; }
  });

  test(`${name}: reset 使 init/File/URL/JSON 旧请求全部失效`, async () => {
    const init = deferred();
    const h = harness(name, config({ init: () => init.promise }));
    const loading = h.call('loadData', new Uint8Array([1]));
    h.call('reset');
    init.resolve();
    await loading;
    assert.equal(h.read('html'), null);
    assert.equal(h.read('loading'), false);
    const cfg = config();
    const ready = harness(name, cfg);
    await ready.call('loadData', new Uint8Array([1]));
    const file = deferred();
    const oldFile = ready.call('loadFile', { arrayBuffer: () => file.promise });
    await tick();
    ready.call('reset');
    file.resolve(new Uint8Array([2]).buffer);
    await oldFile;
    assert.equal(ready.read('html'), null);
    const originalFetch = globalThis.fetch;
    try {
      const response = deferred();
      globalThis.fetch = () => response.promise;
      const url = ready.call('loadUrl', '/old.xlsx');
      await tick();
      ready.call('reset');
      response.resolve({ ok: true, arrayBuffer: async () => new Uint8Array([3]).buffer });
      await url;
      assert.equal(ready.read('html'), null);
    } finally { globalThis.fetch = originalFetch; }
    await ready.call('loadData', new Uint8Array([4]));
    const json = ready.call('getJsonData');
    ready.call('reset');
    assert.equal(await json, null);
    assert.equal(ready.read('data'), null);
    assert.deepEqual(ready.read('sheets'), []);
    assert.equal(ready.read('error'), null);
  });
}

class FakeWorker {
  handlers = {};
  requests = [];
  fail = false;
  addEventListener(type, callback) { (this.handlers[type] ??= []).push(callback); }
  postMessage(request) {
    if (this.fail) throw new Error('无法克隆');
    this.requests.push(request);
  }
  emit(response) { for (const callback of this.handlers.message ?? []) callback({ data: response }); }
  terminate() {}
}

test('ExportWorker: progress 严格/宽松策略、克隆失败清理 pending', async () => {
  const { ExportWorker } = compile('worker/src/index.ts');
  const worker = new FakeWorker();
  const manager = new ExportWorker(worker);
  worker.fail = true;
  await assert.rejects(manager.init(), /无法克隆/);
  assert.equal(manager.pending.size, 0);
  worker.fail = false;
  const init = manager.init();
  worker.emit({ type: 'ready', id: worker.requests.at(-1).id });
  await init;
  worker.fail = true;
  await assert.rejects(manager.generateBytes([]), /无法克隆/);
  assert.equal(manager.pending.size, 0);
  worker.fail = false;
  const warnings = [], originalWarn = console.warn;
  console.warn = (...args) => warnings.push(args);
  try {
    const loose = manager.generateBytes([], { progressCallback() {} }, { onProgress() { throw new Error('回调失败'); } });
    const id = worker.requests.at(-1).id;
    assert.equal(worker.requests.at(-1).options.progressCallback, undefined);
    assert.doesNotThrow(() => worker.emit({ type: 'progress', id, progress: 50 }));
    worker.emit({ type: 'result', id, bytes: new Uint8Array([1]).buffer });
    assert.deepEqual(await loose, new Uint8Array([1]));
    assert.equal(warnings.length, 1);
    const strict = manager.generateBytes([], { strictProgressCallback: true }, { onProgress() { throw new Error('严格回调失败'); } });
    const strictId = worker.requests.at(-1).id;
    worker.emit({ type: 'progress', id: strictId, progress: 50 });
    await assert.rejects(strict, /严格回调失败/);
    assert.equal(manager.pending.size, 0);
    worker.emit({ type: 'result', id: strictId, bytes: new ArrayBuffer(0) });
  } finally { console.warn = originalWarn; }
});

for (const name of frameworks) {
  test(`${name}: Worker 初始化/生成克隆失败不泄漏 pending`, async () => {
    const worker = new FakeWorker();
    const h = harness(name, () => worker, true);
    worker.emit({ type: 'ready', id: worker.requests[0].id });
    await tick();
    assert.equal(h.read('initialized'), true);
    worker.fail = true;
    assert.equal(await h.call('generateBytes', [{ children: [] }]), null);
    assert.equal(h.read('error').message, '无法克隆');
    assert.equal(h.maps[0].size, 0);
    worker.fail = false;
    const result = h.call('generateBytes', [{ children: [] }], { progressCallback() {} });
    assert.equal(worker.requests.at(-1).options.progressCallback, undefined);
    worker.emit({ type: 'result', id: worker.requests.at(-1).id, bytes: new Uint8Array([7]).buffer });
    assert.deepEqual(await result, new Uint8Array([7]));
    h.cleanup();
    const failedWorker = new FakeWorker();
    failedWorker.fail = true;
    const failed = harness(name, () => failedWorker, true);
    await tick();
    assert.equal(failed.read('initialized'), false);
    assert.equal(failed.read('error').message, '无法克隆');
    assert.equal(failed.maps[0].size, 0);
    failed.cleanup();
  });
}

test('Vue: 在临时 outdir 执行真实 build，所有预览 JS/SFC/声明完整', () => {
  const directory = mkdtempSync(path.join(tmpdir(), 'bsg-vue-build-'));
  try {
    const root = path.join(packages, 'vue');
    const { scripts } = JSON.parse(readFileSync(path.join(root, 'package.json'), 'utf8'));
    // 独立构建本地类型，避免安装目录中历史 registry 包掩盖仓库契约。
    const typesOut = path.join(directory, 'types');
    const typesBuild = spawnSync(process.execPath, [require.resolve('typescript/bin/tsc'), '-p', path.join(packages, 'types/tsconfig.json'), '--outDir', typesOut], { encoding: 'utf8' });
    assert.equal(typesBuild.status, 0, typesBuild.stdout + typesBuild.stderr);
    const project = path.join(directory, 'tsconfig.json');
    writeFileSync(project, JSON.stringify({
      extends: path.join(root, 'tsconfig.json'),
      compilerOptions: { paths: {
        '@bsg-export/types': [path.join(typesOut, 'index.d.ts')],
        'belobog-stellar-grid': [path.join(packages, '../pkg/belobog_stellar_grid.d.ts')],
      } },
    }));
    const compiler = createRequire(path.join(root, 'package.json')).resolve('vue-tsc/bin/vue-tsc.js');
    const command = scripts.build.replaceAll('dist', JSON.stringify(directory))
      .replace('vue-tsc', JSON.stringify(process.execPath) + ' ' + JSON.stringify(compiler) + ' --project ' + JSON.stringify(project));
    const result = spawnSync(command, {
      cwd: root, shell: true, encoding: 'utf8',
      env: { ...process.env, PATH: `${path.join(root, 'node_modules/.bin')}${path.delimiter}${process.env.PATH}` },
    });
    assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
    const files = readdirSync(directory);
    for (const file of ['index.js', 'index.d.ts', 'useExcelPreview.js', 'useExcelPreview.d.ts', 'ExcelPreview.vue', 'ExcelPreview.vue.d.ts', 'ExportButton.vue']) {
      assert.ok(files.includes(file), `缺失 Vue 构建产物: ${file}`);
    }
    assert.match(readFileSync(path.join(directory, 'index.js'), 'utf8'), /useExcelPreview/);
  } finally { rmSync(directory, { recursive: true, force: true }); }
});

test('类型: TreeDataRow 默认/自定义 children 可传所有框架公开导出与 Worker 方法', () => {
  const filename = path.join(packages, 'scripts/__tree-contract.ts');
  const source = `
    import type { TreeDataRow, DataRow } from '../types/src/index';
    import { export_data, generate_data_bytes, export_data_streaming } from '../types/src/index';
    import { useExporter as react } from '../react/src/use-exporter';
    import { useExporter as vue } from '../vue/src/use-exporter';
    import { createExporter as svelte } from '../svelte/src/create-exporter';
    import { createExporter as solid } from '../solid/src/create-exporter';
    import { useWorkerExporter as reactWorker } from '../react/src/use-worker-exporter';
    import { useWorkerExporter as vueWorker } from '../vue/src/use-worker-exporter';
    import { createWorkerExporter as svelteWorker } from '../svelte/src/create-worker-exporter';
    import { createWorkerExporter as solidWorker } from '../solid/src/create-worker-exporter';
    import { ExportWorker } from '../worker/src/index';
    const tree: TreeDataRow[] = [{ name: '中文', children: [{ name: '子节点', children: [] }] }];
    const custom: TreeDataRow<'nodes'>[] = [{ name: '<&>', nodes: [{ name: '子节点' }] }];
    const data: DataRow[] = [...tree, ...custom, { name: '直接输入', children: [{ name: '叶' }] }];
    export_data(data); generate_data_bytes(data); export_data_streaming(data);
    declare const exporters: [ReturnType<typeof react>, ReturnType<typeof vue>, ReturnType<typeof svelte>, ReturnType<typeof solid>];
    for (const exporter of exporters) { exporter.exportData(tree); exporter.exportData(custom, { childrenKey: 'nodes' }); }
    declare const workers: [ReturnType<typeof reactWorker>, ReturnType<typeof vueWorker>, ReturnType<typeof svelteWorker>, ReturnType<typeof solidWorker>, ExportWorker];
    for (const worker of workers) { worker.exportData(tree); worker.generateBytes(custom, { childrenKey: 'nodes' }); }
    // @ts-expect-error 子节点必须是数组，不能是标量
    const invalid: TreeDataRow = { children: 1 };
    // @ts-expect-error 普通字段不接受任意嵌套对象
    const badCell: DataRow = { value: { arbitrary: true } };
  `;
  const options = {
    noEmit: true, strict: true, skipLibCheck: true, esModuleInterop: true,
    target: ts.ScriptTarget.ESNext, module: ts.ModuleKind.ESNext, moduleResolution: ts.ModuleResolutionKind.Bundler,
    paths: {
      '@bsg-export/types': [path.join(packages, 'types/src/index.ts')],
      'belobog-stellar-grid': [path.join(packages, '../pkg/belobog_stellar_grid.d.ts')],
    },
  };
  const host = ts.createCompilerHost(options);
  const originalGet = host.getSourceFile.bind(host), originalExists = host.fileExists.bind(host);
  host.getSourceFile = (file, ...args) => file === filename ? ts.createSourceFile(file, source, options.target, true) : originalGet(file, ...args);
  host.fileExists = file => file === filename || originalExists(file);
  const program = ts.createProgram([filename], options, host);
  const diagnostics = ts.getPreEmitDiagnostics(program);
  assert.equal(diagnostics.length, 0, ts.formatDiagnosticsWithColorAndContext(diagnostics, {
    getCanonicalFileName: file => file, getCurrentDirectory: () => packages, getNewLine: () => '\n',
  }));
});
