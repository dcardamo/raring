// Prototype rules: derive the synthetic situation and pick Today's lead by
// applying the design's gates in order (privacy, evidence, circumstances,
// decisions, Garmin guidance, time, relevance). Illustrative only; stage 3
// specifies and tests the real rules in the Rust core.
import {addDays, between, dayLabel, dur, parse, shortDay, weekId, weekStart} from './dates';
import {ACTIVITIES, COMMITMENT_VERSIONS, EXTRA_ACTIVITY, HISTORY_START, NAPS, PARTIAL_HISTORY_START, SCHEDULE, bodyBatteryFor, nightFor, rand, readinessFor, type Activity, type Night, type Readiness, type Schedule} from './world';
import {SPORTS, type Sport} from './sports';

export const CLOCKS = {
  waiting: {label: 'Morning, last night not synced', time: '06:50', phase: 'waiting'},
  morning: {label: 'Morning, synced', time: '07:40', phase: 'morning'},
  day: {label: 'Daytime', time: '14:20', phase: 'day'},
  evening: {label: 'Evening', time: '21:00', phase: 'evening'},
  winddown: {label: 'Wind-down', time: '22:30', phase: 'winddown'},
  late: {label: 'After midnight', time: '00:40', phase: 'late'},
} as const;
export type ClockKey = keyof typeof CLOCKS;
export const PROFILES = {
  simple: {label: 'Just looking (no plan, partial history)'},
  sleep: {label: 'Sleep-first (watch without readiness)'},
  mixed: {label: 'Mixed training + event (interrupted week)'},
  sparse: {label: 'Sparse data (stale upload, gaps)'},
} as const;
export type Profile = keyof typeof PROFILES;

export type Intention = {id: string, sport: Sport, date: string, time: string | null, link: string | null, history: [string, string][]};
export type Pause = {since: string, review: string | null, until?: string};
export type Revision = {week: string, from: string, strength: number, im: number};
export type Commitments = {from: string, strength: number, im: number, note: string};
export type State = {
  profile: Profile, day: string, clock: ClockKey, offline: boolean, follow: boolean, irregular: boolean,
  rest: {day: string} | null, pause: Pause | null, pauses: Pause[], limits: {exclude: Sport[], note: string, routines: Record<string, 'chosen'>},
  rehab: {items: {id: string, name: string, how?: string}[], done: Record<string, boolean>},
  intentions: Intention[], dismissed: Record<string, boolean>, journal: {consent: boolean, open: boolean, downloaded: boolean}, winddown: Record<string, boolean>,
  revisions: Revision[], ownCommitments: Commitments | null, ambitionsAdded: {id: string, name: string, date: string | null, source: string}[],
  celebrated: Record<string, string>, seen: Record<string, boolean>, quiet: boolean, vitalsException: boolean, extraStrength: boolean,
  napCase: 'same' | 'cross' | 'empty' | 'unavailable' | 'stale', calendar: 'complete' | 'partial' | 'stale', notifyBedtime: boolean,
  permission: 'not-asked' | 'granted' | 'declined', channels: Record<'morning' | 'weekly' | 'metrics' | 'vitals' | 'training' | 'status', boolean>,
  signedOut: boolean, dst: boolean, returning: boolean, noRunning: boolean, savedSleepTarget: boolean, winddownIntention: boolean,
  navVariant: 'tabs' | 'header', reviewDraft: string | null,
};

