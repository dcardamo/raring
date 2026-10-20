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
 for(const name of ['today','status','checkin','review','reviews','progress','progressDomains'])samples[name]=await load(name);
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
}
