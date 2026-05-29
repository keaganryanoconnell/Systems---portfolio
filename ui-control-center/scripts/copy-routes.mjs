import { mkdirSync, copyFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const outDir = join(__dirname, '..', 'out');

const pages = ['capstone'];

for (const page of pages) {
  const src = join(outDir, `${page}.html`);
  const destDir = join(outDir, page);
  const destFile = join(destDir, 'index.html');

  mkdirSync(destDir, { recursive: true });
  copyFileSync(src, destFile);
  console.log(`[post-build] Copied ${page}.html → ${page}/index.html`);
}