export function profileWorld(state: State) {
  const p = state.profile;
  const w = {
    readinessSupported: p !== 'sleep',
    commitments: p === 'mixed' || p === 'sparse' || !!state.ownCommitments,
    schedule: p === 'mixed' ? SCHEDULE : [] as Schedule[],
    ambitions: p === 'mixed' ? [
      {id: 'event', name: 'Winter 10K', date: '2026-12-12' as string | null, source: 'Garmin calendar', sport: 'running' as Sport | null},
      {id: 'strength', name: 'Keep strength', date: null as string | null, source: 'Your direction', sport: 'strength' as Sport | null},
    ] : [],
    historyComplete: p === 'mixed' || p === 'sleep',
    historyStart: p === 'simple' ? PARTIAL_HISTORY_START : HISTORY_START,
    timingCredible: !state.irregular,
    activities: [] as Activity[],
    coverageIncompleteFrom: p === 'sparse' ? '2026-11-17' : null as string | null,
    coverageGaps: p === 'sparse' ? ['2026-W39', '2026-W44'] : p === 'mixed' ? ['2026-W44'] : [] as string[],
  };
  let acts = ACTIVITIES;
  if (p === 'simple') acts = acts.filter(a => a.sport === 'walking' && !a.lateUpload && a.date >= PARTIAL_HISTORY_START);
  if (p === 'sleep') acts = acts.filter(a => a.sport === 'walking' && a.date < '2026-11-16' && rand(a.id + 'sleepfirst') < .45);
  if (p === 'sparse') acts = acts.filter(a => a.date <= '2026-11-16' && a.id !== 'a-1202' && !(a.generated && rand(a.id + 'sparse') < .35));
  if (state.extraStrength) acts = [...acts, EXTRA_ACTIVITY].sort((a, b) => (a.date + a.start).localeCompare(b.date + b.start));
  if (state.noRunning) {
    acts = acts.filter(a => a.sport !== 'running');
    w.schedule = w.schedule.filter(a => a.sport !== 'running');
    w.ambitions = w.ambitions.filter(a => a.id !== 'event');
  }
  w.activities = acts;
  return w;
}
export type World = ReturnType<typeof profileWorld>;

export type Lead = {
  id: string, kind: 'waiting' | 'stale' | 'quiet' | 'sleep' | 'paused' | 'rest' | 'done' | 'schedule' | 'intention' | 'suggest' | 'easy' | 'review' | 'conflict' | 'fixed',
  headline: string, line: string | null, action: {label: string, go?: string, sheet?: string}, trace: {gate: string, result: string}[],
  dismissable?: boolean, celebrate?: string | null, activity?: Activity, schedule?: Schedule, intention?: Intention,
};

