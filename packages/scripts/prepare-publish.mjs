import { cpSync, existsSync, lstatSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const defaultPackages = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const packageNames = ['types', 'react', 'vue', 'svelte', 'solid', 'worker'];
const dependencyFields = ['dependencies', 'devDependencies', 'peerDependencies', 'optionalDependencies'];
const localDependency = /^(?:file:|link:|workspace:|\.{1,2}(?:[/\\]|$)|[/\\]|~[/\\]|[a-zA-Z]:[/\\])/;

/** 将已构建子包复制到独立发布目录，仅改写副本中的本地 types 依赖。 */
export function preparePublish(outputDirectory, packagesDirectory = defaultPackages) {
  if (!outputDirectory) throw new Error('用法: node packages/scripts/prepare-publish.mjs <输出根目录>');
  const output = path.resolve(outputDirectory);
  const source = path.resolve(packagesDirectory);
  for (const name of packageNames) {
    const packageRoot = path.join(source, name);
    if (output === packageRoot || output.startsWith(packageRoot + path.sep)) {
      throw new Error('发布目录不能位于源子包内部');
    }
  }
  if (existsSync(output)) {
    if (!lstatSync(output).isDirectory() || readdirSync(output).length > 0) {
      throw new Error('发布目录必须不存在或为空目录，拒绝覆盖已有内容');
    }
  }

  // 先完成全部契约检查，再创建输出，防止版本错误留下半套发布包。
  const manifests = packageNames.map(name => {
    const directory = path.join(source, name);
    const manifest = JSON.parse(readFileSync(path.join(directory, 'package.json'), 'utf8'));
    if (!existsSync(path.join(directory, 'dist')) || !lstatSync(path.join(directory, 'dist')).isDirectory()) {
      throw new Error(`${name} 缺少 dist，请先构建全部子包`);
    }
    if (readdirSync(path.join(directory, 'dist')).length === 0) {
      throw new Error(`${name} 的 dist 为空，请先构建全部子包`);
    }
    if (!existsSync(path.join(directory, 'README.md'))) throw new Error(`${name} 缺少 README.md`);
    return { name, directory, manifest };
  });
  const version = manifests[0].manifest.version;
  if (typeof version !== 'string' || !/^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$/.test(version)) {
    throw new Error('types 包版本必须是有效的 SemVer');
  }
  for (const { name, manifest } of manifests) {
    if (manifest.version !== version) throw new Error(`${name} 版本与 types ${version} 不一致`);
    for (const field of dependencyFields) {
      for (const [dependency, specifier] of Object.entries(manifest[field] ?? {})) {
        if (dependency === '@bsg-export/types' && specifier === 'file:../types') {
          manifest[field][dependency] = `^${version}`;
        } else if (typeof specifier !== 'string' || localDependency.test(specifier)) {
          throw new Error(`${name} 的 ${field}.${dependency} 残留本地依赖: ${specifier}`);
        }
      }
    }
  }

  mkdirSync(output, { recursive: true });
  for (const { name, directory, manifest } of manifests) {
    const destination = path.join(output, name);
    mkdirSync(destination);
    writeFileSync(path.join(destination, 'package.json'), JSON.stringify(manifest, null, 2) + '\n', { flag: 'wx' });
    cpSync(path.join(directory, 'dist'), path.join(destination, 'dist'), { recursive: true, dereference: true, errorOnExist: true, force: false });
    cpSync(path.join(directory, 'README.md'), path.join(destination, 'README.md'), { errorOnExist: true, force: false });
  }
  return output;
}

if (process.argv[1] && import.meta.url === pathToFileURL(path.resolve(process.argv[1])).href) {
  try {
    if (process.argv.length !== 3) throw new Error('用法: node packages/scripts/prepare-publish.mjs <输出根目录>');
    console.log(`已准备发布目录: ${preparePublish(process.argv[2])}`);
  } catch (error) {
    console.error(error instanceof Error ? error.message : String(error));
    process.exitCode = 1;
  }
}
