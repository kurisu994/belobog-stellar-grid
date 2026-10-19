import assert from 'node:assert/strict';
import { existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, rmSync, symlinkSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import test from 'node:test';
import { preparePublish } from './prepare-publish.mjs';

const packages = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const names = ['types', 'react', 'vue', 'svelte', 'solid', 'worker'];

/** 用 authored manifest 和最小构建产物验证发布流程，不碰仓库 dist。 */
function fixture(run) {
  const root = mkdtempSync(path.join(tmpdir(), 'bsg-publish-'));
  const source = path.join(root, 'packages');
  try {
    for (const name of names) {
      const directory = path.join(source, name);
      mkdirSync(path.join(directory, 'dist'), { recursive: true });
      writeFileSync(path.join(directory, 'package.json'), readFileSync(path.join(packages, name, 'package.json')));
      writeFileSync(path.join(directory, 'pnpm-lock.yaml'), '锁文件不得修改\n');
      writeFileSync(path.join(directory, 'README.md'), `# ${name}\n`);
      writeFileSync(path.join(directory, 'dist/index.js'), `export const name = ${JSON.stringify(name)};\n`);
    }
    run({ root, source, output: path.join(root, 'publish') });
  } finally { rmSync(root, { recursive: true, force: true }); }
}

function changeManifest(source, name, update) {
  const filename = path.join(source, name, 'package.json');
  const manifest = JSON.parse(readFileSync(filename, 'utf8'));
  update(manifest);
  writeFileSync(filename, JSON.stringify(manifest));
}

test('发布副本: 六包内容与版本完整，types 转 registry 范围，源 manifest/锁不变', () => fixture(({ source, output }) => {
  const originals = new Map(names.flatMap(name => ['package.json', 'pnpm-lock.yaml'].map(file => {
    const filename = path.join(source, name, file);
    return [filename, readFileSync(filename, 'utf8')];
  })));
  const version = JSON.parse(readFileSync(path.join(source, 'types/package.json'), 'utf8')).version;
  assert.equal(preparePublish(output, source), output);
  assert.deepEqual(readdirSync(output).sort(), [...names].sort());
  for (const name of names) {
    const directory = path.join(output, name);
    const manifest = JSON.parse(readFileSync(path.join(directory, 'package.json'), 'utf8'));
    assert.equal(manifest.version, version);
    if (name !== 'types') assert.equal(manifest.dependencies['@bsg-export/types'], `^${version}`);
    assert.ok(existsSync(path.join(directory, 'dist/index.js')));
    assert.equal(readFileSync(path.join(directory, 'README.md'), 'utf8'), `# ${name}\n`);
    assert.equal(existsSync(path.join(directory, 'pnpm-lock.yaml')), false);
    assert.doesNotMatch(JSON.stringify(manifest), /file:|link:|workspace:/);
  }
  for (const [filename, original] of originals) assert.equal(readFileSync(filename, 'utf8'), original);
}));

test('发布副本: 允许既有空目录，不覆盖已有非空目录或文件', () => fixture(({ root, source, output }) => {
  mkdirSync(output);
  preparePublish(output, source);
  const original = readFileSync(path.join(output, 'vue/package.json'), 'utf8');
  assert.throws(() => preparePublish(output, source), /拒绝覆盖/);
  assert.equal(readFileSync(path.join(output, 'vue/package.json'), 'utf8'), original);
  const file = path.join(root, 'existing.txt');
  writeFileSync(file, '保持原样');
  assert.throws(() => preparePublish(file, source), /拒绝覆盖/);
  assert.equal(readFileSync(file, 'utf8'), '保持原样');
}));

test('发布副本: 拒绝源包内部路径和符号链接输出目录', () => fixture(({ root, source, output }) => {
  assert.throws(() => preparePublish(path.join(source, 'vue/stage'), source), /源子包内部/);
  const empty = path.join(root, 'empty');
  mkdirSync(empty);
  symlinkSync(empty, output, 'dir');
  assert.throws(() => preparePublish(output, source), /拒绝覆盖/);
  assert.deepEqual(readdirSync(empty), []);
}));

test('发布副本: 版本不一致在写入输出前失败', () => fixture(({ source, output }) => {
  changeManifest(source, 'react', manifest => { manifest.version = '0.0.1'; });
  assert.throws(() => preparePublish(output, source), /版本与 types/);
  assert.equal(existsSync(output), false);
}));

for (const field of ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies']) {
  for (const specifier of ['file:../other', 'link:../other', 'workspace:*', '../other', '/tmp/other', 'C:\\other']) {
    test(`发布副本: 拒绝 ${field} 中的 ${specifier}`, () => fixture(({ source, output }) => {
      changeManifest(source, 'worker', manifest => { (manifest[field] ??= {}).local = specifier; });
      assert.throws(() => preparePublish(output, source), /残留本地依赖/);
      assert.equal(existsSync(output), false);
    }));
  }
}

test('发布副本: 缺失产物与空产物均明确报错，不生成部分输出', () => fixture(({ source, output }) => {
  const dist = path.join(source, 'solid/dist');
  rmSync(dist, { recursive: true });
  assert.throws(() => preparePublish(output, source), /缺少 dist/);
  assert.equal(existsSync(output), false);
  mkdirSync(dist);
  assert.throws(() => preparePublish(output, source), /dist 为空/);
  assert.equal(existsSync(output), false);
}));
