import assert from 'node:assert/strict';
import {mkdir,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';
import {contrastFailures} from './contrast.mjs';

const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
const output=resolve('../.data/screens');
await mkdir(output,{recursive:true});
let browser;
try {
 await server.listen();
 const url=`http://127.0.0.1:${server.httpServer.address().port}`;
 browser=await chromium.launch({headless:true,executablePath:'/run/current-system/sw/bin/chromium',args:['--no-sandbox']});
 const page=await browser.newPage({viewport:{width:390,height:844},hasTouch:true,isMobile:true});
 const errors=[];
 for(const width of [390,700,1100]){
  const exportPage=await browser.newPage({viewport:{width,height:844}});
  await exportPage.goto(`${url}/?fixture=export#settings`);
  await exportPage.getByRole('button',{name:'Export your data'}).waitFor();
  assert.match(await exportPage.getByText('Download a ZIP', {exact:false}).textContent(),/only when you share it/);
  await exportPage.getByRole('button',{name:'Export your data'}).scrollIntoViewIfNeeded();
  await exportPage.screenshot({path:resolve(output,`export-settings-${width}.png`),fullPage:true});
  await exportPage.getByRole('button',{name:'Export your data'}).click();
  await exportPage.getByText('Preview: no file created').waitFor();
  await exportPage.close();
 }
 console.log('Export settings at 390/700/1100 and preview handoff: passed');
 page.on('pageerror',e=>errors.push(e.message));
 for(const width of [390,700,1100]){
  const watch=await browser.newPage({viewport:{width,height:844}});
  await watch.goto(`${url}/?fixture=stale#today`);
  await watch.getByText('Open Garmin Connect to sync your watch',{exact:true}).waitFor();
  await watch.getByText("Your watch doesn't record skin temperature",{exact:true}).waitFor();
  await watch.screenshot({path:resolve(output,`watch-stale-${width}.png`)});
  await watch.getByRole('button',{name:'Open Garmin Connect'}).click();
  await watch.getByText('Open Garmin Connect on your phone, then sync your watch there.',{exact:true}).waitFor();
  await watch.close();
 }
 const fresh=await browser.newPage({viewport:{width:390,height:844}});
 await fresh.goto(`${url}/?fixture=fresh#today`);
 await fresh.getByText('Watch synced 2 h ago',{exact:true}).waitFor();
 assert.equal(await fresh.getByText('Open Garmin Connect to sync your watch',{exact:true}).count(),0);
 await fresh.screenshot({path:resolve(output,'watch-fresh-390.png')});
 await fresh.close();
 const sleepUpload=await browser.newPage({viewport:{width:390,height:844}});
 await sleepUpload.goto(`${url}/?fixture=stale-sleep#today`);
 await sleepUpload.getByText("Last night's sleep hasn't uploaded yet.",{exact:true}).waitFor();
 assert.equal(await sleepUpload.getByText('Open Garmin Connect to sync your watch',{exact:true}).count(),1,'missing sleep gives one sync prompt in the upload card');
 assert.equal(await sleepUpload.getByText('About missing readings',{exact:true}).count(),0,'upload delays are not unsupported capabilities');
 await sleepUpload.close();
 console.log('Watch freshness, missing capability, Connect fallback at 390/700/1100: passed');
 const waitRoute=async(route)=>{
  await page.waitForURL(`${url}/#${route}`);
  await page.locator('h1').waitFor();
  await page.waitForFunction(()=>!document.querySelector('.empty'));
 };
 const tab=async(name)=>{await page.getByRole('navigation').getByRole('button',{name,exact:true}).click();await waitRoute(name.toLowerCase())};
 const back=async(route)=>{await page.goBack();await waitRoute(route)};
 await page.goto(`${url}/#today`);
 await waitRoute('today');
 assert.equal(await page.title(),'Raring','browser uses the app name');
 await tab('Sleep');await tab('Training');await tab('Health');
 await back('today');
 await tab('Training');
 await page.getByRole('button',{name:'Settings',exact:true}).click();await waitRoute('settings');
 await back('training');await back('today');
 await page.getByRole('button',{name:/^Body Battery/}).click();await waitRoute('metric/bodyBattery');
 await page.getByRole('button',{name:'Today',exact:true}).first().click();await waitRoute('today');
 await page.getByRole('button',{name:'View HRV',exact:true}).first().click();await waitRoute('metric/hrv');
 await page.getByText('Garmin baseline',{exact:true}).waitFor();
 await back('today');
 await page.getByRole('button',{name:/^Stress/}).click();await waitRoute('metric/stress');
 const timeline=page.getByRole('slider',{name:/^Stress through the day/});
 assert.equal(await timeline.getAttribute('aria-valuetext'),'13:27: '+await page.locator('.readout b').textContent()+', '+await page.locator('.readout em').textContent());
 await timeline.focus();await timeline.press('Home');
 assert.match(await timeline.getAttribute('aria-valuetext'),/^00:00: \d+, Rest$/);
 for(let i=0;i<6;i++)await timeline.press('Shift+ArrowRight');
 for(let i=0;i<4;i++)await timeline.press('ArrowRight');
 assert.equal(await timeline.getAttribute('aria-valuetext'),'06:12: active, not rated','movement is a labelled gap, not zero');
 for(let i=0;i<21;i++)await timeline.press('ArrowRight');
 assert.equal(await timeline.getAttribute('aria-valuetext'),'07:15: no reading','off-wrist time is a gap, not zero');
 assert.equal(await page.locator('.stress-row').count(),5);
 await back('today');
 await tab('Health');
 await page.getByRole('button',{name:/^Stress/}).click();await waitRoute('metric/stress');
 await back('health');
 await page.getByRole('button',{name:'View Respiration',exact:true}).click();await waitRoute('metric/respiration');
 await back('health');await back('today');
 await page.locator('.activity').first().click();await waitRoute('workout/9000000000001');
 await back('today');
 await tab('Training');
 await page.locator('.activity').first().click();await waitRoute('workout/9000000000001');
 await page.reload();await waitRoute('workout/9000000000001');
 await back('training');await back('today');
 await tab('Sleep');await tab('Sleep');await back('today');
 assert.equal(await page.evaluate(()=>window.__raringBack()),false,'Today lets Android leave the app');
 await tab('Training');
 assert.equal(await page.evaluate(()=>window.__raringBack()),true);
 await waitRoute('today');
 console.log('Hierarchical browser, header and native Back: passed');

 await page.getByRole('button',{name:/^This morning/}).click();await waitRoute('readiness');
 await page.getByText('Readiness history',{exact:true}).waitFor();
 await page.getByRole('button',{name:'30 days',exact:true}).click();
 assert.equal(await page.locator('.history-bar').count(),30);
 await back('today');
 // After waking, tonight's plan is one line that opens Sleep.
 await page.getByRole('button',{name:/^Tonight · Garmin plan/}).click();await waitRoute('sleep');
 assert.ok((await page.locator('.sleep-plan').textContent()).includes('23:06'));
 assert.ok((await page.locator('.sleep-plan').textContent()).includes('22:30–23:06'));
 const plan=page.locator('.sleep-plan');
 assert.ok((await plan.textContent()).includes('Wind down22:00'),'the plan leads with the wind-down time');
 assert.ok((await plan.textContent()).includes('Garmin plan'));
 assert.ok((await plan.textContent()).includes("Garmin added 20 min for today's activity and load."));
 await plan.getByRole('button',{name:'Why this plan'}).click();
 const evidence=await plan.locator('.evidence').textContent();
 assert.ok(evidence.includes("Garmin's recommendation for tonight.") && evidence.includes("Training: today's activity and load") && evidence.includes('30 min before the window starts'),evidence);
 const alignment=page.locator('.alignment');
 assert.equal(await alignment.locator('h3 .pill').textContent(),'Aligned');
 assert.equal(await alignment.locator('.night').count(),7,'a week of nights against Garmin\'s optimal window');
 assert.equal(await alignment.locator('rect.optimal').getAttribute('stroke-dasharray'),'4 3','Garmin\'s target is outlined, never shaded like usual windows');
 assert.ok((await alignment.textContent()).includes('Optimal 22:45–06:15'));
 assert.ok((await page.locator('.consistency .averages').textContent()).includes('22:55'));
 await tab('Health');
 assert.equal(await page.locator('.dashboard>.card').first().locator('h3>span').first().textContent(),'Vitals');
 assert.equal(await page.locator('.signal-tile').count(),5);
 await page.getByRole('button',{name:'View HRV',exact:true}).click();await waitRoute('metric/hrv');
 const history=page.locator('.history'),slider=history.getByRole('slider');
 assert.equal(await slider.getAttribute('aria-valuetext'),'Thu 16 Oct: 58 ms');
 await slider.focus();await slider.press('ArrowLeft');
 assert.ok((await slider.getAttribute('aria-valuetext')).startsWith('Wed 15 Oct:'));
 await history.getByRole('button',{name:'30 nights',exact:true}).click();
 await slider.focus();await slider.press('Home');
 assert.ok((await slider.getAttribute('aria-valuetext')).startsWith('Wed 17 Sept:'));
 assert.equal(await history.locator('.history-gap').count(),2,'missing nights remain gaps');
 for(let i=0;i<9;i++)await slider.press('ArrowRight');
 assert.equal(await slider.getAttribute('aria-valuetext'),'Fri 26 Sept: Not recorded');
 assert.equal(await history.locator('.history-reading b').textContent(),'—','missing is never zero');
 await slider.press('End');
 await history.getByRole('button',{name:'Previous reading'}).click();
 assert.ok((await slider.getAttribute('aria-valuetext')).startsWith('Wed 15 Oct:'));
 await history.getByRole('button',{name:'Next reading'}).click();
 const chart=await history.locator('.chart-surface').boundingBox();
 await history.locator('.chart-surface').click({position:{x:chart.width*.55,y:chart.height*.5}});
 assert.notEqual(await slider.inputValue(),'29','chart taps select a reading');
 await page.goto(`${url}/#checkin`);await waitRoute('checkin');
 for(let i=0;i<4;i++)await page.getByRole('button',{name:'Next'}).click();
 const timing=page.locator('.timing-review');
 await timing.waitFor();
 assert.equal(await page.locator('.checkin-controls span').textContent(),'5 / 8','the timing review follows the area cards');
 const review=await timing.textContent();
 assert.ok(review.includes('Bedtimes22:31–00:30') && review.includes('Garmin: 5 aligned · 2 behind') && review.includes('Keep waking near 06:48.'),review);
 assert.equal(await timing.locator('rect.night').count(),7);
 await page.getByRole('button',{name:'Next'}).click();
 assert.ok((await page.locator('.intensity-review').textContent()).includes("Garmin's goal reached"));
 await page.getByRole('button',{name:'Next'}).click();
 const planReview=page.locator('.plan-checkin');
 assert.ok((await planReview.textContent()).includes('Zone 2 and Asleep on time came up short'));
 assert.deepEqual(await planReview.locator('.delta').allTextContents(),['same','▲ 12','▼ 13','▼ 1']);
 await planReview.getByRole('button',{name:'Keep'}).click();
 await page.getByText('THE FOCUS',{exact:true}).waitFor();
 await page.getByRole('button',{name:'Previous'}).click();
 await page.locator('.plan-checkin').getByRole('button',{name:'Change'}).click();await waitRoute('plan-setup');
 await back('plan');
 await page.goto(`${url}/#today`);await waitRoute('today');
 await page.locator('.plan-card').click();await waitRoute('plan');
 assert.equal(await page.locator('.plan-week .goal[data-state="behind"]').count(),1,'Zone 2 alone is behind pace');
 assert.equal(await page.locator('.plan-history .week').count(),8);
 await page.getByRole('button',{name:'Edit targets'}).click();await waitRoute('plan-setup');
 await page.getByRole('button',{name:'More strength sessions'}).click();
 assert.equal(await page.getByRole('radio',{name:'Custom'}).getAttribute('aria-checked'),'true','an edited target makes the plan custom');
 await page.getByRole('radio',{name:'Endurance'}).click();
 assert.equal(await page.locator('.plan-targets .stepper b').first().textContent(),'1');
 assert.equal(await page.getByRole('button',{name:'Mon',exact:true}).getAttribute('aria-pressed'),'true','presets keep rest days');
 await page.getByRole('button',{name:'More intensity minutes'}).click();
 await page.getByRole('button',{name:"Use Garmin's 150"}).click();
 await back('plan');await back('today');
 console.log('Weekly plan, setup, check-in plan and intensity pages: passed');
 for(const width of [390,700,1100]){
  for(const fixture of ['plan-partial','plan-unavailable']){
   const evidence=await browser.newPage({viewport:{width,height:1000}});
   evidence.on('pageerror',e=>errors.push(e.message));
   for(const route of ['plan','checkin']){
    await evidence.goto(`${url}/?fixture=${fixture}#${route}`);
    if(route==='checkin'){
     await evidence.locator('.checkin-controls').waitFor();
     for(let i=0;i<7 && !(await evidence.locator('.plan-checkin').count());i++)await evidence.getByRole('button',{name:'Next',exact:true}).click();
    }
    const card=evidence.locator(route==='plan'?'.plan-week':'.plan-checkin');
    await card.waitFor();
    const text=await card.textContent();
    assert.ok(text.includes(fixture==='plan-partial'?'Partial evidence':'Not enough evidence'));
    assert.ok(text.includes('Workout list incomplete') && text.includes('Needs a saved sleep plan and recorded night'));
    assert.ok(!text.includes('All reached')&&!text.includes('Every target reached'));
    if(route==='plan' && fixture==='plan-partial'){
     assert.equal(await evidence.locator('.plan-history .now b').textContent(),'100*');
     const color=await evidence.locator('.plan-history .now .track i').evaluate(el=>getComputedStyle(el).backgroundColor);
     assert.notEqual(color,'rgb(61, 220, 151)','known-only 100% is not all reached');
    }
    assert.deepEqual(await contrastFailures(evidence),[],`${width}/${fixture}/${route}: contrast`);
    assert.ok(await evidence.evaluate(()=>document.querySelector('main').scrollWidth<=document.querySelector('main').clientWidth+1));
    await evidence.screenshot({path:resolve(output,`weekly-evidence-${fixture}-${width}-${route}.png`)});
   }
   await evidence.close();
  }
 }
 console.log('Partial/unavailable plan evidence at phone, unfolded and desktop widths: passed');
 await page.goto(`${url}/#settings`);await waitRoute('settings');
 assert.equal(await page.locator('.setting.lead').count(),0,'the lead appears with the bedtime reminder');
 await page.locator('label.setting',{hasText:'Bedtime'}).locator('input').check();
 await page.locator('.setting.lead select').selectOption('45');
 await tab('Today');
 await page.getByRole('button',{name:'Settings',exact:true}).click();await waitRoute('settings');
 assert.equal(await page.locator('.setting.lead select').inputValue(),'45');
 console.log('Daily focus, sleep plan, rhythm, timing review, reminder lead and keyboard/touch history: passed');

 await page.setViewportSize({width:390,height:844});
 await page.goto(`${url}/#training`);await waitRoute('training');
 await page.getByRole('button',{name:/^Strength · this week/}).click();await waitRoute('strength');
 await page.getByText('Heaviest set each session',{exact:true}).waitFor();
 const strengthSample=JSON.parse(await readFile(resolve('src/lib/sample/strength.json'),'utf8'));
 const historyWorkouts=JSON.parse(await readFile(resolve('src/lib/sample/strengthWorkoutHistory.json'),'utf8'));
 const assertStrengthWorkout=async(id,date)=>{
  const expected=historyWorkouts[id];
  assert.equal(expected.summary.id,id);
  assert.equal(expected.summary.startLocal.slice(0,10),date);
  await page.locator('.strength-cols').waitFor();
  assert.equal(await page.locator('h1').textContent(),expected.summary.name);
  const label=new Date(date+'T12:00:00').toLocaleDateString('en-GB',{weekday:'short',day:'numeric',month:'short'});
  assert.ok((await page.locator('.top .eyebrow').textContent()).includes(label));
  assert.equal(await page.locator('.strength-cols .hero-n').textContent(),String(expected.strength.sets));
  const rendered=await page.locator('.strength-cols .ex').evaluateAll(cards=>cards.map(card=>({
   name:card.querySelector('.ex-h b').textContent,
   sets:[...card.querySelectorAll('.sbar[role=listitem]')].map(set=>[set.querySelector('b').textContent,set.querySelector('span').textContent])
  })));
  assert.deepEqual(rendered,expected.strength.exercises.map(e=>({name:e.name,sets:e.sets.map(s=>[String(s.reps),String(s.weight)])})));
  await page.locator('.scroll').evaluate(el=>el.scrollTop=0);
  await page.screenshot({path:resolve(output,`strength-older-${id}.png`),fullPage:false});
 };
 assert.ok(strengthSample.exercises[0].sessions.length>=7 && strengthSample.workouts.length>=9);
 assert.equal(await page.locator('.ses').count(),6);
 await page.getByRole('button',{name:'Show all 12 sessions',exact:true}).click();
 assert.equal(await page.locator('.ses').count(),12);
 const oldSession=strengthSample.exercises[0].sessions[6];
 await page.locator('.ses').nth(6).click();await waitRoute('workout/'+oldSession.activityId);
 await assertStrengthWorkout(oldSession.activityId,oldSession.date);
 await back('strength');
 assert.equal(await page.locator('.strength-history .workout-item').count(),8);
 await page.getByRole('button',{name:'Show all 11 workouts',exact:true}).click();
 assert.equal(await page.locator('.strength-history .workout-item').count(),11);
 await page.getByText('No sets recorded',{exact:true}).waitFor();
 const oldWorkout=strengthSample.workouts[9].activity;
 assert.notEqual(oldWorkout.id,oldSession.activityId,'exercise and workout links cover different older IDs');
 await page.locator('.strength-history .workout-item').nth(9).getByRole('button').click();await waitRoute('workout/'+oldWorkout.id);
 await assertStrengthWorkout(oldWorkout.id,oldWorkout.startLocal.slice(0,10));
 await back('strength');
 assert.equal(await page.locator('.heat-card').isVisible(),false,'the weekly heatmap waits for wider screens');
 await page.getByRole('button',{name:'Barbell Bench Press',exact:true}).click();
 assert.equal(await page.locator('.card h3',{hasText:'Barbell Bench Press'}).count(),1);
 await page.getByRole('button',{name:'4 weeks',exact:true}).click();
 await page.getByText('Last 4 weeks',{exact:true}).waitFor();
 await page.getByRole('button',{name:/^Wed 15 Oct/}).click();await waitRoute('workout/9000000000111');
 await back('strength');
 await page.getByRole('button',{name:'Help',exact:true}).click();await waitRoute('help');
 await page.getByRole('button',{name:'How sets are counted'}).click();await waitRoute('help/strength-sets');
 await page.getByText(/1\.8 times/).waitFor();
 await page.getByText(/Later edits will not appear in Raring, even after Refresh/).waitFor();
 await back('help');await back('strength');await back('training');
 await page.getByRole('button',{name:/^Lower A/}).click();await waitRoute('workout/9000000000002');
 await page.getByText('3 things look off',{exact:true}).waitFor();
 await page.getByText('A 0-rep set, a doubled set and an unnamed exercise',{exact:true}).waitFor();
 assert.equal(await page.locator('.sbar.flag').count(),5,'one 0-rep, one doubled and three unnamed sets are flagged');
 assert.equal(await page.getByText('Training load',{exact:true}).count(),0,'strength workouts show sets instead of the effect gauges');
 await page.getByRole('button',{name:'3 things look off'}).click();
 await page.getByRole('dialog').getByText('How sets are counted').waitFor();
 await page.getByRole('button',{name:'More help'}).click();await waitRoute('help/strength-sets');
 await back('workout/9000000000002');
 assert.equal(await page.getByRole('dialog').count(),0,'the sheet closes when its page opens');
 // Synthetic mixed and missing-count responses exercise the hero as well as
 // the pure formatting helper. No account data or network access to Garmin.
 const repPage=await browser.newPage({viewport:{width:390,height:844}});
 repPage.on('pageerror',e=>errors.push(e.message));
 for(const [reps,unknown,label] of [[14,1,'Reps · 1 set uncounted'],[null,2,'Reps unknown'],[0,0,'Reps']]){
  await repPage.route(u=>u.pathname==='/src/lib/sample/strengthWorkout.json',async route=>{
   const v=JSON.parse(await readFile(resolve('src/lib/sample/strengthWorkout.json'),'utf8'));
   Object.assign(v.strength,{reps,unknownRepSets:unknown,missingRepSets:unknown});
   await route.fulfill({contentType:'application/javascript',body:`export default ${JSON.stringify(v)}`});
  });
  await repPage.goto('about:blank');
  await repPage.goto(`${url}/#workout/9000000000002`);
  const cell=repPage.locator('.grid2>div').nth(1);
  await cell.getByText(label,{exact:true}).waitFor();
  assert.equal(await cell.locator('b').textContent(),reps==null?'—':String(reps));
  assert.ok(await repPage.evaluate(()=>document.querySelector('main').scrollWidth<=innerWidth));
  await repPage.unrouteAll();
 }
 await repPage.close();
 console.log('Strength mixed/unknown reps, complete lists and older links, flags and 48-hour help: passed');

 for(const width of [344,360,390,660,707,840,1024]) {
  await page.setViewportSize({width,height:900});
  for(const route of ['today','sleep','training','health','readiness','workout/9000000000001','workout/9000000000002','strength','help','help/strength-sets','metric/hrv','metric/restingHr','metric/bodyBattery','metric/stress','settings','checkin','plan','plan-setup','onboarding']) {
   await page.goto(`${url}/#${route}`);await waitRoute(route);
   const layout=await page.evaluate(()=>{
    // Largest vertical gap between cards stacked in the same grid column.
    const holes=(grid)=>{
     if(!grid || getComputedStyle(grid).display!=='grid')return 0;
     const cards=[...grid.querySelectorAll(':scope>*,:scope>.row>*')].filter(e=>getComputedStyle(e).display!=='contents').map(e=>e.getBoundingClientRect());
     return Math.max(0,...cards.map(c=>{const above=cards.filter(o=>o!==c && o.left<c.right-1 && o.right>c.left+1 && o.bottom<=c.top+1);return above.length?c.top-Math.max(...above.map(o=>o.bottom)):0}));
    };
    // On wide screens Today's headline sits beside the dials, not below them.
    const hero=()=>{const d=document.querySelector('.today-hero>.dials'),h=document.querySelector('.today-hero>.headline');return d&&h?h.getBoundingClientRect().top<d.getBoundingClientRect().bottom:null};
    const main=document.querySelector('main'),grid=document.querySelector('.dashboard'),phone=document.querySelector('.phone');
    return {viewport:innerWidth,body:document.body.scrollWidth,main:main.clientWidth,content:main.scrollWidth,shell:phone.clientWidth,height:phone.clientHeight,columns:grid?getComputedStyle(grid).gridTemplateColumns.split(' ').length:0,holes:holes(grid),hero:hero()};
   });
   assert.equal(layout.shell,width,`${width}/${route}: shell uses available width`);
   assert.equal(layout.height,900,`${width}/${route}: shell uses available height`);
   assert.ok(layout.body<=width && layout.content<=layout.main+1,`${width}/${route}: horizontal overflow ${JSON.stringify(layout)}`);
   if(width>=660 && layout.columns)assert.equal(layout.columns,2,`${width}/${route}: two columns`);
   if(route==='today')assert.equal(layout.hero,width>=660,`${width}/today: headline beside dials only when unfolded`);
   assert.ok(layout.holes<=18,`${width}/${route}: ${layout.holes}px hole between cards`);
   if([390,840].includes(width) && ['today','sleep','training','workout/9000000000001','workout/9000000000002','strength','help/strength-sets','health','plan','plan-setup'].includes(route)) {
    await page.evaluate(()=>document.fonts.ready);
    const failures=await contrastFailures(page);
    assert.deepEqual(failures,[],`${width}/${route}: text contrast`);
   }
   if(width===840 || (width===390 && ['today','sleep','training','workout/9000000000001','health','metric/restingHr','onboarding','settings','plan','plan-setup'].includes(route)) || (width===1024 && ['today','sleep','health','metric/restingHr','plan','plan-setup'].includes(route)) || (width===660 && ['health','metric/restingHr'].includes(route)))await page.screenshot({path:resolve(output,`${width}-${route.replace('/','-')}.png`)});
  }
 }
 console.log('Today, Sleep, Training, Workout, Health and plan text contrast at 390/840: passed');
 // Folding/unfolding should reflow in place, without losing the open detail.
 await page.setViewportSize({width:390,height:844});
 await page.goto(`${url}/#metric/hrv`);await waitRoute('metric/hrv');
 await page.setViewportSize({width:840,height:900});await waitRoute('metric/hrv');
 await page.setViewportSize({width:390,height:844});await waitRoute('metric/hrv');
 console.log('Phone, unfolded and live resize layouts: passed');

 // Why sheets list Garmin's inputs and close by button, scrim, Escape and Android Back.
 await page.goto(`${url}/#readiness`);await waitRoute('readiness');
 await page.getByRole('button',{name:'Why High?',exact:true}).click();
 const whySheet=page.getByRole('dialog',{name:'Why readiness is High'});
 await whySheet.waitFor();
 assert.equal(await whySheet.locator('.why-row').count(),6);
 assert.ok((await whySheet.textContent()).includes('Garmin · Well recovered'));
 assert.equal(await page.evaluate(()=>document.activeElement?.getAttribute('aria-label')),'Close','focus moves into the sheet');
 await page.evaluate(()=>document.fonts.ready);
 assert.deepEqual(await contrastFailures(page),[],'Why sheet text contrast');
 await page.screenshot({path:resolve(output,'390-why-readiness.png')});
 await page.keyboard.press('Escape');await whySheet.waitFor({state:'detached'});
 assert.equal(await page.evaluate(()=>document.activeElement?.textContent),'Why High?','focus returns to the button');
 await page.getByRole('button',{name:'Why High?',exact:true}).click();await whySheet.waitFor();
 assert.equal(await page.evaluate(()=>window.__raringBack()),true);
 await whySheet.waitFor({state:'detached'});
 assert.ok(page.url().endsWith('#readiness'),'Back closes the sheet before leaving the screen');
 await page.goto(`${url}/#metric/bodyBattery`);await waitRoute('metric/bodyBattery');
 await page.getByRole('button',{name:'Why 78?',exact:true}).click();
 const batterySheet=page.getByRole('dialog',{name:'Why Body Battery is 78'});
 assert.ok((await batterySheet.textContent()).includes('Sleep charged it by 62'));
 await page.screenshot({path:resolve(output,'390-why-body-battery.png')});
 await batterySheet.getByRole('button',{name:'Close'}).click();await batterySheet.waitFor({state:'detached'});
 await page.goto(`${url}/#training`);await waitRoute('training');
 await page.getByRole('button',{name:'Why Productive?',exact:true}).click();
 const statusSheet=page.getByRole('dialog',{name:'Why Productive'});
 await statusSheet.waitFor();
 await page.screenshot({path:resolve(output,'390-why-training.png')});
 await page.locator('.why-scrim').click({position:{x:20,y:20}});await statusSheet.waitFor({state:'detached'});
 for(const width of [840,1024]){
  await page.setViewportSize({width,height:900});
  await page.getByRole('button',{name:'Why Productive?',exact:true}).click();await statusSheet.waitFor();
  const box=await statusSheet.boundingBox();
  assert.ok(box.y>=0 && box.y+box.height<=900,`${width}: sheet fits the window`);
  if(width>=1000)assert.ok(Math.abs(box.x+box.width/2-width/2)<2,'desktop shows a centred dialog');
  await page.screenshot({path:resolve(output,`${width}-why-training.png`)});
  await page.keyboard.press('Escape');await statusSheet.waitFor({state:'detached'});
 }
 await page.setViewportSize({width:390,height:844});
 // Scores that differ: one line on Today behind a tap. Vite serves JSON imports as modules.
 const differ=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
 Object.assign(differ,{readiness:{...differ.readiness,value:34,label:'Low',tone:'poor'},headline:'Take it easy',call:'easy',
  reconcile:{key:'bodyBattery',label:'Why Body Battery and readiness differ',tones:['good','poor'],line:'Body Battery charged to 87 overnight, but readiness is held back by acute load.'}});
 const todaySample=u=>u.pathname==='/src/lib/sample/today.json';
 await page.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(differ)}`}));
 // A new query string forces a full load, so the module graph refetches the sample.
 await page.goto(`${url}/?differ#today`);await page.waitForURL(`${url}/?differ#today`);await page.locator('h1').waitFor();
 const reconcile=page.getByRole('button',{name:'Why Body Battery and readiness differ'});
 assert.equal(await reconcile.getAttribute('aria-expanded'),'false');
 assert.equal(await page.getByText(differ.reconcile.line).isVisible(),false,'the line sits behind a tap');
 await reconcile.click();
 assert.equal(await reconcile.getAttribute('aria-expanded'),'true');
 await page.getByText(differ.reconcile.line).waitFor();
 await page.evaluate(()=>document.fonts.ready);
 assert.deepEqual(await contrastFailures(page),[],'reconciliation text contrast');
 await page.screenshot({path:resolve(output,'390-today-reconcile.png')});
 await page.setViewportSize({width:840,height:900});
 await page.screenshot({path:resolve(output,'840-today-reconcile.png')});
 await page.setViewportSize({width:390,height:844});
 await page.unroute(todaySample);
 await page.goto(`${url}/#today`);await waitRoute('today');
 console.log('Why sheets and scores that differ: passed');

 // Today through the day: the same data at different clocks. Only the leading card changes; the hero,
 // its reconciliation pill and the metric cards stay put, and nothing invites effort after wind-down.
 const base=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
 base.reconcile=differ.reconcile;
 const night={asleepMin:468,needMin:450,woke:'06:39'};
 const phases=[
  ['wake','07:10',{phase:'wake',cards:['focus','sleepPlan'],until:'2025-10-16T09:39:00',effort:true,planOpen:false},'Ready to push','.daily-focus'],
  ['day','14:20',{phase:'day',cards:['now','focus','sleepPlan'],until:'2025-10-16T20:30:00',effort:true,planOpen:false},'Ready to push','.now-card'],
  ['evening','20:45',{phase:'evening',cards:['sleepPlan','focus'],until:'2025-10-16T22:00:00',effort:true,planOpen:true},'Wind down at 22:00','.sleep-plan'],
  ['winddown','22:10',{phase:'windDown',cards:['sleepPlan','focus'],until:'2025-10-16T23:06:00',effort:false,planOpen:true},'Time to wind down','.sleep-plan'],
  ['late','23:40',{phase:'lateNight',cards:['sleepPlan','focus'],until:'2025-10-17T06:36:00',effort:false,planOpen:true},"Rest when you're ready",'.sleep-plan'],
  ['missing','2025-10-17T07:30',{phase:'wake',cards:['focus'],until:'2025-10-17T12:00:00',effort:false,planOpen:false,sleepMissing:true,lastNight:null},"Waiting for last night's sleep",'.daily-focus'],
  ['overview','14:20',{phase:'overview',cards:['focus','sleepPlan'],until:null,effort:true,planOpen:true,note:'Fixed layout: your wake times vary too much to plan the day around.'},'Ready to push','.daily-focus'],
 ];
 const phasePage=await browser.newPage({viewport:{width:390,height:844},hasTouch:true});
 phasePage.on('pageerror',e=>errors.push(e.message));
 let heroTop=null;
 for(const [name,time,focus,headline,leader] of phases){
  const clock=new Date(time.includes('T')?time:`2025-10-16T${time}:00`);
  // A recent upload, so the watch notice (which stays above the leading card) is absent.
  const view={...structuredClone(base),headline,watchLastUpload:new Date(clock.getTime()-15*60_000).toISOString(),focus:{sleepMissing:false,lastNight:night,note:null,recoveryHours:18,...focus}};
  await phasePage.clock.setFixedTime(clock);
  await phasePage.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(view)}`}));
  await phasePage.setViewportSize({width:390,height:844});
  await phasePage.goto(`${url}/?phase=${name}#today`);await phasePage.getByRole('heading',{name:headline}).waitFor();
  await phasePage.evaluate(()=>document.fonts.ready);
  const first=await phasePage.locator('.dashboard > *').first().evaluate(el=>el.className);
  assert.ok(first.includes(leader.slice(1)) && first.includes('lead'),`${name}: ${leader} leads (${first})`);
  const top=await phasePage.locator('.dials').evaluate(el=>Math.round(el.getBoundingClientRect().top));
  heroTop??=top;
  assert.equal(top,heroTop,`${name}: the scores stay where they are`);
  const pill=phasePage.getByRole('button',{name:differ.reconcile.label});
  await pill.click();
  await phasePage.getByText(differ.reconcile.line).waitFor();
  assert.equal(await phasePage.locator('.daily-focus .segments').count(),focus.effort?1:0,`${name}: effort call ${focus.effort?'shown':'hidden'}`);
  const text=await phasePage.locator('.dashboard').textContent();
  // The weekly plan's "N days left" counts the week, not the time to bed.
  const advice=await phasePage.locator('.dashboard > :not(.plan-card)').allTextContents();
  assert.ok(!/\bin \d+ ?(min|h)\b|left\b/i.test(advice.join(' ')),`${name}: no countdown`);
  if(name==='evening')assert.equal(await phasePage.locator('.sleep-plan .strip-bar u').count(),1,'the strip marks now against the window');
  if(name==='late')assert.ok(text.includes("Tonight's window was"));
  if(name==='missing')assert.ok(text.includes('Not synced yet') && !text.includes('Lowest readiness factor'),'an unsynced night is neutral');
  if(name==='day')assert.ok(text.includes('As of watch sync') && text.includes('Done today'),'the day card is labeled as synced data');
  if(name==='overview')await phasePage.getByText(focus.note).waitFor();
  if(['wake','day'].includes(name))await phasePage.locator('.plan-line').waitFor();
  assert.deepEqual(await contrastFailures(phasePage),[],`${name}: text contrast`);
  for(const width of [390,...(['day','evening'].includes(name)?[700,1100]:[])]){
   await phasePage.setViewportSize({width,height:844});
   const overflow=await phasePage.locator('.scroll').evaluate(el=>el.scrollWidth-el.clientWidth);
   assert.ok(overflow<=1,`${name} at ${width}: ${overflow}px horizontal overflow`);
   await phasePage.screenshot({path:resolve(output,`${width}-today-phase-${name}.png`),fullPage:true});
  }
  await phasePage.unroute(todaySample);
 }
 await phasePage.close();
 console.log('Today through the day: one leading card, stable hero and pill, calm nights at 390/700/1100: passed');

 const samples={};
 for(const name of ['today','sleep','training','health','status','workout'])samples[name]=JSON.parse(await readFile(resolve(`src/lib/sample/${name}.json`),'utf8'));
 // The sample's phase ended long ago; phase expiry has its own check below.
 samples.today.focus.until=null;
 const onboarding=await browser.newPage({viewport:{width:390,height:844}});
 onboarding.on('pageerror',e=>errors.push(e.message));
 await onboarding.addInitScript(samples=>{
  const state=window.testOnboarding={signedIn:false,signIns:0,permissions:0,failSignIn:true};
  window.isTauri=true;
  window.__TAURI_INTERNALS__={
   transformCallback:()=>1,
   invoke:async(command)=>{
    if(command==='app_status')return {...samples.status,signedIn:state.signedIn,lastSync:new Date().toISOString()};
    if(command==='begin_sign_in'){
     state.signIns++;
     if(state.failSignIn)throw Error('Synthetic sign-in failure');
     state.signedIn=true;
     queueMicrotask(()=>window.dispatchEvent(new Event('raring:resume')));
     return;
    }
    if(command==='request_notification_permission'){state.permissions++;return true}
    if(command==='sign_out'){state.signedIn=false;return}
    if(command==='sync_now')return;
    if(command==='get_settings')return {notifyMorningReadiness:true,notifyWeeklyCheckin:true,notifyMetricChanges:true,notifyVitals:true,notifyTrainingStatus:false,notifyBedtime:false,bedtimeLeadMin:30};
    if(command.startsWith('plugin:event|'))return 1;
    if(command in samples)return structuredClone(samples[command]);
    throw Error('Unexpected onboarding command '+command);
   },
  };
 },samples);
 await onboarding.goto(`${url}/#today`);
 await onboarding.waitForURL('**/#onboarding');
 await onboarding.getByText('Your Garmin account, not your watch',{exact:true}).waitFor();
 await onboarding.getByText('Keep Garmin Connect syncing',{exact:true}).waitFor();
 assert.ok((await onboarding.locator('.about-raring').textContent()).includes('No analytics, telemetry, ads'));
 assert.equal(await onboarding.getByRole('navigation').count(),0,'welcome does not expose signed-in navigation');
 const privacy=onboarding.locator('summary');
 await privacy.focus();await privacy.press('Enter');
 assert.ok(await onboarding.locator('details').getAttribute('open')!==null,'privacy is keyboard accessible');
 await onboarding.getByText(/Sign-in tokens are encrypted/).waitFor();
 await privacy.press('Enter');
 await onboarding.getByRole('button',{name:'Sign in with Garmin'}).click();
 await onboarding.getByRole('alert').getByText('Error: Synthetic sign-in failure',{exact:true}).waitFor();
 await onboarding.evaluate(()=>{window.testOnboarding.failSignIn=false});
 await onboarding.getByRole('button',{name:'Sign in with Garmin'}).click();
 await onboarding.getByRole('heading',{name:'Stay in the loop'}).waitFor();
 await onboarding.getByRole('button',{name:'Not now',exact:true}).click();
 await onboarding.waitForURL('**/#today');await onboarding.locator('.dials').waitFor();
 assert.equal(await onboarding.evaluate(()=>window.testOnboarding.signIns),2,'sign-in button invokes the native command');
 assert.equal(await onboarding.evaluate(()=>window.testOnboarding.permissions),0,'notification permission is optional');
 await onboarding.getByRole('button',{name:'Settings',exact:true}).click();
 await onboarding.getByText('How Raring works',{exact:true}).waitFor();
 await onboarding.locator('summary').click();
 await onboarding.getByText(/This does not delete anything from your Garmin account/).waitFor();
 onboarding.once('dialog',dialog=>dialog.accept());
 await onboarding.getByRole('button',{name:'Sign out',exact:true}).click();
 await onboarding.waitForURL('**/#onboarding');
 assert.equal(await onboarding.evaluate(()=>localStorage.getItem('notifications-step-done')),null);
 await onboarding.getByRole('button',{name:'Sign in with Garmin'}).click();
 await onboarding.getByRole('button',{name:'Enable notifications',exact:true}).click();
 await onboarding.waitForURL('**/#today');
 assert.equal(await onboarding.evaluate(()=>window.testOnboarding.permissions),1);
 assert.equal(await onboarding.evaluate(()=>localStorage.getItem('notifications-step-done')),'1');
 await onboarding.close();
 console.log('Welcome, offline privacy, sign-in errors, notification choices and sign-out: passed');

 const live=await browser.newPage({viewport:{width:840,height:900},timezoneId:'America/New_York'});
 live.on('pageerror',e=>errors.push(e.message));
 await live.clock.install({time:new Date('2026-10-16T14:00:00Z')});
 await live.addInitScript(samples=>{
function trendFixture(q){
     const n=q.range==='days7'?7:q.range==='days90'?90:q.range==='year'?52:30;
     const source=q.metric==='readiness'?samples.today.readinessTrend:q.metric==='stress'?samples.today.stress.trend:samples.health.vitals.readings.find(v=>v.kind===(q.metric==='sleepHr'?'restingHr':q.metric))?.trend ?? [];
     const end=q.end??samples.today.date;
     const points=Array.from({length:n},(_,i)=>{const d=new Date(end+'T12:00:00Z');d.setUTCDate(d.getUTCDate()-n+1+i);const date=d.toISOString().slice(0,10),value=source[source.length-n+i]??null;return {start:date,end:date,value,days:value==null?0:1}});
     const present=points.filter(p=>p.value!=null),series={metric:q.metric,name:q.metric,unit:q.metric==='hrv'?'ms':'',points,average:present.length?present.reduce((s,p)=>s+p.value,0)/present.length:null,days:present.length};
     return {range:q.range,bucket:'day',from:points[0].start,to:end,series,overlay:null,previous:null,change:null,available:[q.metric]};
    }
  localStorage.setItem('notifications-step-done','1');
  const state=window.testBackend={samples,syncs:0,reads:0,lastAttempt:null,failSync:false,failRead:false,holdToday:false,pendingToday:null};
  samples.status.lastSync=new Date(Date.now()-3600_000).toISOString();
  window.isTauri=true;
  window.__TAURI_INTERNALS__={
   transformCallback:()=>1,
   invoke:async(command,args)=>{
 if(command==='trend'){
     const result=trendFixture(args.query);state.trendReads=(state.trendReads??0)+1;
     if(state.holdTrend)await new Promise(resolve=>{state.pendingTrend=resolve});
     return result;
    }
    if(command==='app_status')return structuredClone(samples.status);
    if(command==='sync_now'){
     state.syncs++;state.lastAttempt=Date.now();
     if(state.failSync)throw Error('Synthetic offline error');
     samples.status.lastSync=new Date().toISOString();
     return samples.status;
    }
    if(command==='today'){
     state.reads++;
     if(state.failRead)throw Error('Synthetic cache error');
     if(state.holdToday)await new Promise(resolve=>{state.pendingToday=resolve});
    }
    if(command.startsWith('plugin:event|'))return 1;
    if(command in samples)return structuredClone(samples[command]);
    throw Error('Unexpected command '+command);
   },
  };
 },samples);
 await live.goto(`${url}/#today`);
 await live.waitForFunction(()=>window.testBackend.syncs===1 && document.querySelector('.dial-button.large'));
 await live.evaluate(()=>{window.testBackend.samples.today.readiness.value=64});
 await live.clock.fastForward(30_000);
 await live.waitForFunction(()=>document.querySelector('.dial-button.large .gauge-center b')?.textContent==='64');
 assert.equal(await live.evaluate(()=>window.testBackend.syncs),1,'cache poll does not sync every 30 seconds');
 await live.locator('.dial-button.large').click();
 await live.waitForURL('**/#readiness');
 await live.evaluate(()=>{window.testBackend.samples.today.readiness.value=59});
 await live.clock.fastForward(30_000);
 await live.waitForFunction(()=>document.querySelector('.detail-gauge .gauge-center b')?.textContent==='59');
 await live.clock.fastForward(15*60_000);
 await live.waitForFunction(()=>window.testBackend.syncs===2);
 await live.evaluate(()=>{window.dispatchEvent(new Event('raring:resume'));window.dispatchEvent(new Event('raring:resume'))});
 await live.clock.fastForward(30_000);
 assert.equal(await live.evaluate(()=>window.testBackend.syncs),2,'resume events are throttled');
 await live.evaluate(()=>{window.testBackend.failRead=true});
 await live.clock.fastForward(30_000);
 assert.equal(await live.locator('.detail-gauge .gauge-center b').textContent(),'59','cache errors retain the visible reading');
 await live.evaluate(()=>{window.testBackend.failRead=false;window.testBackend.failSync=true});
 await live.clock.fastForward(15*60_000);
 await live.waitForFunction(()=>window.testBackend.syncs===3);
 await live.clock.fastForward(30_000);
 assert.equal(await live.evaluate(()=>window.testBackend.syncs),3,'failed syncs do not retry every poll');
 await live.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();
 await live.locator('.sync-indicator').getByText('Raring sync delayed · Error: Synthetic offline error').waitFor();
 await live.clock.fastForward(30_000);
 assert.ok((await live.locator('.sync-indicator').textContent()).includes('Raring sync delayed'),'an unchanged cached success must not clear a newer failure');
 await live.evaluate(()=>{
  const s=window.testBackend;
  s.samples.status.lastSync=new Date(s.lastAttempt-1000).toISOString();
  window.dispatchEvent(new Event('raring:resume'));
 });
 await live.clock.fastForward(30_000);
 assert.ok((await live.locator('.sync-indicator').textContent()).includes('Raring sync delayed'),'a previously unseen success before the failed attempt must not clear its warning');
 await live.evaluate(()=>{
  const s=window.testBackend.samples.status;
  s.lastSync=new Date().toISOString();s.lastError='Synthetic partial sync error';
  window.dispatchEvent(new Event('raring:resume'));
 });
 await live.clock.fastForward(30_000);
 assert.ok((await live.locator('.sync-indicator').textContent()).includes('Raring sync delayed'),'partial syncs still show an error');
 await live.evaluate(()=>{
  const s=window.testBackend.samples.status;
  s.lastSync=new Date().toISOString();s.lastError=null;
  window.dispatchEvent(new Event('raring:resume'));
 });
 await live.locator('.sync-indicator').getByText('Watch upload time unavailable',{exact:true}).waitFor({timeout:5_000});
 assert.equal(await live.evaluate(()=>window.testBackend.syncs),3,'background recovery clears the warning without another foreground sync');
 // A late Today response must not overwrite the newly opened Sleep page.
 await live.evaluate(()=>{window.testBackend.holdToday=true;window.dispatchEvent(new Event('raring:resume'))});
 await live.waitForFunction(()=>window.testBackend.pendingToday!==null);
 await live.getByRole('navigation').getByRole('button',{name:'Sleep',exact:true}).click();
 await live.waitForURL('**/#sleep');await live.locator('.sleep-hero').waitFor();
 await live.evaluate(()=>{window.testBackend.pendingToday();window.testBackend.holdToday=false});
 await live.clock.fastForward(1_000);
 assert.equal(await live.locator('h1').textContent(),'Sleep');
 await live.locator('.sleep-hero').waitFor();
 // A mounted long-range chart must refresh on sync; an old pending read must not win.
 await live.goto(`${url}/#readiness`);
 const liveTrend=live.locator('.history');
 await liveTrend.getByRole('button',{name:'30 days',exact:true}).click();
 await liveTrend.getByRole('slider').waitFor();
 await live.evaluate(()=>{window.testBackend.holdTrend=true});
 await liveTrend.getByRole('button',{name:'1 year',exact:true}).click();
 await live.waitForFunction(()=>window.testBackend.pendingTrend);
 await live.evaluate(()=>{
  const s=window.testBackend;s.holdTrend=false;s.samples.today.readinessTrend[29]=93;
  s.samples.status.lastSync=new Date(Date.now()+1000).toISOString();window.dispatchEvent(new Event('raring:resume'));
 });
 await liveTrend.locator('.reading-values b').first().getByText('93',{exact:true}).waitFor();
 await live.evaluate(()=>window.testBackend.pendingTrend());
 await live.clock.fastForward(1000);
 assert.equal(await liveTrend.locator('.reading-values b').first().textContent(),'93','a stale pre-sync trend cannot replace the refreshed one');
 console.log('Live cache polling, throttled sync, failures and route races: passed');

 // A partially calibrated monitor must not turn missing readings into an all-clear.
 await live.evaluate(()=>{
  const s=window.testBackend;s.failSync=false;
  for(const v of s.samples.health.vitals.readings){v.value=null;v.normal=null;v.trend=Array(30).fill(null);v.tone='neutral';v.status=null;v.rangeSource='none'}
  Object.assign(s.samples.health.vitals.readings[0],{value:48,normal:[44,52],scale:[30,90],tone:'good'});
  Object.assign(s.samples.health.vitals.readings[1],{value:58});
  s.samples.health.vitals.baselineReady=true;
 });
 await live.setViewportSize({width:390,height:844});
 await live.getByRole('navigation').getByRole('button',{name:'Health',exact:true}).click();
 await live.getByText('1 reading in range',{exact:true}).waitFor();
 assert.equal(await live.getByText('Within your ranges',{exact:true}).count(),0);
 assert.equal(await live.locator('.signal-grid').getByText('Not recorded',{exact:true}).count(),3);
 await live.screenshot({path:resolve(output,'health-partial.png')});
 await live.evaluate(()=>{
  const v=window.testBackend.samples.health.vitals.readings;
  Object.assign(v[0],{value:74,tone:'poor'});
  Object.assign(v[1],{value:28,normal:[48,62],scale:[14,76],tone:'poor'});
  v[1].trend[29]=28;
  window.dispatchEvent(new Event('raring:resume'));
 });
 await live.getByText('2 outside your ranges',{exact:true}).waitFor();
 await live.getByText('Above usual',{exact:true}).waitFor();
 await live.getByText('Below baseline',{exact:true}).waitFor();
 await live.screenshot({path:resolve(output,'health-outliers.png')});
 await live.getByRole('button',{name:'View SpO₂',exact:true}).click();
 await live.getByText('No readings in this period. Missing data is never shown as zero.',{exact:true}).waitFor();
 assert.equal(await live.locator('.history .history-bar').count(),0);
 assert.equal(await live.locator('.history input').getAttribute('aria-valuetext'),'Thu 16 Oct: Not recorded','history is anchored to stored data, not the system clock');
 await live.screenshot({path:resolve(output,'history-missing.png')});
 // The fixed layout shows the plan in full.
 await live.evaluate(()=>{const t=window.testBackend.samples.today;Object.assign(t.focus,{phase:'overview',planOpen:true,until:null});Object.assign(t.sleepPlan,{bedtimeStart:null,bedtimeStartAt:null,bedtimeSource:'wakeHistory',source:'latest',reason:null})});
 await live.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();
 await live.locator('.sleep-plan').waitFor();
 const fallback=await live.locator('.sleep-plan').textContent();
 assert.ok(fallback.includes('Usual wake') && fallback.includes('Raring estimate') && fallback.includes('Asleep by23:06'),'without Garmin\'s window the plan falls back to wake history');
 await live.evaluate(()=>{window.testBackend.samples.today.sleepPlan=null;window.testBackend.samples.sleep.sleepPlan=null});
 await live.getByRole('navigation').getByRole('button',{name:'Sleep',exact:true}).click();
 await live.locator('.sleep-hero').waitFor();
 assert.equal(await live.locator('.sleep-plan').count(),0,'no inferred sleep plan without Garmin need');
 await live.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();
 await live.locator('.dials').waitFor();
 assert.equal(await live.locator('.sleep-plan, .plan-line').count(),0);
 console.log('Partial baselines, outliers, missing history and unavailable sleep plans: passed');
 await live.evaluate(()=>{Object.assign(window.testBackend.samples.training.acute,{label:'',tone:'neutral',band:null})});
 await live.getByRole('navigation').getByRole('button',{name:'Training',exact:true}).click();
 await live.locator('.semi .gauge-center').waitFor();
 assert.equal(await live.locator('.semi .gauge-center b').textContent(),'612');
 assert.equal(await live.locator('.semi .gauge-center').getByText('Acute load',{exact:true}).count(),1,'missing ranges do not repeat the metric name as a status');
 assert.equal(await live.locator('.semi .gauge-center em').count(),0,'missing ranges do not imply a load assessment');
 await live.screenshot({path:resolve(output,'training-no-range.png')});
 console.log('Training load without an optimal range: passed');
 // Real Garmin shapes are covered in Rust; exercise their partial/mixed-sport UI states here.
 await live.evaluate(()=>{
  const s=window.testBackend.samples;
  Object.assign(s.workout.summary,{startLocal:'2026-10-14T07:23:00',durationS:2100,distanceKm:null});
  s.training.workouts=[structuredClone(s.workout.summary)];
  s.workout.zoneFloors=[];
  s.workout.heartRate=[{tS:0,bpm:98},{tS:60,bpm:140},{tS:2100,bpm:110}];
  s.workout.splits=[{index:1,distanceKm:0,seconds:2100,avgHr:120}];
  window.dispatchEvent(new Event('raring:resume'));
 });
 await live.getByRole('navigation').getByRole('button',{name:'Training',exact:true}).click();
 await live.getByText('Wed 14 Oct · 07:23',{exact:true}).waitFor();
 await live.locator('.activity').first().click();
 await live.locator('.hr-trace').waitFor();
 assert.notEqual(await live.locator('.hr-trace').getAttribute('stroke'),'url(#hr-zones)','HR remains visible without zone boundaries');
 assert.equal(await live.locator('svg[aria-label="Heart rate during workout"] text').last().textContent(),'35:00','HR axis reflects recorded duration, not fixed 45 minutes');
 await live.getByText('Lap duration',{exact:true}).waitFor();
 assert.equal(await live.getByText(/Fastest (km|split)/).count(),0,'a stationary workout is not a fastest kilometre');
 assert.ok(!(await live.locator('main').innerText()).includes('Invalid Date'));
 await live.evaluate(()=>{
  const w=window.testBackend.samples.workout;w.zoneFloors=[100,120,140,160];
  w.splits=[{index:1,distanceKm:.5,seconds:180,avgHr:120},{index:2,distanceKm:1,seconds:300,avgHr:140}];
  window.dispatchEvent(new Event('raring:resume'));
 });
 await live.getByText('Fastest split 2',{exact:true}).waitFor();
 assert.equal(await live.locator('.hr-trace').getAttribute('stroke'),'url(#hr-zones)');
 assert.deepEqual(await live.locator('.split-value b').allTextContents(),['6:00','5:00'],'unequal splits use pace rather than raw lap duration');
 await live.evaluate(()=>{
  const s=window.testBackend.samples.sleep;s.date='2026-10-16';s.startLocal='2026-10-16T01:35:00';
  s.consistency=[{date:'2026-10-14',bedHour:23,wakeHour:7},{date:'2026-10-16',bedHour:1.5,wakeHour:7}];
 });
 await live.getByRole('navigation').getByRole('button',{name:'Sleep',exact:true}).click();
 await live.locator('.time-label').first().waitFor();
 assert.equal(await live.locator('.time-label').first().textContent(),'01:35','sleep chart starts at Garmin bedtime, not fixed 23:00');
 const nights=await live.locator('rect.night').evaluateAll(nodes=>nodes.map(n=>Number(n.getAttribute('x'))));
 assert.equal(nights.length,2);assert.ok(nights[1]>300 && nights[1]-nights[0]>40,'consistency preserves calendar gaps and anchors the latest night on the right');
 await live.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();
 await live.locator('.sync-indicator').waitFor();
 await live.getByRole('button',{name:'Refresh',exact:true}).click();
 await live.locator('.sync-indicator').getByText('Watch upload time unavailable',{exact:true}).waitFor();
 assert.ok(await live.evaluate(()=>window.testBackend.samples.status.lastSync!==null),'Raring refresh does not invent a watch upload time');
 await live.evaluate(()=>{window.testBackend.samples.health.lever.ageIfMet=36.23456789});
 await live.getByRole('navigation').getByRole('button',{name:'Health',exact:true}).click();
 await live.locator('.lever').waitFor();
 assert.ok((await live.locator('.lever').textContent()).includes('age 36.2'),'Garmin fractional target ages are formatted for display');
 assert.ok(!(await live.locator('.lever').textContent()).includes('36.23456789'));
 console.log('Workout dates, partial zones, real chart axes, mixed-sport splits, sleep gaps and target formatting: passed');

 const cached=await browser.newPage({viewport:{width:390,height:844}});
 cached.on('pageerror',e=>errors.push(e.message));
 await cached.clock.install({time:new Date('2026-10-16T14:00:00Z')});
 await cached.addInitScript(samples=>{
function trendFixture(q){
     const n=q.range==='days7'?7:q.range==='days90'?90:q.range==='year'?52:30;
     const source=q.metric==='readiness'?samples.today.readinessTrend:q.metric==='stress'?samples.today.stress.trend:samples.health.vitals.readings.find(v=>v.kind===(q.metric==='sleepHr'?'restingHr':q.metric))?.trend ?? [];
     const end=q.end??samples.today.date;
     const points=Array.from({length:n},(_,i)=>{const d=new Date(end+'T12:00:00Z');d.setUTCDate(d.getUTCDate()-n+1+i);const date=d.toISOString().slice(0,10),value=source[source.length-n+i]??null;return {start:date,end:date,value,days:value==null?0:1}});
     const present=points.filter(p=>p.value!=null),series={metric:q.metric,name:q.metric,unit:q.metric==='hrv'?'ms':'',points,average:present.length?present.reduce((s,p)=>s+p.value,0)/present.length:null,days:present.length};
     return {range:q.range,bucket:'day',from:points[0].start,to:end,series,overlay:null,previous:null,change:null,available:[q.metric]};
    }
  localStorage.setItem('notifications-step-done','1');
  const state=window.testViews={samples,reads:{},completed:{},pending:{},holdNext:{},statusReads:0,statusCompleted:0,syncCalls:0};
  samples.status.lastSync=new Date().toISOString();
  window.isTauri=true;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
 if(command==='trend')return trendFixture(args.query);
   if(command.startsWith('plugin:event|'))return 1;
   if(command==='app_status'){
    state.statusReads++;
    const snapshot=structuredClone(samples.status);
    if(state.holdNext.app_status){
     state.holdNext.app_status--;
     await new Promise(resolve=>{(state.pending.app_status??=[]).push(resolve)});
    }
    state.statusCompleted++;
    return snapshot;
   }
   if(command==='sync_now'){state.syncCalls++;return;}
   if(command in samples){
    state.reads[command]=(state.reads[command]??0)+1;
    const snapshot=structuredClone(samples[command]);
    if(state.holdNext[command]){
     state.holdNext[command]--;
     await new Promise(resolve=>{(state.pending[command]??=[]).push(resolve)});
    }
    state.completed[command]=(state.completed[command]??0)+1;
    return snapshot;
   }
   throw Error('Unexpected view command '+command);
  }};
 },samples);
 const tabCached=async name=>cached.getByRole('navigation').getByRole('button',{name,exact:true}).click();
 const hold=command=>cached.evaluate(command=>{window.testViews.holdNext[command]=(window.testViews.holdNext[command]??0)+1},command);
 const release=command=>cached.evaluate(command=>window.testViews.pending[command].shift()(),command);
 const pending=command=>cached.waitForFunction(command=>window.testViews.pending[command]?.length,command);
 await cached.goto(`${url}/#today`);
 await cached.locator('.dials').waitFor();
 await cached.waitForFunction(()=>['sleep','training','health'].every(key=>window.testViews.completed[key]>=1));
 const statusReads=await cached.evaluate(()=>window.testViews.statusReads);
 const todayOffset=await cached.locator('.scroll').evaluate(el=>{el.scrollTop=350;return el.scrollTop});
 assert.ok(todayOffset>100,'Today can be scrolled for restoration');
 await hold('sleep');
 await tabCached('Sleep');
 await pending('sleep');
 assert.equal(await cached.locator('.sleep-hero').isVisible(),true,'prefetched Sleep appears before navigation read finishes');
 assert.equal(await cached.locator('.empty').count(),0,'cached tab switches without loading state');
 assert.equal(await cached.evaluate(()=>window.testViews.statusReads),statusReads,'navigation does not query app status');
 await release('sleep');
 await cached.locator('.scroll').evaluate(el=>{el.scrollTop=120});
 await hold('today');
 await tabCached('Today');
 await pending('today');
 assert.equal(await cached.locator('.dials').isVisible(),true,'cached Today is visible before its read finishes');
 await cached.waitForFunction(expected=>document.querySelector('.scroll').scrollTop===expected,todayOffset);
 await release('today');

 const touch=()=>cached.evaluate(()=>{
  const main=document.querySelector('main');
  const point=new Touch({identifier:1,target:main,clientY:100});
  main.dispatchEvent(new TouchEvent('touchstart',{bubbles:true,touches:[point]}));
 });
 await cached.evaluate(()=>{window.testViews.samples.today.headline='After touch cancel'});
 await hold('today');await touch();
 await cached.evaluate(()=>window.dispatchEvent(new Event('raring:resume')));
 await pending('today');await release('today');
 await cached.waitForFunction(()=>window.testViews.completed.today>=3);
 assert.notEqual(await cached.locator('.headline h2').textContent(),'After touch cancel','view swap is deferred while touching');
 await cached.locator('main').evaluate(el=>el.dispatchEvent(new TouchEvent('touchcancel',{bubbles:true})));
 await cached.getByRole('heading',{name:'After touch cancel'}).waitFor();
 await cached.evaluate(()=>{window.testViews.samples.today.headline='After hide'});
 await hold('today');await touch();
 await cached.evaluate(()=>window.dispatchEvent(new Event('raring:resume')));
 await pending('today');await release('today');
 await cached.waitForFunction(()=>window.testViews.completed.today>=4);
 assert.notEqual(await cached.locator('.headline h2').textContent(),'After hide','hidden-page swap is deferred while touching');
 await cached.evaluate(()=>{Object.defineProperty(document,'visibilityState',{configurable:true,value:'hidden'});document.dispatchEvent(new Event('visibilitychange'));delete document.visibilityState});
 await cached.getByRole('heading',{name:'After hide'}).waitFor();

 // A cached Today whose phase has ended is read again rather than shown, so its leading card can't swap after appearing.
 await cached.evaluate(()=>{Object.assign(window.testViews.samples.today,{headline:'Old phase'});window.testViews.samples.today.focus.until='2000-01-01T00:00:00';window.dispatchEvent(new Event('raring:resume'))});
 await cached.getByRole('heading',{name:'Old phase'}).waitFor();
 await tabCached('Sleep');await cached.locator('.sleep-hero').waitFor();
 await cached.evaluate(()=>{Object.assign(window.testViews.samples.today,{headline:'New phase'});window.testViews.samples.today.focus.until=null});
 await hold('today');await tabCached('Today');await pending('today');
 assert.equal(await cached.getByRole('heading',{name:'Old phase'}).count(),0,'an ended phase is not served from the cache');
 await release('today');
 await cached.getByRole('heading',{name:'New phase'}).waitFor();

 // A pre-sync request must neither be reused nor overwrite a post-sync reading.
 await hold('today');
 await cached.evaluate(()=>window.dispatchEvent(new Event('raring:resume')));
 await pending('today');
 await cached.evaluate(()=>{const s=window.testViews.samples;s.today.headline='After sync';s.status.lastSync='2026-10-16T14:01:00Z'});
 await cached.getByRole('button',{name:'Refresh',exact:true}).click();
 await cached.getByRole('heading',{name:'After sync'}).waitFor();
 const completedBeforeOld=await cached.evaluate(()=>window.testViews.completed.today);
 await release('today');
 await cached.waitForFunction(previous=>window.testViews.completed.today>previous,completedBeforeOld);
 assert.equal(await cached.locator('.headline h2').textContent(),'After sync','old in-flight result cannot replace synced view');
 const previousSleep=await cached.evaluate(()=>window.testViews.completed.sleep);
 await cached.evaluate(()=>{const s=window.testViews.samples;s.today.headline='Background sync';s.sleep.score.value=51;s.status.lastSync='2026-10-16T14:02:00Z';window.dispatchEvent(new Event('raring:resume'))});
 await cached.getByRole('heading',{name:'Background sync'}).waitFor();
 await cached.waitForFunction(previous=>window.testViews.completed.sleep>previous,previousSleep);
 await hold('sleep');await tabCached('Sleep');await pending('sleep');
 assert.equal(await cached.locator('.sleep-hero .gauge-center b').textContent(),'51','changed lastSync refreshes cached tabs');
 await release('sleep');await tabCached('Today');
 await cached.getByRole('heading',{name:'Background sync'}).waitFor();

 // Midnight on resume invalidates both current data and in-flight reads; prefetch runs again.
 await hold('today');
 await cached.evaluate(()=>window.dispatchEvent(new Event('raring:resume')));
 await pending('today');
 const sleepReads=await cached.evaluate(()=>window.testViews.completed.sleep);
 await cached.clock.setFixedTime(new Date('2026-10-17T14:00:00Z'));
 await cached.evaluate(()=>{const s=window.testViews.samples;s.status.syncing=true;s.today.headline='Next day';s.sleep.score.value=42;window.dispatchEvent(new Event('raring:resume'))});
 await cached.getByRole('heading',{name:'Next day'}).waitFor();
 await cached.waitForFunction(previous=>window.testViews.completed.sleep>previous,sleepReads);
 await release('today');
 await cached.waitForFunction(()=>window.testViews.pending.today?.length===0);
 assert.equal(await cached.locator('.headline h2').textContent(),'Next day','previous-day result cannot replace next day');
 await hold('sleep');await tabCached('Sleep');await pending('sleep');
 assert.equal(await cached.locator('.sleep-hero .gauge-center b').textContent(),'42','next-day prefetch is visible before navigation read');
 await release('sleep');
 await tabCached('Today');
 await cached.locator('.dials').waitFor();
 const pollReads=await cached.evaluate(()=>window.testViews.completed.sleep);
 await cached.clock.setFixedTime(new Date('2026-10-18T14:00:00Z'));
 await cached.evaluate(()=>{const s=window.testViews.samples;s.today.headline='Poll rollover';s.sleep.score.value=43});
 await cached.clock.fastForward(30_000);
 await cached.getByRole('heading',{name:'Poll rollover'}).waitFor();
 await cached.waitForFunction(previous=>window.testViews.completed.sleep>previous,pollReads);
 assert.equal(await cached.evaluate(()=>window.testViews.syncCalls),1,'day rollover never needs a Garmin sync');

 // Navigation must not discard a concurrent status read or its sign-in gate.
 const healthReads=await cached.evaluate(()=>window.testViews.completed.health);
 await cached.evaluate(()=>{const s=window.testViews.samples;s.health.fitnessAge.fitnessAge=37;s.status.lastSync='2026-10-18T14:01:00Z'});
 await hold('app_status');
 await cached.evaluate(()=>window.dispatchEvent(new Event('raring:resume')));
 await pending('app_status');
 await tabCached('Sleep');
 await cached.locator('.sleep-hero').waitFor();
 await release('app_status');
 await cached.waitForFunction(previous=>window.testViews.completed.health>previous,healthReads);
 await hold('health');await tabCached('Health');await pending('health');
 assert.equal(await cached.locator('.age-number').textContent(),'37','in-flight status invalidates other tabs after navigation');
 await release('health');
 await cached.evaluate(()=>{window.testViews.samples.status.signedIn=false});
 await hold('app_status');
 await cached.evaluate(()=>window.dispatchEvent(new Event('raring:resume')));
 await pending('app_status');
 await tabCached('Today');
 await cached.locator('.dials').waitFor();
 await release('app_status');
 await cached.waitForURL('**/#onboarding');
 assert.equal(await cached.getByRole('navigation').count(),0,'in-flight sign-out status gates navigation');
 await cached.evaluate(()=>{window.testViews.samples.status.signedIn=true});
 await hold('app_status');
 await cached.evaluate(()=>window.dispatchEvent(new Event('raring:resume')));
 await pending('app_status');
 const statuses=await cached.evaluate(()=>window.testViews.statusCompleted);
 await cached.evaluate(()=>{window.testViews.samples.status.signedIn=false;window.dispatchEvent(new Event('raring:resume'))});
 await cached.waitForFunction(previous=>window.testViews.statusCompleted>previous,statuses);
 await release('app_status');
 await cached.waitForFunction(()=>window.testViews.pending.app_status?.length===0);
 assert.equal(new URL(cached.url()).hash,'#onboarding','older signed-in status cannot undo a newer sign-out');
 await cached.close();
 console.log('Cached switching, sync/rollover invalidation, scroll, touch cancellation and status races: passed');
 for(const width of [390,700,1100]){
  const trends=await browser.newPage({viewport:{width,height:900}});
  trends.on('pageerror',e=>errors.push(e.message));
  async function shot(name){
   const height=await trends.locator('.scroll').evaluate(e=>e.scrollHeight+100);
   await trends.setViewportSize({width,height:Math.max(900,height)});
   await trends.locator('.scroll').evaluate(e=>{e.scrollTop=0;e.scrollLeft=0});
   await trends.screenshot({path:resolve(output,`${name}-${width}.png`)});
   await trends.setViewportSize({width,height:900});
  }
  await trends.goto(`${url}/#readiness`);
  const history=trends.locator('.history').first();
  await history.getByRole('button',{name:'90 days',exact:true}).click();
  await history.getByText('Reading your history…').waitFor({state:'detached'});
  assert.equal(await history.locator('.history-bar').count()+await history.locator('.history-gap').count(),90,'90 days are daily');
  await history.getByRole('button',{name:'1 year',exact:true}).click();
  await history.getByText('Weekly average of recorded days',{exact:true}).waitFor();
  assert.equal(await history.locator('.history-bar').count()+await history.locator('.history-gap').count(),52,'a year is 52 weeks');
  const fills=new Set(await history.locator('.history-bar').evaluateAll(bars=>bars.map(b=>b.getAttribute('fill'))));
  assert.deepEqual([...fills],['#E4E8F0'],'weekly averages are ink, not Garmin levels');
  assert.ok((await history.getByRole('slider').getAttribute('aria-valuetext')).startsWith('Week of '));
  assert.match(await history.locator('.history-reading small').textContent(),/^\d of \d days recorded$/);
  await history.getByRole('button',{name:'Previous',exact:true}).click();
  await history.locator('.history-previous').waitFor();
  assert.ok((await history.locator('.period-averages').textContent()).includes('Period before'));
  await shot('trends-previous');
  await history.getByRole('button',{name:'All history',exact:true}).click();
  await history.getByText(/^\d+ \/ \d+ weeks$/).waitFor();
  assert.ok(await history.getByRole('button',{name:'Previous',exact:true}).isDisabled(),'All has no period before it');
  assert.equal(await history.locator('.history-previous').count(),0);
  await history.getByRole('button',{name:'90 days',exact:true}).click();
  await history.getByRole('button',{name:'Metric',exact:true}).click();
  await history.locator('.history-overlay').waitFor();
  assert.equal(await history.getByRole('combobox',{name:'Metric to overlay'}).inputValue(),'sleepTime');
  assert.ok((await history.locator('.history-legend').textContent()).includes('Time asleep, right axis'));
  assert.match(await history.locator('.overlay-reading b').textContent(),/^\d+:\d\d$/,'time asleep reads as hours and minutes');
  await history.getByRole('combobox',{name:'Metric to overlay'}).selectOption('hrv');
  await history.getByText('HRV, right axis',{exact:true}).waitFor();
  await shot('trends-overlay');
  await history.getByRole('combobox',{name:'Primary metric',exact:true}).selectOption('sleepTime');
  await history.getByRole('combobox',{name:'Metric to overlay'}).selectOption('acuteLoad');
  await history.getByText('Acute load, right axis',{exact:true}).waitFor();
  await history.getByRole('button',{name:'Custom dates',exact:true}).click();
  await history.getByLabel('Block start',{exact:true}).fill('2025-08-01');
  await history.getByLabel('Block end',{exact:true}).fill('2025-08-28');
  await history.getByRole('button',{name:'Previous',exact:true}).click();
  await history.getByLabel('Comparison block start',{exact:true}).fill('2025-05-01');
  await history.getByText(/Against 2025-05-01 – 2025-05-28/).waitFor();
  await history.getByLabel('Block start',{exact:true}).fill('');
  await history.getByText(/Couldn't read this range/).waitFor();
  await history.getByLabel('Block start',{exact:true}).fill('2025-08-01');
  await history.getByText(/Against 2025-05-01 – 2025-05-28/).waitFor();
  await history.scrollIntoViewIfNeeded();
  await shot('trends-custom');
  const overflow=await trends.evaluate(()=>document.documentElement.scrollWidth-document.documentElement.clientWidth);
  assert.ok(overflow<=1,`no horizontal overflow at ${width}px`);
  assert.equal(await trends.locator('.phone').evaluate(e=>e.scrollLeft),0,'chart focus cannot scroll the clipped hero glow sideways');
  await trends.goto(`${url}/#metric/hrv`);
  const nightly=trends.locator('.history').first();
  await nightly.getByRole('button',{name:'All history',exact:true}).click();
  await nightly.getByText(/^(Weekly|Monthly) average of recorded days$/).waitFor();
  await trends.goto(`${url}/#metric/stress`);
  const stress=trends.locator('.history').first();
  await stress.getByRole('button',{name:'All history',exact:true}).click();
  await stress.getByText('Monthly average of recorded days',{exact:true}).waitFor();
  assert.ok((await stress.getByRole('slider').getAttribute('aria-valuetext')).startsWith('October 2025:'));
  await trends.goto(`${url}/#metric/hrv`);
  await nightly.getByRole('button',{name:'All history',exact:true}).click();
  assert.equal(await nightly.getByText('Current Garmin baseline',{exact:true}).count(),0,'past baselines differed, so averages show none');
  await shot('trends-all');
  await trends.close();
 }
 console.log('Trend ranges, weekly and monthly averages, overlays and the period before at 390/700/1100: passed');

 // "Follow my day": a slow save must not leave Today in the layout that was just switched off.
 const layout=await browser.newPage({viewport:{width:390,height:844}});
 layout.on('pageerror',e=>errors.push(e.message));
 await layout.addInitScript(samples=>{
  localStorage.setItem('notifications-step-done','1');
  samples.status.lastSync=new Date().toISOString();
  const stored={notifyMorningReadiness:true,notifyWeeklyCheckin:true,notifyMetricChanges:true,notifyVitals:true,notifyTrainingStatus:false,notifyBedtime:false,bedtimeLeadMin:30,adaptiveToday:true};
  const state=window.testLayout={stored,saves:[],failNext:false};
  const focus=()=>stored.adaptiveToday
   ?{...samples.today.focus,phase:'day',cards:['now','focus','sleepPlan'],planOpen:false}
   :{...samples.today.focus,phase:'overview',cards:['focus','sleepPlan'],planOpen:true};
  window.isTauri=true;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
   if(command.startsWith('plugin:event|'))return 1;
   if(command==='sync_now')return;
   if(command==='app_status')return structuredClone(samples.status);
   if(command==='get_settings')return structuredClone(stored);
   if(command==='set_settings'){
    await new Promise(resolve=>state.saves.push(resolve));
    if(state.failNext){state.failNext=false;throw Error('Synthetic save failure')}
    Object.assign(stored,args.settings);return;
   }
   if(command==='today')return {...structuredClone(samples.today),focus:focus()};
   if(command in samples)return structuredClone(samples[command]);
   throw Error('Unexpected command '+command);
  }};
 },samples);
 const leader=()=>layout.locator('.dashboard > *').first().evaluate(el=>el.className);
 const followSwitch=()=>layout.locator('label.setting',{hasText:'Follow my day'}).locator('input');
 const saved=()=>layout.evaluate(()=>window.testLayout.saves.shift()());
 await layout.goto(`${url}/#today`);
 await layout.locator('.now-card').waitFor();
 await layout.evaluate(()=>{location.hash='#settings'});
 await followSwitch().waitFor();
 await followSwitch().uncheck();
 await layout.waitForFunction(()=>window.testLayout.saves.length===1);
 assert.equal(await followSwitch().isDisabled(),true,'the switch is locked until its save finishes');
 // A quick second click must not enqueue an opposing save that can finish out of order.
 await followSwitch().evaluate(el=>el.click());
 assert.equal(await followSwitch().isChecked(),false);
 assert.equal(await layout.evaluate(()=>window.testLayout.saves.length),1,'only one layout save is in flight');
 await layout.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();
 await layout.locator('.now-card').waitFor();
 assert.ok((await leader()).includes('now-card'),'before the save lands, Today still has the stored layout');
 await saved();
 // Well inside the 30-second cache poll, so the save itself must bring the new layout.
 await layout.waitForFunction(()=>!document.querySelector('.now-card') && document.querySelector('.dashboard > *')?.className.includes('daily-focus'),null,{timeout:5000});
 assert.equal(await layout.evaluate(()=>location.hash),'#today','the user stays where they went');
 // A failed save is reported and the switch returns to the stored choice.
 await layout.evaluate(()=>{location.hash='#settings'});
 await followSwitch().waitFor();
 assert.equal(await followSwitch().isChecked(),false);
 await layout.evaluate(()=>{window.testLayout.failNext=true});
 await followSwitch().check();
 await layout.waitForFunction(()=>window.testLayout.saves.length===1);
 await saved();
 await layout.getByText('Synthetic save failure').waitFor();
 assert.equal(await followSwitch().isChecked(),false,'a failed save is not treated as success');
 assert.equal(await followSwitch().isEnabled(),true,'a failed save unlocks the switch');
 await layout.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();
 await layout.locator('.dashboard > *').first().waitFor();
 assert.ok((await leader()).includes('daily-focus') && await layout.locator('.now-card').count()===0,'Today keeps the stored fixed layout');
 // A later change is accepted after the pending save has finished.
 await layout.evaluate(()=>{location.hash='#settings'});
 await followSwitch().waitFor();
 await followSwitch().check();
 await layout.waitForFunction(()=>window.testLayout.saves.length===1);
 assert.equal(await followSwitch().isDisabled(),true);
 await saved();
 await layout.waitForFunction(()=>window.testLayout.stored.adaptiveToday && ![...document.querySelectorAll('label.setting')].find(l=>l.textContent.includes('Follow my day')).querySelector('input').disabled);
 assert.equal(await followSwitch().isChecked(),true,'the switch and store agree on the last accepted change');
 await layout.close();
 console.log('Follow my day: slow save, rapid clicks, navigation, failure and retry: passed');
 assert.deepEqual(errors,[],'no browser errors');
} finally {
 await browser?.close();
 await server.close();
}
