import assert from 'node:assert/strict';
import {mkdir,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';

const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
const output=resolve('../.data/screens');
await mkdir(output,{recursive:true});
let browser;
try {
 await server.listen();
 const url=`http://127.0.0.1:${server.httpServer.address().port}`;
 browser=await chromium.launch({headless:true,executablePath:'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 const page=await browser.newPage({viewport:{width:390,height:844},hasTouch:true,isMobile:true});
 const errors=[];
 page.on('pageerror',e=>errors.push(e.message));
 const waitRoute=async(route)=>{
  await page.waitForURL(`${url}/#${route}`);
  await page.locator('h1').waitFor();
  await page.waitForFunction(()=>!document.querySelector('.empty'));
 };
 const tab=async(name)=>{await page.getByRole('navigation').getByRole('button',{name,exact:true}).click();await waitRoute(name.toLowerCase())};
 const back=async(route)=>{await page.goBack();await waitRoute(route)};
 await page.goto(`${url}/#today`);
 await waitRoute('today');
 await tab('Sleep');await tab('Training');await tab('Health');
 await back('today');
 await tab('Training');
 await page.getByRole('button',{name:'Settings',exact:true}).click();await waitRoute('settings');
 await back('training');await back('today');
 await page.getByRole('button',{name:/^Body Battery/}).click();await waitRoute('metric/bodyBattery');
 await page.getByRole('button',{name:'Today',exact:true}).first().click();await waitRoute('today');
 await page.getByRole('button',{name:'View HRV',exact:true}).first().click();await waitRoute('metric/hrv');
 await page.getByText('Garmin baseline',{exact:true}).waitFor();
 await back('today');
 await tab('Health');
 await page.getByRole('button',{name:'View Respiration',exact:true}).click();await waitRoute('metric/respiration');
 await back('health');await back('today');
 await page.locator('.activity').first().click();await waitRoute('workout/9000000000001');
 await back('today');
 await tab('Training');
 await page.locator('.activity').first().click();await waitRoute('workout/9000000000001');
 await page.reload();await waitRoute('workout/9000000000001');
 await back('training');await back('today');
 await tab('Sleep');await tab('Sleep');await back('today');
 assert.equal(await page.evaluate(()=>window.__pepometerBack()),false,'Today lets Android leave the app');
 await tab('Training');
 assert.equal(await page.evaluate(()=>window.__pepometerBack()),true);
 await waitRoute('today');
 console.log('Hierarchical browser, header and native Back: passed');

 await page.getByRole('button',{name:/^Today's focus/}).click();await waitRoute('readiness');
 await page.getByText('Readiness history',{exact:true}).waitFor();
 await page.getByRole('button',{name:'30 days',exact:true}).click();
 assert.equal(await page.locator('.history-bar').count(),30);
 await back('today');
 await page.getByRole('button',{name:/^Tonight's sleep/}).click();await waitRoute('sleep');
 assert.ok((await page.locator('.sleep-plan').textContent()).includes('23:06'));
 assert.ok((await page.locator('.sleep-plan').textContent()).includes('Usual wake'));
 await tab('Health');
 assert.equal(await page.locator('.dashboard>.card').first().locator('h3>span').first().textContent(),'Vitals');
 assert.equal(await page.locator('.signal-tile').count(),5);
 await page.getByRole('button',{name:'View HRV',exact:true}).click();await waitRoute('metric/hrv');
 const history=page.locator('.history'),slider=history.getByRole('slider');
 assert.equal(await slider.getAttribute('aria-valuetext'),'Thu 16 Oct: 58 ms');
 await slider.focus();await slider.press('ArrowLeft');
 assert.ok((await slider.getAttribute('aria-valuetext')).startsWith('Wed 15 Oct:'));
 await history.getByRole('button',{name:'30 nights',exact:true}).click();
 await slider.focus();await slider.press('Home');
 assert.ok((await slider.getAttribute('aria-valuetext')).startsWith('Wed 17 Sept:'));
 assert.equal(await history.locator('.history-gap').count(),2,'missing nights remain gaps');
 for(let i=0;i<9;i++)await slider.press('ArrowRight');
 assert.equal(await slider.getAttribute('aria-valuetext'),'Fri 26 Sept: Not recorded');
 assert.equal(await history.locator('.history-reading b').textContent(),'—','missing is never zero');
 await slider.press('End');
 await history.getByRole('button',{name:'Previous reading'}).click();
 assert.ok((await slider.getAttribute('aria-valuetext')).startsWith('Wed 15 Oct:'));
 await history.getByRole('button',{name:'Next reading'}).click();
 const chart=await history.locator('.chart-surface').boundingBox();
 await history.locator('.chart-surface').click({position:{x:chart.width*.55,y:chart.height*.5}});
 assert.notEqual(await slider.inputValue(),'29','chart taps select a reading');
 console.log('Daily focus, sleep plan and keyboard/touch history: passed');

 for(const width of [360,390,720,840,1024]) {
  await page.setViewportSize({width,height:900});
  for(const route of ['today','sleep','training','health','readiness','workout/9000000000001','metric/hrv','metric/bodyBattery','settings','checkin']) {
   await page.goto(`${url}/#${route}`);await waitRoute(route);
   const layout=await page.evaluate(()=>{
    const main=document.querySelector('main'),grid=document.querySelector('.dashboard'),phone=document.querySelector('.phone');
    return {viewport:innerWidth,body:document.body.scrollWidth,main:main.clientWidth,content:main.scrollWidth,shell:phone.clientWidth,height:phone.clientHeight,columns:grid?getComputedStyle(grid).gridTemplateColumns.split(' ').length:0};
   });
   assert.equal(layout.shell,width,`${width}/${route}: shell uses available width`);
   assert.equal(layout.height,900,`${width}/${route}: shell uses available height`);
   assert.ok(layout.body<=width && layout.content<=layout.main+1,`${width}/${route}: horizontal overflow ${JSON.stringify(layout)}`);
   if(width>=720 && layout.columns)assert.equal(layout.columns,2,`${width}/${route}: two columns`);
   if(width===840 || (width===390 && route==='today'))await page.screenshot({path:resolve(output,`${width}-${route.replace('/','-')}.png`)});
  }
 }
 // Folding/unfolding should reflow in place, without losing the open detail.
 await page.setViewportSize({width:390,height:844});
 await page.goto(`${url}/#metric/hrv`);await waitRoute('metric/hrv');
 await page.setViewportSize({width:840,height:900});await waitRoute('metric/hrv');
 await page.setViewportSize({width:390,height:844});await waitRoute('metric/hrv');
 console.log('Phone, unfolded and live resize layouts: passed');

 const samples={};
 for(const name of ['today','sleep','training','health','status'])samples[name]=JSON.parse(await readFile(resolve(`src/lib/sample/${name}.json`),'utf8'));
 const live=await browser.newPage({viewport:{width:840,height:900}});
 live.on('pageerror',e=>errors.push(e.message));
 await live.clock.install({time:new Date('2026-10-16T14:00:00Z')});
 await live.addInitScript(samples=>{
  localStorage.setItem('notifications-step-done','1');
  const state=window.testBackend={samples,syncs:0,reads:0,failSync:false,failRead:false,holdToday:false,pendingToday:null};
  samples.status.lastSync=new Date(Date.now()-3600_000).toISOString();
  window.isTauri=true;
  window.__TAURI_INTERNALS__={
   transformCallback:()=>1,
   invoke:async(command)=>{
    if(command==='app_status')return structuredClone(samples.status);
    if(command==='sync_now'){
     state.syncs++;
     if(state.failSync)throw Error('Synthetic offline error');
     samples.status.lastSync=new Date().toISOString();
     return samples.status;
    }
    if(command==='today'){
     state.reads++;
     if(state.failRead)throw Error('Synthetic cache error');
     if(state.holdToday)await new Promise(resolve=>{state.pendingToday=resolve});
    }
    if(command.startsWith('plugin:event|'))return 1;
    if(command in samples)return structuredClone(samples[command]);
    throw Error('Unexpected command '+command);
   },
  };
 },samples);
 await live.goto(`${url}/#today`);
 await live.waitForFunction(()=>window.testBackend.syncs===1 && document.querySelector('.dial-button.large'));
 await live.evaluate(()=>{window.testBackend.samples.today.readiness.value=64});
 await live.clock.fastForward(30_000);
 await live.waitForFunction(()=>document.querySelector('.dial-button.large .gauge-center b')?.textContent==='64');
 assert.equal(await live.evaluate(()=>window.testBackend.syncs),1,'cache poll does not sync every 30 seconds');
 await live.locator('.dial-button.large').click();
 await live.waitForURL('**/#readiness');
 await live.evaluate(()=>{window.testBackend.samples.today.readiness.value=59});
 await live.clock.fastForward(30_000);
 await live.waitForFunction(()=>document.querySelector('.detail-gauge .gauge-center b')?.textContent==='59');
 await live.clock.fastForward(15*60_000);
 await live.waitForFunction(()=>window.testBackend.syncs===2);
 await live.evaluate(()=>{window.dispatchEvent(new Event('pepometer:resume'));window.dispatchEvent(new Event('pepometer:resume'))});
 await live.clock.fastForward(30_000);
 assert.equal(await live.evaluate(()=>window.testBackend.syncs),2,'resume events are throttled');
 await live.evaluate(()=>{window.testBackend.failRead=true});
 await live.clock.fastForward(30_000);
 assert.equal(await live.locator('.detail-gauge .gauge-center b').textContent(),'59','cache errors retain the visible reading');
 await live.evaluate(()=>{window.testBackend.failRead=false;window.testBackend.failSync=true});
 await live.clock.fastForward(15*60_000);
 await live.waitForFunction(()=>window.testBackend.syncs===3);
 await live.clock.fastForward(30_000);
 assert.equal(await live.evaluate(()=>window.testBackend.syncs),3,'failed syncs do not retry every poll');
 // A late Today response must not overwrite the newly opened Sleep page.
 await live.evaluate(()=>{window.testBackend.holdToday=true;window.dispatchEvent(new Event('pepometer:resume'))});
 await live.waitForFunction(()=>window.testBackend.pendingToday!==null);
 await live.getByRole('navigation').getByRole('button',{name:'Sleep',exact:true}).click();
 await live.waitForURL('**/#sleep');await live.locator('.sleep-hero').waitFor();
 await live.evaluate(()=>{window.testBackend.pendingToday();window.testBackend.holdToday=false});
 await live.clock.fastForward(1_000);
 assert.equal(await live.locator('h1').textContent(),'Sleep');
 await live.locator('.sleep-hero').waitFor();
 console.log('Live cache polling, throttled sync, failures and route races: passed');

 // A partially calibrated monitor must not turn missing readings into an all-clear.
 await live.evaluate(()=>{
  const s=window.testBackend;s.failSync=false;
  for(const v of s.samples.health.vitals.readings){v.value=null;v.normal=null;v.trend=Array(30).fill(null);v.tone='neutral'}
  Object.assign(s.samples.health.vitals.readings[0],{value:48,normal:[44,52],scale:[30,90],tone:'good'});
  Object.assign(s.samples.health.vitals.readings[1],{value:58});
  s.samples.health.vitals.baselineReady=true;
 });
 await live.setViewportSize({width:390,height:844});
 await live.getByRole('navigation').getByRole('button',{name:'Health',exact:true}).click();
 await live.getByText('1 reading in range',{exact:true}).waitFor();
 assert.equal(await live.getByText('Within your usual ranges',{exact:true}).count(),0);
 assert.equal(await live.getByText('Not recorded',{exact:true}).count(),3);
 await live.screenshot({path:resolve(output,'health-partial.png')});
 await live.evaluate(()=>{
  const v=window.testBackend.samples.health.vitals.readings;
  Object.assign(v[0],{value:74,tone:'poor'});
  Object.assign(v[1],{value:28,normal:[48,62],scale:[14,76],tone:'poor'});
  v[1].trend[29]=28;
  window.dispatchEvent(new Event('pepometer:resume'));
 });
 await live.getByText('2 outside your usual range',{exact:true}).waitFor();
 await live.getByText('Above usual',{exact:true}).waitFor();
 await live.getByText('Below baseline',{exact:true}).waitFor();
 await live.screenshot({path:resolve(output,'health-outliers.png')});
 await live.getByRole('button',{name:'View SpO₂',exact:true}).click();
 await live.getByText('No readings in this period. Missing data is never shown as zero.',{exact:true}).waitFor();
 assert.equal(await live.locator('.history .history-bar').count(),0);
 assert.equal(await live.locator('.history input').getAttribute('aria-valuetext'),'Thu 16 Oct: Not recorded','history is anchored to stored data, not the system clock');
 await live.screenshot({path:resolve(output,'history-missing.png')});
 await live.evaluate(()=>{window.testBackend.samples.today.sleepPlan=null;window.testBackend.samples.sleep.sleepPlan=null});
 await live.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();
 await live.locator('.dials').waitFor();
 assert.equal(await live.locator('.sleep-plan').count(),0,'no inferred sleep plan without Garmin need');
 await live.getByRole('navigation').getByRole('button',{name:'Sleep',exact:true}).click();
 await live.locator('.sleep-hero').waitFor();
 assert.equal(await live.locator('.sleep-plan').count(),0);
 console.log('Partial baselines, outliers, missing history and unavailable sleep plans: passed');
 assert.deepEqual(errors,[],'no browser errors');
} finally {
 await browser?.close();
 await server.close();
}
