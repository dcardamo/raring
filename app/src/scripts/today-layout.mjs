// Whole-section geometry, not just page overflow: the Fold regression had
// overlapping controls and a detached heading while remaining inside the page.
import assert from 'node:assert/strict';
import {mkdir,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';

const output=resolve('../.data/screens');
const base=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
const lead=(kind,headline)=>({id:`layout:${kind}`,kind,headline,line:null,action:null,dismissable:false,kept:false,garmin:null,activity:null,intention:null,reasons:[],session:null});
const done={...structuredClone(base),watchLastUpload:'2025-10-16T13:30:00',call:null,lead:lead('done','Done today'),focus:{...base.focus,phase:'day',cards:['day','focus','sleepPlan'],effort:false}};
done.activities.push({...done.activities[0],id:'9000000000002',name:'Indoor Cycling',icon:'bike',startLocal:'2025-10-16T12:25:00',durationS:1800,distanceKm:null,load:31,avgHr:115});
done.day.events.push({kind:'activity',id:'9000000000002',name:'Indoor Cycling',icon:'bike',startMin:745,endMin:775,durationMin:30,bodyBatteryChange:null});
const morning={...structuredClone(base),lead:lead('session','A steady day fits')};
morning.lead.session={name:'Base Builder 1',source:'coach',sourceLabel:'Garmin Coach · week 1 of 4',time:null,what:'Strength · gym · about 60 min',effect:null,garminEffect:null,note:null,fit:null};
const evening={...structuredClone(done),lead:lead('sleep','Make room for sleep'),focus:{...done.focus,phase:'evening',cards:['sleepPlan','day','focus'],planOpen:true}};
const waiting={...structuredClone(base),lead:lead('waiting','Last night hasn’t synced yet'),missingMetricReasons:{readiness:'This watch cannot record Training Readiness.'},focus:{...base.focus,phase:'wake',effort:false,sleepMissing:true,lastNight:null}};
const empty={...structuredClone(done),activities:[],day:null};
const cases=[['done',done],['morning',morning],['evening',evening],['waiting',waiting],['empty',empty]];
function scaleText(k){
 const els=[...document.querySelectorAll('body *')],sizes=els.map(el=>parseFloat(getComputedStyle(el).fontSize));
 els.forEach((el,i)=>el.style.setProperty('font-size',`${sizes[i]*k}px`,'important'));
}
function geometry(){
 const rect=el=>{const r=el.getBoundingClientRect();return {l:r.left,r:r.right,t:r.top,b:r.bottom,w:r.width}};
 const box=sel=>{const el=document.querySelector(sel);return el?rect(el):null};
 const spills=[];
 const root=document.querySelector('.today-hero');
 const walker=document.createTreeWalker(root,NodeFilter.SHOW_TEXT);
 for(let node=walker.nextNode();node;node=walker.nextNode()){
  const el=node.parentElement;
  if(!node.data.trim()||el.closest('script,style,title,.sr-only')||!el.checkVisibility())continue;
  const boundary=el.closest('.stat,.card,.your-day,.session,.top,.sync-indicator')??root;
  const r=boundary.getBoundingClientRect(),range=document.createRange();range.selectNodeContents(node);
  if([...range.getClientRects()].some(t=>t.width>0&&(t.left<r.left-1||t.right>r.right+1)))spills.push(node.data.trim());
 }
 return {header:box('.top'),sync:box('.sync-indicator'),stats:box('.morning'),sections:box('.today-sections'),lead:box('.today-lead'),timeline:box('.today-timeline'),recorded:box('.today-lead .recorded'),heading:box('.hero'),
  controls:[...document.querySelectorAll('.top button,.sync-indicator button')].map(rect),statTops:[...document.querySelectorAll('.morning .stat')].map(e=>Math.round(e.getBoundingClientRect().top)),
  overflow:document.querySelector('main').scrollWidth-document.querySelector('main').clientWidth,spills};
}
const overlaps=(a,b)=>Math.min(a.r,b.r)-Math.max(a.l,b.l)>1&&Math.min(a.b,b.b)-Math.max(a.t,b.t)>1;
const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
let browser;
try{
 await mkdir(output,{recursive:true});await server.listen();
 const url=`http://127.0.0.1:${server.httpServer.address().port}`;
 browser=await chromium.launch({headless:true,executablePath:'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 for(const [name,view] of cases){
  const page=await browser.newPage({viewport:{width:390,height:900},reducedMotion:'reduce'});
  const errors=[];page.on('pageerror',e=>errors.push(e.message));
  await page.clock.setFixedTime(new Date(name==='evening'?'2025-10-16T20:45:00':'2025-10-16T14:05:00'));
  await page.route(u=>u.pathname==='/src/lib/sample/today.json',r=>r.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(view)}`}));
  for(const width of [344,390,659,660,707,840,1100])for(const scale of [1,2]){
   await page.setViewportSize({width,height:900});await page.goto('about:blank');await page.goto(`${url}/#today`);
   await page.locator('.morning').waitFor();await page.evaluate(()=>document.fonts.ready);
   if(scale!==1)await page.evaluate(scaleText,scale);
   await page.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
   const g=await page.evaluate(geometry),label=`${name} ${width}px ×${scale}`;
   assert.ok(g.sync.t>=g.header.b-1,`${label}: sync is below header`);
   assert.ok(g.stats.t>=g.sync.b-1,`${label}: metrics are below sync`);
   assert.ok(g.sections.t>=g.stats.b-1,`${label}: metrics span above whole sections`);
   assert.equal(g.stats.w,g.sections.w,`${label}: metrics use content width`);
   for(let i=0;i<g.controls.length;i++)for(let j=i+1;j<g.controls.length;j++)assert.ok(!overlaps(g.controls[i],g.controls[j]),`${label}: controls overlap`);
   if(g.timeline){
    assert.ok(!overlaps(g.lead,g.timeline),`${label}: whole sections do not overlap`);
    if(width>=840&&scale===1)assert.ok(Math.abs(g.lead.t-g.timeline.t)<1,`${label}: sections side by side`);
    if(width===390||scale===2&&width<=840)assert.ok(g.timeline.t>=g.lead.b,`${label}: sections stack`);
   }
   if(g.recorded)assert.ok(g.recorded.l>=g.lead.l-1&&g.recorded.r<=g.lead.r+1&&g.recorded.t>=g.heading.b,`${label}: recordings stay with their heading`);
   assert.ok(g.overflow<=1,`${label}: horizontal overflow`);
   assert.deepEqual(g.spills,[],`${label}: text stays inside its section`);
   if(name!=='evening'&&width===390&&scale===2)assert.equal(new Set(g.statTops).size,2,`${label}: stats wrap 2×2`);
   assert.equal(await page.locator('.recorded .tags,.recorded .zbar,.recorded .incomplete').count(),0,`${label}: details stay off summaries`);
   if(name==='evening'){
    const plan=await page.locator('.sleep-plan').boundingBox(),day=await page.locator('.your-day').boundingBox();
    assert.ok(plan.y+plan.height<=day.y+1,`${label}: tonight before day timeline`);
   }
   if(['done','morning','evening'].includes(name)&&[390,840].includes(width))await page.screenshot({path:resolve(output,`today-layout-${name}-${width}-x${scale}.png`)});
  }
  // Same document: fold/unfold must reflow without navigation or duplicate content.
  await page.goto('about:blank');await page.goto(`${url}/#today`);await page.locator('.morning').waitFor();
  for(const width of [390,840,390]){
   await page.setViewportSize({width,height:900});assert.equal(await page.locator('.morning').count(),1);
   assert.ok(page.url().endsWith('#today'));assert.ok((await page.evaluate(geometry)).overflow<=1);
  }
  if(name==='done'){
   await page.locator('.recorded .activity').first().click();await page.waitForURL('**/#workout/9000000000001');
   await page.locator('h1').waitFor();
  }
  assert.deepEqual(errors,[],`${name}: page errors`);await page.close();
 }
 console.log('Today: coherent sections, separate header/sync, compact recordings, phase order and live folding; 344–1100px at 1×/2× text passed');
}finally{await browser?.close();await server.close()}
