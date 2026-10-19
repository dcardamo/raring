// Run inside nix develop with a debug build open. APP_ID may select a separate test install.
// Seeds synthetic data only; refuses to overwrite a signed-in app.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {mkdir} from 'node:fs/promises';
const {chromium}=createRequire(new URL('../app/package.json',import.meta.url))('playwright');
const adb=(...args)=>execFileSync('adb',args,{encoding:'utf8'}).trim();
const app=process.env.APP_ID??'app.raring.debug';
const pid=adb('shell','pidof',app);
const port=adb('forward','tcp:0',`localabstract:webview_devtools_remote_${pid}`);
const size=adb('shell','wm','size').match(/Override size: (\d+x\d+)/)?.[1]??'reset';
const densityInfo=adb('shell','wm','density');
const densityOverride=densityInfo.match(/Override density: (\d+)/)?.[1]??'reset';
const density=Number(densityInfo.match(/Override density: (\d+)/)?.[1]??densityInfo.match(/Physical density: (\d+)/)?.[1]);
let browser;
try {
 const readyBy=Date.now()+10_000;
 while(true){
  try{if((await (await fetch(`http://127.0.0.1:${port}/json`)).json()).some(t=>JSON.parse(t.description||'{}').attached))break}catch{}
  assert.ok(Date.now()<readyBy,'WebView DevTools becomes ready');
  await new Promise(resolve=>setTimeout(resolve,100));
 }
 browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`,{noDefaults:true});
 const targets=await (await fetch(`http://127.0.0.1:${port}/json`)).json();
 const target=targets.find(t=>t.url.startsWith('http://tauri.localhost/') && JSON.parse(t.description||'{}').attached);
 let page;
 for(const candidate of browser.contexts().flatMap(c=>c.pages())){
  const session=await candidate.context().newCDPSession(candidate);
  const {targetInfo}=await session.send('Target.getTargetInfo');
  await session.detach();
  if(targetInfo.targetId===target?.id){page=candidate;break}
 }
 assert.ok(page,'attached app WebView is open');
 const status=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('app_status'));
 assert.equal(status.signedIn,false,'use a signed-out test install, never a personal account');
 const demo=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('load_demo_data'));
 const workout=demo.split(':')[1];
 await page.goto('http://tauri.localhost/?fixture=demo#today');
 const waitRoute=async(route)=>{
  await page.waitForFunction(route=>location.hash===`#${route}` && !document.querySelector('.empty') && document.querySelector('h1'),route);
 };
 const tab=async(name)=>{await page.getByRole('navigation').getByRole('button',{name,exact:true}).click();await waitRoute(name.toLowerCase())};
 const back=async(route)=>{adb('shell','input','keyevent','4');await waitRoute(route)};
 await waitRoute('today');
 await tab('Plan');await tab('Training');await tab('Health');await tab('Progress');await back('today');
 await page.locator('.morning .sleep-fact').click();await waitRoute('sleep');await back('today');
 await page.locator('.activity').first().click();await waitRoute(`workout/${workout}`);await back('today');
 await tab('Training');
 await page.locator('.activity').first().click();await waitRoute(`workout/${workout}`);await back('training');
 await page.getByRole('button',{name:'Profile and settings'}).click();await waitRoute('profile');
 await page.getByRole('button',{name:/^Settings/}).click();await waitRoute('settings');await back('profile');await back('training');await back('today');
 await page.getByRole('button',{name:/^Body Battery/}).click();await waitRoute('metric/bodyBattery');await back('today');
 await tab('Health');
 await page.getByRole('button',{name:'View HRV',exact:true}).click();await waitRoute('metric/hrv');await back('health');await back('today');
 await tab('Plan');
 await page.getByRole('button',{name:'Goals',exact:true}).click();await waitRoute('plan/commitments');await back('plan');await back('today');
 console.log('Android hardware Back: tab siblings, workouts, settings and metrics passed');
 // Notification and widget targets (VaultPlugin loads them as fragments), including the
 // earlier check-in route, open their current pages in the WebView.
 for(const [target,route] of [['checkin','review/latest'],['sleep','sleep'],['readiness','readiness'],['metric/stress','metric/stress'],['metric/bodyBattery','metric/bodyBattery'],['health','health'],['training','training'],['today','today']]){
  await page.goto(`http://tauri.localhost/?fixture=demo#${target}`);
  await waitRoute(route);
 }
 console.log('Android native targets and the check-in alias: passed');
 // Adapt goes through the real core: rest replaces the lead, offline and at once, and turns off again.
 await page.goto('http://tauri.localhost/?fixture=demo#today');await waitRoute('today');
 await page.locator('.top .adapt').click();
 await page.getByRole('dialog',{name:'Adapt today'}).getByRole('button',{name:/^Rest today/}).click();
 await page.getByRole('heading',{name:'Rest day',exact:true}).waitFor();
 assert.equal(await page.locator('.top .adapt').textContent(),'Rest day');
 assert.equal(await page.locator('.hero .effort').count(),0,'no effort scale while resting');
 await page.locator('.top .adapt').click();
 await page.getByRole('dialog',{name:'Adapt today'}).getByRole('button',{name:/^Rest day is on/}).click();
 await page.waitForFunction(()=>document.querySelector('.top .adapt')?.textContent==='Adapt today');
 console.log('Android Adapt: rest on and off through the core: passed');

 await page.evaluate(()=>{
  window.__foldSentinel=true;
  window.__resumeCount=0;
  window.addEventListener('raring:resume',()=>window.__resumeCount++);
 });
 adb('shell','wm','size',`${Math.round(840*density/160)}x${Math.round(900*density/160)}`);
 await page.waitForFunction(()=>innerWidth>=720 && document.querySelector('.dashboard') && getComputedStyle(document.querySelector('.dashboard')).gridTemplateColumns.split(' ').length===2).catch(async error=>{
  console.error('Unfold state',await page.evaluate(()=>({width:innerWidth,scale:devicePixelRatio,hash:location.hash,columns:document.querySelector('.dashboard')&&getComputedStyle(document.querySelector('.dashboard')).gridTemplateColumns,text:document.querySelector('main')?.innerText.slice(0,160)})));
  throw error;
 });
 assert.equal(await page.evaluate(()=>window.__foldSentinel),true,'unfold reflows without reloading');
 assert.equal(await page.evaluate(()=>document.querySelector('main').scrollWidth<=document.querySelector('main').clientWidth),true);
 await mkdir('.data/emulator-screens',{recursive:true});
 await page.screenshot({path:'.data/emulator-screens/fold-today.png'});
 adb('shell','wm','density',String(density-20));
 await page.waitForFunction(expected=>Math.abs(devicePixelRatio-expected)<.01,(density-20)/160);
 assert.equal(await page.evaluate(()=>window.__foldSentinel),true,'density changes preserve the WebView too');
 adb('shell','input','keyevent','4');
 // WebView animation frames stop in the background; poll from Node instead.
 const deadline=Date.now()+10_000;
 while(await page.evaluate(()=>document.visibilityState!=='hidden')){
  assert.ok(Date.now()<deadline,'Today backgrounds the WebView');
  await new Promise(resolve=>setTimeout(resolve,100));
 }
 assert.ok(!adb('shell','dumpsys','activity','activities').split('\n').filter(line=>line.includes('topResumedActivity')).some(line=>line.includes(app)),'Today backgrounds the task');
 adb('shell','am','start','-n',`${app}/app.raring.MainActivity`);
 await page.waitForFunction(()=>document.visibilityState==='visible' && window.__resumeCount>0);
 await waitRoute('today');
 console.log('Android unfolded layout, root exit and resume event: passed');
} finally {
 adb('shell','wm','size',size);adb('shell','wm','density',densityOverride);
 await browser?.close();
 adb('forward','--remove',`tcp:${port}`);
}
