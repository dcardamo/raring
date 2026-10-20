import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';

export async function visualLanguageChecks(browser,url,output){
 const sample=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
 const page=await browser.newPage({viewport:{width:390,height:844}});
 await page.clock.setFixedTime(new Date('2025-10-16T08:00:00'));
 try{
  for(const effort of ['rest','easy','steady','push']){
   const view={...structuredClone(sample),lead:null,call:effort,focus:{...sample.focus,effort:true}};
   const match=u=>u.pathname==='/src/lib/sample/today.json';
   await page.route(match,r=>r.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(view)}`}));
   for(const width of [360,390,720])for(const scale of [1,2]){
    await page.setViewportSize({width,height:844});await page.goto('about:blank');await page.goto(`${url}/#today`);
    const bar=page.getByRole('img',{name:`Suggested effort from Garmin readiness: ${effort}`,exact:true});await bar.waitFor();await page.evaluate(()=>document.fonts.ready);
    if(scale===2)await bar.evaluate(el=>{const nodes=[el,...el.querySelectorAll('*')],sizes=nodes.map(n=>getComputedStyle(n).fontSize);nodes.forEach((n,i)=>n.style.fontSize=parseFloat(sizes[i])*2+'px')});
    const geometry=await bar.evaluate(el=>{
     const svg=el.querySelector('svg'),paths=[...svg.querySelectorAll('.seg')],pick=svg.querySelector('.pick'),labels=[...el.querySelectorAll('.sw')].map(n=>n.getBoundingClientRect());
     const gaps=[];for(const y of [9,16,23])for(let x=4;x<397;x++)if(!paths.some(p=>p.isPointInFill(new DOMPoint(x,y))))gaps.push([x,y]);
     const box=svg.getBoundingClientRect(),b=pick.getBBox();
     return {gaps,bands:paths.map(p=>({y:p.getBBox().y,height:p.getBBox().height})),pick:{x:Math.round(b.x),y:b.y,height:b.height},picks:svg.querySelectorAll('.pick').length,caps:svg.querySelectorAll('.cap').length,
      centers:labels.map(r=>(r.left+r.right)/2-box.left),width:box.width,overflow:el.scrollWidth>el.clientWidth+1};
    });
    const index=['rest','easy','steady','push'].indexOf(effort);
    assert.deepEqual(geometry.gaps,[],`${effort} ${width} ×${scale}: joined chevrons have no holes`);
    assert.deepEqual(geometry.bands,Array(4).fill({y:8,height:16}),'selection never changes the shared rail');
    assert.deepEqual(geometry.pick,{x:index*100,y:3,height:26},'the suggested level is its own chevron, taller above and below the rail');
    assert.equal(geometry.picks,1);assert.equal(geometry.caps,0,'no separate raised tab');
    geometry.centers.forEach((c,i)=>assert.ok(Math.abs(c-geometry.width*(i+.5)/4)<1,'labels stay centered on equal segments'));
    assert.equal(geometry.overflow,false);assert.equal(await bar.locator('.sw.on').count(),1);assert.equal(await bar.locator('button').count(),0);
    if(width===390)await bar.screenshot({path:resolve(output,`effort-${effort}-x${scale}.png`)});
   }
   await page.unroute(match);
  }
  await page.goto(`${url}/#health`);
  const range=page.getByRole('group',{name:'History period'}).first();await range.waitFor();
  await range.getByRole('button',{name:'90 days',exact:true}).click();
  const active=range.locator('button.active');assert.equal(await active.getAttribute('aria-label'),'90 days');
  assert.deepEqual(await active.evaluate(el=>{const s=getComputedStyle(el);return [s.backgroundColor,s.color,s.borderRadius,s.boxShadow]}),['rgb(244, 241, 234)','rgb(17, 19, 23)','10px','none'],'range picker restores the light selected segment without an underline');
  await range.screenshot({path:resolve(output,'range-picker-restored.png')});
  console.log('Visual language: four effort levels at 360/390/720 and 1/2× text, one taller chevron on a shared rail, centered labels and original range selection: passed');
 }finally{await page.close()}
}
