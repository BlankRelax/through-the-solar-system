// Builds the game for the web.
//
//   node scripts/build.mjs
//
// 1. compiles the Rust crate to WebAssembly (release),
// 2. generates the JavaScript bindings with wasm-bindgen,
// 3. writes dist/ — a static site (index.html, the module, textures) that any
//    web server can host,
// 4. writes dist-single/through-the-solar-system.html — one self-contained
//    file with the WebAssembly, textures and styles inlined, which opens
//    straight from disk in a browser (no server needed).
//
// Needs: Rust with the wasm32-unknown-unknown target, and wasm-bindgen-cli at
// the exact version pinned in Cargo.toml (cargo install wasm-bindgen-cli --version <v>).

import { execFileSync } from 'node:child_process';
import { cpSync, mkdirSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from 'node:fs';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';

const ROOT = fileURLToPath(new URL('..', import.meta.url));
const DIST = join(ROOT, 'dist');
const SINGLE = join(ROOT, 'dist-single');
const WEB = join(ROOT, 'web');
const NAME = 'through_the_solar_system';

const run = (cmd, args) => execFileSync(cmd, args, { cwd: ROOT, stdio: 'inherit' });

run('cargo', ['build', '--release', '--target', 'wasm32-unknown-unknown']);
rmSync(DIST, { recursive: true, force: true });
run('wasm-bindgen', [
  '--target', 'web',
  '--no-typescript',
  '--out-dir', join(DIST, 'pkg'),
  join(ROOT, 'target', 'wasm32-unknown-unknown', 'release', `${NAME}.wasm`),
]);

for (const file of ['index.html', 'tss.css', 'page.css', 'main.js']) {
  cpSync(join(WEB, file), join(DIST, file));
}
cpSync(join(WEB, 'textures'), join(DIST, 'textures'), { recursive: true });
cpSync(join(ROOT, 'TEXTURES.md'), join(DIST, 'TEXTURES.md'));

// --- the single-file build ------------------------------------------------------
const glue = readFileSync(join(DIST, 'pkg', `${NAME}.js`), 'utf8');
const wasm = readFileSync(join(DIST, 'pkg', `${NAME}_bg.wasm`)).toString('base64');
const textures = Object.fromEntries(
  readdirSync(join(WEB, 'textures'))
    .filter((f) => f.endsWith('.webp'))
    .map((f) => [f.replace(/\.webp$/, ''), `data:image/webp;base64,${readFileSync(join(WEB, 'textures', f)).toString('base64')}`]),
);
const css = readFileSync(join(WEB, 'tss.css'), 'utf8') + readFileSync(join(WEB, 'page.css'), 'utf8');
if (glue.includes('</script')) throw new Error('bindings contain </script>');

const page = readFileSync(join(WEB, 'index.html'), 'utf8')
  .replace(/\s*<link rel="stylesheet"[^>]*>/g, '')
  .replace('</head>', `    <style>\n${css}\n    </style>\n  </head>`)
  .replace(
    /<script type="module" src="main.js"><\/script>/,
    () => `<script type="module">
${glue}
const tssModule = Uint8Array.from(atob(${JSON.stringify(wasm)}), (c) => c.charCodeAt(0));
initSync({ module: tssModule });
const tssContainer = document.getElementById('game');
start(tssContainer, { textures: ${JSON.stringify(textures)} });
tssContainer.focus();
</script>`,
  );
rmSync(SINGLE, { recursive: true, force: true });
mkdirSync(SINGLE);
writeFileSync(join(SINGLE, 'through-the-solar-system.html'), page);

const size = (p) => `${(statSync(p).size / 1024 / 1024).toFixed(2)} MB`;
console.log(`dist/pkg/${NAME}_bg.wasm  ${size(join(DIST, 'pkg', `${NAME}_bg.wasm`))}`);
console.log(`dist-single/through-the-solar-system.html  ${size(join(SINGLE, 'through-the-solar-system.html'))}`);
