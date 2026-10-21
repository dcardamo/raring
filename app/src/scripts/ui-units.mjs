// Distances on Training, its tolerance detail, Progress and Profile's gear follow
// Garmin's measurement setting, run from ui-regression.mjs.
import assert from 'node:assert/strict';
import {readFile} from 'node:fs/promises';
import {resolve} from 'node:path';

const load=name=>readFile(resolve(`src/lib/sample/${name}.json`),'utf8').then(JSON.parse);
const KM_PER_MI=1.609344;
// The expected text, worked out here rather than with the app's formatter.
const trim=(v,d)=>v.toFixed(d).replace(/\.?0+$/,'');

export async function unitChecks({browser,url,errors}){
 const samples={training:await load('training'),progress:await load('progress'),profile:await load('profile')};
 for(const [unit,gearUnit] of [['km','km'],['mi','mi'],['km','m']]){
  const statute=unit==='mi',inUnit=km=>statute?km/KM_PER_MI:km;
  const gear={...samples.training.gear,unit:gearUnit};
  const views={
   training:{...samples.training,distanceUnit:unit,gear},
   progress:{...samples.progress,distanceUnit:unit,milestones:samples.progress.milestones.map(m=>m.distanceKm!=null?{...m,distanceUnit:unit}:m)},
   profile:{...samples.profile,gear},
  };
  const page=await browser.newPage({viewport:{width:390,height:844}});
  page.on('pageerror',e=>errors.push(e.message));
  for(const [name,view] of Object.entries(views))
   await page.route(u=>u.pathname===`/src/lib/sample/${name}.json`,r=>r.fulfill({contentType:'text/javascript',body:`export default ${JSON.stringify(view)}`}));
  const tolerance=samples.training.runningToleranceKm;
  await page.goto(`${url}/#training`);
  const tile=page.locator('.metric-link').filter({hasText:'Run tolerance'});
  assert.equal(await tile.locator('b').textContent(),`${trim(inUnit(tolerance),2)} ${unit}`,'Training tile');
  await page.goto(`${url}/#metric/runningTolerance`);
  await page.locator('.metric-value').waitFor();
  assert.equal((await page.locator('.metric-value').textContent()).replace(/\s+/g,''),`${trim(inUnit(tolerance),2)}${unit}`,'tolerance detail reads like the tile');
  await page.goto(`${url}/#progress`);
  const totals=page.locator('.totals');await totals.waitFor();
  const distance=totals.locator('div').filter({hasText:'Distance'});
  assert.equal((await distance.locator('b').textContent()).replace(/\s+/g,' ').trim(),`${trim(inUnit(samples.progress.totals.distanceKm),1)}${unit}`,'Progress distance total');
  const longest=samples.progress.milestones.find(m=>m.kind==='longest');
  assert.match(await page.locator('.ms').filter({hasText:longest.title}).textContent(),new RegExp(`${trim(inUnit(longest.distanceKm),2).replace('.','\\.')} ${unit} ·`),'longest milestone');
  await page.goto(`${url}/#profile`);
  const card=page.locator('.gear-card');await card.waitFor();
  const shoes=gear.items.find(i=>i.thresholdKm!=null);
  const gearText=km=>gearUnit==='m'?`${Math.round(km*1000)} m`:`${trim(inUnit(km),2)} ${gearUnit}`;
  assert.match(await card.textContent(),new RegExp(`${gearText(shoes.distanceKm)}of ${gearText(shoes.thresholdKm)} Garmin threshold`),`gear in ${gearUnit}`);
  await page.close();
 }
 console.log('Distances in km, mi and metres on Training, tolerance, Progress and gear: passed');
}
