// Native Progress smoke checks on existing cached data. No seed, sign-out,
// sync, settings mutation, uninstall or clear. Run inside nix develop.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
const {chromium}=createRequire(new URL('../app/package.json',import.meta.url))('playwright');
const serial=process.env.ANDROID_SERIAL??'emulator-5560';
const app=process.env.APP_ID??'app.raring.debug';
const adb=(...args)=>execFileSync('adb',['-s',serial,...args],{encoding:'utf8'}).trim();
const pid=adb('shell','pidof',app);
const port=adb('forward','tcp:0',`localabstract:webview_devtools_remote_${pid}`);
let browser, page;
try {
 const deadline=Date.now()+10_000;
 for(;;){
  try { if((await(await fetch(`http://127.0.0.1:${port}/json`)).json()).some(t=>JSON.parse(t.description||'{}').attached))break; }catch{}
  assert.ok(Date.now()<deadline,'WebView DevTools becomes ready');
  await new Promise(resolve=>setTimeout(resolve,100));
 }
 browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`,{noDefaults:true});
 const targets=await(await fetch(`http://127.0.0.1:${port}/json`)).json();
 const target=targets.find(t=>t.url.startsWith('http://tauri.localhost/')&&JSON.parse(t.description||'{}').attached);
 for(const candidate of browser.contexts().flatMap(c=>c.pages())){
  const session=await candidate.context().newCDPSession(candidate);
  const {targetInfo}=await session.send('Target.getTargetInfo');
  await session.detach();
  if(targetInfo.targetId===target?.id){page=candidate;break}
 }
 assert.ok(page,'attached app WebView');
 const invoke=(command,args)=>page.evaluate(([command,args])=>window.__TAURI_INTERNALS__.invoke(command,args),[command,args]);
 assert.equal((await invoke('app_status')).signedIn,true,'requires an existing active cached session');
 // Fixture mode disables automatic network sync, not the real Tauri commands.
 await page.goto('http://tauri.localhost/?fixture=progress-check#progress');
 // A repeated same-fragment goto may preserve the previous run's UI selection.
 // Reload the WebView, not the app database, for a known initial month.
 await page.reload();
 await page.locator('.report').waitFor();
 await page.waitForFunction(()=>document.querySelector('.report')?.getAttribute('aria-busy')==='false');
 await page.evaluate(()=>{
  window.__progressResumes=0;
  window.__progressResumeListener=()=>window.__progressResumes++;
  window.addEventListener('raring:resume',window.__progressResumeListener);
 });
 const timing=[];
 for(const period of ['month','week','quarter','year']){
  const start=performance.now();
  const result=await invoke('progress',{period});
  timing.push({period,localReadMs:Math.round(performance.now()-start)});
  assert.equal(result.period,period);
  assert.ok(result.from<=result.end&&result.end<=result.to);
  assert.ok(result.totals.activeDays<=result.totals.coveredDays);
  assert.ok(result.totals.coveredDays<=result.totals.days);
  if(result.partial)assert.equal(result.previous,null,'partial activity coverage omits comparison');
  if(period==='month'||period==='week')assert.ok(result.calendar.length>0);
  else assert.equal(result.calendar.length,0);
 }
 const periodButton=name=>page.getByRole('group',{name:'Period',exact:true}).getByRole('button',{name,exact:true});
 async function pick(name){
  await periodButton(name).click();
  await page.waitForFunction(name=>[...document.querySelectorAll('.periods button')].some(b=>b.textContent===name&&b.getAttribute('aria-pressed')==='true')&&document.querySelector('.report')?.getAttribute('aria-busy')==='false',name);
 }
 await pick('Week');
 const currentWeek=await page.locator('.rep-h h2').textContent();
 await page.getByRole('button',{name:'Previous period',exact:true}).click();
 await page.waitForFunction(label=>document.querySelector('.rep-h h2')?.textContent!==label&&document.querySelector('.report')?.getAttribute('aria-busy')==='false',currentWeek);
 const previousWeek=await page.locator('.rep-h h2').textContent();
 await page.getByRole('button',{name:'Next period',exact:true}).click();
 await page.waitForFunction(label=>document.querySelector('.rep-h h2')?.textContent===label,currentWeek);
 await pick('3 months');
 assert.equal(await page.locator('.cal').count(),0);
 await pick('Year');
 await pick('Month');
 assert.ok(await page.locator('.cal .day').count()>0);
 assert.equal(await page.evaluate(()=>document.querySelector('main').scrollWidth<=document.querySelector('main').clientWidth+1),true,'phone Progress has no horizontal overflow');
 console.log('Android Progress: real core period commands, rendered period switches, previous/next, coverage and calendar passed');
 console.log('Cached synthetic/local read timings (not a real-history benchmark):',JSON.stringify(timing));
 // Keep the selected historical week through a detail visit, Android Back
 // and an ordinary resume refresh. This is part of caller-state parity.
 await pick('Week');
 await page.getByRole('button',{name:'Previous period',exact:true}).click();
 await page.waitForFunction(label=>document.querySelector('.rep-h h2')?.textContent===label,previousWeek).catch(async error=>{
  console.error('Period selection mismatch',await page.evaluate(expected=>({expected,actual:document.querySelector('.rep-h h2')?.textContent,busy:document.querySelector('.report')?.getAttribute('aria-busy')}),previousWeek));
  throw error;
 });
 await page.locator('.tiles2 button').filter({hasText:'HRV'}).click();
 await page.waitForFunction(()=>location.hash==='#metric/hrv');
 adb('shell','input','keyevent','4');
 await page.waitForFunction(()=>location.hash==='#progress');
 await page.locator('.report').waitFor();
 assert.equal(await page.locator('.rep-h h2').textContent(),previousWeek,'Android Back preserves the selected historical week');
 const beforeResume=await page.evaluate(()=>window.__progressResumes);
 adb('shell','input','keyevent','3');
 await page.waitForFunction(()=>document.visibilityState==='hidden',null,{polling:100});
 adb('shell','am','start','-n',`${app}/app.raring.MainActivity`);
 await page.waitForFunction(n=>document.visibilityState==='visible'&&window.__progressResumes>n,beforeResume);
 assert.equal(await page.locator('.rep-h h2').textContent(),previousWeek,'native resume retains the selected historical week');
 await page.getByRole('button',{name:'Next period',exact:true}).click();
 await page.waitForFunction(label=>document.querySelector('.rep-h h2')?.textContent===label&&document.querySelector('.report')?.getAttribute('aria-busy')==='false',currentWeek);
 console.log('Android Progress: detail/Back retains the historical week; native Home/resume and forward navigation passed');
} finally {
 await page?.evaluate(()=>{window.removeEventListener('raring:resume',window.__progressResumeListener);delete window.__progressResumeListener;delete window.__progressResumes;}).catch(()=>{});
 await browser?.close();
 adb('forward','--remove',`tcp:${port}`);
}
