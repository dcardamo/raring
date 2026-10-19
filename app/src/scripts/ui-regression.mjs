import assert from 'node:assert/strict';
import {mkdir,readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {createServer} from 'vite';
import {chromium} from 'playwright';
import {contrastFailures} from './contrast.mjs';

const server=await createServer({configFile:resolve('vite.config.ts'),server:{host:'127.0.0.1',port:0}});
const output=resolve('../.data/screens');
// Sleep opens from Today; Settings sits in Profile, behind each root header's avatar.
const openSleep=async page=>{if(!page.url().endsWith('#today')){await page.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();await page.waitForURL('**/#today');}await page.locator('.morning .sleep-fact').first().click();};
const openSettings=async page=>{await page.getByRole('button',{name:'Profile and settings'}).first().click();await page.getByRole('button',{name:/^Settings/}).click();};
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
 const privatePage=await browser.newPage({viewport:{width:390,height:844}});
 await privatePage.goto(`${url}/?fixture=private#sleep`);
 await privatePage.getByText('Wed 15 Oct 23:45 – Thu 16 Oct 00:15',{exact:true}).waitFor();
 await privatePage.getByText('Start date/time unavailable – Thu 16 Oct 16:30',{exact:true}).waitFor();
 await privatePage.screenshot({path:resolve(output,'naps-dated-midnight-390.png'),fullPage:true});
 await privatePage.getByRole('navigation').getByRole('button',{name:'Health'}).click();
 await privatePage.getByRole('button',{name:'Allow local journal reads'}).waitFor();
 assert.equal(await privatePage.getByText('Stretching',{exact:true}).count(),0,'journal entries hidden before opt-in');
 await openSettings(privatePage);
 await privatePage.getByText('Already downloaded logs remain on this device and in future exports until you erase local data.',{exact:false}).waitFor();
 assert.equal(await privatePage.locator('.card').filter({has:privatePage.getByRole('heading',{name:'Notifications',exact:true})}).locator('input[type=checkbox]').count(),6,'journal consent is not a notification switch');
 const consent=privatePage.locator('.card').filter({hasText:'Journal privacy'}).locator('input[type=checkbox]');
 assert.equal(await consent.isChecked(),false,'consent defaults off');
 await privatePage.getByRole('navigation').getByRole('button',{name:'Health'}).click();
 await privatePage.getByRole('button',{name:'Allow local journal reads'}).click();
 await privatePage.getByRole('button',{name:'Open journal'}).click();
 await privatePage.getByText('Stretching',{exact:true}).waitFor();
 await privatePage.screenshot({path:resolve(output,'journal-consented-390.png'),fullPage:true});
 await openSettings(privatePage);
 const enabled=privatePage.locator('.card').filter({hasText:'Journal privacy'}).locator('input[type=checkbox]');
 assert.equal(await enabled.isChecked(),true);
 await enabled.uncheck();
 await privatePage.getByText('Previously downloaded journal logs are included even if journal reads are now off',{exact:false}).waitFor();
 await privatePage.getByRole('navigation').getByRole('button',{name:'Health'}).click();
 await privatePage.getByRole('button',{name:'Allow local journal reads'}).waitFor();
 assert.equal(await privatePage.getByText('Stretching',{exact:true}).count(),0,'cached private entries hidden on opt-out');
 await privatePage.screenshot({path:resolve(output,'journal-disabled-390.png'),fullPage:true});
 await privatePage.getByRole('button',{name:'Allow local journal reads'}).click();
 await privatePage.getByRole('button',{name:'Open journal'}).click();
 await privatePage.getByText('Stretching',{exact:true}).waitFor();
 await privatePage.close();
 console.log('Dated naps, opt-in/out cache, retained journal export copy and notification separation: passed');
 page.on('pageerror',e=>errors.push(e.message));
 for(const width of [390,700,1100]){
  const watch=await browser.newPage({viewport:{width,height:844}});
  // Stale-upload prompts are intentionally suppressed at night.
  await watch.clock.setFixedTime(new Date('2025-10-16T14:00:00'));
  await watch.goto(`${url}/?fixture=stale#today`);
  await watch.getByText('Open Garmin Connect to sync your watch',{exact:true}).waitFor();
  await watch.getByText("Your watch doesn't record skin temperature",{exact:true}).waitFor();
  await watch.screenshot({path:resolve(output,`watch-stale-${width}.png`)});
  await watch.getByRole('button',{name:'Open Garmin Connect'}).click();
  await watch.getByText('Open Garmin Connect on your phone, then sync your watch there.',{exact:true}).waitFor();
  await watch.close();
 }
 for(const width of [390,700,1100]){
  const surface=await browser.newPage({viewport:{width,height:1000}});
  await surface.goto(`${url}/?fixture=status-offer#today`);
  const enable=surface.getByRole('button',{name:'Enable persistent status',exact:true});
  await enable.waitFor();
  await enable.scrollIntoViewIfNeeded();
  await surface.screenshot({path:resolve(output,`status-offer-${width}.png`)});
  await surface.getByRole('button',{name:'Not now',exact:true}).click();
  await openSettings(surface);
  const toggle=surface.getByRole('checkbox',{name:/Persistent status/});
  await toggle.waitFor();
  assert.equal(await toggle.isChecked(),false,'status is opt-in');
  assert.equal(await surface.getByRole('checkbox').count(),10,'internal offer flag is not a toggle (the plan prompt is)');
  await toggle.check();
  await surface.screenshot({path:resolve(output,`status-settings-${width}.png`)});
  await toggle.uncheck();
  await surface.getByRole('button',{name:'Today',exact:true}).first().click();
  await surface.locator('h1').waitFor();
  assert.equal(await enable.count(),0,'offer is not repeated after declining');
  await surface.close();
 }
 const desktopSettings=await browser.newPage();
 await desktopSettings.goto(`${url}/#settings`);
 await desktopSettings.getByRole('heading',{name:'Settings',exact:true}).waitFor();
 assert.equal(await desktopSettings.getByRole('checkbox',{name:/Persistent status/}).count(),0,'native status is Android-only');
 await desktopSettings.close();
 console.log('Android status offer once, opt-in settings and fixture screenshots at 390/700/1100: passed');
 const quietWatch=await browser.newPage({viewport:{width:390,height:844}});
 await quietWatch.clock.setFixedTime(new Date('2025-10-16T23:30:00'));
 await quietWatch.goto(`${url}/?fixture=stale#today`);
 await quietWatch.getByText('Watch synced 7 h ago',{exact:true}).waitFor();
 assert.equal(await quietWatch.getByText('Open Garmin Connect to sync your watch',{exact:true}).count(),0,'stale-upload prompts remain quiet at night');
 await quietWatch.close();
 const fresh=await browser.newPage({viewport:{width:390,height:844}});
 await fresh.clock.setFixedTime(new Date('2025-10-16T14:00:00'));
 await fresh.goto(`${url}/?fixture=fresh#today`);
 await fresh.getByText('Watch synced 2 h ago',{exact:true}).waitFor();
 assert.equal(await fresh.getByText('Open Garmin Connect to sync your watch',{exact:true}).count(),0);
 await fresh.screenshot({path:resolve(output,'watch-fresh-390.png')});
 await fresh.close();
 const sleepUpload=await browser.newPage({viewport:{width:390,height:844}});
 await sleepUpload.clock.setFixedTime(new Date('2025-10-16T14:00:00'));
 await sleepUpload.goto(`${url}/?fixture=stale-sleep#today`);
 await sleepUpload.getByText("Last night's sleep hasn't uploaded yet.",{exact:true}).waitFor();
 assert.equal(await sleepUpload.getByText('Open Garmin Connect to sync your watch',{exact:true}).count(),1,'missing sleep gives one sync prompt in the upload card');
 assert.equal(await sleepUpload.getByText('About missing readings',{exact:true}).count(),0,'upload delays are not unsupported capabilities');
 await sleepUpload.close();
 console.log('Watch freshness, missing capability, Connect fallback at 390/700/1100: passed');
 for(const width of [390,700,1100]){
  const planned=await browser.newPage({viewport:{width,height:844}});
  await planned.goto(`${url}/?fixture=planned#today`);
  await planned.getByRole('region',{name:'Your day on one timeline'}).waitFor();
  assert.equal(await planned.getByText('Easy Run',{exact:true}).count(),0,"Garmin's next session lives in Plan and Training, not Today");
  await planned.screenshot({path:resolve(output,`planned-today-${width}.png`),fullPage:true});
  await planned.getByRole('navigation').getByRole('button',{name:'Training'}).click();
  await planned.getByText('Easy Run',{exact:true}).waitFor();
  await planned.getByText('City 10K',{exact:true}).waitFor();
  await planned.screenshot({path:resolve(output,`planned-training-${width}.png`),fullPage:true});
  await planned.close();
 }
 const emptyPlanned=await browser.newPage({viewport:{width:390,height:844}});
 const emptyToday=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
 const emptyTraining=JSON.parse(await readFile(resolve('src/lib/sample/training.json'),'utf8'));
 emptyToday.planned={state:'empty',session:null,coach:null,coverageStart:'2025-10-16',coverageEnd:'2025-10-30'};
 emptyTraining.planned=emptyToday.planned;emptyTraining.race=null;emptyTraining.raceState='empty';
 await emptyPlanned.route(u=>['/src/lib/sample/today.json','/src/lib/sample/training.json'].includes(u.pathname),route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(route.request().url().includes('today.json')?emptyToday:emptyTraining)}`}));
 await emptyPlanned.goto(`${url}/?fixture=planned-empty#training`);
 await emptyPlanned.getByText(/No workouts in Garmin’s cached schedule.*16 Oct.*30 Oct/).waitFor();
 await emptyPlanned.getByText('No upcoming race in Garmin’s next 90 days.',{exact:true}).waitFor();
 emptyTraining.raceState='partial';
 await emptyPlanned.goto(`${url}/?fixture=planned-partial#training`);
 await emptyPlanned.getByText('Race schedule incomplete: Garmin returned the first 10 events.',{exact:true}).waitFor();
 await emptyPlanned.close();
 console.log('Planned training, race and empty states at phone, unfolded and desktop widths: passed');
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
 await tab('Plan');await tab('Training');await tab('Health');await tab('Progress');
 await back('today');
 await tab('Training');
 await openSettings(page);await waitRoute('settings');
 await back('profile');await back('training');await back('today');
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
 await page.getByRole('button',{name:'Stress through today'}).click();await waitRoute('metric/stress');
 await back('health');
 await page.getByRole('button',{name:'View Respiration',exact:true}).click();await waitRoute('metric/respiration');
 await back('health');await back('today');
 await page.locator('.activity').first().click();await waitRoute('workout/9000000000001');
 await back('today');
 await tab('Training');
 await page.getByRole('tab',{name:'Activities'}).click();
 await page.locator('.activity').first().click();await waitRoute('workout/9000000000001');
 await page.reload();await waitRoute('workout/9000000000001');
 await back('training');await back('today');
 await tab('Plan');await tab('Plan');await back('today');
 assert.equal(await page.evaluate(()=>window.__raringBack()),false,'Today lets Android leave the app');
 await tab('Training');
 assert.equal(await page.evaluate(()=>window.__raringBack()),true);
 await waitRoute('today');
 console.log('Hierarchical browser, header and native Back: passed');

 await page.getByRole('button',{name:/^Readiness \d+/}).click();await waitRoute('readiness');
 await page.getByText('Readiness history',{exact:true}).waitFor();
 await page.getByRole('button',{name:'30 days',exact:true}).click();
 assert.equal(await page.locator('.history-bar').count(),30);
 await back('today');
 // After waking, tonight's plan is one line that opens Sleep.
 await page.getByRole('button',{name:/^Tonight · Garmin plan/}).click();await waitRoute('sleep/tonight');
 assert.equal(await page.getByRole('tab',{name:'Tonight'}).getAttribute('aria-selected'),'true','the tonight line opens the Tonight tab');
 const plan=page.locator('.tonight-plan');
 assert.ok((await plan.textContent()).includes('22:30–23:06'));
 assert.equal(await plan.getByRole('img',{name:/^Tonight: now/}).count(),1,'a compact dial with labelled times');
 assert.ok((await plan.textContent()).includes('Wind down22:00'),'the plan leads with the wind-down time');
 assert.ok((await plan.textContent()).includes('Garmin plan'));
 assert.ok((await plan.textContent()).includes("Garmin added 20 min for today's activity and load."));
 await plan.getByRole('button',{name:'Why this plan'}).click();
 const evidence=await plan.locator('.evidence').textContent();
 assert.ok(evidence.includes("Garmin's recommendation for tonight.") && evidence.includes("Training: today's activity and load") && evidence.includes('30 min before the window starts'),evidence);
 const recentNights=page.locator('.tonight-nights');
 assert.ok((await recentNights.textContent()).includes('2 of 6 recorded nights below Garmin’s need.'),await recentNights.textContent());
 assert.ok((await recentNights.getByRole('img').getAttribute('aria-label')).includes('Sun 12 Oct not recorded'),'a missing night is labelled, not zero');
 assert.equal(await page.locator('.tonight-goals').count(),1);
 await page.getByRole('tab',{name:'Last night'}).click();await waitRoute('sleep/last');
 assert.equal(await page.locator('.tonight-plan').count(),0);
 await page.goBack();await waitRoute('today');
 await page.getByRole('button',{name:/^Tonight · Garmin plan/}).click();await waitRoute('sleep/tonight');
 await page.getByRole('tab',{name:'Last night'}).click();await waitRoute('sleep/last');
 const alignment=page.locator('.alignment');
 assert.equal(await alignment.locator('h3 .pill').textContent(),'Aligned');
 assert.equal(await alignment.locator('.night').count(),7,'a week of nights against Garmin\'s optimal window');
 assert.equal(await alignment.locator('rect.optimal').getAttribute('stroke-dasharray'),'4 3','Garmin\'s target is outlined, never shaded like usual windows');
 assert.ok((await alignment.textContent()).includes('Optimal 22:45–06:15'));
 assert.ok((await page.locator('.consistency .averages').textContent()).includes('22:55'));
 await tab('Health');
 assert.equal(await page.locator('.dashboard>.card').first().locator('h3>span').first().textContent(),'Long-term health · Fitness Age','long-term health leads (U55)');
 const titles=await page.locator('.health-dashboard>.card h3>span:first-child').allTextContents();
 assert.ok(titles.indexOf('Vitals')<titles.indexOf('Resting heart rate'),'vitals come before the metric charts: '+titles.join(', '));
 assert.equal(await page.getByText('Heart rate · all day',{exact:true}).count(),0,'Your day stays on Today');
 assert.equal(await page.locator('.over-time .history').count(),4,'daily metrics over a range');
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
 await page.goto(`${url}/#checkin`);await waitRoute('review/latest');
 for(let i=0;i<4;i++)await page.getByRole('button',{name:'Next'}).click();
 const timing=page.locator('.timing-review');
 await timing.waitFor();
 assert.equal(await page.locator('.checkin-controls span').textContent(),'5 / 7','the timing review follows the area cards');
 const review=await timing.textContent();
 assert.ok(review.includes('Bedtimes22:31–00:30') && review.includes('Garmin: 5 aligned · 2 behind') && review.includes('Keep waking near 06:48.'),review);
 assert.equal(await timing.locator('rect.night').count(),7);
 await page.getByRole('button',{name:'Next'}).click();
 assert.ok((await page.locator('.intensity-review').textContent()).includes("Garmin's goal reached"));
 await page.getByRole('button',{name:'Next'}).click();
 await page.getByText('THE FOCUS',{exact:true}).waitFor();
 // Goals: each week against its own goals, then proposals that need acceptance.
 const goalsReview=page.locator('.wr');
 const reviewText=await goalsReview.textContent();
 assert.ok(reviewText.includes('Zone 2 168 of 150 min') && reviewText.includes('Met'),reviewText);
 assert.ok(reviewText.includes('Met in your last two complete weeks, 168 of 150 min last week'));
 assert.equal(await goalsReview.getByRole('button',{name:/^Accept/}).isDisabled(),true,'nothing is accepted by default');
 await goalsReview.getByRole('checkbox').check();
 assert.equal(await goalsReview.getByRole('button',{name:'Accept 1 change'}).isEnabled(),true);
 assert.equal(await goalsReview.getByRole('button',{name:'Keep all'}).count(),1,'Keep all is always offered');
 await goalsReview.getByRole('button',{name:'Edit goals'}).click();await waitRoute('plan/commitments');
 await back('review/latest');
 await page.goto(`${url}/#reviews`);await waitRoute('reviews');await page.locator('.review-row').first().waitFor();
 assert.equal(await page.locator('.review-row').count(),2);
 assert.ok((await page.locator('.review-row').nth(1).textContent()).includes('Not counted · 2 ill days'));
 await page.locator('.review-row').first().click();await waitRoute('review/2025-10-06');
 await back('reviews');
 await page.goto(`${url}/#today`);await waitRoute('today');
 await page.locator('.week-card').click();await waitRoute('plan');
 const week=await page.locator('.wk').textContent();
 assert.ok(week.includes('Strength 3 of 4 sessions · 1 placed'),week);
 assert.ok(week.includes('Zone 2 100 of 150 min · 40 placed · 10 left'),week);
 assert.ok(week.includes('Intensity 124 of 150 min · 26 left'),week);
 assert.equal(await page.locator('.day-cells .dc').count(),7);
 await page.getByRole('button',{name:/^Friday 17 Oct/}).click();
 await page.getByText('Garmin schedule · read-only',{exact:true}).waitFor();
 assert.ok((await page.locator('.days').textContent()).includes('Easy ride'),'your session and Garmin\'s share the day');
 await page.getByRole('button',{name:/^Monday 13 Oct/}).click();
 assert.ok((await page.locator('.days').textContent()).includes('Done · matched by Raring'));
 assert.ok((await page.locator('main').textContent()).includes('Left this week'));
 // The plan loop: a session's choices, the backlog taken for today, events and the disabled Garmin write.
 await page.getByRole('button',{name:/^Friday 17 Oct/}).click();
 await page.locator('.row-item',{hasText:'Easy ride'}).click();
 const item=page.getByRole('dialog',{name:'Easy ride'});
 await item.getByRole('button',{name:/^Move/}).waitFor();
 assert.equal(await item.getByRole('button',{name:/^Mark as missed/}).count(),0,'a future session is never missed');
 assert.equal(await item.getByRole('button',{name:/^Send to Garmin/}).isDisabled(),true,'Garmin writes are not available yet');
 await item.getByRole('button',{name:/^Move/}).click();
 await item.getByRole('group',{name:'Move to'}).waitFor();
 await page.keyboard.press('Escape');
 await page.locator('.left li',{hasText:'10 min Zone 2'}).getByRole('button',{name:'Do it today'}).click();
 const add=page.getByRole('dialog',{name:'Add to my plan'});
 assert.equal(await add.getByRole('spinbutton',{name:'Zone 2 minutes you intend'}).inputValue(),'10','the backlog amount is prefilled');
 assert.equal(await add.getByRole('group',{name:'Day'}).getByRole('button',{name:'Today'}).getAttribute('aria-pressed'),'true');
 assert.deepEqual(await contrastFailures(page),[],'add sheet contrast');
 await page.keyboard.press('Escape');
 assert.ok((await page.locator('.event').first().textContent()).includes('Harbour 10K'));
 await page.getByRole('button',{name:'Add a race or event'}).click();
 await page.getByRole('dialog',{name:'Add a race or event'}).getByRole('button',{name:'Add event'}).waitFor();
 assert.equal(await page.getByRole('dialog',{name:'Add a race or event'}).getByRole('button',{name:'Add event'}).isDisabled(),true,'a name and date are needed');
 await page.keyboard.press('Escape');
 await page.getByRole('button',{name:'Goals',exact:true}).click();await waitRoute('plan/commitments');
 await page.getByRole('button',{name:'More Strength'}).click();
 assert.equal(await page.locator('.eg').first().locator('output').textContent(),'5');
 assert.equal(await page.locator('.eg').nth(3).locator('small').textContent(),'Garmin’s weekly goal');
 await page.getByRole('button',{name:'Save weekly goals'}).click();await waitRoute('plan');
 await back('today');
 console.log('Plan week, goals, check-in plan and intensity pages: passed');
 await page.goto(`${url}/#settings`);await waitRoute('settings');
 assert.equal(await page.locator('.setting.lead').count(),0,'the lead appears with the bedtime reminder');
 await page.locator('label.setting',{hasText:'Bedtime'}).locator('input').check();
 await page.locator('.setting.lead select').selectOption('45');
 await tab('Today');
 await openSettings(page);await waitRoute('settings');
 assert.equal(await page.locator('.setting.lead select').inputValue(),'45');
 console.log('Daily focus, sleep plan, rhythm, timing review, reminder lead and keyboard/touch history: passed');

 await page.setViewportSize({width:390,height:844});
 await page.goto(`${url}/#training`);await waitRoute('training');
 await page.getByRole('tab',{name:'Strength'}).click();
 await page.getByRole('button',{name:/^Strength this week/}).click();await waitRoute('strength');
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
 assert.equal(await page.locator('.heat-card').count(),0,'the 12-week heatmap is replaced by the muscle range');
 await page.getByRole('button',{name:'Barbell Bench Press',exact:true}).click();
 assert.equal(await page.locator('.card h3',{hasText:'Barbell Bench Press'}).count(),1);
 const muscleRange=page.getByRole('group',{name:'Muscle range'});
 await muscleRange.getByRole('button',{name:'7 days'}).click();
 assert.equal(await muscleRange.getByRole('button',{name:'7 days'}).getAttribute('aria-pressed'),'true');
 await page.getByRole('button',{name:/^Wed 15 Oct/}).click();await waitRoute('workout/9000000000111');
 await back('strength');
 await page.getByRole('button',{name:'Help',exact:true}).click();await waitRoute('help');
 await page.getByRole('button',{name:'How sets are counted'}).click();await waitRoute('help/strength-sets');
 await page.getByText(/1\.8 times/).waitFor();
 await page.getByText(/Later edits will not appear in Raring, even after Refresh/).waitFor();
 await back('help');await back('strength');await back('training');
 await page.getByRole('tab',{name:'Activities'}).click();
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
  for(const route of ['today','sleep','training','health','readiness','workout/9000000000001','workout/9000000000002','strength','help','help/strength-sets','metric/hrv','metric/restingHr','metric/bodyBattery','metric/stress','settings','review/latest','plan','plan/commitments','progress','profile','onboarding']) {
   await page.goto(`${url}/#${route}`);await waitRoute(route);
   const layout=await page.evaluate(()=>{
    // Largest vertical gap between cards stacked in the same grid column.
    const holes=(grid)=>{
     if(!grid || getComputedStyle(grid).display!=='grid')return 0;
     const cards=[...grid.querySelectorAll(':scope>*,:scope>.row>*')].filter(e=>getComputedStyle(e).display!=='contents').map(e=>e.getBoundingClientRect());
     return Math.max(0,...cards.map(c=>{const above=cards.filter(o=>o!==c && o.left<c.right-1 && o.right>c.left+1 && o.bottom<=c.top+1);return above.length?c.top-Math.max(...above.map(o=>o.bottom)):0}));
    };
    // On wide screens Today's headline sits beside the dials, not below them.
    const hero=()=>null;
    const main=document.querySelector('main'),grid=document.querySelector('.dashboard'),phone=document.querySelector('.phone');
    return {viewport:innerWidth,body:document.body.scrollWidth,main:main.clientWidth,content:main.scrollWidth,shell:phone.clientWidth,height:phone.clientHeight,columns:grid?getComputedStyle(grid).gridTemplateColumns.split(' ').length:0,holes:holes(grid),hero:hero()};
   });
   assert.equal(layout.shell,width,`${width}/${route}: shell uses available width`);
   assert.equal(layout.height,900,`${width}/${route}: shell uses available height`);
   assert.ok(layout.body<=width && layout.content<=layout.main+1,`${width}/${route}: horizontal overflow ${JSON.stringify(layout)}`);
   if(width>=660 && layout.columns)assert.equal(layout.columns,2,`${width}/${route}: two columns`);
   if(route==='health'){
    await page.locator('.weight-card .fresh').waitFor();
    assert.ok((await page.locator('.weight-card .fresh').textContent()).includes('Last measured Thu 16 Oct'),`${width}/health: dated weigh-in`);
    assert.ok(await page.getByText('recorded weigh-ins only',{exact:false}).isVisible(),`${width}/health: sparse trend label`);
   }
   assert.ok(layout.holes<=18,`${width}/${route}: ${layout.holes}px hole between cards`);
   if([390,840].includes(width) && ['today','sleep','training','workout/9000000000001','workout/9000000000002','strength','help/strength-sets','health','plan','plan/commitments','progress','profile'].includes(route)) {
    await page.evaluate(()=>document.fonts.ready);
    const failures=await contrastFailures(page);
    assert.deepEqual(failures,[],`${width}/${route}: text contrast`);
   }
   if(width===840 || (width===390 && ['today','sleep','training','workout/9000000000001','health','metric/restingHr','onboarding','settings','plan','plan/commitments'].includes(route)) || (width===1024 && ['today','sleep','health','metric/restingHr','plan','plan/commitments'].includes(route)) || (width===660 && ['health','metric/restingHr'].includes(route)))await page.screenshot({path:resolve(output,`${width}-${route.replace('/','-')}.png`)});
  }
 }
 console.log('Today, Sleep, Training, Workout, Health and plan text contrast at 390/840: passed');
 // Generated by the actual Rust projection from independently synthetic weigh-ins.
 const weightSource=await readFile(resolve('src/lib/sample/weight.ts'),'utf8');
 const weightFixtures=JSON.parse(weightSource.match(/export const weightFixtures = (.+) satisfies /)[1]);
 for(const indices of [[2],[1,7]]){
  const sparse=structuredClone(weightFixtures);
  for(const range of Object.keys(sparse.kg)){
   sparse.kg[range].points=Array.from({length:12},(_,i)=>({date:`2025-10-${String(i+1).padStart(2,'0')}`,scale:indices.includes(i)?72+i/10:null,trend:72,breakScale:indices.includes(i)&&i!==indices[0]}));
  }
  const sparsePage=await browser.newPage({viewport:{width:390,height:844}});
  await sparsePage.route('**/src/lib/sample/weight.ts*',route=>route.fulfill({contentType:'text/javascript',body:`export const weightFixtures=${JSON.stringify(sparse)}`}));
  await sparsePage.goto(`${url}/?weight-sparse=${indices.join('-')}#health`);
  const card=sparsePage.locator('.weight-card');
  await card.locator('.fresh').waitFor();
  assert.equal(await card.locator('.scale-mark').count(),indices.length,'each actual scale reading has a marker');
  assert.equal(await card.locator('.trend-mark').count(),indices.length,'trend markers correspond to actual weigh-ins');
  const path=await card.locator('.scale-line').getAttribute('d');
  assert.equal((path.match(/M/g)??[]).length,indices.length,'core-defined long gaps split the scale line');
  assert.ok(!path.includes('L')&&!path.includes('C'),'isolated readings are not joined');
  await sparsePage.close();
 }
 for(const unit of ['kg','lb']){
  const wp=await browser.newPage({viewport:{width:390,height:844}});
  const fixtures={kg:structuredClone(weightFixtures[unit])};
  for(const v of Object.values(fixtures.kg)){v.freshness.ageDays=12;v.freshness.stale=true;}
  await wp.route('**/src/lib/sample/weight.ts*',r=>r.fulfill({contentType:'text/javascript',body:`export const weightFixtures=${JSON.stringify(fixtures)}`}));
  await wp.goto(`${url}/?weight-unit=${unit}#health`);
  const card=wp.locator('.weight-card');await card.locator('.fresh.stale').waitFor();
  assert.ok((await card.locator('.weight-head').textContent()).includes(unit));
  assert.equal(await card.locator('.changes li').count(),6);
  assert.equal(await card.getByRole('button',{name:'Add weight',exact:true}).isDisabled(),true);
  await card.getByRole('group',{name:'Weight range'}).getByRole('button',{name:'14 days',exact:true}).click();
  await wp.waitForFunction(()=>document.querySelector('.weight-card')?.getAttribute('aria-busy')==='false');
  await card.getByRole('slider',{name:'Weight readings'}).press('Home');
  assert.ok((await card.getByRole('slider').getAttribute('aria-valuetext')).includes(unit));
  assert.deepEqual(await contrastFailures(wp),[],`weight ${unit} contrast`);
  await card.getByRole('button',{name:'Weight details'}).click();
  await wp.waitForFunction(()=>location.hash==='#health/weight');
  await wp.locator('.weight-card .fresh').waitFor();
  assert.equal(await wp.getByRole('group',{name:'Weight range'}).getByRole('button',{name:'14 days',exact:true}).getAttribute('aria-pressed'),'true','range persists into canonical detail');
  await wp.close();
 }
 console.log('Weight: core fixtures, gap markers, units, freshness, ranges and canonical detail passed');
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
 // The live lead and Adapt: rest outranks readiness, with no effort scale, and Adapt shows its state.
 {
  const leadPage=await browser.newPage({viewport:{width:390,height:844}});
  leadPage.on('pageerror',e=>errors.push(e.message));
  const sample=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
  const rest={...sample,call:null,focus:{...sample.focus,effort:false},
   lead:{id:'rest:2025-10-16',kind:'rest',headline:'Rest day',line:'Through tonight',action:{label:'View plan',route:'plan',adapt:false},dismissable:false,activity:null,intention:null,
    reasons:[{gate:'Privacy',result:'Journal entries never feed Today.'},{gate:'Circumstances',result:'Rest day: no exercise prompts today or tonight; your plan stays as it is.'}]},
   adapt:{rest:true,pause:null,limits:{excluded:['running'],note:''},label:'Rest day',reviewDue:false,blocked:true,routines:[{item:{id:'r1',name:'Wall sit 3 × 30 s',how:'From my physio'},done:false}]}};
  await leadPage.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(rest)}`}));
  await leadPage.goto(`${url}/#today`);
  await leadPage.getByRole('heading',{name:'Rest day',exact:true}).waitFor();
  assert.equal(await leadPage.locator('.hero .effort').count(),0,'no effort scale on a rest day');
  assert.equal(await leadPage.locator('.top .adapt').textContent(),'Rest day','the header names the circumstance');
  assert.equal(await leadPage.getByRole('button',{name:'Why this?'}).count(),0,'the reason line replaces Why this');
  await leadPage.locator('.top .adapt').click();
  const sheet=leadPage.getByRole('dialog',{name:'Adapt today'});
  await sheet.getByText('Rest day is on').waitFor();
  assert.ok((await sheet.textContent()).includes('Avoiding Run'));
  await sheet.getByRole('button',{name:/^Limits/}).click();
  await leadPage.getByRole('dialog',{name:'Your limits'}).waitFor();
  assert.equal(await leadPage.getByRole('button',{name:'Run',exact:true}).getAttribute('aria-pressed'),'true');
  assert.deepEqual(await contrastFailures(leadPage),[],'Adapt sheet contrast');
  await leadPage.keyboard.press('Escape');
  assert.equal(await leadPage.getByRole('dialog').count(),0);
  // My routines: the person's own checklist, self-reported, sent as local actions only.
  await leadPage.locator('.top .adapt').click();
  await leadPage.getByRole('dialog',{name:'Adapt today'}).getByRole('button',{name:/^My routines/}).click();
  const routines=leadPage.getByRole('dialog',{name:'My routines'});
  await routines.waitFor();
  assert.ok((await routines.textContent()).includes('Wall sit 3 × 30 s') && (await routines.textContent()).includes('From my physio'));
  assert.equal(await routines.getByRole('button',{name:'Mark done today'}).count(),1);
  assert.ok((await routines.textContent()).includes('no Garmin activity, intensity minutes or records'));
  assert.ok(await routines.getByRole('button',{name:'Add to checklist'}).isDisabled(),'a name is required');
  await routines.getByLabel('Exercise').fill('Calf raises');
  assert.ok(!(await routines.getByRole('button',{name:'Add to checklist'}).isDisabled()));
  assert.deepEqual(await contrastFailures(leadPage),[],'Routines contrast');
  await leadPage.keyboard.press('Escape');
  await leadPage.getByRole('navigation').getByRole('button',{name:'Plan',exact:true}).click();
  await leadPage.waitForURL('**/#plan');
  assert.equal(await leadPage.locator('.top .adapt').textContent(),'Rest day','Adapt keeps its place on every root page');
  // An option to train is dismissable for the day.
  const suggest={...sample,lead:{id:'suggest:2025-10-16',kind:'suggest',headline:'Training is an option today',line:'After 7:48 of sleep',action:{label:'Plan a session',route:'plan',adapt:false},dismissable:true,activity:null,intention:null,reasons:[]},
   adapt:{rest:false,pause:null,limits:{excluded:[],note:''},label:'Adapt today',reviewDue:false,blocked:false,routines:[]}};
  await leadPage.close();
  const optionPage=await browser.newPage({viewport:{width:390,height:844}});
  await optionPage.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(suggest)}`}));
  await optionPage.goto(`${url}/#today`);
  await optionPage.getByRole('heading',{name:'Training is an option today'}).waitFor();
  await optionPage.locator('.hero .effort').waitFor();
  assert.equal(await optionPage.getByRole('button',{name:'Not today'}).count(),1);
  await optionPage.screenshot({path:resolve(output,'390-today-lead.png')});
  await optionPage.close();
 }
 console.log('Today lead, Why this and Adapt sheet: passed');
 // Your day: one 00–24 axis with sleep, Body Battery, stress, heart rate and events, nothing past the
 // upload; the nap is listed under Recorded today. A kept session offers Keep · Choose another.
 {
  const dayPage=await browser.newPage({viewport:{width:390,height:844}});
  dayPage.on('pageerror',e=>errors.push(e.message));
  const sample=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
  const board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
  const own={...board.items[1],intention:{...board.items[1].intention,id:'i-today',date:'2025-10-16',time:'18:00',name:'Hill run',sport:'running'}};
  board.items.push(own);board.garmin.push({date:'2025-10-16',name:'Tempo'});
  const nap=sample.day.events.find(e=>e.kind==='nap');nap.bodyBatteryChange=6;
  const session={...sample,lead:{id:'session:i-today',kind:'session',headline:'Hill run at 18:00',line:'Your plan',action:{label:'Your plan',route:'plan',adapt:false},dismissable:false,kept:false,garmin:null,activity:null,intention:'i-today',reasons:[]},
   adapt:{rest:false,pause:null,limits:{excluded:['cycling'],note:''},label:'Limits on',reviewDue:false,blocked:false,routines:[]}};
  await dayPage.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(session)}`}));
  await dayPage.route(u=>u.pathname==='/src/lib/sample/board.json',route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(board)}`}));
  await dayPage.goto(`${url}/#today`);
  const yd=dayPage.getByRole('region',{name:'Your day on one timeline'});
  await yd.waitFor();
  assert.equal(await yd.getByText('Watch upload 13:30').count(),1);
  assert.equal(await yd.getByText('Now 14:05').count(),1,'the now marker is labelled');
  assert.equal(await yd.locator('.track.sleep i').count(),sample.day.sleep.length,'every stage after midnight is drawn');
  const bbPath=await yd.locator('.track.chart path').first().getAttribute('d');
  assert.ok(Math.max(...bbPath.match(/[ML](\d+)/g).map(t=>+t.slice(1)))<=810,'nothing is drawn after the upload');
  assert.equal(await yd.getByRole('button',{name:'Body Battery 78 at 13:30, nap +6'}).count(),1,'the nap change is on the Body Battery row');
  assert.equal(await yd.getByText('+6 nap').count(),1);
  assert.equal(await yd.getByRole('button',{name:/^Heart rate through the day, resting 54/}).count(),1);
  assert.equal(await yd.getByRole('button',{name:'Morning Run 06:12, 52 min'}).count(),1);
  assert.equal(await dayPage.locator('.dashboard').getByRole('button',{name:/^Body Battery/}).count(),0,'Your day replaces the separate Body Battery card');
  const recorded=dayPage.locator('.card').filter({hasText:'Recorded today'});
  assert.ok((await recorded.textContent()).includes('Body Battery +6 · separate from last night'),'the nap row shows its Body Battery change');
  assert.equal(await dayPage.getByRole('button',{name:'Keep',exact:true}).count(),1);
  const week=dayPage.locator('.week-card');
  const left=week.getByRole('group',{name:'Left this week'});
  assert.equal(await left.getByRole('button',{name:'Do it today'}).count(),board.backlog.filter(b=>b.kind!=='sleepTiming').length,'This week and what is left share one card');
  await dayPage.getByText('That’s today.',{exact:true}).waitFor();
  const length=await dayPage.evaluate(()=>{const el=document.querySelector('main.scroll');return el.scrollHeight/el.clientHeight});
  console.log(`Today at 390×844 is ${length.toFixed(2)} viewports`);
  await left.getByRole('button',{name:'Do it today'}).first().click();
  await dayPage.getByRole('dialog').first().waitFor();
  assert.equal(await dayPage.url().endsWith('#today'),true,'Do it today stays on Today');
  await dayPage.keyboard.press('Escape');
  await dayPage.getByRole('button',{name:'Choose another'}).click();
  const choose=dayPage.getByRole('dialog',{name:'Choose today’s session'});
  await choose.waitFor();
  assert.equal(await choose.getByRole('button',{name:/^Tempo/}).count(),1,"Garmin's session today is offered");
  assert.equal(await choose.getByRole('button',{name:/^Hill run · 18:00/}).count(),1);
  assert.ok(await choose.getByRole('button',{name:/^Easy ride/}).isDisabled(),'an excluded sport cannot be chosen');
  assert.equal(await choose.getByRole('button',{name:/^Strength.*move to today/}).count(),1,'a later session can be moved to today');
  assert.equal(await choose.getByRole('button',{name:/Do it today$/}).count(),board.backlog.length);
  assert.deepEqual(await contrastFailures(dayPage),[],'Choose sheet contrast');
  await choose.getByRole('button',{name:/^Add a session/}).click();
  assert.equal(await choose.count(),0,'Add a session closes Choose');
  await dayPage.getByRole('dialog').first().waitFor();
  for(const width of [320,700]){await dayPage.setViewportSize({width,height:900});await dayPage.keyboard.press('Escape');
   assert.equal(await dayPage.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`Your day fits ${width}px`);}
  await dayPage.close();
  const many=await browser.newPage({viewport:{width:390,height:844}});
  const extra=[1,2,3].map(i=>({...sample.activities[0],id:`900000000010${i}`,name:`Walk ${i}`}));
  await many.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify({...sample,activities:[...sample.activities,...extra]})}`}));
  await many.goto(`${url}/#today`);
  const recordedMany=many.locator('.card').filter({hasText:'Recorded today'});
  await recordedMany.waitFor();
  assert.equal(await recordedMany.locator('.activity').count(),3,'two activities and the nap');
  await recordedMany.getByRole('button',{name:/^All today’s activities \(4\)/}).click();
  await many.waitForURL('**/#activities');
  await many.close();
  const shifted=await browser.newPage({viewport:{width:390,height:844}});
  const dst={...sample,day:{...sample.day,clockChange:true,bodyBattery:[],stress:[],heartRate:[]}};
  await shifted.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(dst)}`}));
  await shifted.goto(`${url}/#today`);
  await shifted.getByRole('region',{name:'Your day on one timeline'}).waitFor();
  assert.equal(await shifted.getByText('Clock change today · open the detail').count(),3,'a clock-change day sends intraday rows to their details');
  assert.equal(await shifted.getByRole('note').filter({hasText:'Clocks changed by an hour today'}).count(),1,'U39: the clock change is said once');
  await shifted.close();
 }
 console.log('Your day timeline, nap Body Battery and Keep / Choose another: passed');
 // Sleep: a plain #sleep opens the closer tab; Tonight is a plan with a self-report, at every width.
 {
  const sp=await browser.newPage({viewport:{width:390,height:844}});
  sp.on('pageerror',e=>errors.push(e.message));
  const sleepSample=JSON.parse(await readFile(resolve('src/lib/sample/sleep.json'),'utf8'));
  await sp.route(u=>u.pathname==='/src/lib/sample/sleep.json',route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify({...sleepSample,tonightFirst:true,woundDown:true})}`}));
  await sp.goto(`${url}/#sleep`);
  await sp.getByRole('tab',{name:'Tonight'}).waitFor();
  assert.equal(await sp.getByRole('tab',{name:'Tonight'}).getAttribute('aria-selected'),'true','evening opens Tonight');
  const wound=sp.getByRole('button',{name:/Wound down · self-reported/});
  assert.ok(await wound.isDisabled(),'a reported wind-down is shown, once');
  for(const width of [320,700,1100]){
   await sp.setViewportSize({width,height:900});
   assert.equal(await sp.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`Tonight fits ${width}px`);
   await sp.getByRole('tab',{name:'Last night'}).click();
   const stack=sp.locator('.stage-stack').first();
   if(await stack.count()){const box=await stack.boundingBox(),card=await stack.locator('xpath=ancestor::section[1]').boundingBox();
    assert.ok(box.x>=card.x && box.x+box.width<=card.x+card.width+1,`stage totals stay inside their card at ${width}px`)}
   assert.equal(await sp.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`Last night fits ${width}px`);
   await sp.getByRole('tab',{name:'Tonight'}).click();
  }
  assert.deepEqual(await contrastFailures(sp),[],'Tonight contrast');
  await sp.close();
 }
 // Training's week lanes: one per goal kind against the goal and the usual week, labelled.
 {
  const tp=await browser.newPage({viewport:{width:390,height:844}});
  tp.on('pageerror',e=>errors.push(e.message));
  await tp.goto(`${url}/#training`);
  const card=tp.locator('.week-lanes');
  await card.waitFor();
  const zone=card.getByRole('img',{name:/^Zone 2:/});
  assert.equal(await zone.getAttribute('aria-label'),'Zone 2: 30 min so far of a 150 goal; usual week by now 65');
  assert.equal(await card.getByRole('img',{name:/^Strength: 2 sessions so far of a 4 goal/}).count(),1);
  assert.ok((await card.textContent()).includes('middle half of 8 recent complete weeks'));
  assert.equal(await card.locator('svg path[opacity=".13"]').count(),2,'a usual-week band per lane');
  for(const width of [320,700]){await tp.setViewportSize({width,height:900});assert.equal(await tp.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`lanes fit ${width}px`)}
  assert.deepEqual(await contrastFailures(tp),[],'Lanes contrast');
  await tp.close();
 }
 console.log('Training week lanes: goal, usual-week band and labels: passed');
 // Training sections: Fitness's three questions, Strength's muscles with gaps, Activities in Garmin's words.
 {
  const ts=await browser.newPage({viewport:{width:390,height:844}});
  ts.on('pageerror',e=>errors.push(e.message));
  await ts.goto(`${url}/#training`);
  await ts.getByRole('tablist',{name:'Training section'}).waitFor();
  for(const q of ['Enough, or too much?','Is my fitness improving?','What am I ready for?'])assert.equal(await ts.getByText(q,{exact:true}).count(),1,q);
  assert.equal(await ts.locator('.semi').count(),0,'no full-width load gauge');
  assert.ok((await ts.locator('.q-load .lbar').getAttribute('aria-label')).includes("Garmin's optimal range 420–760"));
  assert.equal(await ts.locator('.gear').count(),0,'gear lives in Profile');
  await ts.getByRole('tab',{name:'Strength'}).click();
  const mus=ts.locator('.q-muscles');
  await mus.getByRole('list',{name:'Sets per muscle group'}).waitFor();
  assert.ok((await mus.textContent()).includes('Every group had more than 2 sets'),'no gaps over 30 days in the sample');
  await mus.getByRole('group',{name:'Muscle range'}).getByRole('button',{name:'7 days'}).click();
  await ts.waitForFunction(()=>document.querySelector('.q-muscles')?.textContent.includes('Few or no sets since'));
  assert.ok(await mus.locator('li.gap').count()>0,'gaps are marked');
  await ts.getByRole('tab',{name:'Activities'}).click();
  const acts=ts.locator('.q-activities');
  assert.ok((await acts.textContent()).includes('Aerobic 3.8 · Improving'),'Garmin’s training effect label');
  assert.ok((await acts.textContent()).includes('Primary benefit · Tempo'));
  // The chosen section holds across a detail and Back.
  await acts.locator('.activity').first().click();await ts.waitForURL(/#workout\//);
  await ts.goBack();await ts.waitForURL(/#training$/);
  assert.equal(await ts.getByRole('tab',{name:'Activities'}).getAttribute('aria-selected'),'true');
  for(const width of [320,700,1100]){await ts.setViewportSize({width,height:900});assert.equal(await ts.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`Training fits ${width}px`)}
  assert.deepEqual(await contrastFailures(ts),[],'Training sections contrast');
  await ts.close();
 }
 console.log('Training sections: three questions, muscles with gaps, activity contributions, section kept: passed');
 // "?" help: short sheets on root pages and cards; Back and Escape close them before navigating.
 {
  const hp=await browser.newPage({viewport:{width:390,height:844}});
  hp.on('pageerror',e=>errors.push(e.message));
  const seen=new Set();
  for(const [route,count] of [['today',1],['plan',2],['training',2],['health',2],['progress',2],['sleep/tonight',1],['profile',1],['workout/9000000000001',1]]){
   await hp.goto(`${url}/#${route}`);
   await hp.locator('h1').waitFor();
   const buttons=hp.getByRole('button',{name:/^Help: /});
   await buttons.first().waitFor();
   assert.ok(await buttons.count()>=count,`${route} has ${count}+ help buttons`);
   for(const name of await buttons.evaluateAll(bs=>bs.map(b=>b.getAttribute('aria-label'))))seen.add(name);
  }
  assert.ok(seen.size>=10,'help covers many topics: '+[...seen].join(', '));
  await hp.goto(`${url}/#training`);
  await hp.getByRole('button',{name:'Help: On track this week?'}).click();
  const sheet=hp.getByRole('dialog',{name:'On track this week?'});
  await sheet.waitFor();
  assert.ok((await sheet.textContent()).includes('usual week'));
  assert.equal(await hp.evaluate(()=>window.__raringBack()),true,'Back closes the sheet');
  await sheet.waitFor({state:'detached'});
  assert.equal(await hp.evaluate(()=>location.hash),'#training');
  await hp.getByRole('button',{name:'Help: Acute load'}).click();
  await hp.keyboard.press('Escape');
  assert.equal(await hp.getByRole('dialog').count(),0);
  await hp.getByRole('button',{name:'Help: Acute load'}).click();
  assert.deepEqual(await contrastFailures(hp),[],'Help sheet contrast');
  await hp.getByRole('dialog').getByRole('button',{name:'All help'}).click();
  await hp.waitForURL('**/#help');
  await hp.close();
 }
 console.log('Help: ? sheets on root pages and cards, Back/Escape close, link to all help: passed');
 // Stage-5 journeys: activity rows name average HR (U05), unsupported readiness says why (U07),
 // Fitness Age is Garmin's dated estimate (U48), and Settings says Raring doesn't learn from use (U54).
 {
  const sp=await browser.newPage({viewport:{width:390,height:844}});
  sp.on('pageerror',e=>errors.push(e.message));
  const sample=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
  await sp.goto(`${url}/#today`);
  assert.ok((await sp.locator('.card').filter({hasText:'Recorded today'}).textContent()).includes('avg HR 152'),'U05');
  const rp=await browser.newPage({viewport:{width:390,height:844}});
  await rp.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify({...sample,readiness:null,missingMetricReasons:{readiness:'This watch can’t record Training Readiness.'}})}`}));
  await rp.goto(`${url}/#readiness`);
  await rp.locator('.detail-gauge').getByText('This watch can’t record Training Readiness.',{exact:true}).waitFor();
  await rp.close();
  await sp.goto(`${url}/#health`);
  assert.ok((await sp.locator('.age-source').textContent()).startsWith('Garmin estimate · '),'U48');
  await sp.goto(`${url}/#settings`);
  await sp.getByText(/Raring doesn’t learn from how you use the app/).waitFor();
  await sp.close();
 }
 console.log('Stage-5 U05/U07/U48/U54 content: passed');
 // U58: a link this version can't open explains itself on Today; U46: tonight's need says when it was read;
 // U17: bests are in available history; U41: the migrated observation offers Review timing.
 {
  const lk=await browser.newPage({viewport:{width:390,height:844}});
  lk.on('pageerror',e=>errors.push(e.message));
  await lk.goto(`${url}/#no-such-page`);
  await lk.getByRole('status').filter({hasText:'That link isn’t in this version of Raring, so Today opened.'}).waitFor();
  await lk.getByRole('button',{name:'Dismiss'}).click();
  assert.equal(await lk.getByText(/That link isn’t in this version/).count(),0);
  await lk.goto(`${url}/#sleep/tonight`);
  assert.ok((await lk.locator('.tonight-plan').textContent()).includes('Garmin · read '),'U46');
  await lk.goto(`${url}/#strength`);
  await lk.getByText('in available history',{exact:true}).first().waitFor();
  await lk.close();
  // The sample module is cached once loaded, so the observation case uses a fresh page.
  const ob=await browser.newPage({viewport:{width:390,height:844}});
  const board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
  board.explainObservation=true;
  await ob.route(u=>u.pathname==='/src/lib/sample/board.json',route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(board)}`}));
  await ob.goto(`${url}/#plan`);
  await ob.getByRole('button',{name:'Review timing'}).click();
  await ob.waitForURL('**/#sleep/tonight');
  await ob.close();
 }
 console.log('U58/U46/U17/U41: invalid link, need read time, bests in available history, Review timing: passed');
 // U36: in the evening a session still planned today is a quiet line, not a prompt.
 {
  const ev=await browser.newPage({viewport:{width:390,height:844}});
  ev.on('pageerror',e=>errors.push(e.message));
  const sample=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
  const board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
  board.items.push({...board.items[1],intention:{...board.items[1].intention,id:'i-eve',date:'2025-10-16',time:'19:30',name:'Evening run',sport:'running'},state:'planned'});
  const evening={...sample,call:null,focus:{...sample.focus,phase:'evening',effort:false},lead:{id:'evening:2025-10-17',kind:'sleep',headline:'Make room for sleep',line:'Wind down from 22:00',action:{label:'Tonight’s plan',route:'sleep/tonight',adapt:false},dismissable:false,kept:false,garmin:null,activity:null,intention:null,reasons:[]}};
  await ev.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(evening)}`}));
  await ev.route(u=>u.pathname==='/src/lib/sample/board.json',route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(board)}`}));
  await ev.goto(`${url}/#today`);
  await ev.getByRole('heading',{name:'Make room for sleep'}).waitFor();
  await ev.getByRole('button',{name:/^Evening run 19:30 · still in your plan/}).waitFor();
  const hero=await ev.locator('.hero').textContent();
  assert.ok(!/missed|Hurry/i.test(hero),'no pressure in the evening');
  await ev.close();
 }
 console.log('U36: an evening session stays as a quiet still-in-your-plan line: passed');
 // U59: larger text keeps every root page inside the phone width; reduced motion stops animation.
 {
  const big=await browser.newPage({viewport:{width:390,height:844}});
  big.on('pageerror',e=>errors.push(e.message));
  await big.emulateMedia({reducedMotion:'reduce'});
  for(const route of ['today','plan','training','health','progress','sleep/tonight','profile']){
   await big.goto(`${url}/#${route}`);
   await big.locator('h1').waitFor();
   await big.addStyleTag({content:'html{zoom:1.3}'});
   await big.waitForTimeout(150);
   const overflow=await big.evaluate(()=>document.documentElement.scrollWidth-innerWidth);
   assert.ok(overflow<=2,`${route} at 130% text: ${overflow}px horizontal overflow`);
   const running=await big.evaluate(()=>document.getAnimations().filter(a=>a.playState==='running'&&(a.effect?.getTiming().iterations===Infinity)).length);
   assert.equal(running,0,`${route}: no endless animation with reduced motion`);
  }
  await big.close();
 }
 console.log('U59: 130% text without horizontal overflow and reduced motion on root pages: passed');
 // U31/U32: under your limits Garmin's workouts need review (contents unknown); Garmin's schedule has a read-only detail.
 {
  const lp=await browser.newPage({viewport:{width:390,height:844}});
  lp.on('pageerror',e=>errors.push(e.message));
  const sample=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
  const board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
  const lift={id:'coach:2025-10-18:1',date:'2025-10-18',name:'Coach strength',kind:'strength',durationMin:40,effect:null,restDay:false,garminStatus:'Not complete'};
  board.coach={name:'Synthetic Coach',state:'available',sessions:[lift]};
  board.items.push({intention:{id:lift.id,date:lift.date,time:null,sport:'strength',typeKey:null,name:lift.name,durationMin:40,allocation:{strengthSessions:1,zone2Min:0,highIntensityMin:0,intensityMin:0,sleepNights:0},kind:'workout',garminId:lift.id,status:'open',history:[]},state:'planned',recording:null,matchedBy:null,candidates:[],coach:lift});
  const limited={...sample,adapt:{rest:false,pause:null,limits:{excluded:['strength'],note:'No deep knee bends'},label:'Limits on',reviewDue:false,blocked:false,routines:[]}};
  await lp.route(todaySample,route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(limited)}`}));
  await lp.route(u=>u.pathname==='/src/lib/sample/board.json',route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(board)}`}));
  await lp.goto(`${url}/#today`);
  await lp.locator('.top .adapt').click();
  await lp.getByRole('dialog',{name:'Adapt today'}).getByRole('button',{name:/^My routines/}).click();
  const routines=lp.getByRole('dialog',{name:'My routines'});
  const rt=await routines.textContent();
  assert.ok(rt.includes('Easy Run') && rt.includes('Contents unknown to Raring') && rt.includes('Review needed'),'U31: '+rt);
  assert.ok(!/Compatible|Safe/.test(rt));
  await lp.keyboard.press('Escape');
  await lp.getByRole('navigation').getByRole('button',{name:'Plan',exact:true}).click();
  await lp.getByRole('button',{name:/^Friday 17 Oct/}).click();
  await lp.getByRole('button',{name:/Easy Run/}).click();
  const g=lp.getByRole('dialog',{name:'Easy Run'});
  const gt=await g.textContent();
  assert.ok(gt.includes('read-only') && gt.includes('Review needed') && gt.includes('Contents unknown to Raring') && gt.includes('completion flag isn’t read'),'U32: '+gt);
  assert.ok(!/Removed from Garmin|Compatible|Safe/.test(gt));
  assert.deepEqual(await contrastFailures(lp),[],'Garmin sheet contrast');
  await lp.keyboard.press('Escape');
  await lp.getByRole('button',{name:/^Saturday 18 Oct/}).click();
  await lp.locator('.days').getByRole('button',{name:/Coach strength/}).click();
  await lp.getByRole('dialog',{name:'Coach strength'}).getByText('Conflicts with your excluded activity.').waitFor();
  await lp.close();
 }
 console.log('U31/U32: Garmin workouts need review under limits; scheduled workouts have a read-only detail; Coach conflict named: passed');

 // Lifts: one Max weight / Volume choice and the shared range picker for every lift chart.
 {
  const lp=await browser.newPage({viewport:{width:390,height:844}});
  lp.on('pageerror',e=>errors.push(e.message));
  await lp.goto(`${url}/#strength`);
  const chart=lp.getByRole('img',{name:/^Heaviest set each session, \d+ sessions/}).first();
  await chart.waitFor();
  const before=await chart.getAttribute('aria-label');
  await lp.getByRole('group',{name:'Lift measure'}).getByRole('button',{name:'Volume'}).click();
  const vol=lp.getByRole('img',{name:/^Volume each session/}).first();
  await vol.waitFor();
  assert.notEqual(await vol.getAttribute('aria-label'),before.replace('Heaviest set','Volume'),'volume plots different values');
  assert.ok((await lp.locator('.strength-history').textContent()).includes('weight × reps over weighted sets'));
  assert.ok((await lp.locator('.strength-history').textContent()).includes('No estimated one-rep max'));
  const picker=lp.getByRole('group',{name:'Lift range'});
  for(const label of ['7D','14D','30D','90D','1Y','Max'])assert.equal(await picker.getByRole('button',{name:new RegExp(label==='Max'?'All history':label.replace('D',' days').replace('1Y','1 year'))}).count(),1,`${label} offered`);
  await picker.getByRole('button',{name:'7 days'}).click();
  await lp.waitForFunction(()=>[...document.querySelectorAll('.strength-history p.dim')].some(p=>/Fewer than two sessions|No sessions in this range/.test(p.textContent))||document.querySelector('svg.trend'));
  // The choice holds when switching exercise.
  const chips=lp.getByRole('group',{name:'Exercise'}).getByRole('button');
  if(await chips.count()>1){await chips.nth(1).click();assert.equal(await lp.getByRole('group',{name:'Lift measure'}).getByRole('button',{name:'Volume'}).getAttribute('aria-pressed'),'true')}
  assert.deepEqual(await contrastFailures(lp),[],'Lift controls contrast');
  await lp.close();
 }
 console.log('Lifts: Max weight / Volume, shared range picker, excluded sets note: passed');
 // Workout detail: stored streams with heart rate on one axis, effects in plain words, only recorded fields.
 {
  const wp=await browser.newPage({viewport:{width:390,height:844}});
  wp.on('pageerror',e=>errors.push(e.message));
  await wp.goto(`${url}/#workout/9000000000001`);
  const along=wp.getByRole('region',{name:'Along the way'});
  await along.waitFor();
  for(const name of [/^Heart rate over/,/^Pace over .*minutes per km/,/^Elevation over/,/^Cadence over .*spm/])assert.equal(await along.getByRole('img',{name}).count(),1,String(name));
  assert.equal(await along.getByRole('img',{name:/^Power over/}).count(),0,'no power without a power meter');
  const did=await wp.getByRole('region',{name:'What it did'}).textContent();
  for(const want of ['3.8','Improving','builds your aerobic base','Minor benefit','62','14 moderate + 2 × 24 vigorous','-9'])assert.ok(did.includes(want),`${want} in ${did}`);
  const rec=await wp.getByRole('region',{name:'Recorded by your watch'}).textContent();
  for(const want of ['Moving time','Descent41 m','Stride length113.4 cm','Recovery heart rate31 bpm'])assert.ok(rec.includes(want),`${want} in ${rec}`);
  assert.equal(await wp.getByText(/new VO₂ max estimate/).count(),0,'no VO₂ max link without a new estimate');
  const hic=await wp.getByRole('region',{name:'How it counted'}).textContent();
  assert.ok(hic.includes('18 min Zone 2') && hic.includes('100 of 150 this week') && hic.includes('Best in available history'),'U15: '+hic);
  assert.ok(!/Fitness Age will|younger/.test(hic));
  for(const width of [320,700]){await wp.setViewportSize({width,height:900});assert.equal(await wp.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`workout fits ${width}px`)}
  assert.deepEqual(await contrastFailures(wp),[],'Workout detail contrast');
  await wp.goto(`${url}/#workout/9000000000002`);
  await wp.locator('h1').waitFor();
  assert.equal(await wp.getByRole('region',{name:'Along the way'}).count(),0,'strength sessions show sets, not streams');
  assert.ok((await wp.getByRole('region',{name:'How it counted'}).textContent()).includes('2 of 4 strength sessions'),'U15 strength');
  await wp.close();
 }
 console.log('Workout detail: aligned streams, effects in plain words, recorded fields only: passed');
 // An event groups this week's sessions of its sport; each still counts toward weekly goals.
 {
  const ep=await browser.newPage({viewport:{width:390,height:844}});
  ep.on('pageerror',e=>errors.push(e.message));
  const board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
  const run={...board.items[1],state:'planned',intention:{...board.items[1].intention,id:'i-run',date:'2025-10-18',sport:'running',name:'Tempo run'}};
  board.items.push(run);
  board.eventWeek={event:board.events[0],sport:'running',items:['i-run'],recordedMin:52};
  await ep.route(u=>u.pathname==='/src/lib/sample/board.json',route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(board)}`}));
  await ep.goto(`${url}/#plan`);
  const group=ep.getByRole('group',{name:'Harbour 10K this week'});
  await group.waitFor();
  assert.ok((await group.textContent()).includes('Run this week: 52 min recorded'),await group.textContent());
  assert.equal(await group.getByRole('button',{name:/Tempo run/}).count(),1,'the run is grouped under the event');
  assert.equal(await group.getByRole('button',{name:/Easy ride/}).count(),0,'other sports are not');
  assert.ok((await group.textContent()).includes('once per goal'));
  assert.deepEqual(await contrastFailures(ep),[],'Event group contrast');
  await ep.close();
  const plain=await browser.newPage({viewport:{width:390,height:844}});
  await plain.goto(`${url}/#plan`);
  await plain.getByText('Harbour 10K',{exact:true}).first().waitFor();
  assert.equal(await plain.getByRole('group',{name:/this week$/}).count(),0,'without its sessions the event only counts down');
  await plain.close();
 }
 console.log('Event week: sessions of its sport grouped, others not, countdown only otherwise: passed');
 // Garmin Coach: read-only planned work grouped under the plan, rest days shown, Garmin's status never completion.
 {
  const cp=await browser.newPage({viewport:{width:390,height:844}});
  cp.on('pageerror',e=>errors.push(e.message));
  const board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
  const session=(date,n,name,kind,min,extra={})=>({id:`coach:${date}:${n}`,date,name,kind,durationMin:min,effect:null,restDay:false,garminStatus:'Not complete',...extra});
  const lift=session('2025-10-16',1,'Strength','strength',40),run=session('2025-10-18',1,'Threshold','cardio',45,{effect:'Lactate threshold'});
  const rest={...session('2025-10-17',1,'Rest day',null,null),kind:null,restDay:true};
  board.coach={name:'Synthetic Coach',state:'available',sessions:[lift,rest,run]};
  const asItem=(c,state)=>({intention:{id:c.id,date:c.date,time:null,sport:c.kind==='strength'?'strength':'other',typeKey:c.kind==='cardio'?'garmin_coach_cardio':null,name:c.name,durationMin:c.durationMin,
   allocation:{strengthSessions:c.kind==='strength'?1:0,zone2Min:0,highIntensityMin:0,intensityMin:0,sleepNights:0},kind:'workout',garminId:c.id,status:'open',history:[]},state,recording:null,matchedBy:null,candidates:[],coach:c});
  board.items.push(asItem(lift,'planned'),asItem(run,'planned'));
  await cp.route(u=>u.pathname==='/src/lib/sample/board.json',route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(board)}`}));
  await cp.goto(`${url}/#plan`);
  const group=cp.getByRole('group',{name:'Synthetic Coach this week'});
  await group.waitFor();
  assert.ok((await group.textContent()).includes('Garmin Coach · read-only') && (await group.textContent()).includes('0 of 2 done'));
  assert.ok((await group.textContent()).includes('plan no zone or intensity minutes'));
  await cp.getByRole('button',{name:/^Friday 17 Oct/}).click();
  assert.ok((await cp.locator('.days').textContent()).includes('Rest day'),'Garmin Coach rest day is listed');
  await group.getByRole('button',{name:/Threshold/}).click();
  const sheet=cp.getByRole('dialog',{name:'Threshold'});
  await sheet.waitFor();
  const text=await sheet.textContent();
  assert.ok(text.includes('Garmin Coach · read-only') && text.includes('Lactate threshold') && text.includes('Garmin: Not complete') && text.includes('only from a matching recording'),text);
  for(const name of [/^Edit/,/^Move/,/^Let it go/,/^Remove/,/^Send to Garmin/])assert.equal(await sheet.getByRole('button',{name}).count(),0,`no ${name} for Garmin's session`);
  assert.deepEqual(await contrastFailures(cp),[],'Coach contrast');
  await cp.close();
 }
 console.log('Garmin Coach: grouped read-only sessions, rest day, Garmin status not completion: passed');
 // Profile: Garmin's facts with dates, limitations into Adapt's limits, gear, Settings; no Garmin writes.
 {
  const pp=await browser.newPage({viewport:{width:390,height:844}});
  pp.on('pageerror',e=>errors.push(e.message));
  const sample=JSON.parse(await readFile(resolve('src/lib/sample/profile.json'),'utf8'));
  await pp.route(u=>u.pathname==='/src/lib/sample/profile.json',route=>route.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify({...sample,limits:{excluded:['running'],note:'No overhead pressing'},weightFreshness:{...sample.weightFreshness,ageDays:12,stale:true}})}`}));
  await pp.goto(`${url}/#profile`);
  const facts=pp.getByRole('region',{name:'What Garmin knows'});
  await facts.waitFor();
  const text=await facts.textContent();
  for(const want of ['Age46','72.4 kg','Last weigh-in Tue 14 Oct','Metric · km, kg','Last upload'])assert.ok(text.includes(want),`${want} in ${text}`);
  assert.ok(await pp.getByRole('button',{name:'Add a weight'}).isDisabled(),'adding a weight stays disabled');
  assert.ok((await pp.getByRole('region',{name:'Limitations and injuries'}).textContent()).includes('Avoiding run · “No overhead pressing”'));
  await pp.getByRole('button',{name:/^Manage limitations/}).click();
  await pp.getByRole('dialog',{name:'Your limits'}).waitFor();
  await pp.keyboard.press('Escape');
  assert.equal(await pp.locator('.gear').count(),1,'gear lives in Profile');
  for(const width of [320,700]){await pp.setViewportSize({width,height:900});assert.equal(await pp.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`Profile fits ${width}px`)}
  assert.deepEqual(await contrastFailures(pp),[],'Profile contrast');
  await pp.close();
 }
 console.log('Profile: Garmin facts with dates, limitations, gear, disabled weight write: passed');
 console.log('Sleep tabs: closer default, Tonight plan, wind-down self-report, recent nights and widths: passed');

 // Today through the day: the same data at different clocks. Only the leading card changes; the hero,
 // its reconciliation pill and the metric cards stay put, and nothing invites effort after wind-down.
 const base=JSON.parse(await readFile(resolve('src/lib/sample/today.json'),'utf8'));
 base.reconcile=differ.reconcile;
 const night={asleepMin:468,needMin:450,woke:'06:39'};
 const phases=[
  ['wake','07:10',{phase:'wake',cards:['focus','sleepPlan'],until:'2025-10-16T09:39:00',effort:true,planOpen:false},'Ready to push','.plan-line'],
  ['day','14:20',{phase:'day',cards:['now','focus','sleepPlan'],until:'2025-10-16T20:30:00',effort:true,planOpen:false},'Ready to push','.now-card'],
  ['evening','20:45',{phase:'evening',cards:['sleepPlan','focus'],until:'2025-10-16T22:00:00',effort:true,planOpen:true},'Wind down at 22:00','.sleep-plan'],
  ['winddown','22:10',{phase:'windDown',cards:['sleepPlan','focus'],until:'2025-10-16T23:06:00',effort:false,planOpen:true},'Time to wind down','.sleep-plan'],
  ['late','23:40',{phase:'lateNight',cards:['sleepPlan','focus'],until:'2025-10-17T06:36:00',effort:false,planOpen:true},"Rest when you're ready",'.sleep-plan'],
  ['missing','2025-10-17T07:30',{phase:'wake',cards:['focus'],until:'2025-10-17T12:00:00',effort:false,planOpen:false,sleepMissing:true,lastNight:null},"Waiting for last night's sleep",null],
  ['overview','14:20',{phase:'overview',cards:['focus','sleepPlan'],until:null,effort:true,planOpen:true,note:'Fixed layout: your wake times vary too much to plan the day around.'},'Ready to push','.sleep-plan'],
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
  if(leader){
   const first=await phasePage.locator('.dashboard > *').first().evaluate(el=>el.className);
   // The one-line plan is first without taking the lead: the hero leads mornings.
   assert.ok(first.includes(leader.slice(1)) && (leader==='.plan-line' || first.includes('lead')),`${name}: ${leader} leads (${first})`);
  }
  const top=await phasePage.locator('.morning').evaluate(el=>Math.round(el.getBoundingClientRect().top));
  heroTop??=top;
  assert.equal(top,heroTop,`${name}: the scores stay where they are`);
  const pill=phasePage.getByRole('button',{name:differ.reconcile.label});
  await pill.click();
  await phasePage.getByText(differ.reconcile.line).waitFor();
  assert.equal(await phasePage.locator('.hero .effort').count(),focus.effort?1:0,`${name}: effort call ${focus.effort?'shown':'hidden'}`);
  assert.equal(await phasePage.locator('.planned-session').count(),0,`${name}: Garmin's next session isn't repeated on Today`);
  assert.equal(await phasePage.locator('.week-card').count(),['winddown','late'].includes(name)?0:1,`${name}: no week card during sleep phases`);
  const text=await phasePage.locator('.dashboard').textContent();
  // The week card's "left" counts the week's goals, not the time to bed; it is hidden in sleep phases.
  const advice=await phasePage.locator('.dashboard > :not(.week-card)').allTextContents();
  assert.equal(await phasePage.locator('.week-card').count(),['winddown','late'].includes(name)?0:1,`${name}: no goal remainders during sleep phases`);
  assert.ok(!/\bin \d+ ?(min|h)\b|left\b/i.test(advice.join(' ')),`${name}: no countdown`);
  if(name==='evening')assert.equal(await phasePage.locator('.sleep-plan .strip-bar u').count(),1,'the strip marks now against the window');
  if(name==='late')assert.ok(text.includes("Tonight's window was"));
  if(name==='missing'){const all=await phasePage.locator('main').textContent();assert.ok(all.includes('Not synced yet') && !all.includes('holds it back'),'an unsynced night is neutral')}
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
 for(const name of ['today','sleep','training','health','status','workout','profile'])samples[name]=JSON.parse(await readFile(resolve(`src/lib/sample/${name}.json`),'utf8'));
 samples.plan_board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
 samples.progress=JSON.parse(await readFile(resolve('src/lib/sample/progress.json'),'utf8'));
 samples.progress_domains=JSON.parse(await readFile(resolve('src/lib/sample/progressDomains.json'),'utf8'));
 samples.weight_trend=weightFixtures.kg.days30;
 // Progress controls invoke the Rust contract. These responses are synthetic;
 // period arithmetic, evidence coverage and awards are tested in the core.
 {
  const progressPage=await browser.newPage({viewport:{width:390,height:844}});
  await progressPage.addInitScript(samples=>{
   window.isTauri=true;
   window.progressCalls=[];
   window.progressStatus=samples.status;
   window.progressReplies=0;
   window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
    if(command.startsWith('plugin:event|'))return 1;
    if(command==='app_status')return samples.status;
    if(command==='progress'){
     window.progressCalls.push(args);
     const p=structuredClone(samples.progress);
     if(args.period==='year'){p.period='year';p.from='2024-11-01';p.to='2025-10-31';p.calendar=[];p.ringDays=[];}
     if(args.period==='week'){
      p.period='week';p.from='2025-10-13';p.to='2025-10-19';
      if(args.date==='2025-10-12'){p.from='2025-10-06';p.to=p.end='2025-10-12';p.current=false;}
     }
     if(window.holdProgress){window.holdProgress=false;await new Promise(resolve=>window.releaseProgress=resolve);}
     window.progressReplies++;
     return p;
    }
    if(command==='trend')return {range:'days30',bucket:'day',from:'2025-09-17',to:'2025-10-16',series:{metric:'hrv',name:'HRV',unit:'ms',points:[],average:null,days:0},overlay:null,previous:null,change:null,available:[]};
    if(command in samples)return structuredClone(samples[command]);
    throw Error('Unexpected progress test command '+command);
   }};
  },samples);
  await progressPage.goto(`${url}/?fixture=progress#progress`);
  await progressPage.getByRole('heading',{name:'October 2025',exact:true}).waitFor();
  assert.ok((await progressPage.locator('.report').textContent()).includes('Active on 4 of 16 days'));
  assert.ok((await progressPage.locator('.report').textContent()).includes('Intensity: known total from 4 of 16 days'));
  assert.equal(await progressPage.locator('.ring-cal .dc.future').count(),17,'the month’s whole weeks, future days empty');
  assert.equal(await progressPage.locator('.ring-cal .dc.outside').count(),4,'days outside October are faint');
  assert.ok((await progressPage.locator('.ring-cal .dc').filter({hasText:'8'}).first().getAttribute('aria-label')).includes('ill'));
  assert.ok((await progressPage.locator('.ms').first().textContent()).includes('Longest run'));
  await progressPage.getByRole('group',{name:'Period',exact:true}).getByRole('button',{name:'Year',exact:true}).click();
  await progressPage.waitForFunction(()=>window.progressCalls.some(c=>c.period==='year'));
  await progressPage.getByRole('button',{name:'Year',exact:true}).getAttribute('aria-pressed').then(v=>assert.equal(v,'true'));
  assert.equal(await progressPage.locator('.cal, .ring-cal').count(),0);
  await progressPage.getByRole('group',{name:'Period',exact:true}).getByRole('button',{name:'Week',exact:true}).click();
  await progressPage.waitForFunction(()=>window.progressCalls.some(c=>c.period==='week'));
  await progressPage.getByRole('button',{name:'Previous period',exact:true}).click();
  await progressPage.waitForFunction(()=>document.querySelector('.rep-h h2')?.textContent.includes('6 Oct')&&document.querySelector('.report')?.getAttribute('aria-busy')==='false');
  const historicalLabel=await progressPage.locator('.rep-h h2').textContent();
  await progressPage.getByRole('tab',{name:'Long-term'}).click();
  await progressPage.locator('.tiles2 button').filter({hasText:'HRV'}).click();
  await progressPage.waitForFunction(()=>location.hash==='#metric/hrv');
  await progressPage.goBack();
  await progressPage.waitForFunction(()=>document.querySelector('[role=tab][aria-selected=true]')?.textContent==='Long-term');
  await progressPage.getByRole('tab',{name:'Overall'}).click();
  await progressPage.waitForFunction(label=>location.hash==='#progress'&&document.querySelector('.rep-h h2')?.textContent===label,historicalLabel);
  const beforeResume=await progressPage.evaluate(()=>window.progressReplies);
  await progressPage.evaluate(()=>{window.progressStatus.lastSync='2025-10-16T12:34:56Z';window.dispatchEvent(new Event('raring:resume'));});
  await progressPage.waitForFunction(n=>window.progressReplies>n,beforeResume);
  assert.equal(await progressPage.locator('.rep-h h2').textContent(),historicalLabel,'refresh keeps historical period');
  // A held old-period read cannot replace a selection made while it was pending.
  await progressPage.evaluate(()=>{window.holdProgress=true;window.dispatchEvent(new Event('raring:resume'));});
  await progressPage.waitForFunction(()=>typeof window.releaseProgress==='function');
  await progressPage.getByRole('group',{name:'Period',exact:true}).getByRole('button',{name:'Month',exact:true}).click();
  await progressPage.getByRole('heading',{name:'October 2025',exact:true}).waitFor();
  const beforeRelease=await progressPage.evaluate(()=>window.progressReplies);
  await progressPage.evaluate(()=>window.releaseProgress());
  await progressPage.waitForFunction(n=>window.progressReplies>n,beforeRelease);
  assert.equal(await progressPage.locator('.rep-h h2').textContent(),'October 2025','old period read cannot overwrite the selected month');
  await progressPage.close();
 }
 console.log('Progress contract: periods, coverage, calendar, records, Back/resume and stale-read selection passed');
 // Progress sections: Cardio's weekly zone and intensity against goals with VO₂ max, Strength's sessions
 // and bests, Long-term's months; marks and unknown weeks stay visible, never zero.
 {
  const pd=await browser.newPage({viewport:{width:390,height:844}});
  pd.on('pageerror',e=>errors.push(e.message));
  await pd.goto(`${url}/#progress`);
  await pd.getByRole('tablist',{name:'Progress section'}).waitFor();
  await pd.getByRole('tab',{name:'Cardio'}).click();
  const z2=pd.getByRole('region',{name:'Zone 2 minutes per week'});
  await z2.waitFor();
  assert.ok((await z2.textContent()).includes('goal 150'));
  const label=await z2.getByRole('img').getAttribute('aria-label');
  assert.ok(label.includes('unknown'),'an unknown week stays unknown: '+label);
  assert.equal(await pd.getByRole('region',{name:'Intensity minutes per week'}).count(),1);
  assert.equal(await pd.getByRole('region',{name:'Zone 4–5 minutes per week'}).count(),0,'no lane without its goal');
  assert.ok((await pd.getByRole('region',{name:'VO₂ max, same weeks'}).getByRole('img').getAttribute('aria-label')).includes('none'),'weeks without an estimate stay empty');
  await pd.getByRole('tab',{name:'Strength'}).click();
  await pd.getByRole('region',{name:'Strength sessions per week'}).waitFor();
  const bests=pd.getByRole('region',{name:'Bests in available history'});
  assert.ok((await bests.textContent()).includes('Barbell Back Squat') && (await bests.textContent()).includes('105 kg × 3'));
  const squat=pd.getByRole('region',{name:'Barbell Back Squat over the year'});
  await squat.waitFor();
  assert.ok((await squat.getByRole('img').getAttribute('aria-label')).startsWith('Barbell Back Squat: heaviest set per session'));
  assert.equal(await pd.getByRole('region',{name:/over the year$/}).count(),2,'the two heaviest lifts');
  await pd.getByRole('tab',{name:'Overall'}).click();
  await pd.getByRole('button',{name:/^All records and milestones/}).click();
  await pd.waitForURL('**/#progress/milestones');
  await pd.getByRole('heading',{name:'Records and milestones',level:1}).waitFor();
  const months=pd.locator('.archive-month');
  assert.ok(await months.count()>=1,'grouped by month');
  assert.ok(await months.first().locator('.ms').count()>=1);
  await pd.goBack();await pd.waitForURL(/#progress$/);
  await pd.getByRole('tab',{name:'Strength'}).click();
  await pd.getByRole('tab',{name:'Long-term'}).click();
  for(const r of ['Training per month','Resting heart rate by month','Overnight HRV by month','VO₂ max and Fitness Age by month'])await pd.getByRole('region',{name:r}).waitFor();
  assert.ok((await pd.getByRole('region',{name:'Training per month'}).getByRole('img').getAttribute('aria-label')).includes('Nov unknown'),'before history is unknown');
  for(const width of [320,700,1100]){await pd.setViewportSize({width,height:900});assert.equal(await pd.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),true,`Progress fits ${width}px`)}
  assert.deepEqual(await contrastFailures(pd),[],'Progress sections contrast');
  await pd.close();
 }
 console.log('Progress sections: Cardio, Strength and Long-term from the core, unknown weeks kept: passed');
 // U57: a declined permission is remembered; Settings stops offering to ask.
 {
  const np=await browser.newPage({viewport:{width:390,height:844}});
  await np.addInitScript(samples=>{
   window.isTauri=true;window.asked=0;localStorage.setItem('notifications-step-done','1');
   window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command)=>{
    if(command.startsWith('plugin:'))return 1;
    if(command==='request_notification_permission'){window.asked++;return false}
    if(command==='app_status')return structuredClone(samples.status);
    if(command==='get_settings')return {};
    if(command in samples)return structuredClone(samples[command]);
    return null;
   }};
  },samples);
  await np.goto(`${url}/#settings`);
  await np.getByRole('button',{name:'Enable notifications'}).click();
  await np.getByText(/Notifications were declined\. Raring won’t ask again/).waitFor();
  assert.equal(await np.getByRole('button',{name:'Enable notifications'}).count(),0);
  await np.reload();
  await np.getByText(/Notifications were declined/).waitFor();
  assert.equal(await np.evaluate(()=>window.asked),0,'not asked again after a reload');
  await np.close();
 }
 console.log('U57: declined notifications are remembered and not asked again: passed');
 // Plan edits and tonight's wind-down go to the core as local actions.
 {
  const ep=await browser.newPage({viewport:{width:390,height:844}});
  ep.on('pageerror',e=>errors.push(e.message));
  const board=JSON.parse(await readFile(resolve('src/lib/sample/board.json'),'utf8'));
  board.goals.targets={...board.goals.targets,sleepNights:4,sleepTime:'22:30',sleepKind:'windDown'};
  board.ledger.push({kind:'sleepTiming',target:4,original:null,revisedOn:null,recorded:2,planned:0,remaining:2,reached:false,review:0,unavailable:null,observation:false});
  const mock={...samples,plan_board:board,today:{...samples.today,adapt:{rest:false,pause:{id:'p1',since:'2025-10-14',until:null,reason:'ill',review:null},limits:{excluded:[],note:''},label:'Ill',reviewDue:false,blocked:true,routines:[]}}};
  await ep.addInitScript(samples=>{
   window.isTauri=true;window.actions=[];localStorage.setItem('notifications-step-done','1');
   window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
    if(command.startsWith('plugin:'))return 1;
    if(command==='local_action'){window.actions.push(args.action);return null}
    if(command==='app_status')return structuredClone(samples.status);
    if(command==='trend')return {range:'days30',bucket:'day',from:'2025-09-17',to:'2025-10-16',series:{metric:'hrv',name:'HRV',unit:'ms',points:[],average:null,days:0},overlay:null,previous:null,change:null,available:[],freshness:null};
    if(command in samples)return structuredClone(samples[command]);
    if(command==='get_settings')return {};
    return null;
   }};
  },mock);
  await ep.goto(`${url}/#plan`);
  await ep.getByRole('button',{name:/^Friday 17 Oct/}).click();
  await ep.getByRole('button',{name:/Easy ride/}).first().click();
  const sheet=ep.getByRole('dialog',{name:'Easy ride'});
  await sheet.getByRole('button',{name:/^Edit/}).click();
  const form=sheet.getByRole('group',{name:'Edit session'});
  await form.getByLabel('Time').fill('07:15');
  await form.getByLabel('Minutes').fill('50');
  await form.getByRole('button',{name:'More Zone 2'}).click();
  await form.getByRole('button',{name:'Save'}).click();
  await ep.waitForFunction(()=>window.actions.some(a=>a.action==='editIntention'));
  const edit=(await ep.evaluate(()=>window.actions)).find(a=>a.action==='editIntention');
  assert.equal(edit.id,'i1760380000001');
  assert.equal(edit.time,'07:15');assert.equal(edit.durationMin,50);assert.equal(edit.name,'Easy ride');
  assert.ok(edit.allocation.zone2Min>board.items[1].intention.allocation.zone2Min,'the Zone 2 share went up');
  await ep.goto(`${url}/#sleep/tonight`);
  await ep.getByRole('button',{name:'Make tonight one of them'}).click();
  await ep.waitForFunction(()=>window.actions.some(a=>a.action==='addIntention'));
  const wind=(await ep.evaluate(()=>window.actions)).find(a=>a.action==='addIntention');
  assert.deepEqual([wind.kind,wind.date,wind.time,wind.allocation.sleepNights,wind.name],['windDown','2025-10-16','22:30',1,'Wind down 22:30']);

  await ep.locator('.top .adapt').click();
  await ep.getByRole('dialog',{name:'Adapt today'}).getByRole('button',{name:/^Resume training/}).click();
  const resume=ep.getByRole('dialog',{name:'Resume training'});
  await resume.waitFor();
  assert.ok((await resume.textContent()).includes('Strength: 4 sessions') && (await resume.textContent()).includes('Nothing is added to catch up'),await resume.textContent());
  assert.equal((await ep.evaluate(()=>window.actions)).filter(a=>a.action==='resume').length,0,'opening Resume changes nothing');
  await resume.getByRole('button',{name:/^Resume and change goals/}).click();
  await ep.waitForURL('**/#plan/commitments');
  assert.equal((await ep.evaluate(()=>window.actions)).filter(a=>a.action==='resume').length,1);
  await ep.close();
 }
 console.log('Plan: edit time, minutes and goal shares; place tonight’s wind-down; resume shows goals first: passed');
 // U34: a lighter week is one tap from Adapt; the original targets stay on record (core rule).
 {
  const lw=await browser.newPage({viewport:{width:390,height:844}});
  await lw.addInitScript(samples=>{
   window.isTauri=true;window.actions=[];localStorage.setItem('notifications-step-done','1');
   window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
    if(command.startsWith('plugin:'))return 1;
    if(command==='local_action'){window.actions.push(args.action);return null}
    if(command==='app_status')return structuredClone(samples.status);
    if(command==='get_settings')return {};
    if(command in samples)return structuredClone(samples[command]);
    return null;
   }};
  },{...samples,today:{...samples.today,adapt:{rest:false,pause:null,limits:{excluded:[],note:''},label:'Adapt today',reviewDue:false,blocked:false,routines:[]}}});
  await lw.goto(`${url}/#today`);
  await lw.locator('.top .adapt').click();
  const sheet=lw.getByRole('dialog',{name:'Adapt today'});
  assert.ok((await sheet.getByRole('button',{name:/^Lighter week/}).textContent()).includes('Original targets stay on record'));
  await sheet.getByRole('button',{name:/^Lighter week/}).click();
  await lw.waitForFunction(()=>window.actions.some(a=>a.action==='lighterWeek'));
  await lw.close();
 }
 console.log('U34: Lighter week from Adapt: passed');
 // The sample's phase ended long ago; phase expiry has its own check below.
 samples.today.focus.until=null;
 const statusDenied=await browser.newPage({viewport:{width:390,height:844}});
 await statusDenied.addInitScript(samples=>{
  const state=window.testStatus={permission:'denied',failSave:false,saves:0,settings:{notifyMorningReadiness:true,notifyWeeklyCheckin:true,notifyMetricChanges:true,notifyVitals:true,notifyTrainingStatus:false,notifyBedtime:false,notifyStatus:false,statusOfferSeen:false,adaptiveToday:true,bedtimeLeadMin:30}};
  window.isTauri=true;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
   if(command==='app_status')return samples.status;
   if(command==='get_settings')return structuredClone(state.settings);
   if(command==='patch_settings'){state.saves++;if(state.failSave)throw Error('Synthetic snapshot write failure');Object.assign(state.settings,args.patch);return}
   if(command==='request_notification_permission'){
    await new Promise(r=>setTimeout(r,100));
    if(state.permission==='error')throw Error('Synthetic permission failure');
    return state.permission==='granted';
   }
   if(command.startsWith('plugin:event|'))return 1;
   if(command in samples)return structuredClone(samples[command]);
   throw Error('Unexpected status command '+command);
  }};
 },samples);
 await statusDenied.goto(`${url}/?fixture=status-offer#settings`);
 const deniedToggle=statusDenied.getByRole('checkbox',{name:/Persistent status/});
 await deniedToggle.waitFor();
 for(const permission of ['denied','error']){
  await statusDenied.evaluate(p=>window.testStatus.permission=p,permission);
  await deniedToggle.click();
  await statusDenied.getByRole('status').filter({hasText:permission==='denied'?'Notifications are blocked':'Synthetic permission failure'}).waitFor();
  assert.equal(await deniedToggle.isChecked(),false,'denial/exception cannot leave the switch visually on');
  assert.equal(await statusDenied.evaluate(()=>window.testStatus.saves),0,'permission denial never enables persistent status');
 }
 await statusDenied.evaluate(()=>{window.testStatus.permission='granted';window.testStatus.failSave=true});
 await deniedToggle.click();
 await statusDenied.getByRole('status').filter({hasText:'Synthetic snapshot write failure'}).waitFor();
 assert.equal(await deniedToggle.isChecked(),false,'failed publication rolls the switch back');
 assert.equal(await statusDenied.evaluate(()=>window.testStatus.settings.notifyStatus),false);
 await statusDenied.evaluate(()=>window.testStatus.failSave=false);
 await deniedToggle.click();
 await statusDenied.waitForFunction(()=>window.testStatus.settings.notifyStatus);
 assert.equal(await deniedToggle.isChecked(),true);
 await statusDenied.close();
 console.log('Persistent status permission denial/exception leaves the checkbox and setting off: passed');
 // Deferred native calls expose the stale-snapshot race without relying on network timing.
 const patches=await browser.newPage({viewport:{width:390,height:844}});
 patches.on('pageerror',e=>errors.push(e.message));
 await patches.addInitScript(samples=>{
  const stored={notifyMorningReadiness:true,notifyWeeklyCheckin:true,notifyMetricChanges:true,notifyVitals:true,notifyTrainingStatus:false,notifyBedtime:false,notifyStatus:true,statusOfferSeen:true,adaptiveToday:true,bedtimeLeadMin:30,journalEnabled:true};
  const state=window.testPatch={stored,patches:[],pending:[],holdKeys:[],holdReads:false,reads:[],readsCompleted:0,settingsReads:0,failOptIn:false};
  window.isTauri=true;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
   if(command.startsWith('plugin:event|'))return 1;
   if(command==='app_status')return structuredClone(samples.status);
   if(command==='get_settings'){
    state.settingsReads++;
    const snapshot=structuredClone(stored);
    if(state.holdReads){await new Promise(resolve=>state.reads.push(resolve));state.readsCompleted++}
    return snapshot;
   }
   if(command==='patch_settings'){
    state.patches.push(structuredClone(args.patch));
    if(Object.keys(args.patch).some(key=>state.holdKeys.includes(key)))await new Promise(resolve=>state.pending.push(resolve));
    if(state.failOptIn && args.patch.notifyStatus===true){state.failOptIn=false;throw Error('Synthetic opt-in failure')}
    Object.assign(stored,args.patch);return;
   }
   if(command==='request_notification_permission')return true;
   if(command==='health')return {...structuredClone(samples.health),journalEnabled:stored.journalEnabled};
   if(command in samples)return structuredClone(samples[command]);
   throw Error('Unexpected settings command '+command);
  }};
 },samples);
 await patches.goto(`${url}/?fixture=status-offer#settings`);
 const morning=patches.getByRole('checkbox',{name:'Morning readiness'});
 const weekly=patches.getByRole('checkbox',{name:'Weekly check-in'});
 const privacy=patches.getByRole('checkbox',{name:/Persistent status/});
 await morning.waitFor();
 assert.equal(await patches.getByRole('checkbox').count(),10,'journal has its own consent control');
 assert.equal(await patches.locator('.card').filter({has:patches.getByRole('heading',{name:'Notifications',exact:true})}).getByRole('checkbox').count(),7,'journal consent is not a notification switch');
 await patches.evaluate(()=>window.testPatch.holdKeys=['notifyMorningReadiness','notifyStatus']);
 await morning.uncheck();
 await patches.waitForFunction(()=>window.testPatch.pending.length===1);
 assert.equal(await morning.isDisabled(),true,'the same notification switch is locked during its write');
 assert.equal(await privacy.isEnabled(),true,'unrelated privacy switch remains usable');
 await morning.evaluate(el=>el.click());
 assert.equal(await patches.evaluate(()=>window.testPatch.patches.length),1,'rapid same-key click is ignored');
 await privacy.click();
 await patches.waitForFunction(()=>window.testPatch.pending.length===2);
 assert.equal(await privacy.isDisabled(),true,'rapid privacy toggle is locked during its write');
 await privacy.evaluate(el=>el.click());
 assert.equal(await patches.evaluate(()=>window.testPatch.patches.length),2);
 await patches.evaluate(()=>window.testPatch.pending.pop()());
 await patches.waitForFunction(()=>window.testPatch.stored.notifyStatus===false);
 await patches.evaluate(()=>window.testPatch.pending.shift()());
 await patches.waitForFunction(()=>window.testPatch.stored.notifyMorningReadiness===false);
 assert.equal(await privacy.isChecked(),false,'late unrelated write cannot turn status back on');
 assert.deepEqual(await patches.evaluate(()=>window.testPatch.patches.slice(0,2)),[{notifyMorningReadiness:false},{notifyStatus:false,statusOfferSeen:true}]);
 await patches.evaluate(()=>{window.testPatch.holdKeys=[];window.testPatch.failOptIn=true});
 await privacy.click();
 await patches.getByRole('status').filter({hasText:'Synthetic opt-in failure'}).waitFor();
 assert.equal(await privacy.isChecked(),false,'failed opt-in rolls back only its own keys');
 await weekly.uncheck();
 await patches.waitForFunction(()=>window.testPatch.stored.notifyWeeklyCheckin===false);
 assert.equal(await privacy.isChecked(),false,'unrelated patch after failed opt-in cannot revive consent');
 assert.equal(await patches.evaluate(()=>window.testPatch.stored.notifyStatus),false);
 await patches.getByRole('checkbox',{name:'Bedtime'}).check();
 await patches.waitForFunction(()=>window.testPatch.stored.notifyBedtime===true);
 await patches.getByLabel('Wind-down reminder').selectOption('45');
 await patches.waitForFunction(()=>window.testPatch.stored.bedtimeLeadMin===45);
 assert.deepEqual(await patches.evaluate(()=>window.testPatch.patches.slice(-2)),[{notifyBedtime:true},{bedtimeLeadMin:45}]);
 await privacy.click();
 await patches.waitForFunction(()=>window.testPatch.stored.notifyStatus===true);
 await patches.evaluate(()=>{window.testPatch.holdReads=true;window.dispatchEvent(new Event('raring:resume'))});
 await patches.waitForFunction(()=>window.testPatch.reads.length===1);
 await privacy.click();
 await patches.waitForFunction(()=>window.testPatch.stored.notifyStatus===false);
 await patches.evaluate(()=>window.testPatch.reads.shift()());
 await patches.waitForFunction(()=>window.testPatch.readsCompleted===1);
 assert.equal(await privacy.isChecked(),false,'stale full read cannot visually re-enable status');
 assert.equal(await patches.evaluate(()=>window.testPatch.stored.notifyStatus),false);
 assert.ok((await patches.evaluate(()=>window.testPatch.patches)).every(p=>Object.keys(p).length<=2),'no settings snapshot is ever submitted');
 const journalToggle=patches.getByRole('checkbox',{name:/Allow Garmin journal reads/});
 await patches.evaluate(()=>{window.testPatch.holdReads=false;window.testPatch.holdKeys=['notifyMorningReadiness','journalEnabled']});
 await morning.check();
 await patches.waitForFunction(()=>window.testPatch.pending.length===1);
 await journalToggle.uncheck();
 await patches.waitForFunction(()=>window.testPatch.pending.length===2);
 assert.equal(await journalToggle.isDisabled(),true);
 await journalToggle.evaluate(el=>el.click());
 await patches.evaluate(()=>window.testPatch.pending.pop()());
 await patches.waitForFunction(()=>window.testPatch.stored.journalEnabled===false);
 await patches.evaluate(()=>window.testPatch.pending.shift()());
 await patches.waitForFunction(()=>window.testPatch.stored.notifyMorningReadiness===true);
 assert.equal(await journalToggle.isChecked(),false,'late unrelated save cannot restore journal consent');
 assert.equal(await patches.evaluate(()=>window.testPatch.stored.journalEnabled),false);
 assert.deepEqual(await patches.evaluate(()=>window.testPatch.patches.slice(-2)),[{notifyMorningReadiness:true},{journalEnabled:false}]);
 await patches.getByRole('navigation').getByRole('button',{name:'Health',exact:true}).click();
 const allowJournal=patches.getByRole('button',{name:'Allow local journal reads',exact:true});
 await allowJournal.waitFor();
 const settingsReads=await patches.evaluate(()=>window.testPatch.settingsReads);
 await allowJournal.click();
 await patches.waitForFunction(()=>window.testPatch.pending.length===1);
 assert.equal(await allowJournal.isDisabled(),true,'Health allow uses the same per-field write guard');
 await patches.evaluate(()=>window.testPatch.pending.shift()());
 await patches.getByRole('button',{name:'Open journal',exact:true}).waitFor();
 assert.deepEqual(await patches.evaluate(()=>window.testPatch.patches.at(-1)),{journalEnabled:true},'Health allow sends only consent');
 assert.equal(await patches.evaluate(()=>window.testPatch.settingsReads),settingsReads,'Health allow never reads/resubmits whole Settings');
 await patches.close();
 const marker=await browser.newPage({viewport:{width:390,height:844}});
 marker.on('pageerror',e=>errors.push(e.message));
 await marker.addInitScript(samples=>{
  const stored={notifyMorningReadiness:true,notifyWeeklyCheckin:true,notifyMetricChanges:true,notifyVitals:true,notifyTrainingStatus:false,notifyBedtime:false,notifyStatus:false,statusOfferSeen:false,adaptiveToday:true,bedtimeLeadMin:30};
  const state=window.testMarker={stored,patches:[]};
  window.isTauri=true;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
   if(command.startsWith('plugin:event|'))return 1;
   if(command==='app_status')return structuredClone(samples.status);
   if(command==='get_settings')return structuredClone(stored);
   if(command==='patch_settings'){state.patches.push(structuredClone(args.patch));Object.assign(stored,args.patch);return}
   if(command in samples)return structuredClone(samples[command]);
   throw Error('Unexpected offer command '+command);
  }};
 },samples);
 await marker.goto(`${url}/?fixture=status-offer#today`);
 await marker.getByRole('button',{name:'Enable persistent status'}).waitFor();
 assert.deepEqual(await marker.evaluate(()=>window.testMarker.patches),[{statusOfferSeen:true}],'offer writes only its marker');
 assert.equal(await marker.evaluate(()=>window.testMarker.stored.notifyStatus),false);
 await marker.close();
 console.log('Settings patches: deferred writes, failed opt-in, stale reads and offer marker: passed');
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
 const privacySummary=onboarding.locator('summary');
 await privacySummary.focus();await privacySummary.press('Enter');
 assert.ok(await onboarding.locator('details').getAttribute('open')!==null,'privacy is keyboard accessible');
 await onboarding.getByText(/Sign-in tokens are encrypted/).waitFor();
 await privacySummary.press('Enter');
 await onboarding.getByRole('button',{name:'Sign in with Garmin'}).click();
 await onboarding.getByRole('alert').getByText('Error: Synthetic sign-in failure',{exact:true}).waitFor();
 await onboarding.evaluate(()=>{window.testOnboarding.failSignIn=false});
 await onboarding.getByRole('button',{name:'Sign in with Garmin'}).click();
 await onboarding.getByRole('heading',{name:'Stay in the loop'}).waitFor();
 await onboarding.getByRole('button',{name:'Not now',exact:true}).click();
 await onboarding.waitForURL('**/#today');await onboarding.locator('.morning').waitFor();
 assert.equal(await onboarding.evaluate(()=>window.testOnboarding.signIns),2,'sign-in button invokes the native command');
 assert.equal(await onboarding.evaluate(()=>window.testOnboarding.permissions),0,'notification permission is optional');
 await openSettings(onboarding);
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
     return {range:q.range,bucket:'day',from:points[0].start,to:end,series,overlay:null,previous:null,change:null,available:[q.metric],freshness:(q.end??samples.today.date)!==samples.today.date?null:samples.freshness?.[q.metric]??null};
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
 await live.waitForFunction(()=>window.testBackend.syncs===1 && document.querySelector('.ready-fact'));
 await live.evaluate(()=>{window.testBackend.samples.today.readiness.value=64});
 await live.clock.fastForward(30_000);
 await live.waitForFunction(()=>document.querySelector('.ready-fact .gauge-center b')?.textContent==='64');
 assert.equal(await live.evaluate(()=>window.testBackend.syncs),1,'cache poll does not sync every 30 seconds');
 await live.locator('.ready-fact').click();
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
 await openSleep(live);
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
 await live.getByRole('button',{name:'View Resting HR',exact:true}).click();
 await live.getByText('Your resting heart rate from Garmin. The shaded band shows your usual range, not a medical target.',{exact:true}).waitFor();
 assert.equal(await live.getByText('Garmin’s average heart rate during sleep, compared with its Health Status baseline. This is not your resting heart rate or a medical target.',{exact:true}).count(),0,'fallback resting HR must not claim to be Sleep HR');
 await live.getByRole('navigation').getByRole('button',{name:'Health',exact:true}).click();
 await live.getByText('1 reading in range',{exact:true}).waitFor();
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
 await openSleep(live);
 await live.locator('.sleep-hero').waitFor();
 assert.equal(await live.locator('.sleep-plan').count(),0,'no inferred sleep plan without Garmin need');
 await live.getByRole('navigation').getByRole('button',{name:'Today',exact:true}).click();
 await live.locator('.morning').waitFor();
 assert.equal(await live.locator('.sleep-plan, .plan-line').count(),0);
 console.log('Partial baselines, outliers, missing history and unavailable sleep plans: passed');
 await live.evaluate(()=>{Object.assign(window.testBackend.samples.training.acute,{label:'',tone:'neutral',band:null})});
 await live.getByRole('navigation').getByRole('button',{name:'Training',exact:true}).click();
 if(await live.getByRole('tab',{name:'Fitness'}).getAttribute('aria-selected')!=='true')await live.getByRole('tab',{name:'Fitness'}).click();
 const load=live.locator('.q-load .load');
 await load.waitFor();
 assert.equal(await load.locator('b').textContent(),'612');
 assert.equal(await load.getByText('Acute load · Garmin',{exact:true}).count(),1);
 assert.equal((await load.locator('div').first().locator('span').textContent()).trim(),'','missing ranges do not imply a load assessment');
 assert.equal(await load.getByText(/^Optimal/).count(),0,'no optimal range is drawn without Garmin\'s');
 assert.equal(await live.locator('.q-load .lbar .band').count(),0);
 await live.screenshot({path:resolve(output,'training-no-range.png')});
 console.log('Training load without an optimal range: passed');
 // A daily metric whose newest reading is older than two days says so on its chart.
 await live.evaluate(()=>{window.testBackend.samples.freshness={restingHr:{recordedOn:'2025-10-12',ageDays:4,stale:true,thresholdDays:2},hrv:{recordedOn:'2025-10-16',ageDays:0,stale:false,thresholdDays:2}};window.dispatchEvent(new Event('raring:resume'))});
 await live.getByRole('navigation').getByRole('button',{name:'Health',exact:true}).click();
 await live.locator('.over-time').first().getByRole('note').waitFor();
 assert.equal(await live.locator('.over-time').first().getByRole('note').textContent(),'Last reading Sun 12 Oct · 4 days ago');
 assert.equal(await live.getByRole('note').filter({hasText:/^Last reading/}).count(),1,'only the stale metric is flagged');
 console.log('Health: stale daily readings are flagged on their chart: passed');
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
 await live.getByRole('tab',{name:'Activities'}).click();
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
 await openSleep(live);
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
     return {range:q.range,bucket:'day',from:points[0].start,to:end,series,overlay:null,previous:null,change:null,available:[q.metric],freshness:(q.end??samples.today.date)!==samples.today.date?null:samples.freshness?.[q.metric]??null};
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
 await cached.locator('.morning').waitFor();
 await cached.waitForFunction(()=>['sleep','training','health'].every(key=>window.testViews.completed[key]>=1));
 const statusReads=await cached.evaluate(()=>window.testViews.statusReads);
 const todayOffset=await cached.locator('.scroll').evaluate(el=>{el.scrollTop=350;return el.scrollTop});
 assert.ok(todayOffset>100,'Today can be scrolled for restoration');
 await hold('sleep');
 await cached.evaluate(()=>document.querySelector('.morning .sleep-fact').click());
 await pending('sleep');
 assert.equal(await cached.locator('.sleep-hero').isVisible(),true,'prefetched Sleep appears before navigation read finishes');
 assert.equal(await cached.locator('.empty').count(),0,'cached tab switches without loading state');
 assert.equal(await cached.evaluate(()=>window.testViews.statusReads),statusReads,'navigation does not query app status');
 await release('sleep');
 await cached.locator('.scroll').evaluate(el=>{el.scrollTop=120});
 await hold('today');
 await tabCached('Today');
 await pending('today');
 assert.equal(await cached.locator('.morning').isVisible(),true,'cached Today is visible before its read finishes');
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
 assert.notEqual(await cached.locator('.hero h2').textContent(),'After touch cancel','view swap is deferred while touching');
 await cached.locator('main').evaluate(el=>el.dispatchEvent(new TouchEvent('touchcancel',{bubbles:true})));
 await cached.getByRole('heading',{name:'After touch cancel'}).waitFor();
 await cached.evaluate(()=>{window.testViews.samples.today.headline='After hide'});
 await hold('today');await touch();
 await cached.evaluate(()=>window.dispatchEvent(new Event('raring:resume')));
 await pending('today');await release('today');
 await cached.waitForFunction(()=>window.testViews.completed.today>=4);
 assert.notEqual(await cached.locator('.hero h2').textContent(),'After hide','hidden-page swap is deferred while touching');
 await cached.evaluate(()=>{Object.defineProperty(document,'visibilityState',{configurable:true,value:'hidden'});document.dispatchEvent(new Event('visibilitychange'));delete document.visibilityState});
 await cached.getByRole('heading',{name:'After hide'}).waitFor();

 // A cached Today whose phase has ended is read again rather than shown, so its leading card can't swap after appearing.
 await cached.evaluate(()=>{Object.assign(window.testViews.samples.today,{headline:'Old phase'});window.testViews.samples.today.focus.until='2000-01-01T00:00:00';window.dispatchEvent(new Event('raring:resume'))});
 await cached.getByRole('heading',{name:'Old phase'}).waitFor();
 await cached.evaluate(()=>document.querySelector('.morning .sleep-fact').click());await cached.locator('.sleep-hero').waitFor();
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
 assert.equal(await cached.locator('.hero h2').textContent(),'After sync','old in-flight result cannot replace synced view');
 const previousSleep=await cached.evaluate(()=>window.testViews.completed.sleep);
 await cached.evaluate(()=>{const s=window.testViews.samples;s.today.headline='Background sync';s.sleep.score.value=51;s.status.lastSync='2026-10-16T14:02:00Z';window.dispatchEvent(new Event('raring:resume'))});
 await cached.getByRole('heading',{name:'Background sync'}).waitFor();
 await cached.waitForFunction(previous=>window.testViews.completed.sleep>previous,previousSleep);
 await hold('sleep');await cached.evaluate(()=>document.querySelector('.morning .sleep-fact').click());await pending('sleep');
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
 assert.equal(await cached.locator('.hero h2').textContent(),'Next day','previous-day result cannot replace next day');
 await hold('sleep');await cached.evaluate(()=>document.querySelector('.morning .sleep-fact').click());await pending('sleep');
 assert.equal(await cached.locator('.sleep-hero .gauge-center b').textContent(),'42','next-day prefetch is visible before navigation read');
 await release('sleep');
 await tabCached('Today');
 await cached.locator('.morning').waitFor();
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
 await cached.evaluate(()=>document.querySelector('.morning .sleep-fact').click());
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
 await cached.locator('.morning').waitFor();
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
  const state=window.testLayout={stored,saves:[],patches:[],failNext:false};
  const focus=()=>stored.adaptiveToday
   ?{...samples.today.focus,phase:'day',cards:['now','focus','sleepPlan'],planOpen:false}
   :{...samples.today.focus,phase:'overview',cards:['focus','sleepPlan'],planOpen:true};
  window.isTauri=true;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
   if(command.startsWith('plugin:event|'))return 1;
   if(command==='sync_now')return;
   if(command==='app_status')return structuredClone(samples.status);
   if(command==='get_settings')return structuredClone(stored);
   if(command==='patch_settings'){
    state.patches.push(structuredClone(args.patch));
    await new Promise(resolve=>state.saves.push(resolve));
    if(state.failNext){state.failNext=false;throw Error('Synthetic save failure')}
    Object.assign(stored,args.patch);return;
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
 await layout.waitForFunction(()=>!document.querySelector('.now-card') && document.querySelector('.dashboard > *')?.className.includes('sleep-plan'),null,{timeout:5000});
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
 assert.ok((await leader()).includes('sleep-plan') && await layout.locator('.now-card').count()===0,'Today keeps the stored fixed layout');
 // A later change is accepted after the pending save has finished.
 await layout.evaluate(()=>{location.hash='#settings'});
 await followSwitch().waitFor();
 await followSwitch().check();
 await layout.waitForFunction(()=>window.testLayout.saves.length===1);
 assert.equal(await followSwitch().isDisabled(),true);
 await saved();
 await layout.waitForFunction(()=>window.testLayout.stored.adaptiveToday && ![...document.querySelectorAll('label.setting')].find(l=>l.textContent.includes('Follow my day')).querySelector('input').disabled);
 assert.equal(await followSwitch().isChecked(),true,'the switch and store agree on the last accepted change');
 assert.deepEqual(await layout.evaluate(()=>window.testLayout.patches),[{adaptiveToday:false},{adaptiveToday:true},{adaptiveToday:true}],'layout sends only its changed field');
 await layout.close();
 console.log('Follow my day: slow save, rapid clicks, navigation, failure and retry: passed');
 for(const width of [390,700,1100]){
  const hr=await browser.newPage({viewport:{width,height:844}});
  await hr.goto(`${url}/?fixture=heart-rate#health`);
  await hr.getByRole('button',{name:'Heart rate through today'}).waitFor();
  await hr.screenshot({path:resolve(output,`heart-rate-health-${width}.png`)});
  await hr.getByRole('button',{name:'Heart rate through today'}).click();
  await hr.getByRole('slider',{name:/Heart rate through the day/}).waitFor();
  assert.equal(await hr.getByText('Garmin daily resting HR',{exact:false}).count(),1);
  assert.ok(await hr.getByText('Asleep',{exact:true}).count());
  await hr.screenshot({path:resolve(output,`heart-rate-detail-${width}.png`)});
  await hr.getByRole('button',{name:'Previous'}).click();
  await hr.waitForFunction(()=>document.querySelector('.metric-hero')?.textContent?.includes('15 Oct'));
  assert.ok(await hr.evaluate(()=>document.documentElement.scrollWidth<=innerWidth),`heart-rate detail overflow at ${width}`);
  await hr.close();
 }
 console.log('All-day heart rate at phone, unfolded and desktop widths: passed');
 const hrSwitch=await browser.newPage({viewport:{width:390,height:844}});
 hrSwitch.on('pageerror',e=>errors.push(e.message));
 await hrSwitch.addInitScript(samples=>{
  localStorage.setItem('notifications-step-done','1');
  window.isTauri=true;
  window.__TAURI_INTERNALS__={transformCallback:()=>1,invoke:async(command,args)=>{
   if(command.startsWith('plugin:event|'))return 1;
   if(command==='app_status')return {...samples.status,signedIn:true,lastSync:new Date().toISOString()};
   if(command==='heart_rate_day'){
    const day=structuredClone(samples.health.heartRateDay);day.date=args.date;
    if(args.date===samples.health.date){day.timeline=Array(120).fill(70);day.timeline[118]=null;day.timeline[119]=90}
    return day;
   }
   if(command in samples)return structuredClone(samples[command]);
   throw Error('Unexpected HR command '+command);
  }};
 },samples);
 await hrSwitch.goto(`${url}/#health`);
 await hrSwitch.getByRole('button',{name:'Heart rate through today'}).click();
 await hrSwitch.getByRole('button',{name:'Previous'}).click();
 const hrSlider=hrSwitch.getByRole('slider',{name:/Heart rate through the day/});
 await hrSlider.focus();await hrSwitch.keyboard.press('End');
 assert.equal(await hrSlider.getAttribute('aria-valuenow'),'719','late old-day sample selected');
 await hrSwitch.getByRole('button',{name:'Next'}).click();
 await hrSwitch.waitForFunction(()=>document.querySelector('[role="slider"]')?.getAttribute('aria-valuemax')==='119');
 assert.equal(await hrSlider.getAttribute('aria-valuenow'),'119','new day resets to its real latest reading');
 assert.match(await hrSlider.getAttribute('aria-valuetext'),/90 bpm/);
 await hrSlider.focus();await hrSwitch.keyboard.press('ArrowLeft');
 assert.equal(await hrSlider.getAttribute('aria-valuenow'),'118');
 assert.match(await hrSlider.getAttribute('aria-valuetext'),/no reading/,'known gap stays a gap');
 const trace=hrSwitch.locator('[role="slider"] path[fill]');
 assert.equal(await trace.first().getAttribute('fill'),'#E4E8F0','ungraded HR uses neutral ink');
 await hrSwitch.close();
 console.log('Heart-rate date change resets selection; gaps and ungraded palette: passed');
 assert.deepEqual(errors,[],'no browser errors');
} finally {
 await browser?.close();
 await server.close();
}
