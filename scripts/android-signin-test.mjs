import { execFileSync } from 'node:child_process';
import { mkdirSync, writeFileSync } from 'node:fs';
const adb = (...args) => execFileSync('adb', args, { encoding: 'utf8' });
const pids = [adb('shell', 'pidof', 'ca.hld.pepometer.debug').trim()];
let page;
for (const pid of pids) {
  try {
    adb('forward', 'tcp:9229', `localabstract:webview_devtools_remote_${pid}`);
    const targets = await (await fetch('http://127.0.0.1:9229/json')).json();
    page = targets.find(x => x.type === 'page');
    if (page) break;
  } catch { /* another WebView */ }
}
if (!page) throw new Error('Pepometer WebView not found');
if (!page.url.startsWith('http://tauri.localhost/')) throw new Error(`Expected app index, got ${page.url}`);
const socket = new WebSocket(page.webSocketDebuggerUrl);
await new Promise((resolve, reject) => { socket.onopen = resolve; socket.onerror = reject; });
let nextId = 1;
const pending = new Map();
socket.onmessage = ({ data }) => {
  const message = JSON.parse(data);
  const callback = pending.get(message.id);
  if (callback) { pending.delete(message.id); callback(message); }
};
function call(method, params) {
  const id = nextId++;
  return new Promise(resolve => { pending.set(id, resolve); socket.send(JSON.stringify({ id, method, params })); });
}
await call('Runtime.evaluate', { expression: "window.__TAURI_INTERNALS__.invoke('begin_sign_in')", awaitPromise: true });
await new Promise(resolve => setTimeout(resolve, 10000));
const garmin = (await (await fetch('http://127.0.0.1:9229/json')).json()).find(x => x.id === page.id)?.url;
console.log('Garmin page:', garmin);
if (!garmin?.startsWith('https://sso.garmin.com/portal/')) throw new Error('Garmin sign-in did not load');
const remoteIpc = await call('Runtime.evaluate', { expression: 'typeof window.__TAURI_INTERNALS__' });
console.log('Remote IPC bridge:', remoteIpc.result?.result?.value);
const remoteInvoke = await call('Runtime.evaluate', { expression: "window.__TAURI_INTERNALS__.invoke('app_status').then(() => 'ALLOWED', () => 'DENIED')", awaitPromise: true });
console.log('Remote IPC command:', remoteInvoke.result?.result?.value);
if (remoteInvoke.result?.result?.value !== 'DENIED') throw new Error('Remote Garmin page can call app commands');
mkdirSync('.data', { recursive: true });
writeFileSync('.data/garmin-sign-in.png', execFileSync('adb', ['exec-out', 'screencap', '-p']));
console.log('Navigation:', await call('Runtime.evaluate', { expression: "const a=document.createElement('a'); a.href='https://connect.garmin.com/app?ticket=ST-test'; document.body.appendChild(a); a.click();" }));
await new Promise(resolve => setTimeout(resolve, 3000));
const home = (await (await fetch('http://127.0.0.1:9229/json')).json()).find(x => x.id === page.id)?.url;
console.log('After ticket:', home);
if (home !== 'http://tauri.localhost/') throw new Error('Ticket redirect did not return to app');
socket.close();
