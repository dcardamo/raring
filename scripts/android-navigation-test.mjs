// Run inside nix develop with a debug build open. APP_ID may select a separate test install.
// Default mode seeds only a signed-out synthetic test install. --existing uses
// its cached data without reseeding or changing circumstances.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
import {mkdir} from 'node:fs/promises';
const {chromium}=createRequire(new URL('../app/package.json',import.meta.url))('playwright');
const adb=(...args)=>execFileSync('adb',['-s',process.env.ANDROID_SERIAL??'emulator-5560',...args],{encoding:'utf8'}).trim();
const existing=process.argv.includes('--existing');
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
 let workout;
 if(existing){
  assert.equal(status.signedIn,true,'existing-data checks require an active cached session');
  const training=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('training'));
  workout=training.workouts[0]?.id;
  assert.ok(workout,'existing-data workout navigation needs a cached workout');
 }else{
  // Only a signed-out install or the synthetic demo account may be re-seeded.
  const demo=status.signedIn && await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('demo_session'));
  assert.ok(!status.signedIn || demo,'use a signed-out test install or the synthetic demo, never a personal account');
  const seeded=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('load_demo_data'));
  workout=seeded.split(':')[1];
 }
 await page.goto('http://tauri.localhost/?fixture=demo#today');
 const waitRoute=async(route)=>{
  await page.waitForFunction(route=>location.hash===`#${route}` && !document.querySelector('.empty') && document.querySelector('h1'),route);
 };
 const tab=async(name)=>{await page.getByRole('navigation').getByRole('button',{name,exact:true}).click();await waitRoute(name.toLowerCase())};
 const back=async(route)=>{adb('shell','input','keyevent','4');await waitRoute(route)};
 await waitRoute('today');
 // Your day from the real core: one region with its four rows; nothing past the upload.
 const yd=page.getByRole('region',{name:'Your day on one timeline'});
 await yd.waitFor();
 for(const row of [/^Sleep/,/^Body Battery/,/^Stress/,/^Heart rate/])assert.equal(await yd.getByRole('button',{name:row}).count(),1,`Your day has a ${row} row`);
 const day=await page.evaluate(()=>window.__TAURI_INTERNALS__.invoke('today').then(v=>v.day));
 assert.ok(day && day.nowMin>=0 && day.nowMin<=1500,'the live Today carries a timeline');
 const limit=Math.min(day.uploadMin??day.nowMin,day.nowMin);
 assert.ok(day.bodyBattery.length*3<=limit+3 && day.heartRate.length*2<=limit+2,'the core draws nothing past the upload or now');
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,'Your day fits the phone width');
 await yd.getByRole('button',{name:/^Body Battery/}).click();await waitRoute('metric/bodyBattery');await back('today');
 const choose=page.getByRole('button',{name:'Choose another'});
 if(await choose.count()){await choose.click();await page.getByRole('dialog',{name:'Choose today’s session'}).waitFor();adb('shell','input','keyevent','4');
  await page.waitForFunction(()=>!document.querySelector('[role=dialog]'));assert.equal(await page.evaluate(()=>location.hash),'#today','Back closes Choose before navigating')}
 console.log(`Android Your day: rows, core limit and width passed${await choose.count()?'; Choose another opens and Back closes it':' (no session today, Choose not shown)'}`);
 await tab('Plan');await tab('Training');await tab('Health');await tab('Progress');await back('today');
 await page.locator('.morning .sleep-fact').click();await waitRoute('sleep');await back('today');
 await page.locator('.activity').first().click();await waitRoute(`workout/${workout}`);await back('today');
 await tab('Training');
 await page.getByRole('tab',{name:'Activities'}).click();
 await page.locator('.activity').first().click();await waitRoute(`workout/${workout}`);await back('training');
 assert.equal(await page.getByRole('tab',{name:'Activities'}).getAttribute('aria-selected'),'true','Back keeps the Training section');
 await page.getByRole('tab',{name:'Strength'}).click();
 await page.locator('.q-muscles').waitFor();
 await page.getByRole('tab',{name:'Fitness'}).click();
 await page.getByRole('button',{name:'Profile and settings'}).click();await waitRoute('profile');
 await page.getByRole('button',{name:/^Settings/}).click();await waitRoute('settings');await back('profile');await back('training');await back('today');
 await page.getByRole('region',{name:'Your day on one timeline'}).getByRole('button',{name:/^Body Battery/}).click();await waitRoute('metric/bodyBattery');await back('today');
 await tab('Health');
 await page.getByRole('button',{name:'View HRV',exact:true}).click();await waitRoute('metric/hrv');await back('health');await back('today');
 await tab('Plan');
 await page.getByRole('button',{name:'Goals',exact:true}).click();await waitRoute('plan/commitments');await back('plan');await back('today');
 console.log('Android hardware Back: tab siblings, workouts, settings and metrics passed');
 // Notification and widget targets (VaultPlugin loads them as fragments), including the
 // earlier check-in route, open their current pages in the WebView.
 for(const [target,route] of [['checkin','review/latest'],['sleep','sleep'],['sleep/tonight','sleep/tonight'],['readiness','readiness'],['metric/stress','metric/stress'],['metric/bodyBattery','metric/bodyBattery'],['health','health'],['training','training'],['today','today']]){
  await page.goto(`http://tauri.localhost/?fixture=demo#${target}`);
  await waitRoute(route);
 }
 console.log('Android fragment destinations and the check-in alias: passed (not native intent delivery)');
 // A native target delivered by intent, as the bedtime reminder sends it.
 await page.goto('http://tauri.localhost/?fixture=demo#today');await waitRoute('today');
 // Live mode (no fixture) honours onboarding; a test install that never answered the
 // notification step would open onboarding instead, so answer it as "Not now" would.
 if(await page.evaluate(()=>localStorage.getItem('notifications-step-done'))!=='1'){
  await page.evaluate(()=>localStorage.setItem('notifications-step-done','1'));
  console.log('Android: marked the onboarding notification step done (as "Not now") for live-mode intents');
 }
 adb('shell','am','start','-n',`${app}/app.raring.MainActivity`,'--es','target','sleep/tonight');
 await waitRoute('sleep/tonight');
 assert.equal(await page.getByRole('tab',{name:'Tonight'}).getAttribute('aria-selected'),'true');
 await back('today');
 console.log('Android intent target sleep/tonight opens Tonight and Back returns to Today: passed');
 if(!existing){
 // Adapt goes through the real core: rest replaces the lead, offline and at once, and turns off again.
 await page.goto('http://tauri.localhost/?fixture=demo#today');await waitRoute('today');
 await page.locator('.top .adapt').click();
 await page.getByRole('dialog',{name:'Adapt today'}).getByRole('button',{name:/^Rest today/}).click();
 await page.getByRole('heading',{name:'Rest day',exact:true}).waitFor();
 assert.equal(await page.locator('.top .adapt').textContent(),'Rest day');
 assert.equal(await page.locator('.hero .effort').count(),0,'no effort scale while resting');
 // U28: with the phone offline, a reload keeps the rest day (local records, no network).
 adb('shell','cmd','connectivity','airplane-mode','enable');
 try{
  await page.reload();await waitRoute('today');
  await page.waitForFunction(()=>document.querySelector('.top .adapt')?.textContent==='Rest day');
  console.log('Android Adapt: the rest day holds through an offline reload: passed');
 }finally{adb('shell','cmd','connectivity','airplane-mode','disable')}
 await page.locator('.top .adapt').click();
 await page.getByRole('dialog',{name:'Adapt today'}).getByRole('button',{name:/^Rest day is on/}).click();
 await page.waitForFunction(()=>document.querySelector('.top .adapt')?.textContent==='Adapt today');
 console.log('Android Adapt: rest on and off through the core: passed');
 // Keep / Choose another through the real core with two of today's sessions; removed afterwards.
 const invoke=(cmd,args)=>page.evaluate(([c,a])=>window.__TAURI_INTERNALS__.invoke(c,a),[cmd,args]);
 // The planned demo variant records the morning run the day before, so a session can lead today.
 await invoke('load_demo_data',{planned:true});
 const today=(await invoke('today',{})).date;
 const add=name=>invoke('local_action',{action:{action:'addIntention',date:today,time:name==='Evening lift'?'21:00':'20:00',sport:name==='Evening lift'?'strength':'running',typeKey:null,name,durationMin:30,
  allocation:{strengthSessions:0,zone2Min:0,highIntensityMin:0,intensityMin:0,sleepNights:0},kind:'workout',garminId:null}});
 await add('Evening run');await add('Evening lift');
 const added=(await invoke('plan_board',{})).items.filter(p=>p.intention.date===today&&['Evening run','Evening lift'].includes(p.intention.name)).map(p=>p.intention.id);
 try{
  await page.goto('http://tauri.localhost/?fixture=demo#plan');await waitRoute('plan');await tab('Today');
  const lead=(await invoke('today',{})).lead;
  console.log(`Android Keep/Choose: lead is ${lead?.kind} (${lead?.headline})`);
  if(lead?.kind==='session'){
   await page.getByRole('button',{name:'Choose another'}).click();
   const sheet=page.getByRole('dialog',{name:'Choose today’s session'});await sheet.waitFor();
   const other=lead.intention===added[0]?'Evening lift':'Evening run';
   await sheet.getByRole('button',{name:new RegExp('^'+other)}).click();
   await page.getByRole('button',{name:'Kept for today'}).waitFor();
   const kept=(await invoke('today',{})).lead;
   assert.equal(kept.kept,true,'the chosen session is kept');
   assert.notEqual(kept.intention,lead.intention,'Choose another changes the lead');
   await page.getByRole('button',{name:'Choose another'}).click();await sheet.waitFor();
   adb('shell','input','keyevent','4');await page.waitForFunction(()=>!document.querySelector('[role=dialog]'));
   assert.equal(await page.evaluate(()=>location.hash),'#today','Back closes Choose before navigating');
   console.log('Android Keep/Choose: Choose another keeps the other session through the core; Back closes the sheet: passed');
  }else assert.fail(`a planned session should lead the planned demo day, not ${lead?.kind}`);
 }finally{
  for(const id of added)await invoke('local_action',{action:{action:'removeIntention',id}});
  await invoke('load_demo_data',{});
  await page.goto('http://tauri.localhost/?fixture=demo#today');await waitRoute('today');
 }
 }else console.log('Android existing-data mode: seeding and circumstance mutations skipped');

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
