import assert from 'node:assert/strict';
import {existsSync} from 'node:fs';
import {mkdir,readFile} from 'node:fs/promises';
import {createServer} from 'node:http';
import {extname,resolve,sep} from 'node:path';
import {chromium} from 'playwright';

const root=resolve('../site'),output=resolve('../.data/screens/site');
const types={'.html':'text/html; charset=utf-8','.css':'text/css','.svg':'image/svg+xml','.png':'image/png','.woff2':'font/woff2'};
const server=createServer(async(req,res)=>{
 try{
  const pathname=decodeURIComponent(new URL(req.url,'http://localhost').pathname);
  const file=resolve(root,'.'+(pathname.endsWith('/')?pathname+'index.html':pathname));
  if(!file.startsWith(root+sep)){res.writeHead(403).end();return}
  const data=await readFile(file);
  res.writeHead(200,{'Content-Type':types[extname(file)]??'text/plain'}).end(data);
 }catch{res.writeHead(404).end()}
});
let browser;
try{
 await mkdir(output,{recursive:true});
 await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
 const origin=`http://127.0.0.1:${server.address().port}`;
 const executablePath=process.env.PLAYWRIGHT_CHROMIUM_EXECUTABLE_PATH
  ??(existsSync('/run/current-system/sw/bin/chromium')?'/run/current-system/sw/bin/chromium':undefined);
 browser=await chromium.launch({headless:true,executablePath,args:['--no-sandbox']});
 const context=await browser.newContext({javaScriptEnabled:false});
 const unexpected=[],failures=[],errors=[];
 await context.route('**/*',route=>{
  if(new URL(route.request().url()).origin===origin)return route.continue();
  unexpected.push(route.request().url());return route.abort();
 });
 const page=await context.newPage();
 page.on('pageerror',e=>errors.push(e.message));
 page.on('response',response=>{if(response.status()>=400)failures.push(`${response.status()} ${response.url()}`)});
 const links=new Set();
 for(const width of [320,390,768,1280]){
  await page.setViewportSize({width,height:900});
  for(const path of ['/','/privacy.html']){
   await page.goto(origin+path);
   assert.equal(await page.locator('h1').count(),1);
   assert.equal(await page.locator('script,iframe,form').count(),0,'static pages have no scripts, embeds or forms');
   await page.evaluate(()=>document.fonts.ready);
   if(path==='/'){
    assert.equal(await page.getByText('The app, with sample data',{exact:true}).count(),0);
    assert.equal(await page.locator('.free-note').textContent(),'Free to use. No subscription.');
    const free=await page.locator('.free-note').boundingBox();
    assert.ok(free && free.y+free.height<=900,`${width}: free-to-use message is above the fold`);
    assert.deepEqual(await page.locator('.screen-gallery img').evaluateAll(images=>images.map(img=>img.getAttribute('src'))),['assets/sleep.png','assets/training.png','assets/health.png']);
   }
   for(const img of await page.locator('img').all()){
    await img.scrollIntoViewIfNeeded();
    await img.evaluate(image=>image.decode());
   }
   if(path==='/')await page.locator('.screen-gallery').evaluate(gallery=>{gallery.scrollLeft=0});
   await page.evaluate(()=>scrollTo(0,0));
   const layout=await page.evaluate(()=>({width:innerWidth,body:document.body.scrollWidth,document:document.documentElement.scrollWidth}));
   assert.ok(layout.body<=width && layout.document<=width,`${width}${path}: overflow ${JSON.stringify(layout)}`);
   assert.ok(await page.locator('img').evaluateAll(images=>images.every(img=>img.complete && img.naturalWidth>0)),'images load');
   for(const href of await page.locator('a[href]').evaluateAll(anchors=>anchors.map(a=>a.href)))links.add(href);
   if(width===390 || width===1280)await page.screenshot({path:resolve(output,`${width}-${path==='/'?'home':'privacy'}.png`),fullPage:true});
  }
 }
 await page.setViewportSize({width:390,height:900});
 await page.goto(origin);
 const gallery=page.getByRole('region',{name:'App screenshots',exact:true});
 await gallery.focus();
 await gallery.press('ArrowRight');
 await page.waitForFunction(()=>document.querySelector('.screen-gallery').scrollLeft>0);
 await gallery.locator('figure').last().scrollIntoViewIfNeeded();
 assert.ok(await gallery.evaluate(el=>el.scrollLeft+el.clientWidth>=el.scrollWidth-2),'last screen is reachable without JavaScript');
 // Follow every local link, including fragments, without fetching third-party sites.
 for(const href of links){
  const target=new URL(href);
  if(target.origin!==origin){assert.equal(target.protocol,'https:');continue}
  const response=await page.goto(href);
  if(response)assert.equal(response.status(),200,href);
  if(target.hash)assert.equal(await page.locator(`[id="${target.hash.slice(1)}"]`).count(),1,href);
 }
 await page.goto(origin);
 assert.equal(await page.locator('meta[property="og:image"]').getAttribute('content'),'https://raring.app/assets/social.png');
 const social=await readFile(resolve(root,'assets/social.png'));
 assert.equal(social.subarray(1,4).toString(),'PNG');
 assert.equal(social.readUInt32BE(16),1200);
 assert.equal(social.readUInt32BE(20),630);
 await page.keyboard.press('Tab');
 assert.equal(await page.locator(':focus').textContent(),'Skip to content');
 await page.keyboard.press('Enter');
 assert.equal(new URL(page.url()).hash,'#main');
 await page.getByRole('link',{name:'Get Raring for Android'}).click();
 assert.equal(new URL(page.url()).hash,'#install');
 assert.equal(await page.getByRole('link',{name:'Open the latest release'}).getAttribute('href'),'https://github.com/dcardamo/raring/releases/latest');
 await page.getByRole('link',{name:'Read the privacy details'}).click();
 assert.ok(page.url().endsWith('/privacy.html'));
 await page.getByRole('heading',{name:'This website and downloads'}).waitFor();
 assert.ok((await page.locator('main').textContent()).includes('GitHub receives your IP address'));
 assert.equal((await context.cookies()).length,0,'site sets no cookies');
 assert.deepEqual(unexpected,[],'no third-party resource requests');
 assert.deepEqual(failures,[],'no broken resources');
 assert.deepEqual(errors,[],'no browser errors');
 console.log('Static site: 320–1280px layouts, free-to-use copy, screenshot gallery, assets, links, install/privacy navigation and no third-party requests passed with JavaScript disabled.');
}finally{
 await browser?.close();
 await new Promise(resolve=>server.close(resolve));
}
