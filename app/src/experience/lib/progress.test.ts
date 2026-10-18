import {describe, expect, it} from 'vitest';
import {derive, type State} from './rules';
import {summary, calendarMonth, covered, strengthWeeks, bounds, totals, trendTile, TREND_TILES} from './progress';
import trendSparkSource from '../components/TrendSpark.svelte?raw';
import {setsFor,stagesFor,nightFor,DAYS} from './world';
import {toMin} from './dates';
import {todayView} from './views';
// Browser state defaults without importing the Svelte runtime singleton.
const base = (): State => ({profile:'mixed',day:'2026-11-18',clock:'day',offline:false,follow:true,irregular:false,rest:null,pause:null,pauses:[],limits:{exclude:[],note:'',routines:{}},rehab:{items:[],done:{}},intentions:[],dismissed:{},journal:{consent:false,open:false,downloaded:false},winddown:{},revisions:[],ownCommitments:null,ambitionsAdded:[],celebrated:{},seen:{},quiet:false,vitalsException:false,extraStrength:false,napCase:'same',calendar:'complete',notifyBedtime:false,permission:'not-asked',channels:{morning:true,weekly:true,metrics:true,vitals:true,training:false,status:false},signedOut:false,dst:false,returning:false,noRunning:false,savedSleepTarget:false,winddownIntention:false,navVariant:'tabs',reviewDraft:null});
describe('prototype evidence and period rules',()=>{
 it('compares the same elapsed days and suppresses incomplete comparison',()=>{
  const c=derive(base()), s=summary(c,'month',c.calendarDate);
  expect([s.prevFrom,s.prevEnd]).toEqual(['2026-10-01','2026-10-18']);
  expect(s.prev).toBeNull(); // 1 November belongs to the incomplete October week.
  const complete=derive({...base(),profile:'simple'});
  expect(summary(complete,'month',complete.calendarDate).prev).not.toBeNull();
  const sparse=derive({...base(),profile:'sparse'});
  expect(summary(sparse,'month',sparse.calendarDate).partial).toBe(true);
  expect(summary(sparse,'month',sparse.calendarDate).prev).toBeNull();
 });
 it('keeps known activity marks separate from coverage',()=>{
  const c=derive(base()), october=calendarMonth(c,'2026-10-01');
  const marked=october.days.find(d=>d.date==='2026-10-26')!;
  expect(marked.acts.length).toBeGreaterThan(0);
  expect(marked.covered).toBe(false);
  expect(october.days.filter(d=>d.covered).length).toBe(25);
  expect(covered(c,'2026-11-19')).toBe(false);
 });
 it('never includes earlier-week sets in a month',()=>{
  const c=derive(base()), weeks=strengthWeeks(c,'2026-11-01','2026-11-18');
  expect(weeks[0].sets).toBe(0);
  const exact=c.world.activities.filter(a=>a.date>='2026-11-01'&&a.date<='2026-11-18').reduce((n,a)=>n+(setsFor(a)??[]).reduce((t,e)=>t+e.sets.length,0),0);
  expect(weeks.reduce((n,w)=>n+w.sets,0)).toBe(exact);
 });
 it('only counts covered active days in the covered denominator',()=>{
  const c=derive(base()),t=totals(c,'2026-10-01','2026-10-31');
  expect(t.coveredDays).toBe(25);
  expect(t.activeDays).toBeLessThanOrEqual(t.coveredDays);
 });
 it('clips current periods and reserves future chart days',()=>{
  const p=bounds('month','2026-11-18','2026-11-18');
  expect(p.end).toBe('2026-11-18');expect(p.to).toBe('2026-11-30');
 });
 it('suppresses effort in the canonical readiness adapter during rest, pauses and limits',()=>{
  for(const patch of [{rest:{day:'2026-11-20'}},{pause:{since:'2026-11-19',review:null}},{limits:{exclude:[],note:'Avoid deep knee bends',routines:{}}}]){
   const c=derive({...base(),day:'2026-11-20',...patch});
   expect(c.readiness?.value).toBe(84);expect(todayView(c).call).toBeNull();
  }
 });
 it('keeps rest on the actual day through midnight and nightly self-report identity',()=>{
  const s={...base(),rest:{day:'2026-11-18'}};
  expect(derive({...s,clock:'late'}).restActive).toBe(true);
  expect(derive({...s,day:'2026-11-19',clock:'morning'}).restActive).toBe(false);
  expect(derive({...s,clock:'late',follow:false}).tonight.nightOf).toBe('2026-11-18');
  expect(derive({...s,clock:'late'}).bodyBattery).toBeNull();
 });
 it('retains a whole-week pause after it ended',()=>{
  const c=derive({...base(),day:'2026-10-25',pauses:[{since:'2026-10-19',until:'2026-10-25',review:null}]});
  expect(c.plan?.wholePause).toBe(true);
  expect(c.plan?.tracks.every(t=>t.state==='paused')).toBe(true);
 });
 it('never exposes unsynced or unfinished nights',()=>{
  expect(derive({...base(),clock:'waiting'}).nightOf('2026-11-18')).toBeNull();
  expect(derive({...base(),clock:'late'}).nightOf('2026-11-19')).toBeNull();
 });
 it('bounds scalar tiles to the last watch upload',()=>{
  const c=derive({...base(),profile:'sparse'});
  const tile=TREND_TILES.find(t=>t.id==='bodyBattery')!;
  const v=trendTile(c,tile,'2026-11-16','2026-11-18','2026-11-09','2026-11-11');
  expect(v.count).toBe(1);expect(v.latestDate).toBe('2026-11-16');expect(v.change).toBeNull();
 });
 it('does not compare an older estimate with itself',()=>{
  const c=derive(base()),tile=TREND_TILES.find(t=>t.id==='vo2max')!;
  const v=trendTile(c,tile,'2026-11-16','2026-11-18','2026-11-09','2026-11-11');
  expect(v.count).toBe(0);expect(v.value).toBe(47);expect(v.latestDate).toBe('2026-11-10');expect(v.change).toBeNull();
 });
 it('suppresses effort for sport exclusions without a movement note',()=>{
  const c=derive({...base(),day:'2026-11-20',limits:{exclude:['running'],note:'',routines:{}}});
  expect(c.lead.kind).toBe('review');expect(todayView(c).call).toBeNull();
 });
 it('counts only this period in strength bars',()=>{
  const c=derive(base());expect(strengthWeeks(c,'2026-11-16','2026-11-18').reduce((n,w)=>n+w.sessions,0)).toBe(1);
 });
 it('keeps synthetic stages consistent with sleep duration and endpoints',()=>{
  for(const day of DAYS)for(const profile of [false,true]){
   const n=nightFor(day,profile)!;const stages=stagesFor(day,n.asleep);
   expect(stages.filter(s=>s.stage!=='awake').reduce((t,s)=>t+s.minutes,0)).toBe(n.asleep);
   expect(stages.at(-1)!.startMin+stages.at(-1)!.minutes).toBe((toMin(n.wake)-toMin(n.bed)+1440)%1440);
  }
 });
 it('applies a nap adjustment only after the recorded nap arrives',()=>{
  const morning=derive({...base(),profile:'sleep',clock:'morning'});
  const afternoon=derive({...base(),profile:'sleep',clock:'day'});
  expect(morning.tonight.need).toBe(490);expect(afternoon.tonight.need).toBe(505);
  expect(afternoon.tonight.savedNeed).toBe(morning.tonight.savedNeed);
 });
 it('does not bridge gaps in prototype trend fills',()=>{
  expect(trendSparkSource).toContain('{#each runs as run}');
 });
});