export function derive(state: State): Ctx {
  const clock = CLOCKS[state.clock];
  const world = profileWorld(state);
  const day = state.day;
  const calendarDate = clock.phase === 'late' ? addDays(day, 1) : day;
  // Nothing recorded after the synthetic "now" exists yet.
  world.activities = world.activities.filter(a => a.date < calendarDate || (a.date === calendarDate && a.start <= clock.time));
  const sleepFirst = state.profile === 'sleep';
  const night = (d: string): Night => state.profile === 'sparse' && (d === '2026-11-17' || d === '2026-11-18') ? null : nightFor(d, sleepFirst);
  // Displayed (sleep/readiness) day keeps last night until the handover.
  const displayedDay = clock.phase === 'waiting' ? addDays(day, -1) : day;
  const lastNight = night(displayedDay);
  const readinessOf = (d: string): Readiness | null => !world.readinessSupported || (state.profile === 'sparse' && d >= '2026-11-17') ? null : readinessFor(d);
  const readiness = readinessOf(displayedDay);
  const stale = state.profile === 'sparse';
  const fixedReason = !state.follow ? 'Follow my day is off, so Today keeps a fixed order.'
    : !world.timingCredible ? 'Fixed order: your wake times vary by more than 3 hours.' : null;
  const phase = fixedReason && clock.phase !== 'waiting' ? 'fixed' : clock.phase;
  const uploadAt = stale ? '21:40' : clock.phase === 'waiting' ? '22:51' : clock.phase === 'late' ? '23:58' : ({'07:40': '07:22', '14:20': '14:02', '21:00': '20:42'} as Record<string, string>)[clock.time] ?? '22:12';
  const upload = stale ? {label: '2 days ago', detail: 'Mon 16 Nov, 21:40', date: '2026-11-16', at: uploadAt}
    : clock.phase === 'waiting' ? {label: '8 h ago', detail: `${dayLabel(addDays(day, -1))}, 22:51`, date: addDays(day, -1), at: uploadAt}
    : clock.phase === 'late' ? {label: '42 min ago', detail: `${dayLabel(day)}, 23:58`, date: day, at: uploadAt}
    : {label: '18 min ago', detail: `${dayLabel(day)}, ${uploadAt}`, date: day, at: uploadAt};

  const restActive = !!state.rest && state.rest.day === calendarDate;
  const restCarry = !!state.rest && addDays(state.rest.day, 1) === calendarDate && ['late', 'waiting'].includes(clock.phase);
  const paused = state.pause && state.pause.since <= calendarDate ? state.pause : null;
  const limits = state.limits;
  const hasLimits = limits.exclude.length > 0 || !!limits.note;
  const reviewDue = !!paused && !!paused.review && paused.review <= calendarDate;

  const weekFrom = weekStart(calendarDate);
  const weekTo = addDays(weekFrom, 6);
  const actsToday = world.activities.filter(a => a.date === calendarDate);
  const lateArrivals = world.activities.filter(a => a.lateUpload === calendarDate);
  const scheduleToday = world.schedule.filter(s => s.date === calendarDate);
  const intentionsToday = state.intentions.filter(i => i.date === calendarDate);
  const napList = (state.profile === 'sleep' || state.profile === 'mixed'
    ? (state.napCase === 'cross' ? NAPS.cross : state.napCase === 'same' ? NAPS.same : []) : []).filter(n => {
      const [date, time] = n.end || n.start;
      return date < calendarDate || (date === calendarDate && time <= clock.time);
    });
  const napsToday = napList.filter(n => (n.end ? n.end[0] : n.start[0]) === calendarDate && (n.start[0] < calendarDate || n.start[1] <= clock.time));

  const ctx: Ctx = {
    state, world, clock, phase, fixedReason, day, calendarDate, displayedDay, night: lastNight, nightOf: night, readiness, readinessOf, stale, upload,
    restActive: restActive || restCarry, paused, reviewDue, limits, hasLimits, weekFrom, weekTo, actsToday, lateArrivals,
    scheduleToday, intentionsToday, napList, napsToday, offline: state.offline,
    bodyBattery: stale || clock.phase === 'waiting' ? null : bodyBatteryFor(calendarDate),
    plan: null as PlanStatus | null, tonight: null as unknown as Tonight, notices: [] as Notice[], lead: null as unknown as Lead, adaptLabel: 'Adapt today',
  };
  ctx.plan = planStatus(ctx);
  ctx.tonight = tonight(ctx);
  ctx.notices = notices(ctx);
  ctx.lead = lead(ctx);
  ctx.adaptLabel = paused ? 'Paused' : ctx.restActive ? 'Rest day' : hasLimits ? 'Limits on' : 'Adapt today';
  return ctx;
}
export type Ctx = {
  state: State, world: World, clock: typeof CLOCKS[ClockKey], phase: string, fixedReason: string | null, day: string, calendarDate: string, displayedDay: string,
  night: Night, nightOf: (d: string) => Night, readiness: Readiness | null, readinessOf: (d: string) => Readiness | null, stale: boolean,
  upload: {label: string, detail: string, date: string, at: string}, restActive: boolean, paused: Pause | null, reviewDue: boolean, limits: State['limits'], hasLimits: boolean,
  weekFrom: string, weekTo: string, actsToday: Activity[], lateArrivals: Activity[], scheduleToday: Schedule[], intentionsToday: Intention[], napList: Nap[], napsToday: Nap[],
  offline: boolean, bodyBattery: number | null, plan: PlanStatus | null, tonight: Tonight, notices: Notice[], lead: Lead, adaptLabel: string,
};

