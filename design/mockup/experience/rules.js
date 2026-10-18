// Prototype rules: derive the synthetic situation and pick Today's lead by
// applying the design's gates in order (privacy, evidence, circumstances,
// decisions, Garmin guidance, time, relevance). Illustrative only; stage 3
// specifies and tests the real rules in the Rust core.
(function (root) {
  const D = root.RaringData;
  const WD = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
  const MO = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];

  const parse = iso => { const [y, m, d] = iso.split('-').map(Number); return new Date(Date.UTC(y, m - 1, d)); };
  const iso = dt => dt.toISOString().slice(0, 10);
  const addDays = (s, n) => { const dt = parse(s); dt.setUTCDate(dt.getUTCDate() + n); return iso(dt); };
  const dayLabel = s => { const dt = parse(s); return `${WD[dt.getUTCDay()]} ${dt.getUTCDate()} ${MO[dt.getUTCMonth()]}`; };
  const shortDay = s => WD[parse(s).getUTCDay()];
  const dur = min => min == null ? '—' : `${Math.floor(min / 60)}h ${String(min % 60).padStart(2, '0')}`;
  const weekStart = s => { const dt = parse(s); const wd = (dt.getUTCDay() + 6) % 7; dt.setUTCDate(dt.getUTCDate() - wd); return iso(dt); };
  const weekId = s => {
    const dt = parse(s); const day = (dt.getUTCDay() + 6) % 7; dt.setUTCDate(dt.getUTCDate() - day + 3);
    const first = new Date(Date.UTC(dt.getUTCFullYear(), 0, 4));
    const n = 1 + Math.round(((dt - first) / 86400000 - 3 + ((first.getUTCDay() + 6) % 7)) / 7);
    return `${dt.getUTCFullYear()}-W${String(n).padStart(2, '0')}`;
  };
  const between = (d, a, b) => d >= a && d <= b;

  const CLOCKS = {
    waiting: {label: 'Morning, last night not synced', time: '06:50', phase: 'waiting'},
    morning: {label: 'Morning, synced', time: '07:40', phase: 'morning'},
    day: {label: 'Daytime', time: '14:20', phase: 'day'},
    evening: {label: 'Evening', time: '21:00', phase: 'evening'},
    winddown: {label: 'Wind-down', time: '22:30', phase: 'winddown'},
    late: {label: 'After midnight', time: '00:40', phase: 'late'},
  };

  const PROFILES = {
    simple: {label: 'Just looking (no plan, partial history)'},
    sleep: {label: 'Sleep-first (watch without readiness)'},
    mixed: {label: 'Mixed training + event (interrupted week)'},
    sparse: {label: 'Sparse data (stale upload, gaps)'},
  };

  const SPORTS = {strength: 'Strength', running: 'Run', cycling: 'Ride', walking: 'Walk', hiit: 'HIIT', swimming: 'Swim', yoga: 'Yoga', other: 'Other'};

  function profileWorld(state) {
    const p = state.profile;
    const w = {
      readinessSupported: p !== 'sleep',
      commitments: p === 'mixed' || p === 'sparse' || !!state.ownCommitments,
      schedule: p === 'mixed' ? D.schedule : [],
      ambitions: p === 'mixed' ? [
        {id: 'event', name: 'Winter 10K', date: '2026-12-12', source: 'Garmin calendar'},
        {id: 'strength', name: 'Keep strength', source: 'Your direction · no date'},
      ] : [],
      historyComplete: p === 'mixed',
      timingCredible: !state.irregular,
    };
    let acts = D.activities;
    if (p === 'simple') acts = acts.filter(a => a.sport === 'walking' && !a.lateUpload);
    if (p === 'sleep') acts = [];
    if (p === 'sparse') acts = acts.filter(a => a.date <= '2026-11-16' && a.id !== 'a-1202');
    if (state.extraStrength) acts = [...acts, D.extraActivity];
    if (state.noRunning) { acts = acts.filter(a => a.sport !== 'running'); w.schedule = w.schedule.filter(a => a.sport !== 'running'); w.ambitions = w.ambitions.filter(a => a.id !== 'event'); }
    w.activities = acts;
    w.nights = {...D.nights};
    if (p === 'sleep') Object.assign(w.nights, {
      '2026-11-16': {asleep: 380, need: 485, score: 64, bed: '00:10', wake: '06:30'},
      '2026-11-17': {asleep: 362, need: 495, score: 60, bed: '00:25', wake: '06:27'},
      '2026-11-18': {asleep: 345, need: 500, score: 57, bed: '00:41', wake: '06:26'},
    });
    if (p === 'sparse') { w.nights['2026-11-17'] = null; w.nights['2026-11-18'] = null; }
    w.readiness = w.readinessSupported ? {...D.readiness} : {};
    if (p === 'sparse') { delete w.readiness['2026-11-17']; delete w.readiness['2026-11-18']; }
    w.coverageIncompleteFrom = p === 'sparse' ? '2026-11-17' : null;
    return w;
  }

  function derive(state) {
    const clock = CLOCKS[state.clock];
    const world = profileWorld(state);
    const day = state.day;
    const calendarDate = clock.phase === 'late' ? addDays(day, 1) : day;
    // Nothing recorded after the synthetic "now" exists yet.
    world.activities = world.activities.filter(a => a.date < calendarDate || (a.date === calendarDate && a.start <= clock.time));
    const nightSynced = clock.phase !== 'waiting' && clock.phase !== 'late' ? world.nights[day] != null : false;
    // Displayed (sleep/readiness) day keeps last night until the handover.
    const displayedDay = clock.phase === 'waiting' ? addDays(day, -1) : day;
    const night = world.nights[displayedDay] || null;
    const readiness = world.readiness[displayedDay] || null;
    const stale = state.profile === 'sparse';
    const fixedReason = !state.follow ? 'Follow my day is off in Settings, so Today keeps a fixed order.'
      : !world.timingCredible ? 'Your recent wake times vary by more than 3 hours, so Today keeps a fixed order instead of guessing your day.' : null;
    const phase = fixedReason && clock.phase !== 'waiting' ? 'fixed' : clock.phase;
    const upload = stale ? {label: '2 days ago', detail: 'Mon 16 Nov, 21:40'}
      : clock.phase === 'waiting' ? {label: '8 h ago', detail: `${dayLabel(addDays(day, -1))}, 22:51`}
      : clock.phase === 'late' ? {label: '42 min ago', detail: `${dayLabel(day)}, 23:58`}
      : {label: '18 min ago', detail: `${dayLabel(day)}, ${clock.time === '07:40' ? '07:22' : clock.time === '14:20' ? '14:02' : clock.time === '21:00' ? '20:42' : '22:12'}`};

    const restActive = state.rest && state.rest.day === calendarDate;
    const restCarry = state.rest && addDays(state.rest.day, 1) === calendarDate && ['late', 'waiting'].includes(clock.phase);
    const paused = state.pause && state.pause.since <= calendarDate ? state.pause : null;
    const limits = state.limits;
    const hasLimits = limits.exclude.length > 0 || !!limits.note;
    const reviewDue = paused && paused.review && paused.review <= calendarDate;

    const weekFrom = weekStart(calendarDate);
    const weekTo = addDays(weekFrom, 6);
    const actsToday = world.activities.filter(a => a.date === calendarDate);
    const lateArrivals = world.activities.filter(a => a.lateUpload === calendarDate);
    const scheduleToday = world.schedule.filter(s => s.date === calendarDate);
    const intentionsToday = state.intentions.filter(i => i.date === calendarDate);
    const napList = (state.profile === 'sleep' || state.profile === 'mixed'
      ? (state.napCase === 'cross' ? D.naps.cross : state.napCase === 'same' ? D.naps.same : []) : []).filter(n => {
        const [date, time] = n.end || n.start;
        return date < calendarDate || (date === calendarDate && time <= clock.time);
      });
    const napsToday = napList.filter(n => (n.end ? n.end[0] : n.start[0]) === calendarDate && (n.start[0] < calendarDate || n.start[1] <= clock.time));

    const ctx = {state, world, clock, phase, fixedReason, day, calendarDate, displayedDay, nightSynced, night, readiness, stale, upload,
      restActive: !!restActive || !!restCarry, paused, reviewDue, limits, hasLimits, weekFrom, weekTo, actsToday, lateArrivals,
      scheduleToday, intentionsToday, napList, napsToday, offline: state.offline};
    ctx.plan = planStatus(ctx);
    ctx.tonight = tonight(ctx);
    ctx.notices = notices(ctx);
    ctx.lead = lead(ctx);
    ctx.adaptLabel = paused ? 'Paused' : ctx.restActive ? 'Rest today' : hasLimits ? 'Limits on' : 'Adapt today';
    return ctx;
  }

  function tonight(ctx) {
    const sleepOnly = ctx.state.profile === 'sleep';
    const nightOf = ctx.clock.phase === 'late' ? ctx.day : ctx.calendarDate;
    const need = sleepOnly ? 505 : 490;
    return {
      nightOf, nightLabel: `${shortDay(nightOf)} night → ${shortDay(addDays(nightOf, 1))} morning`,
      window: ctx.stale ? null : '22:45–23:15', windDown: '22:15', need,
      source: ctx.stale ? 'Raring estimate' : 'Garmin plan',
      estimate: ctx.stale ? 'Asleep by 23:20 · from your usual wake 07:10 (12 recorded wake times)' : null,
      reason: sleepOnly ? 'Garmin raised tonight’s need after several short nights, and added 15 min after your nap.' : 'Garmin added 20 min for recent short sleep.',
      savedNeed: sleepOnly ? 490 : 490,
      updated: sleepOnly && ctx.napsToday.length ? 'Need updated 13:58 after Garmin’s nap adjustment. The plan saved this morning for timing observation stays 8h 10.' : null,
    };
  }

  function versionFor(ctx) {
    const wk = weekId(ctx.calendarDate);
    let base = null;
    const versions = ['mixed', 'sparse'].includes(ctx.state.profile) ? D.commitmentVersions : ctx.state.ownCommitments ? [ctx.state.ownCommitments] : [];
    for (const v of versions) if (v.from <= wk) base = v;
    const revisions = ctx.state.revisions.filter(r => r.week === wk && r.from <= ctx.calendarDate);
    return {wk, base, revisions};
  }

  function planStatus(ctx) {
    if (!ctx.world.commitments) return null;
    const {wk, base, revisions} = versionFor(ctx);
    if (!base) return null;
    const inWeek = ctx.world.activities.filter(a => between(a.date, ctx.weekFrom, ctx.calendarDate));
    const strength = inWeek.filter(a => a.sport === 'strength').length;
    const im = inWeek.reduce((n, a) => n + (a.im || 0), 0);
    const last = revisions.at(-1);
    const target = {strength: last ? last.strength : base.strength, im: last ? last.im : base.im};
    const ended = (ctx.state.pauses || []).filter(x => x.since <= ctx.calendarDate && x.until >= ctx.weekFrom).at(-1);
    const pause = ctx.state.pause && ctx.state.pause.since <= ctx.calendarDate ? ctx.state.pause : ended || null;
    const wholePause = pause && !pause.until && pause.since <= ctx.weekFrom;
    const midPause = pause && !wholePause && pause.since <= ctx.calendarDate;
    const daysLeft = Math.max(0, Math.round((parse(ctx.weekTo) - parse(ctx.calendarDate)) / 86400000));
    const incomplete = ctx.world.coverageIncompleteFrom;
    const track = (key, have, unit) => {
      const t = target[key], orig = base[key];
      const rev = revisions.find(r => r[key] !== orig);
      const out = {key, have, target: t, original: orig, unit};
      if (wholePause) return {...out, state: 'paused', label: 'Paused', detail: `${have} recorded · original target ${orig}${unit}`};
      if (midPause) return {...out, state: 'adjusted-paused', label: pause.until ? `Adjusted · paused ${shortDay(pause.since)}–${shortDay(pause.until)}` : `Adjusted · paused since ${shortDay(pause.since)}`, detail: `${have}${unit} recorded · original target ${orig}${unit} · no pace verdict or proration`};
      if (incomplete && have < t) return {...out, state: 'unknown', label: 'Awaiting evidence', detail: `${have}${unit} known · activity list incomplete since ${dayLabel(incomplete)}`};
      if (have >= t && rev) return {...out, state: 'reached-adjusted', label: `Reached · adjusted ${shortDay(rev.from)}`, detail: `${have}${unit} of ${t}${unit} (was ${orig}${unit})`};
      if (have >= t) return {...out, state: 'reached', label: 'Reached', detail: `${have}${unit} of ${t}${unit}`};
      return {...out, state: 'open', label: `${have} of ${t}${unit}`, detail: rev ? `Adjusted ${shortDay(rev.from)} from ${orig}${unit} · ${daysLeft} days left` : `${daysLeft} days left`};
    };
    return {wk, base, revisions, pause, wholePause, midPause, daysLeft,
      tracks: [track('strength', strength, ''), track('im', im, ' min')].filter(t => t.target > 0),
      strength, im, target};
  }

  function notices(ctx) {
    const list = [];
    if (ctx.stale) list.push({id: 'stale', text: 'Your watch hasn’t uploaded to Garmin Connect for 2 days.', action: {label: 'Sync guidance', go: 'help/sync'}});
    if (ctx.state.vitalsException) list.push({id: 'vitals', text: 'Two overnight readings are outside your usual ranges. This isn’t a diagnosis; check how you feel.', action: {label: 'Health', go: 'health'}});
    if (ctx.reviewDue) list.push({id: 'review', text: `Your pause review date (${dayLabel(ctx.paused.review)}) has arrived. Training stays paused until you resume.`, action: {label: 'Review pause', go: 'adapt/pause'}});
    return list;
  }

  function trainBlocked(ctx) {
    if (ctx.paused) return 'paused';
    if (ctx.restActive) return 'rest';
    return null;
  }

  function contribution(ctx, a) {
    if (a.timerOnly) return 'Counted as a strength session · no sets recorded';
    const p = ctx.plan;
    if (p && a.sport === 'strength' && !p.wholePause) {
      const before = ctx.world.activities.filter(x => x.sport === 'strength' && between(x.date, ctx.weekFrom, a.date) && (x.date < a.date || x.start <= a.start)).length;
      return `Counts toward strength: ${before} of ${p.target.strength} this week`;
    }
    if (p && a.im) return `+${a.im} intensity minutes toward your ${p.target.im}`;
    if (a.record) return `Best in available history: ${a.record}`;
    return `${SPORTS[a.sport]} recorded · ${a.min} min`;
  }

  function lead(ctx) {
    const s = ctx.state, r = ctx.readiness, n = ctx.night;
    const trace = [
      {gate: 'Privacy', result: 'Journal entries never feed Today; nothing private is used.'},
    ];
    const done = (l) => ({...l, trace});
    const evidence = [];
    if (ctx.phase === 'waiting') {
      trace.push({gate: 'Evidence', result: 'Last night has not synced; yesterday’s readiness is not presented as today’s.'});
      return done({id: `wait:${ctx.calendarDate}`, kind: 'waiting', headline: 'Last night hasn’t synced yet',
        lines: [`Your watch last uploaded ${ctx.upload.detail}. Sleep and readiness appear once it syncs.`],
        action: {label: 'How syncing works', go: 'help/sync'}});
    }
    if (ctx.stale) {
      trace.push({gate: 'Evidence', result: 'No upload for 2 days and no night or readiness for today; no effort suggestion is possible.'});
      return done({id: `stale:${ctx.calendarDate}`, kind: 'stale', headline: 'Waiting for your watch',
        lines: ['Readings here are from Mon 16 Nov. Open Garmin Connect on your phone to sync the watch.'],
        action: {label: 'Sync guidance', go: 'help/sync'}});
    }
    evidence.push(n ? `Last night ${dur(n.asleep)} of ${dur(n.need)} need` : 'No recorded night');
    evidence.push(r ? `Readiness ${r.value} ${r.level}` : ctx.world.readinessSupported ? 'Readiness missing' : 'Readiness not recorded by this watch');
    trace.push({gate: 'Evidence', result: `${evidence.join(' · ')} · watch upload ${ctx.upload.label}.`});
    if (ctx.fixedReason) return done({id: `fixed:${ctx.calendarDate}`, kind: 'quiet', headline: 'Your recorded context',
      lines: ['Fixed presentation: no assumed sleep schedule or training window.'], action: {label: 'View sleep', go: 'sleep'}});
    const blocked = trainBlocked(ctx);
    trace.push({gate: 'Circumstances', result: blocked === 'paused' ? `Training paused since ${dayLabel(ctx.paused.since)}: no exercise prompts.`
      : blocked === 'rest' ? 'Rest today: no exercise prompts today or tonight.'
      : ctx.hasLimits ? `Limits: ${[ctx.limits.exclude.map(x => SPORTS[x]).join(', ') && `excluded ${ctx.limits.exclude.map(x => SPORTS[x]).join(', ')}`, ctx.limits.note && 'movement note (routines need review)'].filter(Boolean).join('; ')}.` : 'None active.'});

    const sleepPhase = ['evening', 'winddown', 'late'].includes(ctx.phase);
    const tn = ctx.tonight;
    if (sleepPhase) {
      trace.push({gate: 'Decisions', result: ctx.intentionsToday.length ? 'Tonight’s intention stays in Plan; it is not deleted or nagged.' : 'Nothing planned tonight.'});
      trace.push({gate: 'Time', result: `${CLOCKS[ctx.state.clock].label}: sleep preparation leads; no exercise deficit or countdown.`});
      if (ctx.phase === 'late') return done({id: `late:${tn.nightOf}`, kind: 'sleep', headline: 'Rest when you’re ready',
        lines: [`Still ${tn.nightLabel}. No countdown; sleep when you can.`], action: {label: 'Tonight’s sleep', go: 'sleep/tonight'}});
      if (ctx.phase === 'winddown') return done({id: `wind:${tn.nightOf}`, kind: 'sleep', headline: 'Time to wind down',
        lines: [`Garmin’s window ${tn.window} · need ${dur(tn.need)}.`], action: {label: 'Tonight’s plan', go: 'sleep/tonight'}});
      return done({id: `evening:${tn.nightOf}`, kind: 'sleep', headline: 'Make room for sleep',
        lines: [`Wind down from ${tn.windDown} · Garmin’s window ${tn.window}.`], action: {label: 'See tonight’s plan', go: 'sleep/tonight'}});
    }
    if (blocked === 'paused') {
      trace.push({gate: 'Decisions', result: 'Commitments are kept for review when you resume; nothing to catch up.'});
      return done({id: `paused:${ctx.calendarDate}`, kind: 'paused', headline: 'Training is paused',
        lines: [`Since ${dayLabel(ctx.paused.since)}, until you resume.`, r ? `Readiness ${r.value} (${r.level}) is shown as context, not a reason to train.` : 'Your recorded data is all still here.'],
        action: {label: 'Review pause', go: 'adapt/pause'}});
    }
    if (blocked === 'rest') {
      trace.push({gate: 'Decisions', result: 'Your rest day outranks readiness and any schedule.'});
      return done({id: `rest:${ctx.displayedDay}`, kind: 'rest', headline: 'Rest day',
        lines: [r && r.value >= 70 ? `Readiness is ${r.level} (${r.value}). Good to know; your rest day stands.` : 'No exercise prompts today or tonight.',
          ctx.scheduleToday.length ? `${ctx.scheduleToday[0].name} stays on your Garmin calendar, unchanged.` : null].filter(Boolean),
        action: {label: 'View plan', go: 'plan'}});
    }
    // Recognition: a recorded session today leads; a day can finish.
    const recent = ctx.actsToday.filter(a => a.sport !== 'walking' || !ctx.world.commitments).at(-1);
    const sched = ctx.scheduleToday.find(x => !s.dismissed[`sched:${x.id}:${ctx.calendarDate}`]);
    if (recent && !(ctx.phase === 'morning' && recent.start > CLOCKS[ctx.state.clock].time)) {
      const p = ctx.plan;
      const reached = p && recent.sport === 'strength' && p.tracks.find(t => t.key === 'strength')?.state === 'reached' && p.strength === p.target.strength;
      trace.push({gate: 'Decisions', result: `${SPORTS[recent.sport]} recorded at ${recent.start}; recognition leads and no new task is created.`});
      return done({id: `done:${recent.id}`, kind: 'done', celebrate: reached ? `target:${p.wk}:strength` : null,
        headline: reached ? 'Strength target reached' : `${SPORTS[recent.sport]} recorded`,
        lines: [reached ? `${p.target.strength} of ${p.target.strength} sessions this week. That stays done; no higher target follows.` : contribution(ctx, recent)],
        action: {label: 'Open workout', go: `workout/${recent.id}`}});
    }
    if (sched) {
      const conflict = ctx.limits.exclude.includes(sched.sport);
      if (ctx.limits.note && !conflict) return done({id: `review:${sched.id}`, kind: 'schedule', headline: 'Review needed before this session',
        lines: ['Its movements are unknown to Raring. Your choice to use it isn’t exercise clearance.'],
        action: {label: 'Review session', go: `scheduled/${sched.id}`}});
      const lowR = r && r.value < 50;
      trace.push({gate: 'Decisions', result: `Garmin schedule: ${sched.name} today (read-only).`});
      trace.push({gate: 'Garmin guidance', result: conflict ? 'Conflicts with your excluded activity; shown, not changed.' : lowR ? 'Readiness Low: an easier day may fit; the session is not cancelled.' : 'Readiness supports the session as planned.'});
      return done({id: `sched:${sched.id}:${ctx.calendarDate}`, kind: 'schedule', dismissable: true,
        headline: conflict ? 'A Garmin session conflicts with your limits' : lowR ? 'An easier day may fit' : `${sched.name} is on your Garmin schedule`,
        lines: conflict ? [`${sched.name} is still on your Garmin calendar. Raring hasn’t changed it.`]
          : lowR ? [`Readiness is Low (${r.value}) after a short night.`, `${sched.name} stays on your Garmin calendar.`]
          : [r ? `Readiness ${r.level} (${r.value}).` : 'Readiness unavailable.'],
        action: {label: 'Review session', go: `scheduled/${sched.id}`}});
    }
    const intent = ctx.intentionsToday.find(i => !i.link);
    if (intent) {
      if (ctx.limits.note) return done({id: `review-intent:${intent.id}`, kind: 'intention', headline: 'Review your planned session',
        lines: [`${SPORTS[intent.sport]} is your intention; its movements are unknown to Raring. Your movement note still applies.`],
        action: {label: 'Review session', go: `intention/${intent.id}`}});
      trace.push({gate: 'Decisions', result: 'Your own intention for today leads; current evidence qualifies it.'});
      return done({id: `intent:${intent.id}`, kind: 'intention', headline: `${SPORTS[intent.sport]} ${intent.time ? `at ${intent.time}` : 'today'}`,
        lines: [`In your plan${ctx.limits.exclude.includes(intent.sport) ? ', but you excluded this activity' : ''}. ${r ? `Readiness ${r.level} (${r.value}).` : ''}`],
        action: {label: 'Review session', go: `intention/${intent.id}`}});
    }
    trace.push({gate: 'Decisions', result: 'Nothing scheduled, planned or recorded yet today.'});
    if (!r) {
      trace.push({gate: 'Garmin guidance', result: 'No readiness, so no effort suggestion; sleep leads.'});
      return done({id: `sleep:${ctx.displayedDay}`, kind: 'sleep', headline: n ? `You slept ${dur(n.asleep)}` : 'No sleep recorded last night',
        lines: n ? [`${n.asleep < n.need ? dur(n.need - n.asleep) + ' below' : n.asleep === n.need ? 'Matched' : dur(n.asleep - n.need) + ' above'} last night’s need of ${dur(n.need)}.`, `Sleep score ${n.score} (Garmin).${ctx.world.readinessSupported ? '' : ' This watch doesn’t record readiness.'}`] : ['Your watch may not have been worn.'],
        action: {label: 'Last night’s sleep', go: 'sleep'}});
    }
    const id = `suggest:${ctx.calendarDate}`;
    if (ctx.limits.note) return done({id: `limits:${ctx.calendarDate}`, kind: 'quiet', headline: 'Your movement limits apply',
      lines: ['Workout contents need review; readiness cannot establish compatibility.'], action: {label: 'Review limits', go: 'adapt/limits'}});
    if (s.dismissed[id]) {
      trace.push({gate: 'Relevance', result: 'You chose “Not today” for training ideas; it stays dismissed after sync and resume.'});
      return done({id: 'quiet', kind: 'quiet', headline: 'Nothing else needs you today', lines: ['You chose “Not today” for training ideas.'],
        action: {label: 'Tonight’s plan', go: 'sleep/tonight'}});
    }
    if (r.value >= 75) {
      trace.push({gate: 'Garmin guidance', result: `Readiness ${r.level}: Garmin’s guidance allows hard effort if it fits.`});
      trace.push({gate: 'Relevance', result: 'No availability declared: offer planning, not a command or required load.'});
      return done({id, kind: 'suggest', dismissable: true, headline: 'Training is an option today',
        lines: [`Readiness ${r.level} (${r.value}) after ${dur(n.asleep)} of sleep.`, ctx.hasLimits ? 'Your limits apply to anything you plan.' : 'Only if it fits you and your week.'],
        action: {label: 'Plan a session', sheet: 'add'}});
    }
    trace.push({gate: 'Garmin guidance', result: `Readiness ${r.level}: Garmin suggests easier effort.`});
    return done({id, kind: 'easy', dismissable: true, headline: r.value < 50 ? 'An easier day may fit' : 'A steady day',
      lines: [`Readiness ${r.level} (${r.value}); ${r.limit.toLowerCase()} holds it back most.`],
      action: {label: 'Why readiness is ' + r.level.toLowerCase(), go: 'readiness'}});
  }

  function surfaces(ctx) {
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
      status: {bb: ctx.stale || ctx.phase === 'waiting' ? null : D.bodyBattery[ctx.calendarDate] ?? null, stress: ctx.stale || ctx.phase === 'waiting' ? null : 27, hr: ctx.stale || ctx.phase === 'waiting' ? null : 61, focus, go: 'today'},
      bedtime: s.permission === 'granted' && s.notifyBedtime && ctx.phase !== 'waiting' ? {title: 'Wind-down', body: `Tonight’s window ${ctx.tonight.window || '—'} · wind down from ${ctx.tonight.windDown}`, go: 'sleep/tonight'} : null,
      weekly: s.permission === 'granted' && s.channels.weekly ? {title: 'Your week in review', body: 'One win, one observation, one choice', go: 'review/latest'} : null,
    };
  }

  root.RaringRules = {derive, surfaces, contribution, versionFor, CLOCKS, PROFILES, SPORTS, dayLabel, shortDay, dur, addDays, weekId, weekStart, between};
})(typeof window !== 'undefined' ? window : globalThis);
