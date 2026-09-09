import { defineConfig, type Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { execFile } from 'node:child_process';
import { promisify } from 'node:util';
import { fileURLToPath } from 'node:url';

// Development UI verification uses the same app library as Tauri's IPC.
// No HTTP service or subprocess is included in the built desktop frontend.
function syntheticPreview(): Plugin {
  return { name: 'synthetic-preview', apply: 'serve', configureServer(server) {
    server.middlewares.use('/__temporal', async (req, res) => {
      const url = new URL(req.url ?? '/', 'http://127.0.0.1');
      const id = url.searchParams.get('scenarioId') ?? '';
      const minutes = url.searchParams.get('offsetMinutes') ?? '0';
      const args = url.pathname === '/scenario_catalog' ? ['catalog']
        : url.pathname === '/evaluate_scenario' && /^S\d{2}$/.test(id) && /^\d{1,5}$/.test(minutes) ? [id, minutes] : null;
      if (req.method !== 'GET' || !args) { res.statusCode = 400; res.end('Invalid synthetic preview request'); return; }
      try {
        const binary = fileURLToPath(new URL(`./target/debug/scenario-preview${process.platform === 'win32' ? '.exe' : ''}`, import.meta.url));
        const { stdout } = await promisify(execFile)(binary, args, { windowsHide: true, timeout: 10_000, maxBuffer: 2_000_000 });
        res.setHeader('Content-Type', 'application/json'); res.setHeader('Cache-Control', 'no-store'); res.end(stdout);
      } catch {
        res.statusCode = 503; res.end('Synthetic preview unavailable. Run npm run preview, or launch the Windows app with npm run app.');
      }
    });
  } };
}

export default defineConfig({
  plugins: [svelte(), syntheticPreview()],
  clearScreen: false,
  server: { host: '127.0.0.1', port: 1420, strictPort: true, watch: { ignored: ['**/src-tauri/**', '**/crates/**', '**/target/**', '**/.cache/**'] } },
  build: { target: 'chrome105' },
});