export type Tonight = {nightOf: string, nightLabel: string, window: string | null, windowStart: string, windowEnd: string, windDown: string, wake: string, need: number, source: string, estimate: string | null, reason: string, savedNeed: number, updated: string | null};
type Nap = {start: [string, string], end: [string, string] | null, min: number};
function tonight(ctx: {state: State, clock: typeof CLOCKS[ClockKey], day: string, calendarDate: string, stale: boolean, napsToday: Nap[]}): Tonight {
  const sleepOnly = ctx.state.profile === 'sleep';
  const nightOf = ctx.clock.phase === 'late' ? ctx.day : ctx.calendarDate;
  const need = sleepOnly ? 505 : 490;
  return {
    nightOf, nightLabel: `${shortDay(nightOf)} night → ${shortDay(addDays(nightOf, 1))} morning`,
    window: ctx.stale ? null : '22:45–23:15', windowStart: '22:45', windowEnd: '23:15', windDown: '22:15', wake: sleepOnly ? '06:30' : '06:55', need,
    source: ctx.stale ? 'Raring estimate' : 'Garmin plan',
    estimate: ctx.stale ? 'Asleep by 23:20 · from your usual wake 07:10 (12 recorded wake times)' : null,
    reason: sleepOnly ? 'Garmin raised tonight’s need after several short nights, and added 15 min after your nap.' : 'Garmin added 20 min for recent short sleep.',
    savedNeed: 490,
    updated: sleepOnly && ctx.napsToday.length ? 'Need updated 13:58 after Garmin’s nap adjustment. The plan saved this morning for timing stays 8h 10.' : null,
  };
}

export function versionFor(ctx: {state: State, calendarDate: string}) {
  const wk = weekId(ctx.calendarDate);
  let base: Commitments | null = null;
  const versions = ['mixed', 'sparse'].includes(ctx.state.profile) ? COMMITMENT_VERSIONS : ctx.state.ownCommitments ? [ctx.state.ownCommitments] : [];
  for (const v of versions) if (v.from <= wk) base = v;
  const revisions = ctx.state.revisions.filter(r => r.week === wk && r.from <= ctx.calendarDate);
  return {wk, base, revisions, versions};
}

export type Track = {key: 'strength' | 'im', have: number, target: number, original: number, unit: string, state: string, label: string, detail: string};
export type PlanStatus = {wk: string, base: Commitments, revisions: Revision[], pause: Pause | null, wholePause: boolean, midPause: boolean, daysLeft: number, tracks: Track[], strength: number, im: number, target: {strength: number, im: number}};
function planStatus(ctx: {state: State, world: World, calendarDate: string, weekFrom: string, weekTo: string}): PlanStatus | null {
  if (!ctx.world.commitments) return null;
  const {wk, base, revisions} = versionFor(ctx);
  if (!base) return null;
  const inWeek = ctx.world.activities.filter(a => between(a.date, ctx.weekFrom, ctx.calendarDate));
  const strength = inWeek.filter(a => a.sport === 'strength').length;
  const im = inWeek.reduce((n, a) => n + (a.im || 0), 0);
  const last = revisions.at(-1);
  const target = {strength: last ? last.strength : base.strength, im: last ? last.im : base.im};
  const ended = (ctx.state.pauses || []).filter(x => x.since <= ctx.calendarDate && (x.until ?? '') >= ctx.weekFrom).at(-1);
  const pause = ctx.state.pause && ctx.state.pause.since <= ctx.calendarDate ? ctx.state.pause : ended || null;
  const wholePause = !!pause && !pause.until && pause.since <= ctx.weekFrom;
  const midPause = !!pause && !wholePause && pause.since <= ctx.calendarDate;
  const daysLeft = Math.max(0, Math.round((parse(ctx.weekTo).getTime() - parse(ctx.calendarDate).getTime()) / 86400000));
  const incomplete = ctx.world.coverageIncompleteFrom;
  const track = (key: 'strength' | 'im', have: number, unit: string): Track => {
    const t = target[key], orig = base[key];
    const rev = revisions.find(r => r[key] !== orig);
    const out = {key, have, target: t, original: orig, unit};
    if (wholePause) return {...out, state: 'paused', label: 'Paused', detail: `${have}${unit} recorded · target ${orig}${unit}`};
    if (midPause) return {...out, state: 'adjusted-paused', label: pause!.until ? `Paused ${shortDay(pause!.since)}–${shortDay(pause!.until)}` : `Paused since ${shortDay(pause!.since)}`, detail: `${have}${unit} recorded · target ${orig}${unit}`};
    if ((incomplete || ctx.world.coverageGaps.includes(wk)) && have < t) return {...out, state: 'unknown', label: 'Awaiting evidence', detail: incomplete ? `${have}${unit} known · list incomplete since ${dayLabel(incomplete)}` : `${have}${unit} known · list incomplete`};
    if (have >= t && rev) return {...out, state: 'reached-adjusted', label: `Reached · adjusted ${shortDay(rev.from)}`, detail: `${have}${unit} of ${t}${unit} (was ${orig}${unit})`};
    if (have >= t) return {...out, state: 'reached', label: 'Reached', detail: `${have}${unit} of ${t}${unit}`};
    return {...out, state: 'open', label: `${have} of ${t}${unit}`, detail: rev ? `Adjusted ${shortDay(rev.from)} from ${orig}${unit} · ${daysLeft} days left` : `${daysLeft} days left`};
  };
  return {wk, base, revisions, pause, wholePause, midPause, daysLeft,
    tracks: [track('strength', strength, ''), track('im', im, ' min')].filter(t => t.target > 0), strength, im, target};
}

