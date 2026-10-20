// Measures the text inside every chart on every page with one: text below the
// minimum size, text boxes that overlap each other or a marker, and text that
// leaves the screen. Runs at phone, tablet and desktop widths and with text
// scaled the way Android's font size scales a WebView (text only, not layout).
// CHART_REPORT=1 lists every failure instead of stopping at the first page;
// CHART_SHOTS=<name> saves screenshots to ../.data/screens/chart-<name>-*.png.
import assert from 'node:assert/strict';
import {mkdir,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';

const report=!!process.env.CHART_REPORT, shots=process.env.CHART_SHOTS;
const output=resolve('../.data/screens');
const todaySample=u=>u.pathname==='/src/lib/sample/today.json';
const sample=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
const windDownToday={...sample,focus:{...sample.focus,phase:'windDown',cards:['sleepPlan','focus'],until:'2025-10-16T23:06:00'}};
const eveningToday={...sample,focus:{...sample.focus,phase:'evening',cards:['sleepPlan','focus'],until:'2025-10-16T22:00:00'}};

// [name, hash or query, local clock, Today override, section tab to open]
const pages=[
 ['today','#today','2025-10-16T14:05:00'],
 ['today-evening','#today','2025-10-16T20:10:00',eveningToday],
 ['today-wind-down','#today','2025-10-16T22:12:00',windDownToday],
 ['readiness','#readiness'],
 ['sleep-last','#sleep/last'],
 ['sleep-tonight','#sleep/tonight','2025-10-16T21:56:00'],
 ['sleep-tonight-late','#sleep/tonight','2025-10-16T23:20:00'],
 ['sleep-tonight-day','#sleep/tonight','2025-10-16T13:00:00'],
 ['training','#training'],
 ['workout-run','#workout/9000000000001'],
 ['workout-strength','#workout/9000000000002'],
 ['strength','#strength'],
 ['health','#health'],
 ['health-heart-rate','?fixture=heart-rate#health'],
 ['weight','#health/weight'],
 ['progress','#progress'],
 ['progress-fixture','?fixture=progress#progress'],
 ['plan','#plan'],
 ['review','#review/latest'],
 ...['Cardio','Strength','Long-term'].map(tab=>[`progress-${tab.toLowerCase()}`,'#progress',undefined,undefined,tab]),
 ...['bodyBattery','stress','heartRate','restingHr','hrv','respiration','spo2','skinTemp','vo2max','endurance','hill','runningTolerance'].map(m=>[`metric-${m}`,`#metric/${m}`]),
];
const widths=[360,390,720,1100];
const scales=[1,1.3,2];

// Runs in the page: every chart's visible text with its size in CSS pixels.
function measure(){
 const roots=[...document.querySelectorAll('[data-chart],svg')].filter(el=>el.matches('[data-chart]')||el.querySelector('text'));
 const charts=roots.filter(c=>!roots.some(o=>o!==c&&o.contains(c)));
 const visible=el=>el.checkVisibility({visibilityProperty:true,opacityProperty:true,contentVisibilityAuto:true});
 return charts.map(root=>{
  const texts=[],marks=[];
  const walker=document.createTreeWalker(root,NodeFilter.SHOW_TEXT);
  for(let node=walker.nextNode();node;node=walker.nextNode()){
   const el=node.parentElement;
   if(!node.data.trim()||!el||el.closest('title,desc,style,script,.sr-only')||!visible(el))continue;
   const range=document.createRange();range.selectNodeContents(node);
   const rects=[...range.getClientRects()].filter(r=>r.width>0.5&&r.height>0.5);
   if(!rects.length)continue;
   let size=parseFloat(getComputedStyle(el).fontSize);
   if(el instanceof SVGElement){const m=el.getScreenCTM();if(m)size*=Math.min(Math.hypot(m.a,m.b),Math.hypot(m.c,m.d));}
   // Rect heights include line height; shrink each box to the glyphs' em box.
   for(const r of rects){const pad=Math.max(0,(r.height-size)/2);texts.push({text:node.data.trim(),size,box:{l:r.left,r:r.right,t:r.top+pad,b:r.bottom-pad}});}
  }
  for(const el of root.querySelectorAll('[data-mark]')){if(!visible(el))continue;const r=el.getBoundingClientRect();if(r.width||r.height)marks.push({name:el.dataset.mark,box:{l:r.left,r:r.right,t:r.top,b:r.bottom}});}
  const r=root.getBoundingClientRect();
  return {chart:root.dataset.chart||root.getAttribute('aria-label')?.slice(0,40)||root.closest('[class]')?.className.baseVal||root.closest('[class]')?.className||root.tagName,visible:r.width>0&&r.height>0,texts,marks,viewport:innerWidth};
 }).filter(c=>c.visible&&(c.texts.length||c.marks.length));
}

// Runs in the page: gauge numbers that spill out of their ring and, on Today, Sleep
// and Readiness, any text that leaves the screen or its card sideways.
function spills(cards){
 const out=[],walker=cards?document.createTreeWalker(document.querySelector('main')??document.body,NodeFilter.SHOW_TEXT):null;
 for(let node=walker?.nextNode();node;node=walker.nextNode()){
  const el=node.parentElement;
  if(!node.data.trim()||!el||el.closest('title,desc,style,script,.sr-only,select,option,[inert]')||!el.checkVisibility({visibilityProperty:true,opacityProperty:true}))continue;
  // Horizontally scrolled strips (chips, tabs) may run past the edge by design.
  if(el.closest('[data-scroll], .exchips, .tabs-scroll')||[...ancestors(el)].some(a=>/(auto|scroll)/.test(getComputedStyle(a).overflowX)&&a.scrollWidth>a.clientWidth))continue;
  const range=document.createRange();range.selectNodeContents(node);
  const card=el.closest('.card,.your-day,.morning')?.getBoundingClientRect();
  for(const r of range.getClientRects()){
   if(r.width<=0.5)continue;
   if(r.right>innerWidth+1||r.left<-1){out.push(`"${node.data.trim().slice(0,40)}" runs off the screen`);break;}
   if(card&&(r.right>card.right+1||r.left<card.left-1)){out.push(`"${node.data.trim().slice(0,40)}" runs out of its card`);break;}
  }
 }
 for(const g of document.querySelectorAll('.gauge')){
  const b=g.querySelector('.gauge-center b');if(!b||!b.checkVisibility())continue;
  const ring=g.getBoundingClientRect(),range=document.createRange();range.selectNodeContents(b);const t=range.getBoundingClientRect();
  if(t.width>ring.width*.82)out.push(`gauge "${b.textContent}" is wider than its ring`);
 }
 return out;
 function* ancestors(el){for(let a=el.parentElement;a&&a!==document.body;a=a.parentElement)yield a;}
}

// Scales every font the way a larger system font size does: text grows, boxes don't.
function scaleText(k){
 const els=[...document.querySelectorAll('body *')];
 const sizes=els.map(el=>parseFloat(getComputedStyle(el).fontSize));
 els.forEach((el,i)=>el.style.setProperty('font-size',`${sizes[i]*k}px`,'important'));
}

// Axis ticks: numbers, clock times, durations, dates, months and weekdays.
const MONTH='(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sept?|Oct|Nov|Dec)',DAY='(Mon|Tue|Wed|Thu|Fri|Sat|Sun|[MTWFS])';
const tickRe=new RegExp(`^([−\\-+×]?[\\d.,:]+\\s?(′|%|h|m|k|km|mi|kg|lb|bpm|ms|min|°)?|(${DAY} )?(\\d{1,2} )?${MONTH}( \\d{2,4})?|${DAY})$`);
const tick=t=>tickRe.test(t);
const overlap=(a,b,slack=0.75)=>Math.min(a.r,b.r)-Math.max(a.l,b.l)>slack&&Math.min(a.b,b.b)-Math.max(a.t,b.t)>slack;

function problems(charts,width,scale){
 const out=[];
 for(const c of charts){
  for(const t of c.texts){
   // Minimums hold at the default text size; larger text only has to stay clear.
   const min=tick(t.text)?11:12;
   if(scale===1&&width===390&&t.size<min-0.05)out.push(`${c.chart}: "${t.text}" is ${t.size.toFixed(1)}px (< ${min})`);
   if(t.box.l<-1||t.box.r>c.viewport+1)out.push(`${c.chart}: "${t.text}" leaves the screen`);
  }
  for(let i=0;i<c.texts.length;i++)for(let j=i+1;j<c.texts.length;j++){
   const a=c.texts[i],b=c.texts[j];
   if(overlap(a.box,b.box))out.push(`${c.chart}: "${a.text}" overlaps "${b.text}"`);
  }
  for(const t of c.texts)for(const m of c.marks)if(overlap(t.box,m.box))out.push(`${c.chart}: "${t.text}" sits on the ${m.name} marker`);
 }
 return [...new Set(out)];
}

const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
let browser,failures=[];
try {
 await server.listen();
 const url=`http://127.0.0.1:${server.httpServer.address().port}`;
 if(shots)await mkdir(output,{recursive:true});
 browser=await chromium.launch({headless:true,executablePath:'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 for(const [name,path,clock='2025-10-16T14:05:00',today,tab] of pages){
  const ctx=await browser.newContext({viewport:{width:390,height:844},reducedMotion:'reduce'});
  const page=await ctx.newPage();
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.clock.setFixedTime(new Date(clock));
  if(today)await page.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(today)}`}));
  let charts=0;
  for(const width of widths)for(const scale of scales){
   await page.setViewportSize({width,height:900});
   // A hash-only goto keeps the document (and the scaled text); start from a blank page.
   await page.goto('about:blank');
   await page.goto(`${url}/${path}`);
   await page.locator('h1').first().waitFor();
   if(tab){await page.getByRole('tab',{name:tab}).click();await page.locator('.dom').first().waitFor();}
   await page.waitForTimeout(250);
   if(scale!==1){await page.evaluate(scaleText,scale);await page.waitForTimeout(150);}
   // Let resize observers and the label thinning settle.
   await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(()=>setTimeout(r,100)))));
   const found=await page.evaluate(measure);
   charts=Math.max(charts,found.length);
   const bad=[...problems(found,width,scale),...await page.evaluate(spills,/^(today|sleep|readiness)/.test(name))].map(p=>`${name} @${width}px ×${scale}: ${p}`);
   if(shots&&(width===390||width===360)&&(scale===1||scale===2))await page.screenshot({path:resolve(output,`chart-${shots}-${name}-${width}-x${scale}.png`),fullPage:true});
   if(bad.length&&!report)assert.fail(bad.join('\n'));
   failures.push(...bad);
  }
  assert.deepEqual(errors,[],`${name}: page errors`);
  if(report)console.log(`${name}: ${charts} charts`);
  await ctx.close();
 }
 if(report){console.log(failures.join('\n'));console.log(`${failures.length} problems`);}
 else console.log(`Chart text: ${pages.length} pages at ${widths.join('/')} px and ${scales.join('/')}× text, no overlaps or undersized labels: passed`);
} finally {
 await browser?.close();
 await server.close();
}
if(report&&failures.length)process.exitCode=1;
