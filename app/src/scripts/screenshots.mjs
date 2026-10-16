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
  ['today','today',''],['sleep','sleep',''],['training','training',''],['workout','workout/9000000000001',''],['health','health',''],['readiness','readiness',''],['checkin','checkin',''],['onboarding','onboarding',''],['settings','settings',''],['today-missing','today','?fixture=missing'],['today-empty','today','?fixture=empty']
 ]){
  await page.goto(`${url}/${query}#${route}`);
  await page.waitForFunction(()=>!document.querySelector('.empty') && !document.querySelector('.scroll > .empty') || document.querySelector('.empty')?.textContent?.includes('Syncing your Garmin data') && location.search.includes('empty'));
  await page.evaluate(()=>document.fonts.ready);
  await page.screenshot({path:resolve(output,`${name}.png`)});
  console.log(`${name}: ${resolve(output,`${name}.png`)}`);
  if(name==='checkin'){
   for(const area of ['sleep','training','health','summary']){
    await page.getByRole('button',{name:'Next →'}).click();
    await page.screenshot({path:resolve(output,`checkin-${area}.png`)});
    console.log(`checkin-${area}: ${resolve(output,`checkin-${area}.png`)}`);
   }
  }
  if(pageErrors.length)throw new Error(`${name}: ${pageErrors.map(e=>e.message).join('; ')}`);
 }
}finally{await browser?.close();await server.close()}