type Notice = {id: string, text: string, action: {label: string, go: string}};
function notices(ctx: {stale: boolean, state: State, reviewDue: boolean, paused: Pause | null}): Notice[] {
  const list: Notice[] = [];
  if (ctx.stale) list.push({id: 'stale', text: 'No watch upload for 2 days', action: {label: 'Sync help', go: 'help/sync'}});
  if (ctx.state.vitalsException) list.push({id: 'vitals', text: 'Two overnight readings outside your usual ranges', action: {label: 'Health', go: 'health'}});
  if (ctx.reviewDue) list.push({id: 'review', text: `Pause review date ${dayLabel(ctx.paused!.review!)}`, action: {label: 'Review pause', go: 'adapt/pause'}});
  return list;
}

export function contribution(ctx: {plan: PlanStatus | null, world: World, weekFrom: string}, a: Activity): {short: string, ring?: [number, number]} {
  if (a.timerOnly) return {short: 'Strength session · no sets'};
  const p = ctx.plan;
  if (p && a.sport === 'strength' && !p.wholePause) {
    const before = ctx.world.activities.filter(x => x.sport === 'strength' && between(x.date, ctx.weekFrom, a.date) && (x.date < a.date || x.start <= a.start)).length;
    return {short: `${before} of ${p.target.strength} strength this week`, ring: [before, p.target.strength]};
  }
  if (p && a.im && !p.wholePause) return {short: `+${a.im} intensity min`, ring: [Math.min(p.im, p.target.im), p.target.im]};
  if (a.record) return {short: `Best: ${a.record}`};
  return {short: `${SPORTS[a.sport].name} recorded`};
}

export const effortFor = (r: Readiness | null) => !r ? null : r.value < 30 ? 'rest' : r.value < 50 ? 'easy' : r.value < 75 ? 'steady' : 'push';

