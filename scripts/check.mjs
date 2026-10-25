// Opens the built game (dist/) in headless Chrome, flies to a series of
// objects, and saves a screenshot of each to .check/. Fails on any console
// error, page exception or WebGL error.
//
//   node scripts/build.mjs && node scripts/check.mjs [ids...]
//   PAGE=file:///.../dist-single/through-the-solar-system.html node scripts/check.mjs earth
//
// Needs Node 22+ (for the global WebSocket) and Chrome or Edge; set CHROME to
// its path if it is not found.

import { spawn, spawnSync } from 'node:child_process';
import { createServer } from 'node:http';
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { extname, join, normalize } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const DIST = join(ROOT, 'dist');
const OUT = join(ROOT, '.check');
const PROFILE = join(tmpdir(), `tss-check-${process.pid}`);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
mkdirSync(OUT, { recursive: true });

const TYPES = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.wasm': 'application/wasm', '.webp': 'image/webp', '.md': 'text/plain' };
const server = createServer((req, res) => {
  const path = normalize(decodeURIComponent(new URL(req.url, 'http://x').pathname)).replace(/^([/\\])+/, '');
  const file = join(DIST, path || 'index.html');
  if (!file.startsWith(DIST) || !existsSync(file)) {
    res.writeHead(404).end();
    return;
  }
  res.writeHead(200, { 'Content-Type': TYPES[extname(file)] ?? 'application/octet-stream' });
  res.end(readFileSync(file));
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const BASE = `http://127.0.0.1:${server.address().port}/`;

function findChrome() {
  const candidates = [
    process.env.CHROME,
    'C:/Program Files/Google/Chrome/Application/chrome.exe',
    'C:/Program Files (x86)/Microsoft/Edge/Application/msedge.exe',
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    'google-chrome',
    'chromium',
  ].filter(Boolean);
  return candidates.find((c) => (c.includes('/') ? existsSync(c) : spawnSync(c, ['--version']).status === 0));
}

const chrome = spawn(
  findChrome(),
  [
    '--headless=new',
    '--remote-debugging-port=0',
    '--enable-unsafe-swiftshader',
    '--use-angle=swiftshader',
    ...(process.env.CI ? ['--no-sandbox'] : []),
    '--window-size=1280,800',
    '--hide-scrollbars',
    '--no-first-run',
    `--user-data-dir=${PROFILE}`,
    'about:blank',
  ],
  { stdio: 'ignore' },
);

async function socketUrl() {
  for (let i = 0; i < 100; i++) {
    try {
      const port = readFileSync(join(PROFILE, 'DevToolsActivePort'), 'utf8').split('\n')[0].trim();
      const page = (await (await fetch(`http://127.0.0.1:${port}/json`)).json()).find((t) => t.type === 'page');
      if (page) return page.webSocketDebuggerUrl;
    } catch {}
    await sleep(200);
  }
  throw new Error('Chrome did not start');
}

const ws = new WebSocket(await socketUrl());
await new Promise((r) => ws.addEventListener('open', r));
let nextId = 0;
const waiting = new Map();
const listeners = new Map();
ws.addEventListener('message', (e) => {
  const msg = JSON.parse(e.data);
  if (msg.id) {
    const { resolve, reject } = waiting.get(msg.id);
    waiting.delete(msg.id);
    msg.error ? reject(new Error(msg.error.message)) : resolve(msg.result);
  } else {
    for (const fn of listeners.get(msg.method) ?? []) fn(msg.params);
  }
});
const send = (method, params = {}) =>
  new Promise((resolve, reject) => {
    const id = ++nextId;
    waiting.set(id, { resolve, reject });
    ws.send(JSON.stringify({ id, method, params }));
  });
const on = (method, fn) => listeners.set(method, [...(listeners.get(method) ?? []), fn]);

const problems = [];
await send('Page.enable');
await send('Runtime.enable');
await send('Log.enable');
await send('Emulation.setFocusEmulationEnabled', { enabled: true });
on('Runtime.exceptionThrown', (p) => problems.push(`exception: ${p.exceptionDetails.exception?.description ?? p.exceptionDetails.text}`));
on('Runtime.consoleAPICalled', (p) => {
  if (p.type === 'error' || p.type === 'warning') problems.push(`console.${p.type}: ${p.args.map((a) => a.value ?? a.description).join(' ')}`);
});
on('Log.entryAdded', (p) => {
  if (p.entry.level === 'error' || p.entry.level === 'warning') problems.push(`log.${p.entry.level}: ${p.entry.text}`);
});

const evaluate = async (expression) => {
  const { result, exceptionDetails } = await send('Runtime.evaluate', { expression, returnByValue: true, awaitPromise: true });
  if (exceptionDetails) throw new Error(exceptionDetails.exception?.description ?? exceptionDetails.text);
  return result.value;
};
const shot = async (name) => {
  const { data } = await send('Page.captureScreenshot', { format: 'png' });
  writeFileSync(join(OUT, `${name}.png`), Buffer.from(data, 'base64'));
};
const click = (selector) => evaluate(`document.querySelector(${JSON.stringify(selector)}).click()`);

const loaded = new Promise((r) => on('Page.loadEventFired', r));
await send('Page.navigate', { url: process.env.PAGE ?? BASE });
await loaded;
await sleep(4000);
await shot('00-intro');
if (!(await evaluate(`!!document.querySelector('[data-action=explore]')`))) {
  console.log(['the game did not start', ...problems].join(String.fromCharCode(10)));
  chrome.kill();
  server.close();
  process.exit(1);
}
await click('[data-action=explore]');
await sleep(1500);
await shot('01-overview');

const ids = process.argv.slice(2);
const stops = ids.length ? ids : ['sun', 'earth', 'moon', 'mars', 'jupiter', 'io', 'saturn', 'titan', 'uranus', 'neptune', 'pluto', 'asteroid-belt', 'kuiper-belt', 'heliosphere', 'oort-cloud'];
for (const [n, id] of stops.entries()) {
  const ok = await evaluate(`(() => {
    const b = [...document.querySelectorAll('.tss-label')].find(l => l.getAttribute('aria-label').toLowerCase().replace(/^the /, '').startsWith(${JSON.stringify(id.replace(/-/g, ' '))}));
    const i = b ? b.dataset.action.split(':')[1] : null;
    if (i === null) return false;
    const fly = document.createElement('button');
    fly.dataset.action = 'fly:' + i;
    document.querySelector('.tss-labels').append(fly);
    fly.click();
    fly.remove();
    return true;
  })()`);
  if (!ok) {
    problems.push(`no label for ${id}`);
    continue;
  }
  for (let t = 0; t < 120; t++) {
    await sleep(500);
    if ((await evaluate(`document.querySelector('.tss-near').textContent`)).startsWith('Visiting')) break;
  }
  await sleep(800);
  await shot(`${String(n + 2).padStart(2, '0')}-${id}`);
}

const hud = await evaluate(`document.querySelector('.tss-hud').innerText`);
const journal = await evaluate(`document.querySelector('.tss-journal-button').textContent`);
console.log(hud.replace(/\n/g, ' | '));
console.log(journal);
ws.close();
chrome.kill();
server.close();
try {
  rmSync(PROFILE, { recursive: true, force: true });
} catch {}
if (problems.length) {
  console.log(problems.join('\n'));
  process.exit(1);
}
console.log(`ok: screenshots in ${OUT}`);
