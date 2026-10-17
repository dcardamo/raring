import {createServer} from 'vite';
import {chromium} from 'playwright';
import {mkdir} from 'node:fs/promises';
import {resolve} from 'node:path';

const output=resolve('../.data/screens');
await mkdir(output,{recursive:true});
const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
let browser;
try{
 await server.listen();
 const address=server.httpServer.address();
 const url=`http://127.0.0.1:${address.port}`;
 browser=await chromium.launch({headless:true,executablePath:'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 const page=await browser.newPage({viewport:{width:390,height:844},deviceScaleFactor:2,hasTouch:true,isMobile:true});
 const pageErrors=[];
 page.on('pageerror',e=>pageErrors.push(e));
 for(const [name,route,query] of [
  ['today','today',''],['sleep','sleep',''],['training','training',''],['workout','workout/9000000000001',''],['health','health',''],['readiness','readiness',''],['hrv-history','metric/hrv',''],['stress','metric/stress',''],['checkin','checkin',''],['plan','plan',''],['plan-setup','plan-setup',''],['onboarding','onboarding',''],['settings','settings',''],['today-missing','today','?fixture=missing'],['today-empty','today','?fixture=empty']
 ]){
  await page.goto(`${url}/${query}#${route}`);
  await page.waitForFunction(()=>!document.querySelector('.empty') && !document.querySelector('.scroll > .empty') || document.querySelector('.empty')?.textContent?.includes('Syncing your Garmin data') && location.search.includes('empty'));
  await page.evaluate(()=>document.fonts.ready);
  await page.screenshot({path:resolve(output,`${name}.png`)});
  console.log(`${name}: ${resolve(output,`${name}.png`)}`);
  if(name==='workout' && !(await page.locator('.map > path').first().getAttribute('d')).includes('C'))throw new Error('Sparse route was not smoothed');
  if(name==='settings' && await page.locator('input[type="checkbox"]').count()!==7)throw new Error('Expected six notification switches and the Today switch');
  if(name==='today-missing' && await page.getByText("Today's focus").count())throw new Error('Missing-data variant still shows a call');
  if(name==='readiness'){
   await page.getByText('Readiness history').waitFor();
   await page.locator('.scroll').evaluate(el=>{el.scrollTop=el.scrollHeight});
   await page.waitForFunction(()=>document.querySelector('.scroll').scrollTop>0);
   await page.screenshot({path:resolve(output,'readiness-trend.png')});
   console.log(`readiness-trend: ${resolve(output,'readiness-trend.png')}`);
  }
  if(name==='checkin'){
   for(const area of ['sleep','training','health','timing','intensity','plan','summary']){
    await page.getByRole('button',{name:'Next'}).click();
    await page.screenshot({path:resolve(output,`checkin-${area}.png`)});
    console.log(`checkin-${area}: ${resolve(output,`checkin-${area}.png`)}`);
   }
  }
  if(pageErrors.length)throw new Error(`${name}: ${pageErrors.map(e=>e.message).join('; ')}`);
 }
 await page.goto(`${url}/#today`);
 await page.locator('.dial-button.large').click();
 await page.waitForURL('**/#readiness');
 await page.goBack();
 await page.waitForURL('**/#today');
 await page.goto(`${url}/?fixture=demo#workout/9000000000001`);
 await page.locator('.tab.on').getByText('Training').waitFor();
 await page.goto(`${url}/#readiness`);
 await page.locator('.tab.on').getByText('Today').waitFor();
 await page.goto(`${url}/#checkin`);
 await page.locator('.tab.on').getByText('Today').waitFor();
 await page.goto(`${url}/#sleep`);
 await page.getByRole('button',{name:'Settings'}).click();
 await page.waitForURL('**/#settings');
 await page.locator('.tab.on').getByText('Sleep').waitFor();
 await page.goto(`${url}/#training`);
 await page.getByRole('button',{name:'Settings'}).click();
 await page.waitForURL('**/#settings');
 await page.locator('.tab.on').getByText('Training').waitFor();
 if(pageErrors.length)throw new Error(pageErrors.map(e=>e.message).join('; '));
 console.log('Hash navigation and browser back: passed');
}finally{await browser?.close();await server.close()}
