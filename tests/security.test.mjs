import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync, readdirSync } from 'node:fs';
import { resolve } from 'node:path';
const root = resolve(import.meta.dirname, '..');
const read = p => readFileSync(resolve(root,p),'utf8');
const json = p => JSON.parse(read(p));

test('distribution has no updater trust or permission', () => {
  const config=json('src-tauri/tauri.conf.json');
  assert.equal(config.bundle.createUpdaterArtifacts,false);
  assert.equal(config.identifier,'local.autofish.desktop');
  assert.deepEqual(config.plugins,{});
  const cap=json('src-tauri/capabilities/default.json');
  assert.ok(cap.permissions.every(p => !/^(updater|opener|global-shortcut):/.test(p)));
  assert.doesNotMatch(read('src-tauri/Cargo.toml'),/tauri-plugin-updater/);
  assert.doesNotMatch(read('src-tauri/Cargo.lock'),/name = "tauri-plugin-updater"/);
  assert.equal(json('package.json').dependencies['@tauri-apps/plugin-updater'],undefined);
});

test('frontend CSP blocks external requests and executable content', () => {
  const csp=json('src-tauri/tauri.conf.json').app.security.csp;
  assert.match(csp,/object-src 'none'/);
  assert.match(csp,/frame-src 'none'/);
  const connect=csp.split(';').find(s=>s.trim().startsWith('connect-src'));
  assert.equal(connect.trim(),'connect-src ipc: http://ipc.localhost');
  assert.doesNotMatch(csp,/unsafe-eval/);
});

test('locked npm components come from the official registry and have integrity', () => {
  const lock=json('package-lock.json');
  for (const [name,pkg] of Object.entries(lock.packages)) {
    if (!name) continue;
    assert.ok(pkg.resolved.startsWith('https://registry.npmjs.org/'),name);
    assert.match(pkg.integrity,/^sha512-/,name);
  }
  assert.ok(Number(lock.packages['node_modules/source-map-js'].version.split('.')[2])>=2);
});

test('build process never generates signing keys or bypasses execution policy', () => {
  const build=read('MakeItExe.bat');
  assert.match(build,/npm ci --ignore-scripts/);
  assert.match(build,/--locked/);
  assert.doesNotMatch(build,/ExecutionPolicy|TAURI_SIGNING_PRIVATE_KEY|signer generate|npm install/i);
  assert.doesNotMatch(build,/^\s*(?:call\s+)?powershell(?:\.exe)?\s/im);
  assert.match(read('src-tauri/latest-json.ps1'),/throw/);
});

test('compiled UI carries no old updater or developer community destination', () => {
  const assets=readdirSync(resolve(root,'dist/assets')).filter(n=>n.endsWith('.js'));
  assert.ok(assets.length>0,'Build frontend before testing distribution');
  for (const name of assets) {
    const js=read('dist/assets/'+name);
    assert.doesNotMatch(js,/arielldev|unPZxXAtfb|downloadAndInstall|plugin:updater/);
    assert.ok(js.includes('Emergency stop'));
  }
});
