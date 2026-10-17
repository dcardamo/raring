// Synthetic-only strength screenshots. Keep the normal app layout and clipping:
// plain viewport captures, with additional frames for content below the fold.
import assert from 'node:assert/strict';
import {mkdir,writeFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';
const output=resolve(process.env.RARING_SHOTS??'../.data/strength-screens');
await mkdir(output,{recursive:true});
const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
let browser;
const measurements=[];
try{
 await server.listen();
 const url=`http://127.0.0.1:${server.httpServer.address().port}`;
 browser=await chromium.launch({executablePath:'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 for(const [size,width] of [['phone',390],['unfolded',720],['desktop',1280]]){
  const page=await browser.newPage({viewport:{width,height:900},deviceScaleFactor:1});
  for(const [name,route,ready] of [['workout','workout/9000000000002','.strength-cols'],['history','strength','.strength-history'],['help','help/strength-sets','.help-steps']]){
   await page.goto(`${url}/#${route}`);
   await page.locator(ready).waitFor();
   await page.evaluate(()=>document.fonts.ready);
   const dimensions=await page.evaluate(()=>({viewport:innerWidth,document:document.documentElement.scrollWidth,body:document.body.scrollWidth,main:document.querySelector('main').scrollWidth,glowRight:document.querySelector('.glow').getBoundingClientRect().right}));
   measurements.push({size,name,...dimensions});
   assert.equal(dimensions.document,width);
   assert.equal(dimensions.body,width);
   assert.ok(dimensions.main<=width);
   await page.screenshot({path:resolve(output,`${size}-${name}.png`),fullPage:false});
   const count=await page.locator('.scroll').evaluate(el=>1+Math.ceil((el.scrollHeight-el.clientHeight)/(el.clientHeight-100)));
   for(let i=1;i<count;i++){
    await page.locator('.scroll').evaluate((el,i)=>el.scrollTop=i*(el.clientHeight-100),i);
    await page.screenshot({path:resolve(output,`${size}-${name}-${i+1}.png`),fullPage:false});
   }
  }
  if(size==='phone'){
   await page.goto(`${url}/#workout/9000000000002`);
   await page.locator('.strength-cols').waitFor();
   // Reproduce the old capture-only CSS mutation, not a production layout.
   const artifact=await page.evaluate(()=>{
    const scroll=document.querySelector('.scroll'),shell=document.querySelector('.phone');
    scroll.style.position='static';scroll.style.overflow='visible';
    shell.style.height='auto';shell.style.overflow='visible';
    return {viewport:innerWidth,document:document.documentElement.scrollWidth,body:document.body.scrollWidth,glowRight:document.querySelector('.glow').getBoundingClientRect().right};
   });
   measurements.push({captureArtifact:artifact});
  }
  await page.close();
 }
 await writeFile(resolve(output,'widths.json'),JSON.stringify(measurements,null,2)+'\n');
 console.log(JSON.stringify(measurements,null,2));
}finally{await browser?.close();await server.close()}
