// Run inside nix develop with a debug build open on the synthetic demo account.
// Checks Profile, My routines, event grouping, week lanes, Training sections and
// lifts through the real core and WebView. Every local change it makes is
// removed before it exits; it never re-seeds.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {createRequire} from 'node:module';
const {chromium}=createRequire(new URL('../app/package.json',import.meta.url))('playwright');
const adb=(...args)=>execFileSync('adb',['-s',process.env.ANDROID_SERIAL??'emulator-5560',...args],{encoding:'utf8'}).trim();
const app=process.env.APP_ID??'app.raring.debug';
const port=adb('forward','tcp:0',`localabstract:webview_devtools_remote_${adb('shell','pidof',app)}`);
const browser=await chromium.connectOverCDP(`http://127.0.0.1:${port}`,{noDefaults:true});
const cleanup=[];
try{
 const page=browser.contexts().flatMap(c=>c.pages()).find(p=>p.url().startsWith('http://tauri.localhost/'));
 assert.ok(page,'attached app WebView is open');
 const invoke=(cmd,args)=>page.evaluate(([c,a])=>window.__TAURI_INTERNALS__.invoke(c,a),[cmd,args]);
 assert.equal(await invoke('demo_session',{}),true,'only the synthetic demo account');
 const waitRoute=route=>page.waitForFunction(r=>location.hash===`#${r}` && document.querySelector('h1'),route);
 const back=async route=>{adb('shell','input','keyevent','4');await waitRoute(route)};
 const fits=async what=>assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`${what} fits the phone width`);
 const local=action=>invoke('local_action',{action});
 await page.goto('http://tauri.localhost/?fixture=demo#today');await waitRoute('today');

 // Profile.
 await page.getByRole('button',{name:'Profile and settings'}).first().click();await waitRoute('profile');
 const facts=await page.getByRole('region',{name:'What Garmin knows'}).textContent();
 for(const want of ['Age46','72.4 kg','Metric · km, kg','Last upload'])assert.ok(facts.includes(want),`${want} in ${facts}`);
 assert.ok(await page.getByRole('button',{name:'Add a weight'}).isDisabled());
 await page.getByRole('button',{name:/^Manage limitations/}).click();
 await page.getByRole('dialog',{name:'Your limits'}).waitFor();
 adb('shell','input','keyevent','4');await page.waitForFunction(()=>!document.querySelector('[role=dialog]'));
 assert.equal(await page.evaluate(()=>location.hash),'#profile','Back closes the limits sheet first');
 await fits('Profile');
 await back('today');
 console.log('Android Profile: Garmin facts, disabled weight write, limitations sheet and Back: passed');

 // My routines, through the UI, then removed.
 await page.locator('.top .adapt').click();
 await page.getByRole('dialog',{name:'Adapt today'}).getByRole('button',{name:/^My routines/}).click();
 const sheet=page.getByRole('dialog',{name:'My routines'});
 await sheet.getByLabel('Exercise').fill('Emulator wall sit');
 await sheet.getByRole('button',{name:'Add to checklist'}).click();
 await sheet.getByText('Emulator wall sit',{exact:true}).waitFor();
 const added=(await invoke('today',{})).adapt.routines.find(r=>r.item.name==='Emulator wall sit');
 assert.ok(added,'the routine is stored by the core');
 cleanup.push(()=>local({action:'removeRoutine',id:added.item.id}));
 await sheet.getByRole('button',{name:'Mark done today'}).click();
 await sheet.getByRole('button',{name:/Done today · self-reported/}).waitFor();
 assert.equal((await invoke('today',{})).adapt.routines.find(r=>r.item.id===added.item.id).done,true);
 await sheet.getByRole('button',{name:'Remove Emulator wall sit'}).click();
 await page.waitForFunction(()=>!document.querySelector('[role=dialog]')?.textContent.includes('Emulator wall sit'));
 cleanup.pop();
 adb('shell','input','keyevent','4');await page.waitForFunction(()=>!document.querySelector('[role=dialog]'));
 console.log('Android My routines: add, self-report and remove through the core: passed');

 // An event groups today's run; both removed afterwards.
 const today=(await invoke('today',{})).date;
 const later=new Date(Date.parse(today+'T12:00:00Z')+20*864e5).toISOString().slice(0,10);
 await local({action:'addEvent',name:'Emulator 10K',date:later,sport:'running'});
 const ev=(await invoke('plan_board',{})).events.find(e=>e.name==='Emulator 10K');
 cleanup.push(()=>local({action:'removeEvent',id:ev.id}));
 await local({action:'addIntention',date:today,time:'21:30',sport:'running',typeKey:null,name:'Emulator tempo',durationMin:30,
  allocation:{strengthSessions:0,zone2Min:0,highIntensityMin:0,intensityMin:0,sleepNights:0},kind:'workout',garminId:null});
 const run=(await invoke('plan_board',{})).items.find(p=>p.intention.name==='Emulator tempo');
 cleanup.push(()=>local({action:'removeIntention',id:run.intention.id}));
 await page.getByRole('navigation').getByRole('button',{name:'Plan',exact:true}).click();await waitRoute('plan');
 const group=page.getByRole('group',{name:'Emulator 10K this week'});
 await group.waitFor();
 const groupText=await group.textContent();
 // The demo's running event is soonest unless another event with a sport comes first.
 assert.ok(/Emulator tempo/.test(groupText),groupText);
 await fits('Plan with an event group');
 await back('today');
 console.log('Android event week: today’s run grouped under its event: passed');

 // Garmin Coach from the demo's synthetic adaptive plan: read-only, matched by kind through the core.
 const board=await invoke('plan_board',{});
 assert.ok(board.coach && board.coach.sessions.length>=1,'the board carries the Coach week');
 const coachItems=board.items.filter(p=>p.coach);
 const lifted=coachItems.find(p=>p.coach.kind==='strength');
 if(lifted)assert.equal(lifted.state,'done',"yesterday's Coach strength matches the demo's strength activity");
 await page.getByRole('navigation').getByRole('button',{name:'Plan',exact:true}).click();await waitRoute('plan');
 const coachGroup=page.getByRole('group',{name:'Sample Coach this week'});
 await coachGroup.waitFor();
 await coachGroup.getByRole('button').first().click();
 const coachSheet=page.getByRole('dialog');
 await coachSheet.getByText(/Garmin Coach · read-only/).waitFor();
 assert.equal(await coachSheet.getByRole('button',{name:/^Edit|^Move|^Let it go/}).count(),0,'Garmin’s session is read-only');
 adb('shell','input','keyevent','4');await page.waitForFunction(()=>!document.querySelector('[role=dialog]'));
 await fits('Plan with Garmin Coach');
 await back('today');
 console.log(`Android Garmin Coach: ${coachItems.length} sessions as read-only planned work${lifted?', strength matched':''}: passed`);

 // Week lanes and Training sections.
 await page.getByRole('navigation').getByRole('button',{name:'Training',exact:true}).click();await waitRoute('training');
 const lanes=page.locator('.week-lanes');
 await lanes.waitFor();
 assert.ok(await lanes.getByRole('img').count()>=1,'at least one lane');
 const tw=(await invoke('training',{})).lanes;
 assert.ok(tw && tw.lanes.length>=1,'the live training read carries lanes');
 console.log(`Android week lanes: ${tw.lanes.map(l=>`${l.kind} ${l.cumulative.filter(v=>v!=null).at(-1)??'?'}${l.usual?' (band)':''}`).join(', ')}`);
 for(const q of ['Enough, or too much?','Is my fitness improving?','What am I ready for?'])await page.getByText(q,{exact:true}).waitFor();
 await page.getByRole('tab',{name:'Strength'}).click();
 await page.locator('.q-muscles').getByRole('list',{name:'Sets per muscle group'}).waitFor();
 await page.locator('.q-muscles').getByRole('group',{name:'Muscle range'}).getByRole('button',{name:'7 days'}).click();
 await page.getByRole('tab',{name:'Activities'}).click();
 await page.locator('.q-activities .activity').first().waitFor();
 await fits('Training');
 console.log('Android Training: lanes, Fitness questions, muscles with a range, activities: passed');

 // Lifts on the Strength page.
 await page.getByRole('tab',{name:'Strength'}).click();
 await page.getByRole('button',{name:/^Lifts and exercises/}).click();await waitRoute('strength');
 await page.getByRole('group',{name:'Lift measure'}).getByRole('button',{name:'Volume'}).click();
 await page.getByText(/Volume each session: weight × reps/).waitFor();
 await page.getByRole('group',{name:'Lift range'}).getByRole('button',{name:'All history'}).click();
 const series=await invoke('lift_series',{key:(await invoke('strength',{})).exercises[0].key,range:'all'});
 assert.ok(series && series.points.length>=1,'the core returns the lift series');
 await fits('Strength');
 await back('training');await back('today');
 console.log(`Android lifts: Volume and range through the core (${series.points.length} session${series.points.length===1?'':'s'}): passed`);
 // Workout detail from the stored streams.
 const morning=(await invoke('training',{})).workouts.find(w=>w.name==='Morning Run');
 await page.goto(`http://tauri.localhost/?fixture=demo#workout/${morning.id}`);await waitRoute(`workout/${morning.id}`);
 const along=page.getByRole('region',{name:'Along the way'});
 await along.waitFor();
 for(const name of [/^Heart rate over/,/^Pace over/,/^Elevation over/])assert.equal(await along.getByRole('img',{name}).count(),1,String(name));
 const did=await page.getByRole('region',{name:'What it did'}).textContent();
 assert.ok(did.includes('14 moderate + 2 × 24 vigorous') && did.includes('Improving'),did);
 await fits('Workout detail');
 await back('today');
 console.log('Android workout detail: unit-checked streams, intensity minutes and training effect: passed');

 // Health order and Progress sections.
 await page.getByRole('navigation').getByRole('button',{name:'Health',exact:true}).click();await waitRoute('health');
 const titles=await page.locator('.health-dashboard>.card h3>span:first-child').allTextContents();
 assert.equal(titles[0],'Long-term health · Fitness Age',titles.join(', '));
 assert.ok(titles.indexOf('Vitals')<titles.indexOf('Resting heart rate'),titles.join(', '));
 await fits('Health');
 await page.getByRole('navigation').getByRole('button',{name:'Progress',exact:true}).click();await waitRoute('progress');
 await page.locator('.ring-cal .dc').first().waitFor();
 for(const [tab,region] of [['Cardio','Zone 2 minutes per week'],['Strength','Strength sessions per week'],['Long-term','Training per month']]){
  await page.getByRole('tab',{name:tab}).click();
  await page.getByRole('region',{name:region}).waitFor();
  await fits(`Progress ${tab}`);
 }
 const domains=await invoke('progress_domains',{});
 assert.ok(domains.weeks.length>=1 && domains.months.length===12,'the core returns domains');
 await page.getByRole('tab',{name:'Strength'}).click();
 const liftCharts=await page.getByRole('region',{name:/over the year$/}).count();
 await page.getByRole('tab',{name:'Overall'}).click();
 const archive=page.getByRole('button',{name:/^All records and milestones/});
 if(await archive.count()){await archive.click();await waitRoute('progress/milestones');await fits('Milestone archive');await back('progress');}
 await page.getByRole('button',{name:'Help: Rings'}).click();
 await page.getByRole('dialog',{name:'Rings'}).waitFor();
 adb('shell','input','keyevent','4');await page.waitForFunction(()=>!document.querySelector('[role=dialog]'));
 assert.equal(await page.evaluate(()=>location.hash),'#progress','Back closes help first');
 console.log(`Android Progress: ${liftCharts} lift chart(s), archive ${await archive.count()?'opened':'not shown (no milestones)'}, help sheet and Back: passed`);
 await back('today');
 console.log(`Android Health order and Progress sections (${domains.weeks.length} weeks, ${domains.bests.length} best${domains.bests.length===1?'':'s'}): passed`);
}finally{
 for(const undo of cleanup.reverse())await undo().catch(e=>console.error('cleanup failed',e));
 await browser.close();
}