function lead(ctx: Ctx): Lead {
  const s = ctx.state, r = ctx.readiness, n = ctx.night;
  const trace: Lead['trace'] = [{gate: 'Privacy', result: 'Journal entries never feed Today.'}];
  const done = (l: Omit<Lead, 'trace'>): Lead => ({...l, trace});
  if (ctx.phase === 'waiting') {
    trace.push({gate: 'Evidence', result: 'Last night has not synced; yesterday’s readiness is not presented as today’s.'});
    return done({id: `wait:${ctx.calendarDate}`, kind: 'waiting', headline: 'Last night hasn’t synced yet', line: `Watch upload ${ctx.upload.detail}`, action: {label: 'How syncing works', go: 'help/sync'}});
  }
  if (ctx.stale) {
    trace.push({gate: 'Evidence', result: 'No upload for 2 days and no night or readiness for today; no effort suggestion is possible.'});
    return done({id: `stale:${ctx.calendarDate}`, kind: 'stale', headline: 'Waiting for your watch', line: `Last upload ${ctx.upload.detail}`, action: {label: 'Sync help', go: 'help/sync'}});
  }
  const evidence = [n ? `Last night ${dur(n.asleep)} of ${dur(n.need)} need` : 'No recorded night', r ? `Readiness ${r.value} ${r.level}` : ctx.world.readinessSupported ? 'Readiness missing' : 'Readiness not recorded by this watch'];
  trace.push({gate: 'Evidence', result: `${evidence.join(' · ')} · watch upload ${ctx.upload.label}.`});
  if (ctx.fixedReason) return done({id: `fixed:${ctx.calendarDate}`, kind: 'fixed', headline: 'Your recorded day', line: ctx.fixedReason, action: {label: 'View sleep', go: 'sleep'}});
  const blocked = ctx.paused ? 'paused' : ctx.restActive ? 'rest' : null;
  const excluded = ctx.limits.exclude.map(x => SPORTS[x].name).join(', ');
  trace.push({gate: 'Circumstances', result: blocked === 'paused' ? `Training paused since ${dayLabel(ctx.paused!.since)}: no exercise prompts.`
    : blocked === 'rest' ? 'Rest today: no exercise prompts today or tonight.'
    : ctx.hasLimits ? `Limits: ${[excluded && `excluded ${excluded}`, ctx.limits.note && 'movement note (routines need review)'].filter(Boolean).join('; ')}.` : 'None active.'});

  const tn = ctx.tonight;
  if (['evening', 'winddown', 'late'].includes(ctx.phase)) {
    trace.push({gate: 'Decisions', result: ctx.intentionsToday.length ? 'Tonight’s intention stays in Plan.' : 'Nothing planned tonight.'});
    trace.push({gate: 'Time', result: `${CLOCKS[s.clock].label}: sleep preparation leads.`});
    if (ctx.phase === 'late') return done({id: `late:${tn.nightOf}`, kind: 'sleep', headline: 'Rest when you’re ready', line: tn.nightLabel, action: {label: 'Tonight’s sleep', go: 'sleep/tonight'}});
    if (ctx.phase === 'winddown') return done({id: `wind:${tn.nightOf}`, kind: 'sleep', headline: 'Time to wind down', line: tn.window ? `Garmin’s window ${tn.window}` : `Asleep by 23:20`, action: {label: 'Tonight’s plan', go: 'sleep/tonight'}});
    return done({id: `evening:${tn.nightOf}`, kind: 'sleep', headline: 'Make room for sleep', line: `Wind down from ${tn.windDown}`, action: {label: 'Tonight’s plan', go: 'sleep/tonight'}});
  }
  if (blocked === 'paused') {
    trace.push({gate: 'Decisions', result: 'Commitments are kept for when you resume; nothing to catch up.'});
    return done({id: `paused:${ctx.calendarDate}`, kind: 'paused', headline: 'Training is paused', line: `Since ${dayLabel(ctx.paused!.since)}`, action: {label: 'Resume when ready', go: 'adapt/pause'}});
  }
  if (blocked === 'rest') {
    trace.push({gate: 'Decisions', result: 'Your rest day outranks readiness and any schedule.'});
    return done({id: `rest:${ctx.displayedDay}`, kind: 'rest', headline: 'Rest day', line: 'Through tonight', action: {label: 'View plan', go: 'plan'}, schedule: ctx.scheduleToday[0]});
  }
  // Recognition: a recorded session today leads; a day can finish.
  const recent = ctx.actsToday.filter(a => a.sport !== 'walking' || !ctx.world.commitments).at(-1);
  const sched = ctx.scheduleToday.find(x => !s.dismissed[`sched:${x.id}:${ctx.calendarDate}`]);
  if (recent) {
    const p = ctx.plan;
    const reached = !!p && recent.sport === 'strength' && p.tracks.find(t => t.key === 'strength')?.state === 'reached' && p.strength === p.target.strength;
    trace.push({gate: 'Decisions', result: `${SPORTS[recent.sport].name} recorded at ${recent.start}; recognition leads and no new task is created.`});
    return done({id: `done:${recent.id}`, kind: 'done', celebrate: reached ? `target:${p!.wk}:strength` : null, activity: recent,
      headline: reached ? 'Strength target reached' : recent.name, line: reached ? `${p!.target.strength} of ${p!.target.strength} sessions this week` : contribution(ctx, recent).ring || recent.record ? contribution(ctx, recent).short : null,
      action: {label: 'Open workout', go: `workout/${recent.id}`}});
  }
  if (sched) {
    const conflict = ctx.limits.exclude.includes(sched.sport);
    if (ctx.limits.note && !conflict) {
      trace.push({gate: 'Decisions', result: `Garmin schedule: ${sched.name}; its movements are unknown to Raring.`});
      return done({id: `review:${sched.id}`, kind: 'review', headline: 'Review needed', line: `${sched.name} · contents unknown`, action: {label: 'Review session', go: `scheduled/${sched.id}`}, schedule: sched});
    }
    const lowR = !!r && r.value < 50;
    trace.push({gate: 'Decisions', result: `Garmin schedule: ${sched.name} today (read-only).`});
    trace.push({gate: 'Garmin guidance', result: conflict ? 'Conflicts with your excluded activity; shown, not changed.' : lowR ? 'Readiness Low: an easier day may fit; the session is not cancelled.' : 'Readiness supports the session as planned.'});
    return done({id: `sched:${sched.id}:${ctx.calendarDate}`, kind: conflict ? 'conflict' : 'schedule', dismissable: true, schedule: sched,
      headline: conflict ? 'Conflicts with your limits' : lowR ? 'An easier day may fit' : sched.name,
      line: conflict ? `${sched.name} · ${SPORTS[sched.sport].name.toLowerCase()} excluded` : lowR ? `Garmin: ${sched.name}` : 'Garmin schedule',
      action: {label: 'Review session', go: `scheduled/${sched.id}`}});
  }
  const intent = ctx.intentionsToday.find(i => !i.link);
  if (intent) {
    if (ctx.limits.note) {
      trace.push({gate: 'Decisions', result: 'Your intention; its movements are unknown to Raring, so your movement note applies.'});
      return done({id: `review-intent:${intent.id}`, kind: 'review', headline: 'Review your session', line: `${SPORTS[intent.sport].name} · movement note applies`, action: {label: 'Review session', go: `intention/${intent.id}`}, intention: intent});
    }
    trace.push({gate: 'Decisions', result: 'Your own intention for today leads; current evidence qualifies it.'});
    return done({id: `intent:${intent.id}`, kind: 'intention', headline: `${SPORTS[intent.sport].name} ${intent.time ? `at ${intent.time}` : 'today'}`,
      line: ctx.limits.exclude.includes(intent.sport) ? 'Your plan · excluded in your limits' : 'Your plan', action: {label: 'Review session', go: `intention/${intent.id}`}, intention: intent});
  }
  trace.push({gate: 'Decisions', result: 'Nothing scheduled, planned or recorded yet today.'});
  if (!r) {
    trace.push({gate: 'Garmin guidance', result: 'No readiness, so no effort suggestion; sleep leads.'});
    return done({id: `sleep:${ctx.displayedDay}`, kind: 'sleep', headline: n ? `You slept ${dur(n.asleep)}` : 'No sleep recorded',
      line: n ? `${n.asleep < n.need ? dur(n.need - n.asleep) + ' below' : 'Met'} last night’s need` : 'A gap in the record', action: {label: 'Last night’s sleep', go: 'sleep'}});
  }
  const id = `suggest:${ctx.calendarDate}`;
  if (ctx.limits.note) {
    trace.push({gate: 'Relevance', result: 'Workout contents need review; readiness cannot establish compatibility.'});
    return done({id: `limits:${ctx.calendarDate}`, kind: 'review', headline: 'Your movement limits apply', line: 'Workouts need your review', action: {label: 'Review limits', go: 'adapt/limits'}});
  }
  if (s.dismissed[id]) {
    trace.push({gate: 'Relevance', result: 'You chose “Not today” for training ideas; it stays dismissed after sync and resume.'});
    return done({id: 'quiet', kind: 'quiet', headline: 'Nothing else needs you today', line: 'Training ideas hidden for today', action: {label: 'Tonight’s plan', go: 'sleep/tonight'}});
  }
  if (r.value >= 75) {
    trace.push({gate: 'Garmin guidance', result: `Readiness ${r.level}: Garmin’s guidance allows hard effort if it fits.`});
    trace.push({gate: 'Relevance', result: 'No availability declared: offer planning, not a command or required load.'});
    return done({id, kind: 'suggest', dismissable: true, headline: 'Training is an option today', line: n ? `After ${dur(n.asleep)} of sleep` : null, action: {label: 'Plan a session', sheet: 'add'}});
  }
  trace.push({gate: 'Garmin guidance', result: `Readiness ${r.level}: Garmin suggests easier effort.`});
  return done({id, kind: 'easy', dismissable: true, headline: r.value < 50 ? 'An easier day may fit' : 'A steady day', line: `${r.limit} holds it back most`, action: {label: 'Why readiness is ' + r.level.toLowerCase(), go: 'readiness'}});
}

