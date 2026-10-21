// Reading order, usable controls and live folding across the app. Chart text
// is checked separately by chart-legibility; this catches section/shell bugs.
import assert from 'node:assert/strict';
import {mkdir,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';
const output=resolve('../.data/screens');
const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
const sample=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
const board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
const cases=[['today'],['plan'],['training'],['training','Strength'],['training','Activities'],['health'],['progress'],['progress','Cardio'],['progress','Strength'],['progress','Long-term'],['sleep/last'],['sleep/tonight'],['readiness'],['strength'],['workout/9000000000001'],['workout/9000000000002'],['health/weight'],['metric/hrv'],['settings'],['profile']];
function scaleText(k){
 const nodes=[...document.querySelectorAll('body *')],sizes=nodes.map(e=>parseFloat(getComputedStyle(e).fontSize));
 nodes.forEach((e,i)=>e.style.setProperty('font-size',sizes[i]*k+'px','important'));
}
function geometry(){
 const bad=[],rect=e=>e.getBoundingClientRect(),overlap=(a,b)=>Math.min(a.right,b.right)-Math.max(a.left,b.left)>1&&Math.min(a.bottom,b.bottom)-Math.max(a.top,b.top)>1;
 const main=document.querySelector('main'),nav=document.querySelector('.tabs');
 if(main.scrollWidth>main.clientWidth+1)bad.push('page overflow: '+[...main.querySelectorAll('*')].filter(e=>e.checkVisibility()&&rect(e).right>main.clientWidth+1).slice(0,8).map(e=>`${e.tagName}.${e.getAttribute('class')??''} ${e.textContent.slice(0,60)}`).join('; '));
 if(rect(main).bottom>rect(nav).top+1)bad.push('navigation covers scroll area');
 for(const grid of document.querySelectorAll('.dashboard,.today-sections,.strength-cols,.strength-history')){
  if(getComputedStyle(grid).display!=='grid')continue;
  const boxes=[...grid.children].filter(e=>e.checkVisibility()).map(rect);
  for(let i=1;i<boxes.length;i++)if(boxes[i].top<boxes[i-1].top-1)bad.push('section reading order changed');
  for(let i=0;i<boxes.length;i++)for(let j=i+1;j<boxes.length;j++)if(overlap(boxes[i],boxes[j]))bad.push('sections overlap');
  for(const child of grid.children)if(child.style.gridRowEnd)bad.push('height-dependent placement');
 }
 for(const label of document.querySelectorAll('.tab-label')){
  if(!label.checkVisibility())continue;
  const r=rect(label.closest('button')),range=document.createRange();range.selectNodeContents(label);
  for(const t of range.getClientRects())if(t.left<r.left-1||t.right>r.right+1||t.top<r.top-1||t.bottom>r.bottom+1)bad.push('navigation label leaves its button');
 }
 const controls=[...document.querySelectorAll('.top button')].map(rect);
 for(let i=0;i<controls.length;i++)for(let j=i+1;j<controls.length;j++)if(overlap(controls[i],controls[j]))bad.push('header controls overlap');
 return bad;
}
function dialogGeometry(){
 const d=document.querySelector('[role=dialog]'),r=d.getBoundingClientRect(),bad=[];
 if(r.top<0||r.bottom>innerHeight+1||r.left<0||r.right>innerWidth+1)bad.push('dialog leaves viewport');
 if(d.scrollWidth>d.clientWidth+1)bad.push('dialog horizontal overflow');
 const walker=document.createTreeWalker(d,NodeFilter.SHOW_TEXT);
 for(let n=walker.nextNode();n;n=walker.nextNode()){
  const e=n.parentElement;if(!n.data.trim()||!e.checkVisibility()||e.closest('style,script,svg'))continue;
  const range=document.createRange();range.selectNodeContents(n);
  if([...range.getClientRects()].some(t=>t.left<r.left-1||t.right>r.right+1))bad.push('text leaves dialog: '+n.data.trim());
 }
 for(const e of d.querySelectorAll('input,textarea,button')){const b=e.getBoundingClientRect();if(e.checkVisibility()&&(b.left<r.left-1||b.right>r.right+1))bad.push('control leaves dialog');}
 return bad;
}
let browser;
try{
 await mkdir(output,{recursive:true});await server.listen();const url=`http://127.0.0.1:${server.httpServer.address().port}`;
 browser=await chromium.launch({headless:true,executablePath:'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 for(const [route,tab] of cases){
  const p=await browser.newPage({viewport:{width:390,height:900},reducedMotion:'reduce'});
  await p.clock.setFixedTime(new Date('2025-10-16T14:05:00'));
  for(const scale of [1,2]){
   await p.goto('about:blank');await p.goto(`${url}/#${route}`);await p.locator('h1').first().waitFor();
   if(tab)await p.getByRole('tab',{name:tab,exact:true}).click();
   if(route==='strength')await p.locator('.mlist').waitFor();
   if(route==='health/weight')await p.locator('.weight-card .fresh').waitFor();
   await p.evaluate(()=>document.fonts.ready);if(scale!==1)await p.evaluate(scaleText,scale);
   const before=await p.locator('h1').first().textContent();
   for(const width of [344,390,660,707,840,1100,1280,390]){
    await p.setViewportSize({width,height:900});await p.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
    const label=`${route}/${tab??''} ${width} ×${scale}`;
    assert.deepEqual(await p.evaluate(geometry),[],label);
    assert.equal(await p.locator('h1').first().textContent(),before,`${label}: resize preserves screen`);
    if(tab)assert.equal(await p.getByRole('tab',{name:tab,exact:true}).getAttribute('aria-selected'),'true',`${label}: selected section survives`);
    if(route==='strength'){
     const wide=width>=1176&&scale===1,twoColumns=width>=756&&scale===1;
     assert.equal(await p.locator('.exercises-col').isVisible(),wide,`${label}: wide exercise list`);
     assert.equal(await p.locator('.exchips').isVisible(),!wide,`${label}: exactly one exercise selector`);
     if(!wide)assert.equal(await p.locator('.exchips').evaluate(e=>getComputedStyle(e).flexWrap),twoColumns?'wrap':'nowrap',`${label}: selector wraps with page layout, not column width`);
    }
    if([390,840].includes(width))await p.screenshot({path:resolve(output,`adaptive-${route.replaceAll('/','-')}${tab?'-'+tab:''}-${width}-x${scale}.png`)});
    await p.locator('main').evaluate(e=>e.scrollTop=e.scrollHeight);
    assert.ok(await p.locator('.page').evaluate(e=>e.getBoundingClientRect().bottom<=document.querySelector('.tabs').getBoundingClientRect().top+1),`${label}: last content is reachable above navigation`);
    await p.locator('main').evaluate(e=>e.scrollTop=0);
   }
  }
  await p.close();
 }
 console.log('Adaptive sections, navigation labels, last-content reachability and live resize: 20 page/section states at 344–1280px and 1×/2× passed');
 const sheetCases=[
  ['search','today',async p=>{await p.getByRole('button',{name:'Search',exact:true}).click();await p.getByRole('searchbox').fill('sleep')}],
  ['adapt','today',async p=>p.locator('.top .adapt').click()],
  ['limits','today',async p=>{await p.locator('.top .adapt').click();await p.getByRole('dialog').getByRole('button',{name:/^Limits/}).click()}],
  ['routines','today',async p=>{await p.locator('.top .adapt').click();await p.getByRole('dialog').getByRole('button',{name:/^My routines/}).click()}],
  ['why','readiness',async p=>p.getByRole('button',{name:'Why High?',exact:true}).click()],
  ['help','workout/9000000000002',async p=>p.getByRole('button',{name:'Help',exact:true}).click()],
  ['quick-help','training',async p=>p.getByRole('button',{name:'Help: Acute load',exact:true}).click()],
  ['add','plan',async p=>p.getByRole('button',{name:'Add a session',exact:true}).click()],
  ['event','plan',async p=>p.getByRole('button',{name:'Add a race or event',exact:true}).click()],
  ['item','plan',async p=>{await p.getByRole('button',{name:/^Friday 17 Oct/}).click();await p.locator('.row-item',{hasText:'Easy ride'}).click();await p.getByRole('dialog').getByRole('button',{name:/^Edit/}).click()}],
  ['garmin','plan',async p=>p.getByRole('button',{name:/Fold test session/}).click()],
  ['choose','today',async p=>p.getByRole('button',{name:'Choose another',exact:true}).click()],
 ];
 for(const [name,route,open] of sheetCases)for(const scale of [1,2]){
  const p=await browser.newPage({viewport:{width:390,height:900},reducedMotion:'reduce'});
  await p.clock.setFixedTime(new Date('2025-10-16T14:05:00'));
  if(name==='garmin')await p.route(u=>u.pathname==='/src/lib/sample/board.json',r=>r.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify({...board,garmin:[...board.garmin,{name:'Fold test session',date:board.today}]})}`}));
  if(name==='choose')await p.route(u=>u.pathname==='/src/lib/sample/today.json',r=>r.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify({...sample,lead:{id:'session:fold',kind:'session',headline:'A steady day fits',line:null,action:null,dismissable:false,kept:false,garmin:null,activity:null,intention:'fold',reasons:[],session:{name:'Fold test session',source:'own',sourceLabel:'Your plan',time:null,what:'Run · about30 min',effect:null,garminEffect:null,note:null,fit:null}}})}`}));
  await p.goto(`${url}/#${route}`);await p.locator('h1').waitFor();await open(p);const dialog=p.getByRole('dialog');await dialog.waitFor();await p.evaluate(()=>document.fonts.ready);
  if(scale!==1)await p.evaluate(scaleText,scale);
  for(const width of [344,840,390]){
   await p.setViewportSize({width,height:900});await p.evaluate(()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r))));
   assert.deepEqual(await p.evaluate(dialogGeometry),[],`${name} ${width} ×${scale}`);
   for(const control of [dialog.getByRole('button',{name:'Close',exact:true}),dialog.locator('button').last()]){
    await control.scrollIntoViewIfNeeded();assert.ok(await control.evaluate(e=>{const r=e.getBoundingClientRect();return r.top>=0&&r.bottom<=innerHeight+1}),`${name}: control reachable after resize`);
   }
   await dialog.evaluate(e=>e.scrollTop=0);await p.screenshot({path:resolve(output,`adaptive-sheet-${name}-${width}-x${scale}.png`)});
  }
  await dialog.getByRole('button',{name:'Close',exact:true}).click();await dialog.waitFor({state:'detached'});await p.close();
 }
 console.log('Twelve sheet/form states at 344/390/840px and 1×/2× text: controls and close reachable through folding passed');
}finally{await browser?.close();await server.close()}
