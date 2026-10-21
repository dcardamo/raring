import assert from 'node:assert/strict';
import {mkdir,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';
const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
const out=resolve('../.data/screens');await mkdir(out,{recursive:true});
const training=JSON.parse(await readFile(resolve('src/lib/sample/training.json'),'utf8'));
// Vite serves JSON imports as JS modules. Supply the complete synthetic view
// as a default-exporting module, without relying on its named-export format.
const trainingWithGear=(page,gear)=>page.route(/\/src\/lib\/sample\/training\.json/,route=>
 route.fulfill({contentType:'application/javascript',body:`export default ${JSON.stringify({...training,gear})};`}));
let browser;
try{
 await server.listen();const url=`http://127.0.0.1:${server.httpServer.address().port}`;
 browser=await chromium.launch({executablePath:'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 for(const [width,label] of [[390,'phone'],[720,'fold'],[1100,'desktop']]){
  const page=await browser.newPage({viewport:{width,height:900}});
  await page.goto(`${url}/?fixture=gear#training`);
  const card=page.locator('.gear-card');await card.waitFor();
  assert.match(await card.textContent(),/Road shoes.*420.*650 km/s);
  assert.match(await card.textContent(),/retired/i);
  assert.match(await card.textContent(),/Distance from Garmin · fetched/);
  await page.screenshot({path:resolve(out,`gear-${label}.png`),fullPage:true});
  await card.getByRole('button',{name:'View gear'}).click();
  assert.match(await card.textContent(),/Spare shoes.*0.*No Garmin distance threshold/s);
  assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=window.innerWidth),true);
  await page.screenshot({path:resolve(out,`gear-${label}-detail.png`),fullPage:true});
  await page.close();
 }
 const unknown=await browser.newPage({viewport:{width:390,height:900}});
 await trainingWithGear(unknown,{...training.gear,items:training.gear.items.slice(0,2).map(item=>
  ({...item,status:'Status unavailable',distanceKm:null,thresholdKm:650}))});
 await unknown.goto(`${url}/?fixture=gear#training`);
 const unknownCard=unknown.locator('.gear-card');await unknownCard.waitFor();
 assert.match(await unknownCard.textContent(),/Active count unavailable · 2 total/);
 assert.equal(await unknownCard.getByText('Distance unavailable',{exact:true}).count(),2);
 assert.equal(await unknownCard.locator('.gear-heading span').filter({hasText:'Status unavailable'}).count(),2);
 assert.equal(await unknownCard.locator('.gear-track').count(),0);
 assert.doesNotMatch(await unknownCard.textContent(),/0 active/);
 for(const label of await unknownCard.locator('.gear-distance b').allTextContents())
  assert.equal(label,'Distance unavailable');
 await unknown.screenshot({path:resolve(out,'gear-unknown-phone.png'),fullPage:true});
 await unknown.close();
 const empty=await browser.newPage();
 await trainingWithGear(empty,null);
 await empty.goto(`${url}/?fixture=gear#training`);
 await empty.locator('h1').getByText('Training').waitFor();
 assert.equal(await empty.locator('.gear-card').count(),0);
 await empty.close();
 console.log('Gear phone/fold/desktop, expanded, all-unknown and empty account: passed');
}finally{await browser?.close();await server.close()}