export function surfaces(ctx: Ctx) {
  const r = ctx.readiness, s = ctx.state;
  const focus = ctx.phase === 'waiting' || ctx.stale ? 'Waiting for last night'
    : ctx.fixedReason ? 'Recorded context only'
    : ['evening', 'winddown', 'late'].includes(ctx.phase) ? `Wind down from ${ctx.tonight.windDown}`
    : ctx.paused ? 'Training paused' : ctx.restActive ? 'Rest day'
    : ctx.hasLimits ? 'Training suggestions limited' : s.dismissed[`suggest:${ctx.calendarDate}`] ? 'No training suggestion today'
    : !r ? 'Readiness not recorded' : r.value >= 75 ? 'Training is an option' : r.value < 50 ? 'An easier day may fit' : 'A steady day';
  const as = ctx.upload.detail;
  return {
    focus, as,
    morning: s.permission !== 'granted' || !s.channels.morning || ctx.phase === 'waiting' || ctx.stale || !r ? null : {
      title: 'Morning readiness', body: `Readiness ${r.value} · ${r.level}`,
      sub: ctx.paused ? 'Training paused · factual update only' : ctx.restActive ? 'Rest day · no training call' : focus,
      lock: 'Raring · Readiness update', go: 'readiness'},
    statusEnabled: s.permission === 'granted' && s.channels.status,
    status: {bb: ctx.bodyBattery, stress: ctx.stale || ctx.phase === 'waiting' ? null : 27, hr: ctx.stale || ctx.phase === 'waiting' ? null : 61, focus, go: 'today'},
    bedtime: s.permission === 'granted' && s.notifyBedtime && ctx.phase !== 'waiting' ? {title: 'Wind-down', body: `Tonight’s window ${ctx.tonight.window || '—'} · wind down from ${ctx.tonight.windDown}`, go: 'sleep/tonight'} : null,
    weekly: s.permission === 'granted' && s.channels.weekly ? {title: 'Your week in review', body: 'One win, one observation, one choice', go: 'review/latest'} : null,
  };
}

/** Week states for a plan week: v1's history plus the live week. */
export function latestReviewWeek(ctx: Ctx) {
  // The review for a week appears on its Sunday; otherwise the previous week's.
  return new Date(ctx.calendarDate + 'T00:00:00Z').getUTCDay() === 0 ? weekId(ctx.calendarDate) : weekId(addDays(ctx.weekFrom, -1));
}
export const isHistoryDay = (ctx: Ctx, d: string) => d >= ctx.world.historyStart && d <= ctx.calendarDate;
export const strengthCount = (acts: Activity[], from: string, to: string) => acts.filter(a => a.sport === 'strength' && between(a.date, from, to)).length;
export {weekStart};
