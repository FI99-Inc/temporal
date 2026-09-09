import { defineConfig, type Plugin } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { execFile } from 'node:child_process';
import { fileURLToPath } from 'node:url';

// Development UI verification uses the same app library as Tauri's IPC.
// No HTTP service or subprocess is included in the built desktop frontend.
function syntheticPreview(): Plugin {
  return { name: 'synthetic-preview', apply: 'serve', configureServer(server) {
    server.middlewares.use('/__temporal', async (req, res) => {
      const url = new URL(req.url ?? '/', 'http://127.0.0.1');
      const id = url.searchParams.get('scenarioId') ?? '';
      const minutes = url.searchParams.get('offsetMinutes') ?? '0';
      const cache = fileURLToPath(new URL('./.cache/browser-preview/temporal-engine.sqlite3', import.meta.url));
      const frozenTime = '2026-09-09T12:00:00.000Z';
      const importing = url.pathname === '/import_trace_json';
      const args = url.pathname === '/scenario_catalog' ? ['catalog']
        : url.pathname === '/evaluate_scenario' && /^S\d{2}$/.test(id) && /^\d{1,5}$/.test(minutes) ? [id, minutes]
        : url.pathname === '/personal_snapshot' ? ['personal', cache, frozenTime]
        : importing ? ['import', cache, frozenTime] : null;
      if (req.headers.origin && !['http://127.0.0.1:1420','http://localhost:1420'].includes(req.headers.origin)) { res.statusCode=403; res.end('Same-origin preview requests only'); return; }
      if (req.method !== (importing ? 'POST' : 'GET') || !args) { res.statusCode = 400; res.end('Invalid synthetic preview request'); return; }
      try {
        let body = '';
        if (importing) {
          const parts: Buffer[] = []; let size = 0;
          for await (const chunk of req) {
            const part = Buffer.from(chunk); size += part.length;
            if (size > 10 * 1024 * 1024) { res.statusCode=413; res.end('Export exceeds the preview size limit'); return; }
            parts.push(part);
          }
          body = new TextDecoder('utf-8',{fatal:true}).decode(Buffer.concat(parts));
        }
        const binary = fileURLToPath(new URL(`./target/debug/scenario-preview${process.platform === 'win32' ? '.exe' : ''}`, import.meta.url));
        const stdout = await new Promise<string>((resolve,reject) => {
          const child = execFile(binary,args,{windowsHide:true,timeout:10_000,maxBuffer:20_000_000},(error,stdout) => error ? reject(error) : resolve(stdout));
          child.stdin?.on('error',reject); child.stdin?.end(body);
        });
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
