// Plan, Progress and weekly review regressions, run from ui-regression.mjs
// against synthetic Tauri responses.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';

const load=name=>readFile(resolve(`src/lib/sample/${name}.json`),'utf8').then(JSON.parse);

// A page whose Tauri commands answer from `mock`; local actions are recorded.
async function mocked(browser,errors,mock){
 const page=await browser.newPage({viewport:{width:390,height:844}});
 page.on('pageerror',e=>errors.push(e.message));
 await page.addInitScript(mock=>{
  window.isTauri=true;window.actions=[];localStorage.setItem('notifications-step-done','1');
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
   if(command.startsWith('plugin:'))return 1;
   if(command==='local_action'){window.actions.push(args.action);return null}
   if(command==='get_settings')return {};
   if(command in mock)return structuredClone(typeof mock[command]==='function'?mock[command](args):mock[command]);
   return null;
  }};
 },mock);
 return page;
}

export async function planProgressChecks({browser,url,errors}){
 const samples={};
 for(const name of ['today','status','checkin','review','reviews','progress','progressDomains','health'])samples[name]=await load(name);
 samples.plan_board=await load('board');
 const base={...samples,app_status:samples.status,week_review:samples.review,progress_domains:samples.progressDomains};

 // PP-B2/B3: only the latest completed, undecided week offers a decision, and a
 // decision after next week began says it applies from today.
 {
  const page=await mocked(browser,errors,base);
  await page.goto(`${url}/#review/2025-10-06`);
  const card=page.locator('.wr');
  await card.getByRole('button',{name:'Keep all'}).waitFor();
  assert.ok((await card.textContent()).includes('From today, Thu 16 Oct'),await card.textContent());
  await page.close();
  const old=await mocked(browser,errors,{...base,week_review:{...samples.review,decidable:false}});
  await old.goto(`${url}/#review/2025-09-29`);
  await old.locator('.wr').waitFor();
  assert.equal(await old.locator('.wr').getByRole('button',{name:/Keep all|Accept/}).count(),0,'an old week offers no decision');
  assert.equal(await old.locator('.wr input[type=checkbox]').count(),0);
  assert.ok(!(await old.locator('.wr').textContent()).includes('Next week'),'no proposals for a week that can’t decide');
  await old.close();
  const decided=await mocked(browser,errors,{...base,week_review:{...samples.review,decidable:false,decision:{week:'2025-10-06',at:'2025-10-16T09:00:00+01:00',proposals:samples.review.proposals,accepted:[]}}});
  await decided.goto(`${url}/#review/2025-10-06`);
  await decided.locator('.wr').waitFor();
  const text=await decided.locator('.wr').textContent();
  assert.ok(text.includes('Kept (proposed 165)') && !text.includes('Met in your last two'),text);
  await decided.close();
 }
 console.log('Weekly review: decisions only for the latest completed week, from today when late, kept proposals: passed');
 // PP-B5: Plan shows one week, so sessions are placed and moved only within it.
 {
  const page=await mocked(browser,errors,base);
  await page.goto(`${url}/#plan`);
  await page.getByRole('button',{name:'Add a session'}).click();
  const add=page.getByRole('dialog',{name:'Add to my plan'});
  const days=add.getByRole('group',{name:'Day'}).getByRole('button');
  assert.deepEqual(await days.allTextContents(),['Today','Fri 17 Oct','Sat 18 Oct','Sun 19 Oct'],'today through the plan week’s end');
  await add.getByRole('button',{name:'Place today'}).waitFor();
  await days.last().click();
  await add.getByLabel('Session name').fill('Hill repeats');
  await add.getByRole('button',{name:'Place on Sun 19 Oct'}).click();
  const placed=await page.evaluate(()=>window.actions.at(-1));
  assert.equal(placed.date,'2025-10-19');assert.equal(placed.name,'Hill repeats');
  await page.getByRole('button',{name:/^Friday 17 Oct/}).click();
  await page.locator('.row-item',{hasText:'Easy ride'}).click();
  const item=page.getByRole('dialog',{name:'Easy ride'});
  await item.getByRole('button',{name:/^Move/}).click();
  assert.deepEqual(await item.getByRole('group',{name:'Move to'}).getByRole('button').allTextContents(),['Today','Sat 18 Oct','Sun 19 Oct']);
  await page.close();
 }
 console.log('Plan: sessions are placed and moved within the plan week, by name and day: passed');
 // Rings: every goal ring fits (PP-B1), weeks that didn't count say why over
 // faded rings (PP-B12), this week opens Plan, and ring grids carry a key (PP-M11).
 {
  const progress=structuredClone(samples.progress);
  const kinds=['strength','zone2','highIntensity','intensity','sleepTiming'];
  const five=progress.ringDays.findIndex(d=>d.date==='2025-10-06');
  progress.ringDays[five].rings=kinds.map(kind=>({kind,recorded:1,planned:0,fill:1}));
  const page=await mocked(browser,errors,{...base,progress});
  await page.goto(`${url}/#progress`);
  const cal=page.locator('.ring-cal');
  await cal.waitFor();
  const radii=await cal.getByRole('button',{name:/^Monday 6 Oct/}).locator('circle[stroke-width]').evaluateAll(cs=>cs.map(c=>Number(c.getAttribute('r'))));
  assert.equal(radii.length,5,'all five rings draw at 28 px');
  assert.ok(Math.min(...radii)>=3,radii.join());
  assert.ok((await cal.locator('.ring-legend').textContent()).includes('Zone 2'),'the calendar has a colour key');
  const weeks=page.locator('.gw button');
  assert.equal(await weeks.count(),3);
  assert.ok(await weeks.first().evaluate(b=>b.classList.contains('uncounted')&&!!b.querySelector('g[opacity="0.4"]')&&[...b.querySelectorAll('circle')].some(c=>c.getAttribute('fill')?.startsWith('url('))),'an ill week is faded and hatched');
  assert.equal((await weeks.last().textContent()).trim(),'This week');
  assert.equal(await page.getByRole('button',{name:/^Last week/}).count(),0,'the strip replaces the Last week row');
  await page.getByRole('button',{name:/^All weekly reviews/}).waitFor();
  await weeks.last().click();
  await page.waitForFunction(()=>location.hash==='#plan');
  // Plan's days mark what is still planned and Garmin's sessions; the week ring centre is clear.
  const fri=page.getByRole('button',{name:/^Friday 17 Oct/});
  await fri.waitFor();
  assert.ok((await fri.getAttribute('aria-label')).includes('1 session planned'));
  assert.equal(await fri.locator('.marks .pl').count(),1);
  assert.equal(await fri.locator('.marks .gm').count(),1);
  assert.equal(await page.locator('.ws .nc').textContent(),'','no text over the week rings');
  await page.close();
  // Without goals, the calendar shows activity days rather than empty rings (PP-M10).
  const bare=structuredClone(samples.progress);
  bare.ringDays=bare.ringDays.map(d=>({...d,rings:[]}));bare.goalWeeks=[];
  const none=await mocked(browser,errors,{...base,progress:bare});
  await none.goto(`${url}/#progress`);
  await none.locator('.cal').waitFor();
  assert.equal(await none.locator('.ring-cal').count(),0);
  assert.equal(await none.getByRole('button',{name:/weekly reviews/}).count(),0,'no review links without goal weeks');
  await none.close();
 }
 console.log('Rings: every ring fits, uncounted weeks say why, this week opens Plan, colour keys: passed');
 // The weekly review leads with the decision; the readings sit in one grid with
 // labelled days, and only Garmin's scores carry status colours (PP-M6, PP-B6).
 {
  const page=await mocked(browser,errors,base);
  await page.goto(`${url}/#review/2025-10-06`);
  const grid=page.getByRole('region',{name:'Your week'});
  await grid.waitFor();
  const order=await page.locator('.wr, .cw').evaluateAll(els=>els.map(e=>e.className.includes('wr')?'goals':'week'));
  assert.deepEqual(order,['goals','week'],'the goals decision comes first');
  assert.equal(await grid.getByRole('group').count(),4);
  const age=grid.getByRole('group',{name:/^Fitness Age/});
  assert.equal(await age.locator('.n').evaluate(e=>e.style.color),'','Fitness Age has no status colour');
  assert.equal(await age.locator('.cw-bar b').count(),7,'each day labelled');
  assert.ok((await grid.getByRole('group',{name:/^Readiness/}).locator('.n').evaluate(e=>e.style.color))!=='','readiness keeps its score colour');
  assert.equal(await page.locator('.gauge').count(),1,'only the intensity ring is a gauge');
  assert.equal(await page.getByText('Keep the momentum').count(),0);
  await page.close();
 }
 console.log('Weekly review: decision first, one readings grid, score colours only for scores: passed');
 // Progress reads as amounts and dates with years, compares once, and charts
 // each week against its own target (PP-B8, B10, B11, M8, M9, M13–M16).
 {
  const progress=structuredClone(samples.progress);
  progress.previous={...progress.totals,activeMin:91,distanceKm:52.9,activities:4};progress.previousFrom='2025-09-01';progress.previousTo='2025-09-16';
  progress.historyStart='2024-10-23';
  progress.milestones.push({id:'best:ROW/ONE_ARM_DUMBBELL_ROW:1',kind:'best',title:'One Arm Dumbbell Row',value:'12 kg × 10 · heaviest for 1–10 reps',date:'2024-11-02',route:null,new:false,corrected:false});
  const page=await mocked(browser,errors,{...base,progress});
  await page.goto(`${url}/#progress`);
  const report=page.locator('.report');
  await report.waitFor();
  const text=await report.textContent();
  assert.ok(text.includes('4 h 36 min') && text.includes('+3 h 05') && text.includes('−4.3 km'),text);
  assert.equal((text.match(/Compared with/g)??[]).length,1,'one comparison line');
  assert.ok(!text.includes(' vs '),text);
  const chart=report.getByRole('img',{name:/^Activity by day, minutes/});
  await chart.waitFor();
  assert.ok((await chart.getAttribute('aria-label')).includes('Thu 16 Oct: 52 min'));
  assert.ok((await report.locator('text.yu').textContent())==='min','the activity chart has a unit axis');
  const records=await page.locator('.ms').allTextContents();
  assert.ok(records.some(r=>r.includes('12 kg × 10 · heaviest for 1–10 reps · Sat 2 Nov 2024')),records.join('|'));
  assert.ok((await page.locator('main').textContent()).includes('since 23 Oct 2024'));
  await page.getByRole('tab',{name:'Cardio'}).click();
  const im=page.getByRole('region',{name:'Intensity minutes per week'});
  await im.waitFor();
  assert.ok((await im.textContent()).includes('Garmin goal 150'),'the intensity chart follows Garmin’s goal');
  const z2=page.getByRole('region',{name:'Zone 2 minutes per week'});
  const step=await z2.locator('path[stroke-dasharray="4 3"]').getAttribute('d');
  assert.ok(step && step.split('L').length>12,'a stepped goal line across the weeks: '+step);
  assert.ok((await z2.getByRole('img').getAttribute('aria-label')).includes('This week'));
  assert.equal(await z2.locator('rect[fill="none"][stroke-dasharray="3 2"]').count(),1,'this week is outlined');
  await page.getByRole('tab',{name:'Long-term'}).click();
  await page.getByRole('region',{name:'Training per month'}).waitFor();
  const long=await page.locator('main').textContent();
  for(const gone of ['Latest readings','What happened','fitness follows'])assert.ok(!long.includes(gone),gone);
  await page.close();
 }
 console.log('Progress: readable totals, one comparison, dated records, per-week goals, current period marked: passed');
 // Plan's week card gathers what needs you: reviews name the recording and open
 // the session; not-recorded sessions are one row with Reschedule and Let it go
 // (PP-M1, M2); a recording that may be a session says so (PP-B15).
 {
  const board=structuredClone(samples.plan_board);
  const base2=board.items[1];
  const rec={...board.recordings[0],id:'r-run',date:'2025-10-15',name:'Morning Run',startLocal:'2025-10-15T06:12:00',durationMin:52,icon:'run'};
  board.recordings.push(rec);
  const item=(id,date,state,extra={})=>({...base2,state,recording:null,matchedBy:null,candidates:[],coach:null,...extra,intention:{...base2.intention,id,date,sport:'running',name:null,time:null,durationMin:null,status:'open',...(extra.intention??{})}});
  board.items.push(item('i-a','2025-10-15','review',{candidates:['r-run'],intention:{durationMin:45}}),item('i-b','2025-10-15','review',{candidates:['r-run'],intention:{time:'18:00'}}),
   item('i-c','2025-10-13','notRecorded'),item('i-d','2025-10-14','notRecorded',{intention:{name:'Hill ride',sport:'cycling'}}));
  const page=await mocked(browser,errors,{...base,plan_board:board});
  await page.goto(`${url}/#plan`);
  const needs=page.getByRole('group',{name:'Needs you'});
  await needs.waitFor();
  const asks=await needs.getByRole('button',{name:/^Needs review/}).allTextContents();
  assert.deepEqual(asks.map(t=>t.trim()),['Needs review Run (45 min), Wed 15 Oct: Morning Run 06:12?','Needs review Run 18:00, Wed 15 Oct: Morning Run 06:12?'],asks.join('|'));
  await needs.getByRole('button',{name:/^Needs review Run 18:00/}).click();
  await page.getByRole('dialog',{name:'Run'}).getByRole('button',{name:/^This was my session/}).waitFor();
  await page.keyboard.press('Escape');
  await needs.getByRole('button',{name:/^2 not recorded · Mon, Tue/}).click();
  await needs.locator('.nr-row',{hasText:'Hill ride'}).getByRole('button',{name:'Let it go'}).click();
  assert.deepEqual(await page.evaluate(()=>window.actions.at(-1)),{action:'setIntentionStatus',id:'i-d',status:'letGo'});
  await page.getByRole('button',{name:/^Wednesday 15 Oct/}).click();
  const loose=page.locator('.row-item',{hasText:'Morning Run'});
  assert.ok((await loose.textContent()).includes('Needs review · may be Run or Run 18:00'),await loose.textContent());
  await loose.click();
  await page.getByRole('dialog',{name:'Run'}).waitFor();
  await page.keyboard.press('Escape');
  // An event is removed only after confirming.
  page.once('dialog',d=>d.dismiss());
  await page.getByRole('button',{name:/^Remove /}).first().click();
  assert.notEqual((await page.evaluate(()=>window.actions.at(-1))).action,'removeEvent','dismissing keeps the event');
  await page.close();
 }
 console.log('Plan: one Needs you place, recordings named, not-recorded sessions handled inline, candidates labelled: passed');
}
