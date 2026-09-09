// Package the CLI-rendered PNG into a Windows ICO container, without adding
// platform scaffolds or generating icons for platforms the app does not support.
import { readFile, writeFile } from 'node:fs/promises';
const directory = new URL('../src-tauri/icons/', import.meta.url);
const png = await readFile(new URL('128x128.png', directory));
const header = Buffer.alloc(22);
header.writeUInt16LE(1, 2); // icon
header.writeUInt16LE(1, 4); // one image
header[6] = 128; header[7] = 128;
header.writeUInt16LE(1, 10);
header.writeUInt16LE(32, 12);
header.writeUInt32LE(png.length, 14);
header.writeUInt32LE(22, 18);
await writeFile(new URL('icon.ico', directory), Buffer.concat([header, png]));
await writeFile(new URL('icon.png', directory), png);
