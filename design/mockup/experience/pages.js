// Page renderers and actions for the experience prototype. Every value is synthetic.
(function () {
  const D = window.RaringData;
  const R = window.RaringRules;
  const A = () => window.RaringApp;
  const h = v => String(v ?? '').replace(/[&<>"']/g, c => ({'&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;'})[c]);
  const {dayLabel, shortDay, dur, addDays, SPORTS} = R;

  const ICONS = {
    today: '<circle cx="12" cy="12" r="4"/><path d="M12 2v2M12 20v2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M2 12h2M20 12h2M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"/>',
    plan: '<rect x="3" y="5" width="18" height="16" rx="3"/><path d="M3 10h18M8 3v4M16 3v4"/>',
    progress: '<path d="M4 19V9M10 19V5M16 19v-7M22 19H2"/>',
    explore: '<rect x="3" y="3" width="7" height="7" rx="2"/><rect x="14" y="3" width="7" height="7" rx="2"/><rect x="3" y="14" width="7" height="7" rx="2"/><rect x="14" y="14" width="7" height="7" rx="2"/>',
    settings: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.7 1.7 0 0 0 .3 1.8l.1.1a2 2 0 1 1-2.8 2.8l-.1-.1a1.7 1.7 0 0 0-1.8-.3 1.7 1.7 0 0 0-1 1.5V21a2 2 0 1 1-4 0v-.1a1.7 1.7 0 0 0-1.1-1.5 1.7 1.7 0 0 0-1.8.3l-.1.1a2 2 0 1 1-2.8-2.8l.1-.1a1.7 1.7 0 0 0 .3-1.8 1.7 1.7 0 0 0-1.5-1H3a2 2 0 1 1 0-4h.1a1.7 1.7 0 0 0 1.5-1.1 1.7 1.7 0 0 0-.3-1.8l-.1-.1a2 2 0 1 1 2.8-2.8l.1.1a1.7 1.7 0 0 0 1.8.3H9a1.7 1.7 0 0 0 1-1.5V3a2 2 0 1 1 4 0v.1a1.7 1.7 0 0 0 1 1.5 1.7 1.7 0 0 0 1.8-.3l.1-.1a2 2 0 1 1 2.8 2.8l-.1.1a1.7 1.7 0 0 0-.3 1.8V9a1.7 1.7 0 0 0 1.5 1H21a2 2 0 1 1 0 4h-.1a1.7 1.7 0 0 0-1.5 1z"/>',
    back: '<path d="M15 18l-6-6 6-6"/>',
    chev: '<path d="M9 18l6-6-6-6"/>',
    adapt: '<path d="M4 6h10M18 6h2M4 12h4M12 12h8M4 18h12M20 18h0"/><circle cx="16" cy="6" r="2"/><circle cx="10" cy="12" r="2"/><circle cx="18" cy="18" r="2"/>',
    moon: '<path d="M21 12.8A9 9 0 1 1 11.2 3a7 7 0 0 0 9.8 9.8z"/>',
    check: '<path d="M20 6L9 17l-5-5"/>',
    pause: '<path d="M8 5v14M16 5v14"/>',
    info: '<circle cx="12" cy="12" r="9"/><path d="M12 11v6M12 7.5v.5"/>',
    offline: '<path d="M2 2l20 20M8.5 16.5a5 5 0 0 1 7 0M5 12.6a10 10 0 0 1 5-2.6M19 12.6a10 10 0 0 0-2-1.5M12 20h.01"/>',
    heart: '<path d="M20.8 4.6a5.5 5.5 0 0 0-7.8 0L12 5.7l-1-1.1a5.5 5.5 0 0 0-7.8 7.8L12 21l8.8-8.6a5.5 5.5 0 0 0 0-7.8z"/>',
    bolt: '<path d="M13 2L3 14h8l-1 8 10-12h-8l1-8z"/>',
    dumbbell: '<path d="M6 7v10M18 7v10M3 10v4M21 10v4M6 12h12"/>',
    activity: '<path d="M22 12h-4l-3 9L9 3l-3 9H2"/>',
    search: '<circle cx="11" cy="11" r="7"/><path d="M21 21l-4.3-4.3"/>',
    close: '<path d="M18 6L6 18M6 6l12 12"/>',
    lock: '<rect x="4" y="11" width="16" height="10" rx="2"/><path d="M8 11V7a4 4 0 0 1 8 0v4"/>',
  };
  const icon = (name, cls = '') => `<svg class="ic ${cls}" viewBox="0 0 24 24" aria-hidden="true" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">${ICONS[name] || ''}</svg>`;

  const TITLES = {
    onboarding: 'Welcome', today: 'Today', plan: 'Plan', progress: 'Progress', explore: 'Explore', sleep: 'Sleep', readiness: 'Readiness', training: 'Training',
    strength: 'Strength', activities: 'Activities', health: 'Health', gear: 'Gear', trends: 'Trends & comparisons', reviews: 'Weekly reviews',
    settings: 'Settings', help: 'Help', adapt: 'Adapt', surfaces: 'Status surfaces', legacy: 'Old links', 'plan/commitments': 'Commitments',
    'health/weight': 'Weight & body composition', 'sleep/patterns': 'Sleep patterns',
  };
  function title(route) {
    if (TITLES[route]) return TITLES[route];
    const [a, b] = route.split('/');
    if (a === 'sleep') return 'Sleep';
    if (a === 'adapt') return 'Adapt';
    if (a === 'health') return 'Health';
    if (a === 'settings') return 'Settings';
    if (a === 'workout') { const x = activity(b); return x ? `${SPORTS[x.sport]} · ${shortDay(x.date)}` : 'Workout'; }
    if (a === 'metric') return D.metrics[b]?.name || 'Metric';
    if (a === 'strength') return D.exercises.find(e => e.id === b)?.name || 'Exercise';
    if (a === 'intention') return 'Your plan';
    if (a === 'scheduled') return 'Garmin schedule';
    if (a === 'review') return `Week review`;
    if (a === 'help') return D.helpPages[b]?.title || 'Help';
    return route;
  }
  const activity = id => D.activities.concat(D.extraActivity).find(a => a.id === id);

  // ---------- shell ----------
  function shell(env) {
    const {route, ctx, sheet} = env;
    const page = renderPage(env);
    const isRoot = ['today', 'plan', 'progress'].includes(route);
    return `
<div class="proto-strip" role="note"><span><b>Unshipped prototype</b> · synthetic data</span><button class="strip-btn" data-act="scenario" aria-haspopup="dialog">Scenario</button></div>
${env.notice ? `<div class="case-note" role="status"><span>${h(env.notice)}</span><button class="link" data-act="clearNotice" aria-label="Dismiss this message">${icon('close')}</button></div>` : ''}
<div class="layout ${isRoot ? 'is-root' : 'is-detail'}" ${sheet ? 'inert' : ''}>
  <main id="main" tabindex="-1" data-route="${h(route)}">${page}</main>
  <nav class="tabs" aria-label="Main">
    ${['today', 'plan', 'progress'].map(t => `<button data-go="${t}" class="tab" ${env.rootOf === t ? 'aria-current="page"' : ''}>${icon(t)}<span>${TITLES[t]}</span></button>`).join('')}
  </nav>
</div>
<div id="toast" class="toast" role="status" aria-live="polite"></div>
${sheet ? renderSheet(env) : ''}`;
  }

  function rootHeader(env, name) {
    const {ctx, state} = env;
    const adapt = name !== 'progress';
    const offline = state.offline ? `<span class="pill quiet">${icon('offline')} Offline · saved data</span>` : '';
    return `<header class="top">
  <p class="eyebrow">${h(dayLabel(ctx.calendarDate))} · <span class="fresh ${ctx.stale ? 'is-stale' : ''}">Watch upload ${h(ctx.upload.label)}</span></p>
  <div class="hrow">
    <div class="htext"><h1 tabindex="-1">${TITLES[name]}</h1></div>
    <div class="hctl"><button class="hbtn explore" data-go="explore">${icon('explore')}<span>Explore</span></button><button class="hbtn settings" data-go="settings" aria-label="Settings">${icon('settings')}<span class="sl">Settings</span></button></div>
  </div>
  ${adapt || offline ? `<div class="hrow2">${adapt ? `<button class="adapt ${ctx.adaptLabel !== 'Adapt today' ? 'on' : ''}" data-sheet="adapt" aria-haspopup="dialog">${icon(ctx.paused ? 'pause' : ctx.restActive ? 'moon' : 'adapt')}<span>${h(ctx.adaptLabel)}</span></button>` : ''}${offline}${!ctx.world.historyComplete && name === 'today' ? `<span class="pill quiet">History since Sep 2026 · older history downloading</span>` : ''}</div>` : ''}
</header>`;
  }
  function detailHeader(env, t, eyebrow, extra = '') {
    return `<header class="top detail">
  <button class="back" data-act="back">${icon('back')}<span>${h(env.parentLabel)}</span></button>
  <div class="hrow"><div class="htext">${eyebrow ? `<p class="eyebrow">${eyebrow}</p>` : ''}<h1 tabindex="-1">${h(t)}</h1></div>${extra}</div>
</header>`;
  }
  const card = (t, body, cls = '', id = '') => `<section class="card ${cls}" ${id ? `id="${id}"` : ''} ${t ? `aria-labelledby="${id || slug(t)}-t"` : ''}>${t ? `<h2 class="ct" id="${id || slug(t)}-t">${t}</h2>` : ''}${body}</section>`;
  const slug = t => String(t).toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/(^-|-$)/g, '');
  const row = (label, value, go, sub = '', attrs = '') => `<button class="row" ${go ? `data-go="${go}"` : ''} ${attrs}><span class="rl"><span>${label}</span>${sub ? `<small>${sub}</small>` : ''}</span><span class="rv">${value}</span>${icon('chev', 'chev')}</button>`;
  const tag = (text, kind = '') => `<span class="tag ${kind}">${text}</span>`;

  function renderPage(env) {
    const r = env.route;
    const [a, b] = r.split('/');
    switch (a) {
      case 'onboarding': return detailHeader(env, 'Welcome to Raring', 'Prototype only · no sign-in here') + card('Your Garmin data, on your phone', '<p>Raring reads Garmin Connect, not the watch directly. This unofficial connection depends on the data your watch records. No Raring server or analytics.</p><p class="small">Production sign-in stays in the existing app. This isolated prototype accepts no credentials and does not open an account.</p><button class="primary wide" data-go="today">Continue with synthetic data</button>');
      case 'today': return today(env);
      case 'plan': return b === 'commitments' ? commitments(env) : plan(env);
      case 'progress': return progress(env);
      case 'explore': return explore(env);
      case 'sleep': return b === 'patterns' ? sleepPatterns(env) : sleep(env);
      case 'readiness': return readiness(env);
      case 'training': return b === 'performance' ? detailHeader(env, 'Performance readings', 'Garmin · independently dated') + window.RaringEvidence.performance() : training(env);
      case 'gear': return gear(env);
      case 'strength': return b === 'muscles' ? detailHeader(env, 'Muscle coverage', 'Recorded sets, not recovery') + window.RaringEvidence.muscles() : b ? exercise(env, b) : strength(env);
      case 'activities': return activities(env);
      case 'workout': return env.route.endsWith('/evidence') ? detailHeader(env, 'Workout evidence', 'Synthetic Garmin record') + window.RaringEvidence.workout(activity(b)) : workout(env, b);
      case 'health': return b === 'weight' ? weight(env) : health(env);
      case 'metric': return metric(env, b);
      case 'trends': return trends(env);
      case 'intention': return intention(env, b);
      case 'scheduled': return scheduled(env, b);
      case 'review': return review(env, b);
      case 'reviews': return reviews(env);
      case 'adapt': return adapt(env);
      case 'settings': return settings(env);
      case 'help': return b ? helpPage(env, b) : help(env);
      case 'surfaces': return surfaces(env);
      case 'legacy': return legacy(env);
      default: return today(env);
    }
  }

  // ---------- Today ----------
  const KICKER = {waiting: 'Waiting for data', stale: 'Waiting for data', sleep: 'Sleep', paused: 'Training paused', rest: 'Your rest day', done: 'Recorded today',
    schedule: 'Garmin schedule', intention: 'Your plan', suggest: 'Readiness', easy: 'Readiness', quiet: 'Today'};
  function leadCard(env) {
    const {ctx, state} = env;
    const l = ctx.lead;
    const act = l.action.sheet ? `data-sheet="${l.action.sheet}"` : `data-go="${l.action.go}"`;
    const celebrate = l.celebrate && !state.quiet ? `<span class="celebrate ${state.celebrated[l.celebrate] ? 'seen' : ''}" aria-hidden="true">${icon('check')}</span>` : '';
    return `<section class="lead k-${l.kind}" aria-labelledby="lead-h">
  <p class="kicker">${icon(l.kind === 'sleep' ? 'moon' : l.kind === 'paused' ? 'pause' : l.kind === 'done' ? 'check' : l.kind === 'rest' ? 'moon' : 'info')}<span>${KICKER[l.kind]}</span></p>
  <h2 id="lead-h">${celebrate}${h(l.headline)}</h2>
  ${l.lines.map(x => `<p class="lead-line">${h(x)}</p>`).join('')}
  <div class="lead-actions"><button class="primary" ${act}>${h(l.action.label)}</button><button class="link" data-sheet="why">Why this?</button>${l.dismissable ? `<button class="link" data-act="dismiss" data-arg="${h(l.id)}">Not today</button>` : ''}</div>
</section>`;
  }
  function facts(env) {
    const {ctx} = env;
    const waiting = ctx.phase === 'waiting' || ctx.stale;
    const n = ctx.night, r = ctx.readiness;
    const bb = waiting ? null : D.bodyBattery[ctx.calendarDate];
    const f = [];
    // After midnight the readings still belong to the night that ended this morning; say whose they are.
    const of = ctx.calendarDate !== ctx.displayedDay ? `${shortDay(ctx.displayedDay)} · ` : '';
    f.push({label: 'Sleep', go: 'sleep', value: waiting ? 'Waiting' : n ? dur(n.asleep) : 'Not recorded', sub: waiting ? 'Last night not synced' : n ? `${of}${n.asleep < n.need ? 'Below' : 'Met'} need ${dur(n.need)}` : 'No night recorded'});
    f.push({label: 'Readiness', go: 'readiness', value: !ctx.world.readinessSupported ? 'Not on watch' : waiting ? 'Waiting' : r ? `${r.value}` : 'Missing', sub: r && !waiting ? `${of}${r.level}` : !ctx.world.readinessSupported ? 'This watch can’t record it' : 'No reading yet'});
    f.push({label: 'Body Battery', go: 'metric/bodyBattery', value: bb == null ? '—' : `${bb}`, sub: bb == null ? 'Waiting for upload' : 'At last upload'});
    return `<section class="facts" aria-label="Right now">${f.map(x => `<button class="fact" data-go="${x.go}"><span class="fl">${x.label}</span><b class="n">${h(x.value)}</b><small>${h(x.sub)}</small></button>`).join('')}</section>`;
  }
  function eventRows(env) {
    const {ctx, state} = env;
    const items = [
      ...ctx.actsToday.filter(a => ctx.phase !== 'morning' || a.start <= ctx.clock.time).map(a => ({t: a.start, html: row(`${SPORTS[a.sport]} · ${a.start}`, `${a.min} min`, `workout/${a.id}`, h(R.contribution(ctx, a)))})),
      ...ctx.napsToday.map(n => ({t: n.start[1], html: row(`Nap · ${n.start[1]}${n.end ? `–${n.end[1]}` : ''}`, `${n.min} min`, 'sleep/naps', 'Recorded separately from overnight sleep')})),
    ].sort((x, y) => x.t.localeCompare(y.t));
    const shown = items.slice(0, 2);
    const label = ctx.calendarDate === ctx.displayedDay ? 'Today so far' : `${dayLabel(ctx.calendarDate)} so far`;
    const late = ctx.lateArrivals.length ? `<p class="small">${ctx.lateArrivals.length} activity from ${dayLabel(ctx.lateArrivals[0].date)} arrived late and counts on that day. <button class="link inline" data-act="activitiesFor" data-arg="${ctx.lateArrivals[0].date}">See it</button></p>` : '';
    const yesterday = ctx.phase === 'waiting' ? D.activities.filter(a => a.date === ctx.displayedDay && ctx.world.activities.includes(a)) : [];
    return `<section class="counted" aria-labelledby="counted-t"><h2 class="ct" id="counted-t">${label}</h2>
  ${shown.length ? shown.map(x => x.html).join('') : `<p class="small">Nothing recorded yet${ctx.calendarDate === ctx.displayedDay ? ' today' : ''}.</p>`}
  ${yesterday.length ? `<p class="small">Still available: ${yesterday.map(a => `<button class="link inline" data-go="workout/${a.id}">${SPORTS[a.sport]} · ${dayLabel(a.date)}</button>`).join(', ')}</p>` : ''}
  ${items.length > 2 ? `<button class="link" data-act="activitiesFor" data-arg="${ctx.calendarDate}">All ${items.length} from today</button>` : ''}
  ${late}
</section>`;
  }
  function supportLine(env) {
    const {ctx} = env;
    const l = ctx.lead;
    let text, go;
    const pending = ctx.intentionsToday.find(i => !i.link);
    if (l.kind === 'sleep' && ['evening', 'winddown', 'late'].includes(ctx.phase)) {
      if (pending && ctx.phase !== 'late') { text = `${SPORTS[pending.sport]} ${pending.time ? `at ${pending.time} ` : ''}is still in your plan, with no need to fit it in.`; go = `intention/${pending.id}`; }
      else if (ctx.plan && ctx.phase === 'evening') { text = `This week: ${ctx.plan.strength} strength sessions recorded`; go = 'plan'; }
      else return '';
    } else if (ctx.paused) { text = 'Your commitments are kept for review when you resume.'; go = 'plan'; }
    else if (ctx.phase === 'waiting' || ctx.stale) { text = ctx.stale ? `Tonight: ${ctx.tonight.estimate}` : 'Tonight’s plan appears after last night syncs.'; go = 'sleep/tonight'; }
    else { text = `Tonight: wind down from ${ctx.tonight.windDown} · Garmin’s window ${ctx.tonight.window}`; go = 'sleep/tonight'; }
    return `<button class="support" data-go="${go}">${icon(go.startsWith('sleep') ? 'moon' : 'plan')}<span>${h(text)}</span>${icon('chev', 'chev')}</button>`;
  }
  function noticeBlock(env) {
    const list = env.ctx.notices;
    if (!list.length) return '';
    const n = list[0];
    return `<section class="notice" aria-label="Notice">${icon('info')}<p>${h(n.text)}</p><div class="notice-act"><button class="link" data-go="${n.action.go}">${h(n.action.label)}</button>${list.length > 1 ? `<button class="link" data-sheet="notices">${list.length - 1} more notice${list.length > 2 ? 's' : ''}</button>` : ''}</div></section>`;
  }
  function today(env) {
    const {ctx} = env;
    return `${rootHeader(env, 'today')}
${noticeBlock(env)}
${leadCard(env)}
${ctx.fixedReason ? `<p class="small fixed-reason">${icon('info')} ${h(ctx.fixedReason)} <button class="link inline" data-go="settings">Settings</button></p>` : ''}
${env.state.dst ? `<p class="small fixed-reason">${icon('info')} Clocks changed by an hour overnight. Tonight’s plan keeps your usual schedule.</p>` : ''}
${facts(env)}
${eventRows(env)}
${supportLine(env)}
<p class="end">That’s today. <button class="link inline" data-go="explore">Explore all data</button></p>`;
  }

  // ---------- Plan ----------
  function weekStrip(env) {
    const {ctx, state} = env;
    const days = [...Array(7)].map((_, i) => addDays(ctx.weekFrom, i));
    const cells = days.map(d => {
      const marks = [];
      ctx.world.schedule.filter(s => s.date === d).forEach(s => marks.push(['garmin', `Garmin: ${s.name}`]));
      state.intentions.filter(i => i.date === d).forEach(i => marks.push(['intent', `Your plan: ${SPORTS[i.sport]}${i.link ? ' (linked)' : ''}`]));
      ctx.world.activities.filter(a => a.date === d && d <= ctx.calendarDate).forEach(a => marks.push(['done', `Recorded: ${SPORTS[a.sport]}`]));
      if (state.rest && state.rest.day === d) marks.push(['rest', 'Rest day']);
      const paused = state.pause && state.pause.since <= d;
      const isToday = d === ctx.calendarDate;
      return `<li class="day ${isToday ? 'is-today' : ''} ${paused ? 'is-paused' : ''}" aria-label="${dayLabel(d)}${isToday ? ', today' : ''}${paused ? ', paused' : ''}: ${marks.map(m => m[1]).join('; ') || 'nothing'}"><span class="dw">${shortDay(d)}</span><span class="dd n">${Number(d.slice(8))}</span><span class="marks">${marks.map(m => `<i class="m-${m[0]}" title="${h(m[1])}"></i>`).join('')}</span>${paused ? '<span class="pz">Paused</span>' : ''}</li>`;
    }).join('');
    return `<ol class="strip" aria-label="This week">${cells}</ol>
<p class="legend" aria-hidden="true"><span><i class="m-garmin"></i>Garmin schedule</span><span><i class="m-intent"></i>Your plan</span><span><i class="m-done"></i>Recorded</span><span><i class="m-rest"></i>Rest</span></p>`;
  }
  function comingUp(env) {
    const {ctx, state} = env;
    const days = [...Array(7)].map((_, i) => addDays(ctx.calendarDate, i));
    const out = [];
    for (const d of days) {
      const items = [];
      if (state.rest && state.rest.day === d) items.push(`<div class="item k-rest"><span class="it">${tag('Rest day', 'rest')}<span>No exercise prompts${d === ctx.calendarDate ? ' today or tonight' : ''}</span></span></div>`);
      ctx.world.schedule.filter(s => s.date === d).forEach(s => {
        const conflict = state.limits.exclude.includes(s.sport);
        const review = s.routine && state.limits.note && state.limits.routines[s.id] !== 'chosen';
        items.push(`<button class="item k-garmin" data-go="scheduled/${s.id}"><span class="it">${tag('Garmin schedule', 'garmin')}<span>${h(s.name)}<small>Read-only${conflict ? ' · conflicts with your excluded activity' : review ? ' · review needed for your movement note' : ''}${ctx.paused ? ' · paused for you' : ''}</small></span></span>${icon('chev', 'chev')}</button>`);
      });
      state.intentions.filter(i => i.date === d).forEach(i => {
        items.push(`<button class="item k-intent" data-go="intention/${i.id}"><span class="it">${tag('Your plan', 'intent')}<span>${SPORTS[i.sport]}${i.time ? ` · ${i.time}` : ''}<small>${i.link ? 'Linked by you to a recorded activity' : d < ctx.calendarDate ? 'Not linked' : 'On this phone, not Garmin'}</small></span></span>${icon('chev', 'chev')}</button>`);
      });
      if (d === ctx.calendarDate) ctx.actsToday.forEach(a => items.push(`<button class="item k-done" data-go="workout/${a.id}"><span class="it">${tag('Recorded', 'done')}<span>${SPORTS[a.sport]} · ${a.start}<small>${h(R.contribution(ctx, a))}</small></span></span>${icon('chev', 'chev')}</button>`));
      if (items.length) out.push(`<div class="dayblock"><h3>${d === ctx.calendarDate ? 'Today' : dayLabel(d)}</h3>${items.join('')}</div>`);
    }
    return out.join('') || `<p class="small">Nothing planned in the next 7 days. That’s fine; add something when you want to.</p>`;
  }
  function coverage(env) {
    const c = env.state.calendar;
    if (!env.ctx.world.schedule.length) return '';
    return `<p class="small">${c === 'complete' ? 'Garmin schedule checked 07:12, covering 18 Nov–1 Dec. Coach plan: none active.' : c === 'partial' ? 'Garmin returned a full page of events, so later items may be missing. Not “nothing planned”.' : 'Garmin schedule may be out of date: last checked Mon 16 Nov. Not “nothing planned”.'}</p>`;
  }
  function tracksHtml(ctx, compact) {
    if (!ctx.plan) return '';
    return ctx.plan.tracks.map(t => {
      const pct = Math.min(100, Math.round(t.have / Math.max(1, t.target) * 100));
      const viz = t.key === 'strength'
        ? `<span class="sessions" aria-hidden="true">${[...Array(Math.max(t.target, t.have))].map((_, i) => `<i class="${i < t.have ? 'on' : ''}"></i>`).join('')}</span>`
        : `<span class="amount" aria-hidden="true"><i style="width:${pct}%"></i></span>`;
      return `<div class="track s-${t.state}"><div class="tl"><span>${t.key === 'strength' ? 'Strength sessions' : 'Intensity minutes'}</span>${tag(h(t.label), t.state)}</div>${viz}${compact ? '' : `<small>${h(t.detail)}</small>`}</div>`;
    }).join('');
  }
  function plan(env) {
    const {ctx, state} = env;
    const migration = state.savedSleepTarget && !state.seen.timingMigration ? card('A change to sleep timing', `<p>Your saved “asleep on time” target is now a <b>timing observation</b>: Raring shows recorded nights against your saved plan without behind or on-track verdicts. Past weeks keep their original results; this week is marked adjusted.</p><div class="btns"><button class="secondary" data-act="seenMigration">Got it</button><button class="link" data-act="seenMigration" data-arg="winddown">Add a wind-down intention</button></div>`, 'info') : '';
    const circ = ctx.paused || ctx.hasLimits || ctx.restActive ? `<button class="circ" data-go="adapt">${icon(ctx.paused ? 'pause' : 'adapt')}<span>${ctx.paused ? `Training paused since ${dayLabel(ctx.paused.since)}. Resume when you’re ready.` : ctx.restActive ? 'Rest today: no exercise prompts today or tonight.' : `Your limits: ${[state.limits.exclude.map(x => SPORTS[x]).join(', ') && `no ${state.limits.exclude.map(x => SPORTS[x].toLowerCase()).join(', ')}`, state.limits.note && `“${state.limits.note}”`].filter(Boolean).join(' · ')}`}</span>${icon('chev', 'chev')}</button>` : '';
    const ambitions = [...ctx.world.ambitions, ...state.ambitionsAdded];
    return `${rootHeader(env, 'plan')}
${migration}${circ}
${card('This week', `${weekStrip(env)}<button class="primary wide" data-sheet="add">Add to my plan</button>`, '', 'sec-week')}
${card('Coming up', comingUp(env) + coverage(env), '', 'sec-coming')}
${state.intentions.some(i => i.date < ctx.calendarDate) ? card('Earlier intentions', state.intentions.filter(i => i.date < ctx.calendarDate).map(i => row(`${SPORTS[i.sport]} · ${dayLabel(i.date)}`, i.link ? 'Linked by you' : 'Not linked', `intention/${i.id}`, 'Not a missed-workout verdict')).join('')) : ''}
${card('Commitments', ctx.plan ? `${ctx.plan.midPause || ctx.plan.wholePause ? '' : ''}${tracksHtml(ctx)}<p class="small">Plan week ${dayLabel(ctx.weekFrom)}–${dayLabel(ctx.weekTo)}, aligned with Garmin’s intensity week. Intensity minutes are Garmin’s moderate + 2 × vigorous.</p><button class="secondary" data-go="plan/commitments">Edit commitments</button>`
      : `<p>No commitments. You don’t need any to use Raring.</p><p class="small">Garmin’s own weekly intensity goal is 150 min (Garmin’s, not one you chose here).</p><button class="secondary" data-go="plan/commitments">Add a commitment</button>`, '', 'sec-commitments')}
${card('Ambitions', ambitions.length ? ambitions.map(a => `<div class="amb"><b>${h(a.name)}</b><small>${a.date ? `${dayLabel(a.date)} · ${Math.round((Date.parse(a.date) - Date.parse(ctx.calendarDate)) / 864e5)} days · ${h(a.source)} · a date, not a readiness verdict` : h(a.source)}</small></div>`).join('') + `<button class="link" data-sheet="ambition">Add an ambition</button>`
      : `<p class="small">Optional. Events or directions can sit alongside each other; none has to come first.</p><button class="link" data-sheet="ambition">Add an ambition</button>`, '', 'sec-ambitions')}
${state.rehab.items.length || ctx.hasLimits ? card('Routines', `<p class="small">${state.rehab.items.length ? `${state.rehab.items.length} item${state.rehab.items.length > 1 ? 's' : ''} in your checklist (self-reported).` : ''} ${ctx.hasLimits ? 'Garmin routines with unknown contents need your review.' : ''}</p><button class="secondary" data-go="adapt/routines">My routines</button>`) : ''}`;
  }

  function commitments(env) {
    const {ctx, state} = env;
    const p = ctx.plan;
    if (!p) return `${detailHeader(env, 'Commitments', 'Optional')}
${card('Add a commitment', `<p class="small">Just a quantity and a period. Rest days and other refinements are optional. Garmin’s weekly intensity goal already exists, so it isn’t duplicated unless you change it.</p>
<div class="stepper-row"><span>Strength sessions a week</span><button class="secondary" data-act="ownCommit" data-arg="1">1</button><button class="secondary" data-act="ownCommit" data-arg="2">2</button><button class="secondary" data-act="ownCommit" data-arg="3">3</button></div>`)}`;
    const revs = p.revisions;
    return `${detailHeader(env, 'Commitments', `Week of ${dayLabel(ctx.weekFrom)}`)}
${card('This week', `${tracksHtml(ctx)}
<div class="stepper-row"><span>Strength sessions</span><button class="secondary" data-act="revise" data-arg="strength:-1" aria-label="One fewer strength session from today">−</button><b class="n">${p.target.strength}</b><button class="secondary" data-act="revise" data-arg="strength:1" aria-label="One more strength session from today">+</button></div>
<div class="stepper-row"><span>Intensity minutes</span><button class="secondary" data-act="revise" data-arg="im:-30" aria-label="30 fewer minutes from today">−</button><b class="n">${p.target.im}</b><button class="secondary" data-act="revise" data-arg="im:30" aria-label="30 more minutes from today">+</button></div>
<p class="small">Changes apply from today (${dayLabel(ctx.calendarDate)}). Earlier days and weeks keep the targets they had. Lowering a target already met shows “Reached · adjusted”, without a celebration.</p>
<button class="secondary" data-go="adapt/lighter">Busy or travelling?</button>`)}
${card('Zone 2 and timing evidence', '<p>Zone 2 minutes use recorded heart-rate zones, not Garmin’s moderate + twice-vigorous intensity weighting. Missing zones mean unknown, not zero.</p><p class="small">A saved Zone 2 commitment remains its own track at cutover. This stage exercises strength/intensity target edits; full Zone 2 edit and migration arithmetic require the implemented Plan projection, not this synthetic model.</p><button class="secondary" data-go="sleep/timing">Sleep timing observation</button>')}
${card('Versions', `<ol class="versions">${D.commitmentVersions.filter(v => v.from <= p.wk).map(v => `<li><b>${v.from.replace('2026-W', 'Week ')}</b> ${v.strength} strength · ${v.im} min <small>${h(v.note)}</small></li>`).join('')}${revs.map(r => `<li><b>${dayLabel(r.from)}</b> ${r.strength} strength · ${r.im} min <small>Current-week revision; the original ${p.base.strength} · ${p.base.im} stays on record</small></li>`).join('')}${p.pause ? `<li><b>${dayLabel(p.pause.since)}${p.pause.until ? `–${dayLabel(p.pause.until)}` : ''}</b> Paused <small>No proration, no completion for pausing</small></li>` : ''}</ol>`)}
${card('Last week on these targets', `<p class="small">A retrospective comparison only: week 46 had 2 strength sessions and 178 min. This is not an achievement for a week before these targets existed.</p>`)}`;
  }

  // ---------- Progress ----------
  function progress(env) {
    const {ctx, state} = env;
    const p = ctx.plan;
    const profile = state.profile;
    const leadText = profile === 'sleep' ? ['Your sleep this week', 'Garmin recorded your sleep need every night, so each short night is clear. Patterns, not a score.', 'sleep/patterns', 'Sleep patterns']
      : profile === 'simple' || profile === 'sparse' ? [`${ctx.world.activities.length} recorded activities`, profile === 'sparse' ? 'Activity coverage is incomplete. These are known sessions, not a total or a failure verdict.' : 'Recorded by your watch. No targets needed to see what you’ve done.', 'activities', 'All activities']
      : p && (p.midPause || p.wholePause) ? ['You kept going before the pause', `${p.strength} strength session${p.strength === 1 ? '' : 's'} recorded this week. The pause began ${dayLabel(p.pause.since)}; actual work still counts.`, 'workout/a-1201', 'Open workout']
      : ['A new best this week', 'Dumbbell bench press 22 kg × 8, best in your available history (since Sep 2025).', 'workout/a-1201', 'Open workout'];
    return `${rootHeader(env, 'progress')}
<section class="lead k-done" aria-labelledby="plead-h"><p class="kicker">${icon('check')}<span>This week</span></p><h2 id="plead-h">${leadText[0]}</h2><p class="lead-line">${leadText[1]}</p><div class="lead-actions"><button class="primary" data-go="${leadText[2]}">${leadText[3]}</button></div></section>
${p ? card('Follow-through', `${tracksHtml(ctx)}<h3 class="sub">Last 8 plan weeks</h3><ol class="weeks">${D.weeks.map(w => `<li class="w-${w.state}"><button data-go="review/${w.id}"><span class="wk">${w.range}</span>${tag(w.label, w.state)}<small>${h(w.detail)}${w.version ? ` · targets then: ${w.version}` : ''}</small></button></li>`).join('')}<li class="w-current"><button data-go="plan"><span class="wk">This week</span>${tag(p.midPause ? 'Adjusted · paused' : 'In progress', p.midPause ? 'adjusted-paused' : 'open')}<small>Targets: ${p.target.strength} strength · ${p.target.im} min</small></button></li></ol><p class="small">Weeks before your first plan have no verdict. Paused and unknown weeks stay visible; no streak.</p>`, '', 'sec-follow')
      : card('Recorded activity', `<p>${profile === 'sleep' ? 'No recorded workouts. That isn’t a problem; sleep and health are here too.' : `${ctx.world.activities.length} activities recorded in this fixture.`}</p><button class="link" data-go="activities">All activities</button>`)}
${card('Measured change', `${row('Fitness Age', '<span class="n">41</span>', 'metric/fitnessAge', 'Garmin estimate · 3 Nov · unchanged over 8 weeks, separate from your routine')}${profile !== 'sleep' ? row('VO₂ max', '<span class="n">47</span>', 'metric/vo2max', 'Garmin estimate · 3 Nov · sparse history') : ''}${profile === 'mixed' ? row('Strength bests', '2 this month', 'strength', 'Best in available history') : ''}${row('Sleep', 'Patterns', 'sleep/patterns', 'Recorded duration against recorded need')}<p class="small">Participation and measured change are kept apart; a flat estimate doesn’t mean effort was wasted.</p>`)}
${profile === 'mixed' ? card('Milestones', `<ol class="miles">${D.milestones.map(m => `<li><button data-go="${m.route}"><span class="wk">${dayLabel(m.date)}</span><b>${h(m.text)}</b><small>${h(m.kind)}</small></button></li>`).join('')}</ol>`) : ''}
${card('Weekly review', `${row(`Week of ${A().latestReviewWeek() === '2026-W47' ? '16–22 Nov' : '9–15 Nov'}`, 'Open', `review/${A().latestReviewWeek()}`, 'One win, one observation, one choice')}<button class="link" data-go="reviews">All reviews</button>`)}
<p class="end">Looking for something else? <button class="link inline" data-go="explore">Explore all data</button></p>`;
  }
  function weekInfo(id) {
    return D.weeks.find(w => w.id === id) || (id === '2026-W47' ? {id, range: '16–22 Nov', state: 'current'} : null);
  }
  function review(env, id) {
    const {ctx, state} = env;
    const w = weekInfo(id);
    if (!w) return `${detailHeader(env, 'Week review')}${card('', '<p>No review exists for that week.</p>')}`;
    const p = ctx.plan;
    const current = id === '2026-W47';
    const win = current ? (ctx.world.activities.some(a => a.id === 'a-1201') ? 'Monday’s strength session, including a new bench press best.' : 'Your recorded data stays available, with or without targets.') : w.state === 'reached' ? 'Every accepted commitment met against that week’s targets.' : w.state === 'none' ? '3 activities recorded before you had a plan.' : w.state === 'paused' ? '2 activities recorded during a paused week, with no completion award.' : w.state === 'unknown' ? '2 known workouts; the remaining evidence is unavailable.' : w.detail;
    const obs = current ? 'One short night (5h 10) on Tuesday; Garmin rated Wednesday’s readiness Low.' : w.state === 'unknown' ? 'Garmin’s activity list was incomplete, so this week stays “awaiting evidence”.' : 'Bed times varied by 40 min; Garmin rated 5 of 7 nights aligned.';
    return `${detailHeader(env, `Week ${id.slice(6)} review`, w.range)}
${card('The win', `<p>${h(win)}</p>`)}
${card('An observation', `<p>${h(obs)}</p>`)}
${card('Commitments that week', current && p ? tracksHtml(ctx) : `<p>${tag(w.label, w.state)} <small>${h(w.detail)}</small></p>${w.version ? `<p class="small">Measured against the targets saved then: ${w.version}.</p>` : '<p class="small">No plan existed, so there’s no verdict.</p>'}`)}
${card('A choice for next week', `<div class="btns"><button class="secondary" data-act="keep">Keep commitments</button><button class="secondary" data-go="plan/commitments">Change</button><button class="secondary" data-go="adapt/lighter">Lighter week</button></div><p class="small">Nothing to catch up. Choosing is optional.</p>`)}`;
  }
  function reviews(env) {
    return `${detailHeader(env, 'Weekly reviews', 'Dated, kept on this phone')}
${card('', `${['2026-W47', ...D.weeks.map(w => w.id).reverse()].filter(id => id !== '2026-W47' || A().latestReviewWeek() === '2026-W47').map(id => { const w = weekInfo(id); return row(`Week ${id.slice(6)} · ${w.range}`, tag(w.label || 'In progress', w.state), `review/${id}`); }).join('')}`)}`;
  }

  // ---------- Explore ----------
  const LIBRARY = [
    ['Sleep', 'sleep', 'Last night, tonight, naps, patterns'],
    ['Readiness', 'readiness', 'Garmin’s factors and trend'],
    ['Training', 'training', 'Load, VO₂ max, predictions, thresholds, gear'],
    ['Strength', 'strength', 'Sessions, exercises, bests, muscles'],
    ['Activities', 'activities', 'Every recorded workout, filters, routes'],
    ['Health', 'health', 'Vitals, heart rate, stress, weight, Fitness Age, journal'],
  ];
  const INDEX = [
    ...Object.entries(D.metrics).map(([id, m]) => [m.name, `metric/${id}`, m.source]),
    ['Race predictions', 'training/performance', 'Training · running'], ['Thresholds (lactate, FTP)', 'training/performance', 'Training'], ['Load focus', 'training', 'Training'],
    ['Gear (shoes, bikes)', 'gear', 'Training'], ['Naps', 'sleep/naps', 'Sleep'], ['Sleep stages', 'sleep/stages', 'Sleep'], ['Sleep alignment', 'sleep/patterns', 'Sleep'],
    ['Sleep timing', 'sleep/timing', 'Sleep'], ['Garmin journal', 'health/journal', 'Health · private'], ['Intensity minutes', 'plan', 'Plan'],
    ['Weekly reviews', 'reviews', 'Progress'], ['Export your data', 'settings/export', 'Settings'], ['Help', 'help', 'Offline pages'],
    ...D.exercises.map(e => [e.name, `strength/${e.id}`, 'Strength exercise']),
  ].sort((a, b) => a[0].localeCompare(b[0]));
  const SYN = {'pulse ox': 'spo2', spo2: 'pulse ox', 'vo2': 'vo₂', shoes: 'gear', bike: 'gear', hr: 'heart', 'heart rate variability': 'hrv', temperature: 'skin', lifestyle: 'journal', backup: 'export'};
  function explore(env) {
    const q = (env.ui.q || '').trim().toLowerCase();
    const terms = q ? [q, SYN[q]].filter(Boolean) : [];
    const results = q ? INDEX.filter(([n, , s]) => terms.some(t => `${n} ${s}`.toLowerCase().includes(t))) : [];
    return `${detailHeader(env, 'Explore', 'All of your Garmin data')}
<label class="search">${icon('search')}<span class="vh">Search metrics and exercises</span><input type="search" placeholder="Search metrics and exercises" value="${h(env.ui.q || '')}" data-input="search" autocomplete="off"></label>
${q ? card(`${results.length} result${results.length === 1 ? '' : 's'}`, results.map(([n, go, s]) => row(h(n), '', go, h(s))).join('') || '<p class="small">Nothing by that name. Browse the areas below; every metric is in one of them.</p>') : ''}
<nav class="library" aria-label="Areas">${LIBRARY.map(([n, go, s]) => `<button class="lib" data-go="${go}"><b>${n}</b><small>${s}</small></button>`).join('')}</nav>
${card('More', `${row('Trends & comparisons', '', 'trends', 'Every chart with ranges and comparisons')}${row('Gear', '', 'gear', 'Shoes and bikes, Garmin distances')}${row('Weekly reviews', '', 'reviews', 'Past reviews by week')}${row('Help', '', 'help', 'Offline guides')}${row('Settings', '', 'settings', 'Notifications, privacy, export, sign out')}`)}
${card('Everything A–Z', INDEX.map(([n, go, s]) => row(h(n), '', go, h(s))).join(''), 'az')}`;
  }

  // ---------- Sleep ----------
  function sleepBar(n) {
    if (!n) return '<p class="small">No recorded night.</p>';
    const max = Math.max(540, n.need);
    return `<div class="needbar" role="img" aria-label="Asleep ${dur(n.asleep)}, need ${dur(n.need)}"><i style="width:${n.asleep / max * 100}%"></i><u style="left:${n.need / max * 100}%"></u></div>`;
  }
  function sleep(env) {
    const {ctx, state} = env;
    const n = ctx.night, tn = ctx.tonight;
    const wd = state.winddown[addDays(ctx.displayedDay, -1)];
    const tonightMarked = state.winddown[tn.nightOf];
    const naps = napSection(env);
    // One coherent synthetic stage partition per fixture night, not a product inference.
    const deep = n ? Math.round(n.asleep * .18) : 0, rem = n ? Math.round(n.asleep * .22) : 0, light = n ? n.asleep - deep - rem : 0;
    return `${detailHeader(env, 'Sleep', n ? `Night ending ${dayLabel(ctx.displayedDay)}` : 'No recorded night')}
${card('Last night', n ? `<div class="big"><b class="n">${dur(n.asleep)}</b><span>asleep · need ${dur(n.need)} (that night’s recorded need)</span></div>${sleepBar(n)}<p class="small">Sleep score ${n.score} · ${n.score >= 80 ? 'Good' : n.score >= 60 ? 'Fair' : 'Poor'} (Garmin). ${n.note ? h(n.note) + '.' : ''} Bed ${n.bed} · wake ${n.wake}.</p>${wd ? `<p class="selfrep">${icon('check')} You marked wind-down done the evening before. Preparation and the night’s result are recorded separately; one doesn’t grade the other.</p>` : ''}`
      : `<p>${ctx.phase === 'waiting' ? 'Last night hasn’t synced yet.' : 'No sleep was recorded. Maybe the watch wasn’t worn; that’s a gap, not a failure.'}</p>`, '', 'sec-last')}
${card('Tonight', `<p class="eyebrow">${h(tn.nightLabel)}</p>${tn.window ? `<div class="big"><b class="n">${tn.window}</b><span>${tag(tn.source, 'src')} bedtime window</span></div>` : `<div class="big"><b class="n">23:20</b><span>${tag('Raring estimate', 'src')} asleep by</span></div><p class="small">${h(tn.estimate)}</p>`}
<p>Need ${dur(tn.need)} · wind down from ${tn.windDown}.</p><p class="small">${h(tn.reason)}</p>${tn.updated ? `<p class="small">${h(tn.updated)}</p>` : ''}
${['winddown', 'late'].includes(ctx.phase) && ctx.phase === 'late' ? '<p class="small">The window has passed. No countdown and no verdict; rest when you’re ready.</p>' : ''}
<div class="btns"><button class="secondary" data-act="winddown" aria-pressed="${!!tonightMarked}">${tonightMarked ? `${icon('check')} Wind-down marked (self-report)` : 'I wound down (optional)'}</button><button class="link" data-act="bedtime">${state.notifyBedtime ? 'Wind-down reminder on' : 'Remind me to wind down'}</button></div>
<p class="small">This plan doesn’t change your watch’s sleep schedule. Garmin’s longer-term alignment window is separate, in Patterns.</p>`, '', 'sec-tonight')}
${naps}
${state.winddownIntention ? card('Your wind-down intention', '<p>Chosen by you. Preparation is an optional self-report, not inferred from sleep or a reminder.</p>') : ''}
${card('Patterns', `<p class="small">Duration against need, timing and alignment across 14 nights, with gaps.</p><button class="secondary" data-go="sleep/patterns">Open patterns</button>`, '', 'sec-patterns-link')}
${card('Stages & overnight', n ? `<div class="stages" role="img" aria-label="Synthetic stage totals: deep ${dur(deep)}, light ${dur(light)}, REM ${dur(rem)}"><i class="s-deep" style="flex:${deep}"></i><i class="s-light" style="flex:${light}"></i><i class="s-rem" style="flex:${rem}"></i></div><p class="legend"><span>Deep ${dur(deep)}</span><span>Light ${dur(light)}</span><span>REM ${dur(rem)}</span></p><p class="small">Synthetic stage partition; the existing full awake/stage timeline remains the cutover renderer.</p>${row('Overnight HRV', '46 ms', 'metric/hrv')}${row('Sleep HR', '52 bpm', 'metric/sleepHr', 'Overnight average, not daily resting HR')}${row('Respiration', '14.1 brpm', 'metric/respiration')}${row('Pulse Ox', '95%', 'metric/spo2')}` : '<p class="small">Waiting for a recorded night.</p>', '', 'sec-stages')}
${state.savedSleepTarget ? card('Sleep timing', `<p>Observed against the plan saved each evening (asleep by 23:15, your saved reference: 5 of 7 nights).</p><ul class="plain"><li>Mon: asleep 22:56 · plan 23:15</li><li>Tue: asleep 23:18 · plan 23:15</li><li>Wed: asleep 00:52 · plan 23:10</li><li>Thu: no saved plan that evening, so nothing to compare</li></ul><p class="small">No behind/on-track verdict from this week, which is marked adjusted. Week 46 keeps its original result: 4 of 5 nights.</p>`, '', 'sec-timing') : ''}
${card('Sleep score trend', row('Full chart and comparisons', '', 'metric/sleepScore'))}`;
  }
  function napSection(env) {
    const {state, ctx} = env;
    if (state.profile !== 'sleep' && state.profile !== 'mixed') return card('Naps', '<p class="small">No naps recorded.</p>', '', 'sec-naps');
    const c = state.napCase;
    const body = c === 'empty' ? '<p>No naps recorded today.</p><p class="small">Garmin returned an empty list.</p>'
      : c === 'unavailable' ? '<p>Naps unavailable.</p><p class="small">Garmin didn’t return nap data for this day. That isn’t the same as no naps.</p>'
      : c === 'stale' ? '<p>Naps may be out of date.</p><p class="small">Last read Mon 16 Nov; a newer nap may not be here yet.</p>'
      : (ctx.napList.length ? ctx.napList.map(n => `<div class="nap"><b class="n">${n.min} min</b><span>${dayLabel(n.start[0])} ${n.start[1]} → ${n.end ? `${n.end[0] !== n.start[0] ? dayLabel(n.end[0]) + ' ' : ''}${n.end[1]}` : '<em>end time not recorded</em>'}</span></div>`).join('') : '<p>No naps recorded as of the current watch upload.</p>');
    return card('Naps', `${body}<p class="small">Naps are kept apart from overnight sleep, its score and tonight’s timing. Garmin may adjust tonight’s need for a nap; that’s Garmin’s input, shown in Tonight.</p>`, '', 'sec-naps');
  }
  function sleepPatterns(env) {
    const {ctx, state} = env;
    const nights = [...Array(14)].map((_, i) => addDays(ctx.displayedDay, i - 13)).map(d => [d, ctx.world.nights[d]]);
    const max = 560;
    const short = nights.filter(([, n]) => n && n.need - n.asleep >= 60).length;
    const marks = Object.keys(state.winddown).filter(d => state.winddown[d]);
    return `${detailHeader(env, 'Sleep patterns', `14 nights to ${dayLabel(ctx.displayedDay)}`)}
${card('Duration and need', `<ol class="nights">${nights.map(([d, n]) => `<li><span class="wk">${shortDay(d)} ${Number(d.slice(8))}</span>${n ? `<span class="nb"><i style="width:${n.asleep / max * 100}%"></i><u style="left:${n.need / max * 100}%"></u></span><span class="n">${dur(n.asleep)}</span>` : '<span class="gap">No recorded night</span>'}</li>`).join('')}</ol>
<p class="small">${short} of ${nights.filter(([, n]) => n).length} recorded nights were an hour or more below that night’s recorded need. Each night stands alone; there is no running total to repay.</p>`)}
${card('Timing', `<p>Usual wake 06:50 (12 recorded wake times). Bed times ranged 22:40–00:52.</p><p class="small">Garmin’s 7-day average: bed 23:14, wake 06:58.</p>`)}
${card('Alignment', `<p>${tag('Aligned', 'src')} on 5 of 7 rated nights (Garmin).</p><p class="small">Garmin’s optimal sleep window is a longer-term rhythm, separate from tonight’s bedtime window.</p>`)}
${card('Wind-down (self-report)', marks.length ? `<p>You marked wind-down on ${marks.map(d => dayLabel(d)).join(', ')}.</p><p class="small">Kept separate from sleep results. Not inferred from sleep, wear or app use.</p>` : '<p class="small">Nothing marked. Optional; not inferred from sleep, wear or app use.</p>')}
${card('', row('Sleep score: full chart and comparisons', '', 'metric/sleepScore'))}`;
  }

  // ---------- Readiness / Training / Strength ----------
  function readiness(env) {
    const {ctx} = env;
    const r = ctx.readiness;
    if (!ctx.world.readinessSupported) return `${detailHeader(env, 'Readiness')}${card('', '<p>This watch reports that it can’t record Training Readiness.</p><p class="small">Sleep, Body Battery, stress and health readings still work. Nothing here is estimated by Raring.</p>')}`;
    if (!r) return `${detailHeader(env, 'Readiness')}${card('', `<p>No readiness for ${dayLabel(ctx.displayedDay)} yet.</p><p class="small">${ctx.phase === 'waiting' ? 'It arrives after last night syncs.' : 'Your watch hasn’t uploaded a reading.'} An older value isn’t shown as today’s.</p>`)}${card('', row('Trend', '', 'metric/readiness'))}`;
    const factors = [['Sleep score', r.value < 50 ? 'Poor' : 'Good'], ['Recovery time', 'Fair'], ['HRV status', 'Good'], ['Acute load', r.value > 80 ? 'Fair' : 'Good'], ['Sleep history', 'Fair'], ['Stress history', null]];
    return `${detailHeader(env, 'Readiness', `${dayLabel(ctx.displayedDay)} · Garmin`)}
${card('', `<div class="big"><b class="n">${r.value}</b><span>${tag(r.level, r.value >= 70 ? 'good' : r.value >= 50 ? 'fair' : 'poor')} Training Readiness</span></div><p class="small">${h(r.limit)} holds it back most. Readiness is one input, not permission or an instruction.</p>`)}
${card('Garmin’s factors', factors.map(([n, rating]) => `<div class="factor"><span>${n}</span>${rating ? tag(rating, rating.toLowerCase()) : tag('Not rated by Garmin', 'gap')}</div>`).join(''))}
${card('', row('Trend and comparisons', '', 'metric/readiness'))}`;
  }
  function training(env) {
    const {ctx} = env;
    const recent = ctx.world.activities.filter(a => a.date <= ctx.calendarDate).slice(-3).reverse();
    return `${detailHeader(env, 'Training', 'Last 7 days · Garmin')}
${card('Acute load', `<div class="big"><b class="n">412</b><span>${tag('Within Garmin’s range', 'good')} 320–560 for your history</span></div><p>How recent recorded load compares with Garmin’s range for you. It isn’t an amount left to do today.</p><button class="link" data-go="help/acute-load">What acute load is</button>${row('Full chart', '', 'metric/acuteLoad')}`, '', 'sec-load')}
${card('Status and focus', `<p>${tag('Productive', 'src')} Garmin training status</p><div class="focus"><span>Low aerobic</span><span class="amount"><i style="width:62%"></i></span><span>High aerobic</span><span class="amount"><i style="width:48%"></i></span><span>Anaerobic</span><span class="amount"><i style="width:30%"></i></span></div><p class="small">Garmin’s load focus over 4 weeks. Muscle recovery isn’t measured here.</p>`)}
${card('Fitness estimates', `${row('VO₂ max', '<span class="n">47</span>', 'metric/vo2max', 'Garmin · 3 Nov')}${row('Endurance score', '<span class="n">5 870</span>', 'metric/endurance', 'Garmin · 17 Nov')}${row('Hill score', '<span class="n">61</span>', 'metric/hill', 'Garmin · 2 Nov')}`)}
${ctx.world.activities.some(a => a.sport === 'running') ? card('Running', `${row('Running tolerance', '<span class="n">31</span>', 'metric/runningTolerance', 'Garmin weekly')}${row('Race predictions', '10K 49:40', 'training/performance', 'Garmin · 3 Nov · a prediction, not a readiness verdict')}${row('Lactate threshold', '164 bpm · 4:58 /km', 'training/performance', 'Recorded 28 Oct')}<p class="small">Shown because you have recorded runs; it isn’t the centre of Training.</p>`) : card('Other watch metrics', row('Thresholds and race predictions', '', 'training/performance', 'Available on deliberate opening, not promoted from another sport'))}
${card('Cycling', row('Cycling FTP', '238 W', 'training/performance', 'Recorded 12 Oct'))}
${card('Recent activities', recent.map(a => row(`${SPORTS[a.sport]} · ${dayLabel(a.date)}`, `${a.min} min`, `workout/${a.id}`)).join('') + `<button class="link" data-go="activities">All activities</button>`)}
${card('Gear', row('Shoes and bikes', '3 items', 'gear', 'Garmin distances'))}`;
  }
  function gear(env) {
    return `${detailHeader(env, 'Gear', 'Garmin gear list · fetched Wed 18 Nov 07:12')}
${card('', D.gear.map(g => `<div class="factor"><span>${g.name}<small>${g.used}${g.limit ? ` of ${g.limit}` : ''}</small></span>${tag(g.state, 'src')}</div>`).join('') + '<p class="small">A missing distance isn’t zero, and no threshold isn’t a lifespan. Raring doesn’t change gear or suggest replacements.</p>')}`;
  }
  function strength(env) {
    const {ctx} = env;
    const acts = ctx.world.activities.filter(a => a.sport === 'strength');
    if (!acts.length) return `${detailHeader(env, 'Strength')}${card('', `<p>No strength sessions recorded in the last 12 weeks.</p><p class="small">When you record one with lap presses between sets, Garmin’s sets appear here. <button class="link inline" data-go="help/strength-start">How to record sets</button></p>`)}`;
    return `${detailHeader(env, 'Strength', 'Last 12 weeks')}
${card('This week', `<div class="big"><b class="n">${acts.filter(a => a.date >= ctx.weekFrom && a.date <= ctx.calendarDate).length}</b><span>sessions · 11 sets recorded</span></div><div class="focus"><span>Chest</span><span class="amount"><i style="width:70%"></i></span><span>Back</span><span class="amount"><i style="width:60%"></i></span><span>Arms</span><span class="amount"><i style="width:80%"></i></span><span>Core</span><span class="amount"><i style="width:20%"></i></span><span>Legs</span><span class="amount"><i style="width:0%"></i></span></div><p class="small">Recorded sets per muscle. Not muscle freshness, injury status or safety. <button class="link inline" data-go="help/strength-muscles">About the map</button></p>`)}
${card('Muscles', row('Recorded coverage and previous weeks', '', 'strength/muscles'))}
${card('Exercises', D.exercises.map(e => row(e.name, e.best[8] || e.best[10] || '—', `strength/${e.id}`, 'Heaviest set history and bests')).join(''))}
${card('Sessions', acts.slice().reverse().map(a => row(`${dayLabel(a.date)} · ${a.start}`, a.timerOnly ? 'Timer only' : 'Sets', `workout/${a.id}`)).join(''))}
${card('Help', `${row('Lifting with the watch', '', 'help/strength-start')}${row('How sets are counted', '', 'help/strength-sets')}${row('Which watches work', '', 'help/strength-watches')}`)}`;
  }
  function exercise(env, id) {
    const e = D.exercises.find(x => x.id === id);
    return `${detailHeader(env, e.name, 'Strength exercise')}
${card('Best weight by reps', `<div class="reps">${[1, 3, 5, 8, 10].map(n => `<div><small>${n} rep${n > 1 ? 's' : ''}</small><b class="n">${e.best[n] || '—'}</b></div>`).join('')}</div><p class="small">Best in available history (since Sep 2025), not lifetime. Bodyweight sets aren’t weighted records.</p>`)}
${card('Sessions', e.sessions.map(sid => { const a = activity(sid); return row(`${dayLabel(a.date)}`, 'Open', `workout/${sid}`, (D.sets[sid] || []).filter(x => x.exercise === e.name).map(x => x.sets.map(s => `${s[0]}×${s[1] ?? 'BW'}`).join(', ')).join('') || 'Sets'); }).join(''))}`;
  }
  function activities(env) {
    const {ctx} = env;
    const f = env.ui.filter || 'all', date = env.ui.date || null;
    const sports = ['all', ...new Set(ctx.world.activities.map(a => a.sport))];
    const list = ctx.world.activities.filter(a => a.date <= ctx.calendarDate && (f === 'all' || a.sport === f) && (!date || a.date === date)).slice().reverse();
    return `${detailHeader(env, 'Activities', 'Recorded by your watch')}
<div class="chips" role="group" aria-label="Filter by sport">${sports.map(s => `<button class="chip" aria-pressed="${f === s}" data-act="filter" data-arg="${s}">${s === 'all' ? 'All' : SPORTS[s]}</button>`).join('')}${date ? `<button class="chip" aria-pressed="true" data-act="clearDate">${dayLabel(date)} ${icon('close')}<span class="vh">Clear date</span></button>` : ''}</div>
${ctx.world.coverageIncompleteFrom ? `<p class="notice small">${icon('info')} Garmin’s activity list is incomplete since ${dayLabel(ctx.world.coverageIncompleteFrom)}. Known workouts are here; others may arrive later.</p>` : ''}
${card('', list.length ? list.map(a => row(`${SPORTS[a.sport]} · ${dayLabel(a.date)} ${a.start}`, `${a.min} min`, `workout/${a.id}`, a.lateUpload ? `Uploaded later, on ${dayLabel(a.lateUpload)}` : a.timerOnly ? 'Timer only · no sets' : `${a.km ? a.km + ' km · ' : ''}Garmin load ${a.load}`)).join('') : '<p class="small">No recorded activities match.</p>')}`;
  }
  function workout(env, id) {
    const {ctx, state} = env;
    const a = activity(id);
    const sets = D.sets[id];
    const linked = state.intentions.find(i => i.link === id);
    const evidenceCtx = R.derive({...state, day: a.date, clock: 'evening'});
    const p = evidenceCtx.plan;
    const direct = a.timerOnly ? 'Counted as a strength session. No sets were recorded, so there are no set records.' : p && a.sport === 'strength' ? R.contribution(evidenceCtx, a) + '.' : p && a.im ? `+${a.im} intensity minutes (Garmin’s weighting).` : `A ${SPORTS[a.sport].toLowerCase()} session recorded. No target needed.`;
    return `${detailHeader(env, `${SPORTS[a.sport]} · ${a.min} min`, `${dayLabel(a.date)} ${a.start} · recorded by your watch`)}
${card('How it counted', `<dl class="counted-dl"><dt>Direct contribution</dt><dd>${h(direct)}</dd><dt>Garmin’s assessment</dt><dd>${a.benefit} benefit · aerobic Training Effect ${a.aerobic} (Garmin)</dd><dt>Measured progress</dt><dd>${a.record ? `${h(a.record)}: best in available history` : 'No record or measured change from this session, and that’s fine.'}</dd></dl>${linked ? `<p class="selfrep">${icon('check')} Linked by you to your plan’s ${SPORTS[linked.sport].toLowerCase()} on ${dayLabel(linked.date)}. <button class="link inline" data-go="intention/${linked.id}">View</button></p>` : ''}${a.lateUpload ? `<p class="small">Uploaded on ${dayLabel(a.lateUpload)}; counted on the day it was recorded.</p>` : ''}`)}
${card('Evidence', row('Route, heart rate, splits and dynamics', '', `workout/${id}/evidence`, 'Recorded source and gaps'))}
${card('Stats', `<div class="stats"><div><small>Time</small><b class="n">${a.min} min</b></div>${a.km ? `<div><small>Distance</small><b class="n">${a.km} km</b></div>` : ''}<div><small>Garmin load</small><b class="n">${a.load}</b></div><div><small>Avg HR</small><b class="n">${118 + a.load % 30} bpm</b></div></div><div class="zones" role="img" aria-label="Time in zones: zone 1 20%, zone 2 35%, zone 3 30%, zone 4 12%, zone 5 3%"><i class="z1" style="flex:20"></i><i class="z2" style="flex:35"></i><i class="z3" style="flex:30"></i><i class="z4" style="flex:12"></i><i class="z5" style="flex:3"></i></div><p class="legend"><span>Zones 1–5 by time</span></p>`)}
${a.sport === 'strength' ? (sets ? card('Sets', sets.map(x => `<div class="exercise"><b>${h(x.exercise)}</b><small>${h(x.muscles)}</small><div class="sets">${x.sets.map(s => `<span class="set ${s[2] || ''}">${s[0]} × ${s[1] == null ? 'bodyweight' : s[1] + ' kg'}${s[2] === 'record' ? ' ★ best' : s[2] === 'flag' ? ' ⚑ looks off' : ''}</span>`).join('')}</div><small>Last time: ${h(x.last)}</small>${x.flagged ? `<small class="flag">${h(x.flagged)}</small>` : ''}</div>`).join('') + '<p class="small">Garmin’s sets, unchanged by Raring. <button class="link inline" data-go="help/strength-sets">How sets are counted</button></p>')
        : card('Sets', `<p>No sets were recorded. The session was timed without lap presses.</p><button class="link" data-go="help/strength-start">How to record sets</button>`)) : ''}`;
  }

  // ---------- Health ----------
  function journalCard(env) {
    const j = env.state.journal;
    const days = D.journal.filter(d => d.date <= env.ctx.calendarDate && d.date >= addDays(env.ctx.calendarDate, -6));
    let body;
    if (!j.consent) body = `<p>${j.downloaded ? 'Off. New reads have stopped and stored entries are hidden.' : 'Off. Raring doesn’t read your Garmin journal unless you allow it.'}</p>${j.downloaded ? '<p class="small">4 downloaded days remain on this phone and are included in exports you create until you erase local data.</p>' : ''}<button class="secondary" data-sheet="consent">Allow journal reads…</button>`;
    else if (!j.open) body = `<p>Allowed · last 7 days · read-only. Entries stay closed until you open them.</p><button class="secondary" data-act="journalOpen">Open journal</button><p class="small">Never in notifications, Today, Progress or search. <button class="link inline" data-go="settings/journal">Turn off in Settings</button></p>`;
    else body = `<button class="secondary" data-act="journalClose">Close journal</button>${days.map(d => `<div class="jday"><b>${dayLabel(d.date)}</b><small>Garmin · ${d.state === 'empty' ? 'No entries recorded' : d.state === 'unavailable' ? 'Not available' : 'Recorded'}</small>${d.entries.map(([k, v]) => `<div class="factor"><span>${h(k)}</span><span>${h(v)}</span></div>`).join('')}</div>`).join('')}<p class="small">Leaving Health or switching apps hides these entries.</p>`;
    return card(`${icon('lock')} Garmin journal`, body, 'journal', 'sec-journal');
  }
  function health(env) {
    const {state} = env;
    return `${detailHeader(env, 'Health', `${dayLabel(env.ctx.displayedDay)} · Garmin`)}
${state.vitalsException ? `<section class="notice">${icon('info')}<p>Two overnight readings are outside your usual ranges. This isn’t a diagnosis; consider taking it easy and checking how you feel.</p></section>` : ''}
${card('Vitals', `<div class="vitals">${D.vitals.map(v => { const off = state.vitalsException && (v.id === 'hrv' || v.id === 'sleepHr'); return `<button class="vital" data-go="metric/${v.id}"><small>${v.label}</small><b class="n">${off ? (v.id === 'hrv' ? '31' : '61') : v.value}<span>${v.unit}</span></b>${tag(off ? (v.id === 'hrv' ? 'Below range' : 'Above range') : v.state, off ? 'fair' : v.state === 'Within range' ? 'good' : 'gap')}</button>`; }).join('')}</div><p class="small">Ranges are personal Garmin baselines, not medical targets. Missing or baseline-building is neutral, not an all-clear.</p>`)}
${card('Heart rate, stress and energy', `${row('All-day heart rate', 'Resting 54 bpm', 'metric/heartRate', 'Daily resting value; Sleep HR is the overnight average')}${row('Stress', 'Avg 27 · Low', 'metric/stress', 'Garmin, so far today')}${row('Body Battery', '64', 'metric/bodyBattery', 'At last upload, not live')}`)}
${card('Fitness Age', `${row('Fitness Age', '<span class="n">41</span>', 'metric/fitnessAge', 'Garmin estimate · 3 Nov · real age 44')}<p class="small">Garmin’s drivers: VO₂ max, resting HR, body fat, vigorous minutes (its own 4-week window). An estimate, not a grade.</p>`)}
${card('Weight & body composition', `<p class="small">Private to Health. No targets or notifications.</p><button class="secondary" data-go="health/weight">Open weight</button>`)}
${journalCard(env)}`;
  }
  function weight(env) {
    return `${detailHeader(env, 'Weight & body composition', 'Garmin weigh-ins · kg (your Garmin setting)')}
${card('', `<div class="big"><b class="n">74.2 kg</b><span>Last weigh-in Sat 14 Nov</span></div><p class="small">Body fat 19.8% (Garmin, 14 Nov). Weeks without a weigh-in stay blank.</p>${row('12-week trend', '', 'metric/weight')}`)}`;
  }

  // ---------- Metrics ----------
  function series(id, n, salt = '') {
    let seed = [...(id + salt)].reduce((s, c) => s * 31 + c.charCodeAt(0), 7) % 9973;
    const m = D.metrics[id];
    return [...Array(n)].map((_, i) => {
      seed = (seed * 9301 + 49297) % 233280;
      const gap = (m.sparse && i % 6 !== 0) || (!m.sparse && seed % 17 === 0);
      if (id === 'fitnessAge') return gap ? null : 41;
      return gap ? null : +(m.base + (seed / 233280 - .5) * m.spread * 2 + Math.sin(i / 5) * m.spread * .4).toFixed(1);
    });
  }
  function chart(vals, cmp) {
    const all = vals.concat(cmp || []).filter(v => v != null);
    const lo = Math.min(...all), hi = Math.max(...all), w = 320, hgt = 120;
    const pt = (v, i, n) => `${(i / Math.max(1, n - 1)) * w},${hgt - 8 - ((v - lo) / Math.max(1e-6, hi - lo)) * (hgt - 16)}`;
    const path = arr => { let d = '', pen = false; arr.forEach((v, i) => { if (v == null) { pen = false; return; } d += `${pen ? 'L' : 'M'}${pt(v, i, arr.length)} `; pen = true; }); return d; };
    return `<svg class="chart" viewBox="0 0 ${w} ${hgt}" role="img" aria-label="Trend chart; gaps are missing days"><path d="${path(vals)}" class="line"/>${cmp ? `<path d="${path(cmp)}" class="line cmp"/>` : ''}</svg>`;
  }
  function metric(env, id) {
    const m = D.metrics[id];
    const range = env.ui.range || '30D', cmp = env.ui.cmp || 'off', sel = env.ui.sel ?? null;
    const from = env.ui.from || '2026-10-20', to = env.ui.to || env.ctx.calendarDate;
    const n = range === 'Dates' ? Math.max(1, Math.min(90, Math.round((Date.parse(to) - Date.parse(from)) / 864e5) + 1)) : {['7D']: 7, ['30D']: 30, ['90D']: 90, ['1Y']: 52, All: 60}[range];
    const vals = series(id, n), prev = cmp === 'prev' && range !== 'All' ? series(id, n, 'previous') : null;
    const known = vals.filter(v => v != null);
    const avg = known.length ? (known.reduce((a, b) => a + b, 0) / known.length).toFixed(1) : '—';
    const past = sel != null && sel < n - 1;
    const intraday = ['stress', 'bodyBattery', 'heartRate'].includes(id);
    return `${detailHeader(env, m.name, h(m.source))}
${window.RaringEvidence.metricSummary(id, env)}
${card('', `<div class="chips" role="group" aria-label="Range">${['7D', '30D', '90D', '1Y', 'All', 'Dates'].map(r => `<button class="chip" aria-pressed="${r === range}" data-act="range" data-arg="${r}">${r}</button>`).join('')}</div>
<label class="field"><span>Compare</span><select data-change="cmp"><option value="off" ${cmp === 'off' ? 'selected' : ''}>Off</option><option value="prev" ${cmp === 'prev' ? 'selected' : ''} ${range === 'All' ? 'disabled' : ''}>Previous period</option><option value="hrv" ${cmp === 'hrv' ? 'selected' : ''}>HRV (separate axis)</option><option value="sleepTime" ${cmp === 'sleepTime' ? 'selected' : ''}>Time asleep (separate axis)</option></select></label>
${range === 'Dates' ? `<label class="field">From<input type="date" value="${h(from)}" data-change="from"></label><label class="field">To<input type="date" value="${h(to)}" data-change="to"></label>${cmp === 'prev' ? `<label class="field">Comparison starts<input type="date" value="${h(env.ui.compareFrom || addDays(from, -n))}" data-change="compareFrom"></label>` : ''}<p class="small">Inclusive block ${h(from)}–${h(to)}. Fixture covers up to 90 days; comparison aligns elapsed days, never fills missing evidence.</p>` : ''}
${chart(vals, prev)}
${D.metrics[cmp] ? `<p class="small">Second axis: ${D.metrics[cmp].name} (${D.metrics[cmp].unit}) — separate aligned plot, not a combined score.</p>${chart(series(cmp, n))}` : ''}
<p class="small">${range === '1Y' || range === 'All' ? 'Weekly averages' : 'Daily values'} · average ${avg}${m.unit ? ' ' + m.unit : ''}${prev ? ' · dashed: previous period' : ''}. Gaps are days without a reading, never zero.${m.sparse ? ' This estimate updates rarely, so few points exist; no trend is drawn between them.' : ''}</p>
<label class="field"><span>Inspect a day</span><input type="range" min="0" max="${n - 1}" value="${sel ?? n - 1}" data-change="sel" aria-label="Day to inspect"></label>
<p class="small">${past ? `Selected: ${n - 1 - sel} ${range === '1Y' || range === 'All' ? 'weeks' : 'days'} ago · ${vals[sel] ?? 'no reading'}.${intraday ? ' Detailed readings through the day are only downloaded for recent days; this shows the daily value, not a substitute from today.' : ''}` : 'Latest point selected.'}</p>`)}`;
  }
  function trends(env) {
    return `${detailHeader(env, 'Trends & comparisons', 'From the history on this phone')}
${card('', Object.entries(D.metrics).map(([id, m]) => row(m.name, '', `metric/${id}`, h(m.source))).join(''))}`;
  }

  // ---------- Plan details ----------
  function intention(env, id) {
    const {ctx, state} = env;
    const i = state.intentions.find(x => x.id === id);
    if (!i) return `${detailHeader(env, 'Your plan')}${card('', '<p>This intention was removed. Nothing was changed in Garmin.</p>')}`;
    const candidates = ctx.world.activities.filter(a => a.date === i.date && a.sport === i.sport && a.date <= ctx.calendarDate);
    const linked = i.link && activity(i.link);
    const excluded = state.limits.exclude.includes(i.sport);
    return `${detailHeader(env, `${SPORTS[i.sport]} · ${dayLabel(i.date)}`, `Your plan${i.time ? ` · ${i.time}` : ' · any time'}`)}
${card('', `<p>${tag('Your plan', 'intent')} On this phone. Not sent to Garmin, and not proof of completion.</p>${excluded ? `<p class="notice small">${icon('info')} You excluded ${SPORTS[i.sport].toLowerCase()} in your limits. This stays in your plan until you change it.</p>` : ''}${i.sport === 'strength' && state.limits.note ? `<p class="small">Your movement note applies (“${h(state.limits.note)}”). Raring can’t check what this session contains.</p>` : ''}
<div class="btns"><button class="secondary" data-sheet="add" data-arg="${i.id}">Edit</button><button class="secondary" data-act="removeIntention" data-arg="${i.id}">Remove</button></div>`)}
${card('Recorded that day', linked ? `<p class="selfrep">${icon('check')} Linked by you: ${SPORTS[linked.sport]} at ${linked.start}.</p><p class="small">A link you confirmed, not Garmin schedule completion. The workout counts either way.</p><button class="link" data-act="unlink" data-arg="${i.id}">Unlink</button>`
      : candidates.length ? (i.date < ctx.calendarDate ? '<p class="small">Not linked. That’s not a missed workout; recorded work counts either way.</p>' : '') + candidates.map(a => `<div class="cand">${row(`${SPORTS[a.sport]} · ${a.start}`, `${a.min} min`, `workout/${a.id}`, 'Recorded · already counts')}<button class="secondary" data-act="link" data-arg="${i.id}:${a.id}">This was my session</button></div>`).join('') + '<p class="small">Optional. Recorded work counts without a link.</p>'
      : `<p class="small">${i.date < ctx.calendarDate ? 'Not linked. That’s not a missed workout; it simply stays unlinked.' : 'Nothing recorded yet. Matching activities appear here to link if you want.'}</p>`)}
${card('History', `<ol class="versions">${i.history.map(([d, t]) => `<li><b>${h(d)}</b> ${h(t)}</li>`).join('')}</ol>`)}`;
  }
  function scheduled(env, id) {
    const {ctx, state} = env;
    const s = D.schedule.find(x => x.id === id);
    const r = ctx.readiness;
    const conflict = state.limits.exclude.includes(s.sport);
    const review = !!state.limits.note;
    return `${detailHeader(env, s.name, `${dayLabel(s.date)} · Garmin schedule`)}
${card('', `<p>${tag('Garmin schedule', 'garmin')} Read-only. Raring doesn’t change, cancel or mark Garmin items complete.</p>
${conflict ? `<p class="notice small">${icon('info')} Conflicts with your excluded activity (${SPORTS[s.sport].toLowerCase()}). It stays on your Garmin calendar, unchanged. Raring won’t remind you about it.</p>` : ''}
${review ? `<p class="notice small">${icon('info')} Review needed: Raring can’t see this workout’s exercises, so it can’t check them against your note “${h(state.limits.note)}”. ${state.limits.routines[s.id] === 'chosen' ? 'You chose to keep using it; that’s your choice, not a clearance.' : ''}</p>` : ''}
${ctx.paused ? '<p class="small">Training is paused for you; this item stays visible as context.</p>' : ''}
${s.date === ctx.calendarDate && r ? `<p class="small">Today’s readiness: ${r.value} (${r.level}). ${r.value < 50 ? 'An easier day may fit. Whether to do it is your call.' : ''}</p>` : ''}
<div class="btns">${s.date === ctx.calendarDate && !ctx.restActive ? '<button class="secondary" data-act="rest">Rest today</button>' : ''}${s.date === ctx.calendarDate ? `<button class="secondary" data-act="dismiss" data-arg="sched:${s.id}:${ctx.calendarDate}">Not today</button>` : ''}${review && state.limits.routines[s.id] !== 'chosen' ? `<button class="secondary" data-act="chooseRoutine" data-arg="${s.id}">I intend to use this</button>` : ''}</div>
<p class="small">Garmin schedule checked 07:12. Completion and cancellation status from Garmin aren’t verified, so they aren’t shown.</p>`)}`;
  }

  // ---------- Adapt ----------
  function adapt(env) {
    const {ctx, state} = env;
    const L = state.limits;
    const excl = ['running', 'cycling', 'strength', 'hiit', 'swimming'];
    return `${detailHeader(env, 'Adapt', 'Your circumstances, on this phone')}
<p class="small intro">Raring asks what to change, never why. Nothing here diagnoses, clears exercise or suggests substitutes. <button class="link inline" data-go="help/adapt">How this works</button></p>
${card('Today', ctx.restActive ? `<p>${icon('moon')} Rest today is on. It lasts through tonight; exercise prompts return tomorrow morning.</p><button class="secondary" data-act="unrest">Undo rest</button>` : `<p class="small">Rest today covers today and the following night.</p><button class="secondary" data-act="rest">Rest today</button>`, '', 'sec-today')}
${card('Pause training', ctx.paused || state.pause ? `<p>${icon('pause')} Paused since ${dayLabel(state.pause.since)}${state.pause.review ? ` · review on ${dayLabel(state.pause.review)} (a reminder, not an end date)` : ''}.</p><p class="small">No exercise prompts or pace verdicts. Commitments are kept for when you return; nothing to catch up.</p><button class="primary" data-sheet="resume">Resume training…</button>`
      : `<p class="small">For illness, recovery or a break. Lasts until you resume; better readiness or a date won’t end it.</p><label class="field"><span>Review date (optional)</span><select data-change="reviewDate"><option value="">None</option><option value="${addDays(ctx.calendarDate, 3)}">${dayLabel(addDays(ctx.calendarDate, 3))}</option><option value="${addDays(ctx.calendarDate, 7)}">${dayLabel(addDays(ctx.calendarDate, 7))}</option></select></label><button class="secondary" data-act="pause">Pause from today</button>`, '', 'sec-pause')}
${card('Limits', `<p class="small">Activities to avoid are enforced in suggestions. A movement note stays visible; Raring can’t read workout contents, so routines it can’t check need your review.</p>
<div class="chips" role="group" aria-label="Activities to avoid">${excl.map(s => `<button class="chip" aria-pressed="${L.exclude.includes(s)}" data-act="exclude" data-arg="${s}">${L.exclude.includes(s) ? icon('check') : ''}${SPORTS[s]}</button>`).join('')}</div>
<label class="field col"><span>Movements to avoid (your words)</span><textarea rows="2" data-change="note" placeholder="For example: no deep knee bends">${h(L.note)}</textarea></label>
${ctx.world.schedule.filter(s => L.exclude.includes(s.sport) || (s.routine && L.note)).map(s => row(s.name, tag(L.exclude.includes(s.sport) ? 'Conflict' : L.routines[s.id] === 'chosen' ? 'Your choice' : 'Review needed', 'fair'), `scheduled/${s.id}`, `${dayLabel(s.date)} · Garmin, unchanged`)).join('')}
${L.exclude.length || L.note ? '<button class="link" data-act="clearLimits">Remove all limits</button>' : ''}`, '', 'sec-limits')}
${card('Routines', `${ctx.world.schedule.filter(s => s.routine).map(s => `<div class="factor"><span>${h(s.name)}<small>Garmin workout · contents unknown to Raring</small></span>${tag(L.note ? (L.routines[s.id] === 'chosen' ? 'Your choice' : 'Review needed') : 'No limits set', L.note ? 'fair' : 'gap')}</div>`).join('')}
<h3 class="sub">Your checklist</h3>${state.rehab.items.length ? state.rehab.items.map(it => { const k = `${ctx.calendarDate}:${it.id}`; return `<div class="factor"><span>${h(it.name)}${it.how ? `<small>${h(it.how)}</small>` : ''}</span><button class="secondary" data-act="rehabDone" data-arg="${it.id}" aria-pressed="${!!state.rehab.done[k]}">${state.rehab.done[k] ? `${icon('check')} Done today (self-reported)` : 'Mark done today'}</button></div>`; }).join('') : '<p class="small">Add the exercises you or your clinician chose. Raring doesn’t suggest exercises or progress them.</p>'}
<form class="addrow" data-act="noop" onsubmit="return false"><label class="field col"><span>Exercise name</span><input id="rehab-name" placeholder="For example: wall sit, 3 × 30 s"></label><button class="secondary" data-act="rehabAdd">Add to checklist</button></form>
<p class="small">Self-reported completion stays separate: no Garmin activity, intensity minutes or records are created.</p>`, '', 'sec-routines')}
${card('Busy or travelling', ctx.plan ? `<p class="small">Lighten this week without giving a reason. Original targets stay on record; nothing is prorated.</p><div class="btns"><button class="secondary" data-act="lighter">1 strength · 60 min from today</button><button class="secondary" data-act="pause">Pause this week</button></div>` : '<p class="small">You have no commitments to lighten.</p>', '', 'sec-lighter')}`;
  }

  // ---------- Settings / help ----------
  function settings(env) {
    const {state} = env;
    return `${detailHeader(env, 'Settings', 'Raring')}
${card('Notifications', `<p class="small">Permission: ${state.permission === 'declined' ? 'declined. Raring won’t ask again; you can allow it in Android settings.' : state.permission === 'granted' ? 'allowed.' : 'not asked yet. Raring asks only when you turn on a reminder.'}</p>
${[['morning', 'Morning readiness'], ['weekly', 'Weekly check-in'], ['metrics', 'VO₂ max or Fitness Age changed'], ['vitals', 'Vitals off'], ['training', 'Training status changed'], ['status', 'Persistent status']].map(([key, n]) => `<label class="toggle"><span>${n}</span><input type="checkbox" data-change="channel" data-key="${key}" ${state.channels[key] ? 'checked' : ''}></label>`).join('')}
<button class="secondary" data-act="bedtime">${state.notifyBedtime ? 'Wind-down reminder on' : 'Remind me to wind down'}</button>
<p class="small">Widgets do not need notification permission. Use Android’s widget picker; updates follow existing sync and may be delayed.</p>
<p class="small">Reminders follow the same rest, pause and limit rules as Today. Lock screens never show restriction notes, routine names, journal or weight.</p>`)}
${card('Today', `<label class="toggle"><span>Follow my day<small>Lead with this morning, the day or tonight. Turn off for shift work.</small></span><input type="checkbox" ${state.follow ? 'checked' : ''} data-change="follow"></label><label class="toggle"><span>Quiet celebrations<small>Same information, no animation.</small></span><input type="checkbox" ${state.quiet ? 'checked' : ''} data-change="quiet"></label>`)}
${card('Adapt', `<p class="small">Rest, pauses, limits and routines live in Adapt today, not here.</p>${row('Open Adapt', '', 'adapt')}`)}
${card('Personalisation', `<p class="small">This version doesn’t learn from how you use the app. Navigation and controls stay where they are.</p>`)}
${card('Journal privacy', `<label class="toggle"><span>Allow Garmin journal reads<small>Off by default. Turning off stops new reads and hides entries. Already downloaded logs remain on this phone and in future exports until you erase local data.</small></span><input type="checkbox" ${state.journal.consent ? 'checked' : ''} data-change="journal"></label>`, '', 'sec-journal')}
${card('Export', `<p class="small">A ZIP of saved Garmin responses, daily CSVs and Raring settings, including your plans, intentions, circumstances and notes, routine checklist and self-reports, dismissals and any downloaded journal logs. Sign-in tokens are never included. The file leaves the phone only when you share it.</p><button class="secondary" data-act="export">Export your data</button>`, '', 'sec-export')}
${card('Account', `<p class="small">Signing out erases all local Garmin history and everything you added here.</p><button class="danger" data-sheet="signout">Sign out and erase…</button>`)}
${card('Garmin history', '<p>Available history opens immediately from this phone. Older history downloads in the background; nothing requires a full download before use.</p>')}
${card('Privacy & connection', '<p>Raring reads Garmin Connect, not your watch directly. This unofficial connection is watch-dependent. Production data stays on your device: no Raring server, advertising or remote analytics. Exports leave only when shared.</p><p class="small">Settings import/restore is not shipped. Android Auto Backup and device transfer are disabled; an export is not a restore workflow. This prototype connects to no account at all.</p>')}
${card('Help', `${row('Help pages', '', 'help')}${row('How Raring works', '', 'help/sync', 'Your watch, Garmin Connect and this phone')}`)}`;
  }
  function help(env) {
    return `${detailHeader(env, 'Help', 'Offline')}${card('', Object.entries(D.helpPages).map(([id, p]) => row(p.title, '', `help/${id}`)).join(''))}`;
  }
  function helpPage(env, id) {
    const p = D.helpPages[id];
    return `${detailHeader(env, p.title, 'Help')}${card('', `<p>${h(p.body)}</p>${id === 'sync' ? '<button class="secondary" data-act="connect">Open Garmin Connect</button>' : ''}`)}`;
  }

  // ---------- Prototype-only pages ----------
  function surfaces(env) {
    const s = R.surfaces(env.ctx);
    const tile = (go, label, body) => `<button class="surface" data-ext="${go}" data-label="Opened from an illustrative status surface; Back follows the page’s canonical path.">${body}<small class="opens">Opens ${h(title(go.replace('review/latest', 'review/x')))}</small></button>`;
    return `${detailHeader(env, 'Status surfaces', 'Illustrative only · not native rendering')}
<p class="small intro">These reflect the prototype’s current state with the same suppression rules. Tapping opens the app as a cold link. Native notifications and widgets from items 9 and 10 keep their own tested rendering.</p>
${card('Lock screen', `${s.morning ? tile(s.morning.go, '', `<span class="ntitle">${icon('lock')} ${h(s.morning.lock)}</span><span class="nbody">Contents hidden on the lock screen</span>`) : '<p class="small">No morning readiness notification: permission, channel or current evidence is missing.</p>'}`)}
${card('Notification shade', `${s.morning ? tile(s.morning.go, '', `<span class="ntitle">${h(s.morning.title)}</span><span class="nbody">${h(s.morning.body)}</span><span class="nsub">${h(s.morning.sub)}</span>`) : ''}
${s.statusEnabled ? tile('today', '', `<span class="ntitle">Raring · As of ${h(s.as)}</span><span class="nbody">Body Battery ${s.status.bb ?? '—'} · Stress ${s.status.stress ?? '—'} · latest HR ${s.status.hr ?? '—'}</span><span class="nsub">${h(s.focus)}</span>`) : '<p class="small">Persistent status off. Widgets remain independent.</p>'}
${s.bedtime ? tile(s.bedtime.go, '', `<span class="ntitle">${h(s.bedtime.title)}</span><span class="nbody">${h(s.bedtime.body)}</span>`) : '<p class="small">Bedtime reminder is off (not sent).</p>'}
${s.weekly ? tile(s.weekly.go, '', `<span class="ntitle">${h(s.weekly.title)}</span><span class="nbody">${h(s.weekly.body)}</span>`) : '<p class="small">Weekly notification not sent: permission or channel off.</p>'}`)}
${card('Widgets', `<p class="small">All widget readings: As of ${h(s.as)}. Android may defer updates.</p><div class="widgets">${tile(env.ctx.readiness ? 'readiness' : 'today', '', `<span class="wnum n">${env.ctx.readiness && env.ctx.phase !== 'waiting' ? env.ctx.readiness.value : '—'}</span><span class="nsub">${h(s.focus)}</span><span class="nas">As of ${h(s.as)}</span>`)}${tile('sleep', '', `<span class="wnum n">${env.ctx.night && env.ctx.phase !== 'waiting' ? env.ctx.night.score : '—'}</span><span class="nsub">Sleep score</span>`)}${tile('metric/bodyBattery', '', `<span class="wnum n">${s.status.bb ?? '—'}</span><span class="nsub">Body Battery</span>`)}${tile('metric/stress', '', `<span class="wnum n">${s.status.stress ?? '—'}</span><span class="nsub">Stress</span>`)}</div>`)}`;
  }
  function legacy(env) {
    const rows = Object.entries(A().LEGACY).map(([from, to]) => [from, to]).concat([['workout/a-1201', 'workout/a-1201'], ['metric/hrv', 'metric/hrv'], ['metric/vo2max', 'metric/vo2max'], ['help/strength-sets', 'help/strength-sets'], ['nonexistent-page', '(invalid → Today with an explanation)']]);
    return `${detailHeader(env, 'Old links', 'Prototype check of the legacy-link map')}
${card('', rows.map(([from, to]) => { const c = A().canonical(from); const t = A().defaultTrail(c.route); return `<button class="row" data-ext="${from}"><span class="rl"><span>#${h(from)}</span><small>→ ${h(c.route)} · Back: ${t.slice(0, -1).map(x => h(title(x))).reverse().join(' → ') || '—'}</small></span>${icon('chev', 'chev')}</button>`; }).join(''))}`;
  }

  // ---------- Sheets ----------
  function renderSheet(env) {
    const {sheet, ctx, state} = env;
    const wrap = (t, body, cls = '') => `<div class="scrim" data-act="closeSheet"></div><div class="sheet ${cls}" role="dialog" aria-modal="true" aria-labelledby="sheet-t"><div class="sheet-h"><h2 id="sheet-t" tabindex="-1">${t}</h2><button class="iconbtn" data-act="closeSheet" aria-label="Close">${icon('close')}</button></div>${body}</div>`;
    switch (sheet.name) {
      case 'adapt': return wrap('Adapt today', `${ctx.restActive ? `<p>${icon('moon')} Rest today is on, through tonight.</p><button class="secondary wide" data-act="unrest">Undo rest</button>` : `<button class="primary wide" data-act="rest" data-autofocus>Rest today</button><p class="small">No exercise prompts today or tonight. Your plan and data stay as they are.</p>`}
<div class="sheet-list">${row('Pause training', ctx.paused ? 'On' : '', 'adapt/pause', 'Illness, recovery or a break; until you resume')}${row('Avoid activities or movements', ctx.hasLimits ? 'On' : '', 'adapt/limits', 'Exclusions and your own note')}${row('My routines', '', 'adapt/routines', 'A checklist you supply; self-reported')}${row('Busy or travelling', '', 'adapt/lighter', 'A lighter week, no reason needed')}</div>
<p class="small">Raring never asks why.</p>`);
      case 'add': {
        const editing = sheet.arg && state.intentions.find(i => i.id === sheet.arg);
        const d = env.ui.add || {sport: editing?.sport || null, date: editing?.date || ctx.calendarDate, time: editing?.time || ''};
        const days = [...Array(8)].map((_, i) => addDays(ctx.calendarDate, i));
        return wrap(editing ? 'Edit your plan' : 'Add to my plan', `<p class="small">Saved on this phone. Not sent to Garmin.</p>
<fieldset><legend>Activity</legend><div class="chips">${['strength', 'running', 'cycling', 'walking', 'swimming', 'yoga', 'other'].map(s => { const ex = state.limits.exclude.includes(s); return `<button class="chip" aria-pressed="${d.sport === s}" data-act="addField" data-arg="sport:${s}" ${ex ? 'disabled aria-describedby="ex-note"' : ''}>${SPORTS[s]}${ex ? ' · excluded' : ''}</button>`; }).join('')}</div>${state.limits.exclude.length ? '<p class="small" id="ex-note">Excluded by your limits.</p>' : ''}${d.sport === 'strength' && state.limits.note ? `<p class="small">Your movement note applies; Raring can’t check routine contents.</p>` : ''}</fieldset>
<fieldset><legend>Day</legend><div class="chips">${days.map(x => `<button class="chip" aria-pressed="${d.date === x}" data-act="addField" data-arg="date:${x}">${x === ctx.calendarDate ? 'Today' : dayLabel(x)}</button>`).join('')}</div></fieldset>
<label class="field"><span>Time (optional)</span><input type="time" value="${h(d.time)}" data-change="addTime"></label>
${ctx.world.ambitions.length ? `<p class="small">Linking it to an ambition is optional.</p>` : ''}
<button class="primary wide" data-act="saveIntention" data-arg="${editing ? editing.id : ''}" ${d.sport ? '' : 'disabled'}>${editing ? 'Save changes' : 'Add to plan'}</button>`);
      }
      case 'why': return wrap('Why this?', `<p class="small">Checks run in this order. A later one can’t override an earlier one.</p><ol class="trace">${ctx.lead.trace.map(t => `<li><b>${h(t.gate)}</b><span>${h(t.result)}</span></li>`).join('')}</ol>`);
      case 'notices': return wrap(`${ctx.notices.length} notices`, ctx.notices.map(n => `<div class="notice">${icon('info')}<p>${h(n.text)}</p><button class="link" data-go="${n.action.go}">${h(n.action.label)}</button></div>`).join(''));
      case 'permission': return wrap('Allow notifications?', '<p>Prototype permission choice — not an Android permission dialog. The app remains useful if declined.</p><button class="primary wide" data-act="permit" data-arg="granted">Allow</button><button class="secondary wide" data-act="permit" data-arg="declined">Decline</button>');
      case 'winddown': return wrap('A wind-down intention', '<p>Make room to wind down this evening. Marking preparation done is optional and separate from recorded sleep timing or quality.</p><button class="primary wide" data-act="winddownIntention">Add wind-down intention</button>');
      case 'consent': return wrap('Allow Garmin journal reads?', `<ul class="plain"><li>Raring reads the last 7 days of your Garmin lifestyle logs, read-only.</li><li>Entries stay closed until you open them, and hide when you leave Health or switch apps.</li><li>They never appear in Today, Progress, search or notifications.</li><li>Turning this off stops new reads and hides entries. Downloaded logs stay on this phone and in exports you create until you erase local data.</li></ul><button class="primary wide" data-act="journalAllow" data-autofocus>Allow journal reads</button><button class="link wide" data-act="closeSheet">Not now</button>`);
      case 'resume': return wrap('Resume training', `<p>Your commitments as they were:</p>${ctx.plan ? `<p><b>${ctx.plan.target.strength} strength · ${ctx.plan.target.im} min</b> a week</p>` : '<p>No commitments.</p>'}<p class="small">Nothing to catch up. This week stays “Adjusted · paused” for the paused days; later sessions count normally.</p><button class="primary wide" data-act="resume">Resume as they are</button><button class="secondary wide" data-act="resumeChange">Resume and change commitments</button>`);
      case 'ambition': return wrap('Add an ambition', `<label class="field col"><span>Name</span><input id="amb-name" placeholder="For example: Hike in May" data-autofocus></label><label class="field col"><span>Date (optional)</span><input id="amb-date" type="date"></label><p class="small">Several ambitions can coexist. A date here is a local entry, not a Garmin race or a plan.</p><button class="primary wide" data-act="saveAmbition">Add</button>`);
      case 'signout': return wrap('Sign out and erase?', `<p>This erases from this phone:</p><ul class="plain"><li>Downloaded Garmin history, including any journal logs</li><li>Your plans, intentions, commitments and ambitions</li><li>Rest, pauses, limits and notes, routine checklists and self-reports</li><li>Dismissals and settings</li></ul><p class="small">Sign-in tokens are removed. Nothing in Garmin changes. Export first if you want a copy.</p><button class="secondary wide" data-go="settings/export">Export first</button><button class="danger wide" data-act="signout">Sign out and erase</button>`);
      default: return '';
    }
  }

  function scenarioDrawer(env) {
    const {state} = env;
    const opt = (v, l, cur) => `<option value="${v}" ${v === cur ? 'selected' : ''}>${h(l)}</option>`;
    return `<form method="dialog"><div class="sheet-h"><h2 id="scn-t">Prototype controls</h2><button class="iconbtn" value="close" aria-label="Close prototype controls">${icon('close')}</button></div>
<p class="small">For moderators and reviewers. Not part of the app.</p>
<label class="field"><span>Profile</span><select data-change="profile">${Object.entries(R.PROFILES).map(([k, v]) => opt(k, v.label, state.profile)).join('')}</select></label>
<label class="field"><span>Day</span><select data-change="day">${D.DAYS.map(d => opt(d, dayLabel(d), state.day)).join('')}</select></label>
<label class="field"><span>Time</span><select data-change="clock">${Object.entries(R.CLOCKS).map(([k, v]) => opt(k, `${v.time} · ${v.label}`, state.clock)).join('')}</select></label>
<label class="toggle"><span>Offline</span><input type="checkbox" ${state.offline ? 'checked' : ''} data-change="offline"></label>
<label class="toggle"><span>Irregular wake times</span><input type="checkbox" ${state.irregular ? 'checked' : ''} data-change="irregular"></label>
<label class="field"><span>Naps</span><select data-change="napCase">${[['same', 'Same-day nap'], ['cross', 'Across midnight + missing end'], ['empty', 'Empty list'], ['unavailable', 'Unavailable'], ['stale', 'Stale']].map(([k, l]) => opt(k, l, state.napCase)).join('')}</select></label>
<label class="field"><span>Garmin calendar</span><select data-change="calendar">${[['complete', 'Complete'], ['partial', 'Full page (partial)'], ['stale', 'Stale']].map(([k, l]) => opt(k, l, state.calendar)).join('')}</select></label>
<div class="btns"><button class="secondary" data-act="sync" value="">Sync</button><button class="secondary" data-act="sync" data-arg="exception">Sync: new notice</button><button class="secondary" data-act="bg">Background + resume</button><button class="secondary" data-act="reload">Reload (resume)</button></div>
<div class="btns"><button class="secondary" data-ext="surfaces">Status surfaces</button><button class="secondary" data-ext="legacy">Old links</button><button class="secondary" data-act="resetAll">Reset choices</button></div>
<label class="field"><span>Journey case</span><select data-change="case"><option value="">Choose U01–U64…</option>${window.RaringCases.map(c => `<option value="${c.id}">${c.id} · ${h(c.title)}</option>`).join('')}</select></label>
</form>`;
  }

  // ---------- actions ----------
  const S = () => A().state;
  const ui = () => history.state && history.state.__raringExperience && history.state.__raringExperience.ui || {};
  function setUi(patch) {
    const s = history.state.__raringExperience;
    const next = {...(s.ui || {}), ...patch};
    history.replaceState({__raringExperience: {...s, ui: next}}, '', location.hash);
    window.RaringApp._ui(next);
  }
  const endPause = () => st => ({pause: null, pauses: [...(st.pauses || []), {...st.pause, until: R.derive(st).calendarDate}]});
  window.RaringActions = {
    back: () => A().back(),
    closeSheet: () => A().closeSheet(),
    clearNotice: () => A().clearNotice(),
    scenario: () => A().openScenario(),
    dismiss: id => { A().set(s => ({dismissed: {...s.dismissed, [id]: true}}), 'Hidden for today. A different notice can still appear.'); if (document.querySelector('.sheet')) A().closeSheet(); },
    rest: () => { const ctx = R.derive(S()); A().set({rest: {day: ctx.calendarDate}},  'Rest today is on, through tonight'); if (document.querySelector('.sheet')) A().closeSheet(); },
    unrest: () => { A().set({rest: null}, 'Rest removed'); if (document.querySelector('.sheet')) A().closeSheet(); },
    pause: () => { const ctx = R.derive(S()); A().set(s => ({pause: {since: ctx.calendarDate, review: s._review || null}}), 'Training paused until you resume'); },
    resume: () => { A().set(endPause(), 'Training resumed with your commitments as they were. Nothing to catch up.'); A().closeSheet(); },
    resumeChange: () => { A().set(endPause(), 'Training resumed'); A().sheetThen(() => A().navigate('plan/commitments')); },
    exclude: s => A().set(st => ({limits: {...st.limits, exclude: st.limits.exclude.includes(s) ? st.limits.exclude.filter(x => x !== s) : [...st.limits.exclude, s]}}), 'Limits saved'),
    clearLimits: () => A().set({limits: {exclude: [], note: '', routines: {}}}, 'Limits removed'),
    chooseRoutine: id => A().set(st => ({limits: {...st.limits, routines: {...st.limits.routines, [id]: 'chosen'}}}), 'Noted as your choice. Not a clearance.'),
    rehabAdd: () => { const v = document.getElementById('rehab-name').value.trim(); if (v) A().set(st => ({rehab: {...st.rehab, items: [...st.rehab.items, {id: 'r' + Date.now(), name: v}]}}), 'Added to your checklist'); },
    rehabDone: id => { const ctx = R.derive(S()); const k = `${ctx.calendarDate}:${id}`; A().set(st => ({rehab: {...st.rehab, done: {...st.rehab.done, [k]: !st.rehab.done[k]}}}), 'Self-report saved'); },
    lighter: () => { const ctx = R.derive(S()); A().set(st => ({revisions: [...st.revisions, {week: ctx.plan.wk, from: ctx.calendarDate, strength: 1, im: 60}]}), 'Lighter week from today; original targets kept on record'); },
    revise: arg => {
      const [k, dlt] = arg.split(':'); const ctx = R.derive(S()); const p = ctx.plan;
      const next = {week: p.wk, from: ctx.calendarDate, strength: p.target.strength, im: p.target.im};
      next[k] = Math.max(0, next[k] + Number(dlt));
      A().set(st => ({revisions: [...st.revisions, next]}), 'Changed from today; earlier days keep their targets');
    },
    ownCommit: n => { const ctx = R.derive(S()); A().set({ownCommitments: {from: R.weekId(ctx.calendarDate), strength: Number(n), im: 0, note: 'Your first commitment; Garmin intensity goal stays separate'}}, 'Commitment added'); },
    addField: arg => { const [k, v] = arg.split(/:(.*)/); const cur = ui().add || {sport: null, date: R.derive(S()).calendarDate, time: ''}; const sheet = history.state.__raringExperience.sheet; if (sheet?.arg && !ui().add) { const i = S().intentions.find(x => x.id === sheet.arg); Object.assign(cur, {sport: i.sport, date: i.date, time: i.time || ''}); } setUi({add: {...cur, [k]: v}}); },
    addTime: v => { const cur = ui().add || {sport: null, date: R.derive(S()).calendarDate, time: ''}; setUi({add: {...cur, time: v}}); },
    saveIntention: id => {
      const d = ui().add; const st = S();
      const editing = id && st.intentions.find(i => i.id === id);
      const sport = d?.sport || editing?.sport; if (!sport) return;
      const date = d?.date || editing?.date, time = d ? d.time || null : editing?.time || null;
      const when = `${dayLabel(date)}${time ? `, ${time}` : ', any time'}`;
      const now = dayLabel(R.derive(st).calendarDate);
      const list = editing ? st.intentions.map(i => i.id === id ? {...i, sport, date, time, history: [...i.history, [now, `Changed to ${SPORTS[sport]}, ${when}`]]} : i)
        : [...st.intentions, {id: 'n' + Date.now().toString(36), sport, date, time, link: null, history: [[now, `Added for ${when}`]]}];
      A().set({intentions: list}, editing ? 'Plan updated · not sent to Garmin' : 'Added to your plan · on this phone, not Garmin');
      A().closeSheet();
    },
    removeIntention: id => A().set(st => ({intentions: st.intentions.filter(i => i.id !== id)}), 'Removed from your plan. Garmin unchanged.'),
    link: arg => { const [id, aid] = arg.split(':'); const now = dayLabel(R.derive(S()).calendarDate); A().set(st => ({intentions: st.intentions.map(i => i.id === id ? {...i, link: aid, history: [...i.history, [now, 'You linked a recorded activity']]} : i)}), 'Linked by you'); },
    unlink: id => { const now = dayLabel(R.derive(S()).calendarDate); A().set(st => ({intentions: st.intentions.map(i => i.id === id ? {...i, link: null, history: [...i.history, [now, 'Link removed']]} : i)}), 'Link removed'); },
    winddown: () => { const ctx = R.derive(S()); const k = ctx.tonight.nightOf; A().set(st => ({winddown: {...st.winddown, [k]: !st.winddown[k]}}), 'Wind-down self-report saved'); },
    bedtime: () => { const st = S(); if (st.permission === 'declined') { A().toast('Notifications are declined. You can allow them in Android settings; Raring won’t ask again.'); return; } if (st.permission === 'not-asked') return A().openSheet('permission', 'bedtime'); A().set({notifyBedtime: !st.notifyBedtime}, st.notifyBedtime ? 'Wind-down reminder off' : 'Wind-down reminder on'); },
    channel: (v, el) => { const st = S(), key = el.dataset.key; if (el.checked && st.permission !== 'granted') { el.checked = false; if (st.permission === 'declined') return A().toast('Permission declined. Change it in Android settings; no repeated prompt.'); return A().openSheet('permission', key); } A().set({channels: {...st.channels, [key]: el.checked}}); },
    permit: (v) => { const key = history.state.__raringExperience.sheet.arg; A().set(st => ({permission: v, ...(key === 'bedtime' ? {notifyBedtime: v === 'granted'} : {channels: {...st.channels, [key]: v === 'granted'}})})); A().closeSheet(); },
    winddownIntention: () => { A().set({winddownIntention: true}, 'Separate optional wind-down intention added; nothing marked complete'); A().closeSheet(); },
    seenMigration: arg => { A().set(st => ({seen: {...st.seen, timingMigration: true}})); if (arg === 'winddown') A().openSheet('winddown'); },
    journalAllow: () => { A().set(st => ({journal: {...st.journal, consent: true, downloaded: true}}), 'Journal reads allowed'); A().closeSheet(); },
    journalOpen: () => A().set(st => ({journal: {...st.journal, open: true}})),
    journalClose: () => A().set(st => ({journal: {...st.journal, open: false}})),
    export: () => A().toast('Export created (prototype: no file). Includes local plans, circumstances, self-reports and downloaded journal logs; no tokens.'),
    signout: () => { A().sheetThen(() => { A().reset('simple'); A().external('onboarding'); A().toast('Prototype choices erased. Fresh synthetic data shown; no Garmin account was connected.'); }); },
    keep: () => A().toast('Commitments kept for next week'),
    connect: () => A().toast('Would open Garmin Connect (prototype).'),
    filter: f => setUi({filter: f}),
    clearDate: () => setUi({date: null}),
    activitiesFor: date => { A().navigate('activities'); setTimeout(() => setUi({date, filter: 'all'}), 60); },
    range: r => setUi({range: r, sel: null}),
    cmp: v => setUi({cmp: v}),
    from: v => setUi({from: v}),
    to: v => setUi({to: v}),
    compareFrom: v => setUi({compareFrom: v}),
    hrDay: v => setUi({hrDay: addDays(ui().hrDay || R.derive(S()).calendarDate, Number(v))}),
    sel: v => setUi({sel: Number(v)}),
    search: v => { setUi({q: v}); const el = document.querySelector('.search input'); if (el) { el.focus(); el.setSelectionRange(v.length, v.length); } },
    note: v => A().set(st => ({limits: {...st.limits, note: v.trim()}}), v.trim() ? 'Movement note saved' : 'Movement note removed'),
    reviewDate: v => { S()._review = v || null; },
    saveAmbition: () => { const n = document.getElementById('amb-name').value.trim(); const d = document.getElementById('amb-date').value; if (!n) return; A().set(st => ({ambitionsAdded: [...st.ambitionsAdded, {id: 'amb' + Date.now(), name: n, date: d || null, source: d ? 'Your local date · not a Garmin race' : 'Your direction · no date'}]}), 'Ambition added'); A().closeSheet(); },
    follow: (v, el) => A().set({follow: el.checked}),
    quiet: (v, el) => A().set({quiet: el.checked}),
    journal: (v, el) => A().set(st => ({journal: {...st.journal, consent: el.checked, open: false, downloaded: st.journal.downloaded || el.checked}}), el.checked ? 'Journal reads allowed' : 'Journal reads off. Downloaded logs stay until you erase local data.'),
    // Prototype controls
    profile: v => { document.getElementById('scenario')?.close(); A().reset(v); },
    day: v => A().set({day: v}),
    clock: v => A().set({clock: v}),
    offline: (v, el) => A().set({offline: el.checked}),
    irregular: (v, el) => A().set({irregular: el.checked}),
    napCase: v => A().set({napCase: v}),
    calendar: v => A().set({calendar: v}),
    sync: arg => A().sync(arg),
    bg: () => A().background(),
    reload: () => location.reload(),
    resetAll: () => { document.getElementById('scenario')?.close(); A().reset(S().profile); },
    case: id => { if (!id) return; document.getElementById('scenario')?.close(); A().applyCase(window.RaringCases.find(c => c.id === id)); },
    noop: () => {},
  };

  window.RaringPages = {shell, title, scenarioBody: scenarioDrawer};
})();
