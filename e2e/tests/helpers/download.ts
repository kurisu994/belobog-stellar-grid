import type { Download } from '@playwright/test';
import { inflateRawSync } from 'node:zlib';

/** 读取浏览器实际下载的完整文件。 */
export async function readDownload(download: Download): Promise<Buffer> {
  const chunks: Buffer[] = [];
  for await (const chunk of await download.createReadStream()) {
    chunks.push(Buffer.from(chunk));
  }
  return Buffer.concat(chunks);
}

/** 从回归测试用的小型 XLSX（ZIP32）中读取指定 XML，无需引入解压依赖。 */
export function readXlsxEntry(archive: Buffer, entryName: string): string {
  let endOffset = archive.length - 22;
  const minOffset = Math.max(0, endOffset - 0xffff);
  while (endOffset >= minOffset && archive.readUInt32LE(endOffset) !== 0x06054b50) {
    endOffset--;
  }
  if (endOffset < minOffset) throw new Error('下载文件不是有效的 XLSX ZIP');

  const entryCount = archive.readUInt16LE(endOffset + 10);
  let offset = archive.readUInt32LE(endOffset + 16);
  for (let index = 0; index < entryCount; index++) {
    if (archive.readUInt32LE(offset) !== 0x02014b50) {
      throw new Error('XLSX ZIP 中央目录格式错误');
    }
    const method = archive.readUInt16LE(offset + 10);
    const compressedSize = archive.readUInt32LE(offset + 20);
    const nameLength = archive.readUInt16LE(offset + 28);
    const extraLength = archive.readUInt16LE(offset + 30);
    const commentLength = archive.readUInt16LE(offset + 32);
    const name = archive.toString('utf8', offset + 46, offset + 46 + nameLength);
    if (name === entryName) {
      const localOffset = archive.readUInt32LE(offset + 42);
      const dataOffset = localOffset + 30
        + archive.readUInt16LE(localOffset + 26)
        + archive.readUInt16LE(localOffset + 28);
      const data = archive.subarray(dataOffset, dataOffset + compressedSize);
      if (method === 0) return data.toString('utf8');
      if (method === 8) return inflateRawSync(data).toString('utf8');
      throw new Error(`不支持的 XLSX ZIP 压缩方式: ${method}`);
    }
    offset += 46 + nameLength + extraLength + commentLength;
  }
  throw new Error(`XLSX 中缺少文件: ${entryName}`);
}
