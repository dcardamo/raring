// Emulator screenshot QA for debug builds: loads synthetic demo data through the
// debug-only load_demo_data command (via the WebView DevTools socket) and saves a
// screenshot of every screen to .data/emulator-screens/. Run inside nix develop
// with the debug app open on the emulator: node scripts/android-screens.mjs
import {execFileSync} from 'node:child_process';
import {mkdirSync, writeFileSync} from 'node:fs';
const adb=(...args)=>execFileSync('adb',args,{encoding:'utf8'});
const pid=adb('shell','pidof','ca.hld.pepometer.debug').trim();
adb('forward','tcp:9229',`localabstract:webview_devtools_remote_${pid}`);
const pages=await (await fetch('http://127.0.0.1:9229/json')).json();
const page=pages.find(x=>x.type==='page'&&x.url.startsWith('http://tauri.localhost/'));
if(!page)throw Error('app page not found: '+JSON.stringify(pages));
const ws=new WebSocket(page.webSocketDebuggerUrl);
await new Promise((ok,no)=>{ws.onopen=ok;ws.onerror=no});
let id=0;const waiters=new Map();
ws.onmessage=({data})=>{let m=JSON.parse(data);if(waiters.has(m.id)){waiters.get(m.id)(m);waiters.delete(m.id)}};
function c(method,params){return new Promise(ok=>{const n=++id;waiters.set(n,ok);ws.send(JSON.stringify({id:n,method,params}))})}
async function evaluate(expression){const response=await c('Runtime.evaluate',{expression,awaitPromise:true,returnByValue:true});if(response.result?.exceptionDetails)throw Error(JSON.stringify(response.result.exceptionDetails));return response.result?.result?.value}
const demo=await evaluate("window.__TAURI_INTERNALS__.invoke('load_demo_data')");console.log('DEMO',demo);
const status=await evaluate("window.__TAURI_INTERNALS__.invoke('app_status')");console.log('STATUS',status);
const syncStatus=await evaluate("window.__TAURI_INTERNALS__.invoke('sync_now').then(x=>x, e=>({error:String(e)}))");console.log('FOREGROUND_SYNC_NO_TOKENS',syncStatus);
const [date,workout]=demo.split(':');
await evaluate("location.href='http://tauri.localhost/?fixture=demo#today'");
await new Promise(ok=>setTimeout(ok,1400));
mkdirSync('.data/emulator-screens',{recursive:true});
for(const route of ['today','sleep','training',`workout/${workout}`,'health','readiness','checkin','settings']){
 await evaluate(`location.hash=${JSON.stringify(route)}`);
 await new Promise(ok=>setTimeout(ok,1100));
 const state=await evaluate("({hash:location.hash,title:document.querySelector('main')?.innerText.slice(0,250),bodyWidth:document.body.scrollWidth,viewport:window.innerWidth,firstCard:document.querySelector('.card')?.innerText.slice(0,150),scrollTop:document.querySelector('main')?.scrollTop})");
 const name=route.replace('/','-');
 writeFileSync(`.data/emulator-screens/${name}.png`,execFileSync('adb',['exec-out','screencap','-p']));
 console.log('SCREEN',name,JSON.stringify(state));
 if(route==='checkin'){
  for(let step=2;step<=5;step++){
   await evaluate("document.querySelector('.checkin-controls button:last-child')?.click()");
   await new Promise(ok=>setTimeout(ok,300));
   writeFileSync(`.data/emulator-screens/checkin-${step}.png`,execFileSync('adb',['exec-out','screencap','-p']));
  }
 }
 await evaluate("document.querySelector('main').scrollTop=document.querySelector('main').scrollHeight");
 await new Promise(ok=>setTimeout(ok,350));
 writeFileSync(`.data/emulator-screens/${name}-lower.png`,execFileSync('adb',['exec-out','screencap','-p']));
 await evaluate("document.querySelector('main').scrollTop=0");
 if(!state.title||state.title.includes('Syncing your Garmin data'))throw Error(name+' not rendered');
}
await evaluate("window.__TAURI_INTERNALS__.invoke('sign_out')");
await evaluate("location.href='http://tauri.localhost/#onboarding'");
await new Promise(ok=>setTimeout(ok,1500));
writeFileSync('.data/emulator-screens/onboarding.png',execFileSync('adb',['exec-out','screencap','-p']));
console.log('ONBOARDING',await evaluate("document.querySelector('main')?.innerText.slice(0,220)"));
ws.close();
