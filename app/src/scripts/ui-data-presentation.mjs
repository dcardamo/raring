import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';
import {contrastFailures} from './contrast.mjs';

export async function dataPresentationChecks(browser,url,output){
 const page=await browser.newPage({viewport:{width:390,height:844},reducedMotion:'reduce'});
 const errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.clock.setFixedTime(new Date('2025-10-16T14:05:00'));
 try{
  await page.goto(url+'/#metric/hrv');
  const history=page.locator('.history');
  await history.locator('.metric-summary b').waitFor();
  assert.match(await history.locator('.period-summary').textContent(),/Recorded-day average.*56.*28 recorded nights/);
  const mean=await history.locator('.metric-summary b').textContent();
  await history.getByRole('slider').fill('9');
  assert.equal(await history.locator('.history-reading b').textContent(),'—','the selected gap is unknown');
  assert.equal(await history.locator('.metric-summary b').textContent(),mean,'selecting a day does not change the period mean');
  await history.getByRole('button',{name:'Previous',exact:true}).click();
  await history.locator('.summary-comparison').waitFor();
  assert.equal(await history.locator('.metric-summary').count(),3,'period, prior period and change are distinct');
  assert.match(await history.locator('.period-summary').textContent(),/Previous period average.*recorded nights.*Change in average.*Compared with/);
  assert.equal(await history.locator('.period-averages').count(),0,'no duplicate footer comparison');
  assert.match(await history.locator('.history-legend').textContent(),/48–62 ms/);
  await history.scrollIntoViewIfNeeded();await page.screenshot({path:resolve(output,'presentation-period-comparison.png')});

  for(const route of ['metric/stress','metric/bodyBattery','sleep/last','readiness']){
   await page.goto(url+'/#'+route);
   const h=page.locator('.history').first();await h.locator('.period-summary').waitFor();
   assert.deepEqual(await h.locator('.chart-surface svg text').allTextContents(),['0','50','100'],route+' has a stable 0–100 scale');
  }

  await page.goto(url+'/#workout/9000000000001');
  const zones=page.getByRole('group',{name:'Garmin heart-rate zones'});
  await zones.waitFor();assert.equal(await zones.locator('.amount-row').count(),5);
  assert.match(await zones.textContent(),/Zone 5.*167\+ bpm.*3:29/);
  assert.ok(!(await zones.textContent()).includes('%'),'durations, not an unexplained denominator');
  const starts=await zones.locator('.rail').evaluateAll(els=>els.map(e=>e.getBoundingClientRect().left));
  assert.ok(starts.every(x=>Math.abs(x-starts[0])<.5),'each zone has the same zero origin');

  const workout=JSON.parse(await readFile(resolve('src/lib/sample/workout.json'),'utf8'));
  const workoutMatch=u=>u.pathname==='/src/lib/sample/workout.json';
  await page.route(workoutMatch,r=>r.fulfill({contentType:'text/javascript',body:'export default '+JSON.stringify({...workout,summary:{...workout.summary,zoneSeconds:[0,null,60,120,0]}})}));
  await page.goto('about:blank');await page.goto(url+'/#workout/9000000000001');
  await zones.waitFor();
  const unknown=zones.locator('.amount-row').filter({hasText:'Zone 2'});
  assert.match(await unknown.textContent(),/Not recorded/);
  assert.equal(await unknown.locator('.amount').count(),0);
  const zero=zones.locator('.amount-row').filter({hasText:'Zone 1'});
  assert.match(await zero.textContent(),/0:00/);
  assert.equal(await zero.locator('.amount').count(),0);
  await page.unroute(workoutMatch);

  await page.goto(url+'/#progress');await page.getByRole('tab',{name:'Long-term',exact:true}).click();
  const resting=page.getByRole('region',{name:'Resting heart rate by month'});
  await resting.waitFor();await resting.getByRole('button',{name:'Details'}).click();
  await page.waitForURL('**/#metric/heartRate');
  assert.match(await page.locator('.metric-hero').textContent(),/Garmin daily resting HR/);

  const sample=JSON.parse(await readFile(resolve('src/lib/sample/sleep.json'),'utf8'));
  const match=u=>u.pathname==='/src/lib/sample/sleep.json';
  await page.route(match,r=>r.fulfill({contentType:'text/javascript',body:'export default '+JSON.stringify({...sample,asleepMin:null,totals:[
   {stage:'awake',minutes:0,usualDeltaMin:-10,comparisonMin:10,comparisonNights:2},
   {stage:'deep',minutes:96,usualDeltaMin:null,comparisonMin:null,comparisonNights:0}
  ]})}));
  await page.goto('about:blank');await page.goto(url+'/#sleep/last');
  const stages=page.getByRole('group',{name:'Garmin sleep stage durations'});
  await stages.waitFor();
  assert.equal(await stages.locator('.amount-row').count(),2,'unrecorded stages are not fabricated');
  assert.equal(await stages.locator('.amount-row').first().locator('.amount').count(),0,'known zero has no minimum fill');
  assert.match(await stages.locator('.amount-row').first().textContent(),/Awake.*0:00.*vs your 30-night average \(2 nights\) · 0:10/);
  assert.equal(await stages.locator('.amount-row').last().locator('.comparison').count(),0,'no comparator without evidence');
  assert.equal(await page.locator('.needbar i').count(),0,'missing sleep duration never fills a zero bar');
  assert.deepEqual(await contrastFailures(page),[],'sleep stage evidence text contrast');
  await page.unroute(match);
  assert.deepEqual(errors,[]);
  console.log('Data presentation: period vs selected evidence, bounded domains, stage coverage/zero/unknown, aligned zones and resting-HR route: passed');
 }finally{await page.close()}
}
