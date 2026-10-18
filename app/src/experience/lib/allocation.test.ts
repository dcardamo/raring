import {describe, expect, it} from 'vitest';
import {derive, type Intention, type State} from './rules';
import {allocations, dayEvidence, intentionState, movedIntention, recordedFor} from './allocation';
// Browser state defaults without importing the Svelte runtime singleton.
const base = (): State => ({profile:'mixed',day:'2026-11-18',clock:'day',offline:false,follow:true,irregular:false,rest:null,pause:null,pauses:[],limits:{exclude:[],note:'',routines:{}},rehab:{items:[],done:{}},intentions:[],dismissed:{},journal:{consent:false,open:false,downloaded:false},winddown:{},revisions:[],ownCommitments:null,ambitionsAdded:[],celebrated:{},seen:{},quiet:false,vitalsException:false,extraStrength:false,napCase:'same',calendar:'complete',notifyBedtime:false,permission:'not-asked',channels:{morning:true,weekly:true,metrics:true,vitals:true,training:false,status:false},signedOut:false,dst:false,returning:false,noRunning:false,savedSleepTarget:false,winddownIntention:false,navVariant:'tabs',reviewDraft:null});
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
 it('never counts a possible match as planned or as a second recording', () => {
  const c = derive(base()), a = c.world.activities.find(x => x.date === c.calendarDate)!;
  const r = row({...base(), goalVersions: goals({im: 150}), intentions: [plan({sport: a.sport, date: a.date, allocation: {im: 30}})]}, 'im');
  expect(r.ambiguous).toBe(true);
  expect(r.review).toHaveLength(1);
  expect(r.planned).toBe(0);
  expect(r.remaining).toBe(Math.max(0, 150 - r.recorded!));
 });
 it('flags a past intention with a same-day same-sport recording for review', () => {
  const c = derive(base()), a = c.world.activities.find(x => x.date < c.calendarDate && x.date >= c.weekFrom)!;
  const past = plan({sport: a.sport, date: a.date, allocation: {im: 30}});
  const s = {...base(), goalVersions: goals({im: 150}), intentions: [past]};
  expect(intentionState(derive(s), past)).toBe('review');
  const r = row(s, 'im');
  const recorded = recordedFor(c.world.activities.filter(x => x.date >= c.weekFrom && x.date <= c.calendarDate), 'im');
  expect(r).toMatchObject({ambiguous: true, planned: 0, recorded, remaining: Math.max(0, 150 - recorded)});
  const linked = {...past, link: a.id};
  expect(intentionState(derive({...s, intentions: [linked]}), linked)).toBe('recorded');
  expect(row({...s, intentions: [linked]}, 'im')).toMatchObject({ambiguous: false, planned: 0});
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
