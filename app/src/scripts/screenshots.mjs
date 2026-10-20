// Review screenshots of every screen on synthetic fixtures, saved in .data/screens/.
// publish-screens.mjs builds the website images from these. Run from app/.
import {createServer} from 'vite';
import {chromium} from 'playwright';
import {mkdir} from 'node:fs/promises';
import {resolve} from 'node:path';

const output=resolve('../.data/screens');
await mkdir(output,{recursive:true});
// [file, route, query, section tab]. Today's "fresh" fixture gives the header a recent watch upload.
const shots=[
 ['today','today','?fixture=fresh'],['plan','plan'],['training','training'],['health','health'],
 ['progress','progress'],['progress-cardio','progress','','Cardio'],['progress-strength','progress','','Strength'],['progress-long','progress','','Long-term'],
 ['sleep','sleep/last'],['sleep-tonight','sleep/tonight'],['readiness','readiness'],
 ['workout','workout/9000000000001'],['strength-workout','workout/9000000000002'],['strength','strength'],
 ['stress','metric/stress'],['hrv-history','metric/hrv'],['weight','health/weight'],
 ['review','review/latest'],['goals','plan/commitments'],['profile','profile'],['settings','settings'],['onboarding','onboarding'],
 ['today-missing','today','?fixture=missing'],['today-empty','today','?fixture=empty'],
];
const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
let browser;
try{
 await server.listen();
 const url=`http://127.0.0.1:${server.httpServer.address().port}`;
 browser=await chromium.launch({headless:true,executablePath:process.env.CHROMIUM??'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 const context=await browser.newContext({viewport:{width:390,height:844},deviceScaleFactor:2,hasTouch:true,isMobile:true});
 const errors=[];
 for(const [name,route,query='',section] of shots){
  // A new tab per screen, so each opens as a link would, without an earlier screen's Back trail.
  const page=await context.newPage();
  page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`${url}/${query}#${route}`);
  if(name==='today-empty')await page.getByText('Syncing your Garmin data').first().waitFor();
  else await page.locator('main h1').first().waitFor();
  if(section){
   const tab=page.getByRole('tab',{name:section,exact:true});
   await tab.click();
   await page.waitForFunction(el=>el.getAttribute('aria-selected')==='true',await tab.elementHandle());
  }
  // Pages fill in follow-up reads (reviews, trends) after their first frame.
  await page.waitForFunction(()=>!/Loading…|Preparing this view…|Reading your records…/.test(document.querySelector('main')?.textContent??''));
  await page.evaluate(()=>document.fonts.ready);
  await page.evaluate(()=>new Promise(done=>requestAnimationFrame(()=>requestAnimationFrame(done))));
  await page.screenshot({path:resolve(output,`${name}.png`)});
  console.log(`${name}: ${resolve(output,`${name}.png`)}`);
  if(errors.length)throw new Error(`${name}: ${errors.join('; ')}`);
  await page.close();
 }
}finally{await browser?.close();await server.close()}
