// Prototype actions: every user choice the journeys exercise. Local only; nothing
// is sent to Garmin, and offline changes apply immediately.
import {store, setCaseHook, fresh, LEGACY} from './store.svelte';
import {derive, type State, type GoalTargets, type Intention} from './rules';
import {candidates, movedIntention} from './allocation';
import {CASES} from './cases';
import {dayLabel, weekId} from './dates';
import {SPORTS, type Sport} from './sports';

let nextAllocation = 0;
const S = () => store.state;
const ctx = () => derive(store.state);
const now = () => dayLabel(ctx().calendarDate);
const endPause = (st: State): Partial<State> => ({pause: null, pauses: [...(st.pauses || []), {...st.pause!, until: derive(st).calendarDate}]});

export const A = {
 saveGoals(targets: GoalTargets) {
  const c=ctx();
  store.set(st=>({goalVersions:[...(st.goalVersions??[]),{from:c.calendarDate,targets}],
   ...(c.plan ? {revisions:[...st.revisions,{week:c.plan.wk,from:c.calendarDate,strength:targets.strength,im:targets.im}]} : {ownCommitments:{from:weekId(c.calendarDate),strength:targets.strength,im:targets.im,note:'Goals chosen by you'}})}),'Weekly goals saved');
 },
 allocate(draft: Omit<Intention,'id'|'link'|'history'>, close = true) {
  const id='n'+Date.now().toString(36)+(++nextAllocation).toString(36);
  store.set(st=>({intentions:[...st.intentions,{...draft,id,link:null,history:[[now(),`Placed on ${dayLabel(draft.date)}${draft.garminId?' · local allocation for a Garmin session':''}`]]}],...(draft.date===ctx().calendarDate && draft.kind!=='sleep'?{todayChoice:{day:draft.date,id}}:{})}),'Added to your plan');
  if(close) store.closeSheet();
 },
 selectToday(id: string) { store.set({todayChoice:{day:ctx().calendarDate,id}},'Your choice for today'); store.closeSheet(); },
 keepGarmin(id: string) { store.set(st=>({keptGarmin:{...(st.keptGarmin??{}),[id]:true},todayChoice:{day:ctx().calendarDate,id}}),'Kept in today’s agenda · Garmin unchanged'); },
 finishIntention(id: string,status: 'released'|'missed') {
  store.set(st=>({intentions:st.intentions.map(i=>i.id===id?{...i,status,link:null,history:[...i.history,[now(),status==='released'?'Let go by you':'Marked missed by you'] as [string,string]]}:i)}),status==='released'?'Let go':'Marked missed');
 },
 reschedule(id:string,date:string) {store.set(st=>({intentions:st.intentions.map(i=>i.id===id?movedIntention(i,date,now()):i)}),'Rescheduled locally');store.closeSheet();},
 dismiss(id: string) { store.set(s => ({dismissed: {...s.dismissed, [id]: true}}), 'Hidden for today'); store.closeSheet(); },
 rest() { store.set({rest: {day: ctx().calendarDate}}, 'Rest day, through tonight'); store.closeSheet(); },
 unrest() { store.set({rest: null}, 'Rest removed'); store.closeSheet(); },
 pause(review: string | null = null) { store.set({pause: {since: ctx().calendarDate, review}}, 'Training paused until you resume'); },
 resume() { store.set(endPause, 'Training resumed · commitments as they were'); store.closeSheet(); },
 resumeChange() { store.set(endPause, 'Training resumed'); store.sheetThen(() => store.navigate('plan/commitments')); },
 exclude(s: Sport) { store.set(st => ({limits: {...st.limits, exclude: st.limits.exclude.includes(s) ? st.limits.exclude.filter(x => x !== s) : [...st.limits.exclude, s]}}), 'Limits saved'); },
 note(v: string) { store.set(st => ({limits: {...st.limits, note: v.trim()}}), v.trim() ? 'Movement note saved' : 'Movement note removed'); },
 clearLimits() { store.set({limits: {exclude: [], note: '', routines: {}}}, 'Limits removed'); },
 chooseRoutine(id: string) { store.set(st => ({limits: {...st.limits, routines: {...st.limits.routines, [id]: 'chosen'}}}), 'Noted as your choice'); },
 rehabAdd(name: string) { if (name.trim()) store.set(st => ({rehab: {...st.rehab, items: [...st.rehab.items, {id: 'r' + Date.now(), name: name.trim()}]}}), 'Added to your checklist'); },
 rehabDone(id: string) { const k = `${ctx().calendarDate}:${id}`; store.set(st => ({rehab: {...st.rehab, done: {...st.rehab.done, [k]: !st.rehab.done[k]}}}), 'Self-report saved'); },
 lighter() { const c = ctx(); store.set(st => ({revisions: [...st.revisions, {week: c.plan!.wk, from: c.calendarDate, strength: 1, im: 60}]}), 'Lighter week from today'); },
 revise(key: 'strength' | 'im', delta: number) {
  const c = ctx(), p = c.plan!;
  const next = {week: p.wk, from: c.calendarDate, strength: p.target.strength, im: p.target.im};
  next[key] = Math.max(0, next[key] + delta);
  store.set(st => ({revisions: [...st.revisions, next]}), 'Changed from today');
 },
 ownCommit(n: number) { store.set({ownCommitments: {from: weekId(ctx().calendarDate), strength: n, im: 0, note: 'Your first commitment'}}, 'Commitment added'); },
 saveIntention(draft: {sport: Sport, date: string, time: string | null}, id?: string) {
  const st = S();
  const editing = id ? st.intentions.find(i => i.id === id) : undefined;
  const when = `${dayLabel(draft.date)}${draft.time ? `, ${draft.time}` : ', any time'}`;
  const list = editing
   ? st.intentions.map(i => i.id === id ? {...i, ...draft, link: i.sport === draft.sport && i.date === draft.date ? i.link : null, history: [...i.history, [now(), `Changed to ${SPORTS[draft.sport].name}, ${when}${i.link && (i.sport !== draft.sport || i.date !== draft.date) ? '; old link cleared' : ''}`] as [string, string]]} : i)
   : [...st.intentions, {id: 'n' + Date.now().toString(36), ...draft, link: null, history: [[now(), `Added for ${when}`] as [string, string]]}];
  store.set({intentions: list}, editing ? 'Plan updated' : 'Added to your plan');
  store.closeSheet();
 },
 removeIntention(id: string) { store.set(st => ({intentions: st.intentions.filter(i => i.id !== id)}), 'Removed from your plan'); },
 link(id: string, aid: string) { const i=S().intentions.find(i=>i.id===id); if(!i || !candidates(ctx(),i).some(a=>a.id===aid)){store.toast('Choose a recording from the same day and sport');return;} if(S().intentions.some(i=>i.id!==id&&i.link===aid)){store.toast('That recording is already linked to a session');return;} store.set(st => ({intentions: st.intentions.map(i => i.id === id ? {...i, link: aid, history: [...i.history, [now(), 'You linked a recorded activity'] as [string, string]]} : i)}), 'Linked by you'); },
 unlink(id: string) { store.set(st => ({intentions: st.intentions.map(i => i.id === id ? {...i, link: null, history: [...i.history, [now(), 'Link removed'] as [string, string]]} : i)}), 'Link removed'); },
 winddown() { const k = ctx().tonight.nightOf; store.set(st => ({winddown: {...st.winddown, [k]: !st.winddown[k]}}), 'Wind-down self-report saved'); },
 bedtime() {
  const st = S();
  if (st.permission === 'declined') { store.toast('Notifications are declined. Allow them in Android settings; Raring won’t ask again.'); return; }
  if (st.permission === 'not-asked') { store.openSheet('permission', 'bedtime'); return; }
  store.set({notifyBedtime: !st.notifyBedtime}, st.notifyBedtime ? 'Wind-down reminder off' : 'Wind-down reminder on');
 },
 channel(key: keyof State['channels'], on: boolean) {
  const st = S();
  if (on && st.permission !== 'granted') { if (st.permission === 'declined') { store.toast('Permission declined. Change it in Android settings.'); return false; } store.openSheet('permission', key); return false; }
  store.set({channels: {...st.channels, [key]: on}}); return true;
 },
 permit(v: 'granted' | 'declined', key: string) {
  store.set(st => ({permission: v, ...(key === 'bedtime' ? {notifyBedtime: v === 'granted'} : {channels: {...st.channels, [key]: v === 'granted'}})}));
  store.closeSheet();
 },
 winddownIntention() { store.set({winddownIntention: true}, 'Wind-down intention added'); store.closeSheet(); },
 seenMigration(then?: 'winddown') { store.set(st => ({seen: {...st.seen, timingMigration: true}})); if (then) store.openSheet('winddown'); },
 journalAllow() { store.set(st => ({journal: {...st.journal, consent: true, downloaded: true}}), 'Journal reads allowed'); store.closeSheet(); },
 journalOpen(open: boolean) { store.set(st => ({journal: {...st.journal, open}})); },
 journalConsent(on: boolean) { store.set(st => ({journal: {...st.journal, consent: on, open: false, downloaded: st.journal.downloaded || on}}), on ? 'Journal reads allowed' : 'Journal reads off · downloaded logs stay until you erase local data'); },
 exportData() { store.toast('Prototype export disclosure only; no file created. Plans, circumstances, self-reports and downloaded journal logs are included; tokens are excluded.'); },
 signout() { store.sheetThen(() => { store.reset('simple'); store.external('onboarding'); store.toast('Prototype choices erased. No Garmin account was connected.'); }); },
 keep() { store.toast('Commitments kept for next week'); },
 connect() { store.toast('Would open Garmin Connect (prototype)'); },
 saveAmbition(name: string, date: string) { if (!name.trim()) return; store.set(st => ({ambitionsAdded: [...st.ambitionsAdded, {id: 'amb' + Date.now(), name: name.trim(), date: date || null, source: date ? 'Your local date' : 'Your direction'}]}), 'Ambition added'); store.closeSheet(); },
 sync(kind?: 'exception') {
  if (S().offline) { store.toast('Offline: sync waits for a connection. Your choices already apply.'); return; }
  if (kind === 'exception') store.set({vitalsException: true});
  store.toast(kind === 'exception' ? 'Synced: a new notice arrived; dismissed suggestions stay dismissed.' : 'Synced: nothing you chose has changed.');
 },
 background() { store.background(); store.toast('Resumed from the background. Your choices are unchanged.'); },
 applyCase(id: string, cold = false) {
  const c = CASES.find(x => x.id === id);
  if (!c) return;
  store.state = fresh(c.setup.profile || 'simple', c.setup); store.save();
  if (cold) { store.notice = `Case ${c.id}: ${c.title}`; store.startWith(store.defaultTrail(store.canonical(c.start).route)); return; }
  store.external(c.start, `Case ${c.id}: ${c.title}`);
 },
};
setCaseHook((id, cold) => A.applyCase(id, cold));

// Prototype test hooks for the browser checks; the shipped app has none of these.
Object.assign(window as unknown as Record<string, unknown>, {
 RaringApp: {
  get state() { return store.state; }, set: (p: Partial<State>) => store.set(p), external: (r: string) => store.external(r), navigate: (r: string) => store.navigate(r),
  openSheet: (n: string, a?: string) => store.openSheet(n, a), sync: (k?: 'exception') => A.sync(k), background: () => A.background(),
  canonical: (r: string) => store.canonical(r), defaultTrail: (r: string) => store.defaultTrail(r), get LEGACY() { return LEGACY; },
 },
 RaringRules: {derive}, RaringCases: CASES,
});
