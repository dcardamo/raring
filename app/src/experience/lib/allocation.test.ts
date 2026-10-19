import {describe, expect, it} from 'vitest';
import {derive, type Intention, type State} from './rules';
import {allocations, dayEvidence, intentionState, movedIntention, recordedFor} from './allocation';
import {itemMatch, localItem, planItems} from './match';
// Browser state defaults without importing the Svelte runtime singleton. No
// running, so Garmin's scheduled runs don't add planned minutes unless a test asks.
const base = (): State => ({profile:'mixed',day:'2026-11-18',clock:'day',offline:false,follow:true,irregular:false,rest:null,pause:null,pauses:[],limits:{exclude:[],note:'',routines:{}},rehab:{items:[],done:{}},intentions:[],dismissed:{},journal:{consent:false,open:false,downloaded:false},winddown:{},revisions:[],ownCommitments:null,ambitionsAdded:[],celebrated:{},seen:{},quiet:false,vitalsException:false,extraStrength:false,napCase:'same',calendar:'complete',notifyBedtime:false,permission:'not-asked',channels:{morning:true,weekly:true,metrics:true,vitals:true,training:false,status:false},signedOut:false,dst:false,returning:false,noRunning:true,savedSleepTarget:false,winddownIntention:false,navVariant:'tabs',reviewDraft:null});
const goals = (t: Partial<{strength: number, zone2: number, hi: number, im: number, sleep: number, sleepScore: number}>) => [{from: '2026-11-16', targets: {strength: 0, zone2: 0, hi: 0, im: 0, sleep: 0, sleepScore: 0, sleepTime: '22:15', sleepKind: 'winddown' as const, ...t}}];
const plan = (o: Partial<Intention>): Intention => ({id: 'x', sport: 'cycling', date: '2026-11-19', time: null, link: null, history: [], ...o});
const row = (s: State, key: string) => allocations(derive(s)).find(r => r.key === key)!;
describe('weekly goal counting', () => {
 it('counts a recording and its linked intention once', () => {
  const c = derive(base()), a = c.world.activities.find(x => x.date === c.calendarDate)!;
  const linked = {...base(), goalVersions: goals({im: 150}), intentions: [plan({sport: a.sport, date: a.date, link: a.id, allocation: {im: 30}})]};
  const r = row(linked, 'im');
  expect(r.recorded).toBe(recordedFor(c.world.activities.filter(x => x.date >= c.weekFrom && x.date <= c.calendarDate), 'im'));
  expect(r.planned).toBe(0);
  expect(r.remaining).toBe(150 - r.recorded!);
 });
 it('auto-links an unambiguous same-day, same-sport match and counts it once', () => {
  const c = derive(base()), a = c.world.activities.find(x => x.date < c.calendarDate && x.date >= c.weekFrom && x.sport !== 'running')!;
  const p = plan({sport: a.sport, date: a.date, allocation: {im: 30}});
  const s = {...base(), goalVersions: goals({im: 150}), intentions: [p]};
  expect(intentionState(derive(s), p)).toBe('recorded');
  expect(itemMatch(derive(s), localItem(p))).toMatchObject({auto: true, activity: {id: a.id}});
  const recorded = recordedFor(c.world.activities.filter(x => x.date >= c.weekFrom && x.date <= c.calendarDate), 'im');
  expect(row(s, 'im')).toMatchObject({ambiguous: false, planned: 0, recorded});
 });
 it('asks only when ambiguous, and after an unlink', () => {
  const c = derive(base()), a = c.world.activities.find(x => x.date < c.calendarDate && x.date >= c.weekFrom && x.sport !== 'running')!;
  const two = [plan({id: 'p1', sport: a.sport, date: a.date, allocation: {im: 30}}), plan({id: 'p2', sport: a.sport, date: a.date, allocation: {im: 20}})];
  const s = {...base(), goalVersions: goals({im: 150}), intentions: two};
  expect(two.map(p => intentionState(derive(s), p))).toEqual(['review', 'review']);
  const recorded = recordedFor(c.world.activities.filter(x => x.date >= c.weekFrom && x.date <= c.calendarDate), 'im');
  expect(row(s, 'im')).toMatchObject({ambiguous: true, planned: 0, recorded, remaining: Math.max(0, 150 - recorded)});
  const one = plan({sport: a.sport, date: a.date});
  expect(intentionState(derive({...base(), intentions: [one], unlinked: {x: true}}), one)).toBe('review');
  const linked = {...one, link: a.id};
  expect(intentionState(derive({...base(), intentions: [linked]}), linked)).toBe('recorded');
 });
 it('asks about a different sport only for one similar endurance recording on a passed day', () => {
  const st = {...base(), noRunning: false, day: '2026-11-19'};
  const c = derive(st), run = c.world.activities.find(x => x.date === '2026-11-17')!;
  expect(c.world.activities.filter(x => x.date === '2026-11-17').map(x => x.sport)).toEqual(['running']);
  const ride = plan({sport: 'cycling', date: '2026-11-17', durationMin: run.min});
  expect(intentionState(derive({...st, intentions: [ride]}), ride)).toBe('review');
  // An everyday walk is never taken for a planned strength session or run.
  const walkDay = c.world.activities.find(x => x.date === '2026-11-18' && x.sport === 'walking')!;
  const lift = plan({sport: 'strength', date: walkDay.date});
  expect(intentionState(derive({...st, intentions: [lift]}), lift)).toBe('not-recorded');
 });
 it('treats a Garmin scheduled session as planned work, then done by the same rule', () => {
  const s = {...base(), noRunning: false, goalVersions: goals({hi: 30, im: 150})};
  const c = derive({...s, day: '2026-11-17'});
  expect(row({...s, day: '2026-11-17'}, 'hi').planned).toBe(18);
  const g = planItems(c, '2026-11-18', '2026-11-18').find(p => p.source === 'garmin')!;
  expect(itemMatch(c, g).state).toBe('planned');
 });
 it('never infers Zone 2 from duration', () => {
  const s = {...base(), goalVersions: goals({zone2: 120}), intentions: [plan({durationMin: 90})]};
  expect(row(s, 'zone2').planned).toBe(0);
  expect(row({...s, intentions: [plan({durationMin: 90, allocation: {zone2: 40}})]}, 'zone2').planned).toBe(40);
  const c = derive(base()), acts = c.world.activities.slice(0, 5);
  expect(recordedFor(acts, 'zone2')).toBe(Math.floor(acts.reduce((n, a) => n + (a.zones[1] ?? 0), 0) / 60));
 });
 it('keeps unknown apart from zero', () => {
  const sparse = {...base(), profile: 'sparse' as const, goalVersions: goals({strength: 2})};
  const r = row(sparse, 'strength');
  expect(r.complete).toBe(false);
  expect(r.remaining).toBeNull();
  const day = dayEvidence(derive(sparse), '2026-11-17');
  expect(day.unknown).toBe(true);
  expect(dayEvidence(derive(sparse), '2026-11-20')).toMatchObject({future: true, unknown: false});
 });
 it('says Not recorded until the person marks a session missed', () => {
  const c = derive(base()), past = plan({sport: 'swimming', date: '2026-11-17'});
  expect(intentionState(c, past)).toBe('not-recorded');
  expect(intentionState(c, {...past, status: 'missed'})).toBe('missed');
  expect(intentionState(c, {...past, status: 'released'})).toBe('released');
 });
 it('reschedules without debt and keeps the original date in history', () => {
  const moved = movedIntention(plan({sport: 'swimming', date: '2026-11-17', allocation: {im: 20}}), '2026-11-20', '2026-11-18T09:00');
  expect(moved).toMatchObject({date: '2026-11-20', status: 'open', link: null});
  expect(moved.history.at(-1)![1]).toContain('17 Nov');
  expect(row({...base(), goalVersions: goals({im: 150}), intentions: [moved]}, 'im').planned).toBe(20);
 });
});
