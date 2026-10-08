// Regenerate the Windows icon set from the 1024 px source (rendered from
// icons/mark.svg). Only the files the Windows bundle uses are kept; no
// Android, iOS, or macOS scaffolds are added to the repository.
import { execFileSync } from 'node:child_process';
import { copyFile, mkdtemp, rm } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const icons = fileURLToPath(new URL('../src-tauri/icons/', import.meta.url));
const scratch = await mkdtemp(join(tmpdir(), 'temporal-icons-'));
try {
  execFileSync('npx', ['tauri', 'icon', join(icons, 'icon-source.png'), '-o', scratch], { stdio: 'inherit', shell: process.platform === 'win32' });
  for (const name of ['icon.ico', 'icon.png', '32x32.png', '128x128.png', '128x128@2x.png']) {
    await copyFile(join(scratch, name), join(icons, name));
  }
} finally {
  await rm(scratch, { recursive: true, force: true });
}
