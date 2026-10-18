// Experience prototype shell: state, caller-aware navigation, sheets and the
// prototype-only scenario drawer. Pages live in pages.js. Synthetic data only.
(function () {
  const R = window.RaringRules;
  const STORE = 'raring-experience-prototype-v1';
  const ROOTS = ['today', 'plan', 'progress'];
  const DETAIL_DOMAINS = ['sleep', 'readiness', 'training', 'strength', 'activities', 'health'];

  const base = () => ({
    profile: 'simple', day: '2026-11-18', clock: 'day', offline: false, follow: true, irregular: false,
    rest: null, pause: null, pauses: [], limits: {exclude: [], note: '', routines: {}}, rehab: {items: [], done: {}},
    intentions: [], dismissed: {}, journal: {consent: false, open: false, downloaded: false}, winddown: {},
    revisions: [], ownCommitments: null, ambitionsAdded: [], celebrated: {}, seen: {}, quiet: false,
    vitalsException: false, extraStrength: false, napCase: 'same', calendar: 'complete', notifyBedtime: false,
    permission: 'not-asked', channels: {morning: true, weekly: true, metrics: true, vitals: true, training: false, status: false}, signedOut: false, dst: false, returning: false,
  });
  const PROFILE_DEFAULTS = {
    simple: {},
    sleep: {napCase: 'same'},
    mixed: {
      intentions: [
        {id: 'i0', sport: 'strength', date: '2026-11-16', time: '07:00', link: null, history: [['Sun 15 Nov', 'Added for Mon 16 Nov, 07:00']]},
        {id: 'i1', sport: 'strength', date: '2026-11-18', time: '18:30', link: null, history: [['Sun 15 Nov', 'Added for Wed 18 Nov, 18:30']]},
        {id: 'i2', sport: 'cycling', date: '2026-11-20', time: null, link: null, history: [['Mon 16 Nov', 'Added for Fri 20 Nov, any time']]},
      ],
      savedSleepTarget: true,
    },
    sparse: {},
  };
  const fresh = (profile, extra = {}) => ({...base(), ...structuredClone(PROFILE_DEFAULTS[profile] || {}), profile, ...extra});

  let state;
  try { state = {...base(), ...JSON.parse(localStorage.getItem(STORE) || 'null')}; } catch { state = null; }
  if (!state || !state.profile) state = fresh('simple');
  state.journal.open = false; // Explicit opening never survives a cold start.
  const save = () => localStorage.setItem(STORE, JSON.stringify(state));

  // ---------- routes ----------
  const LEGACY = {
    // Old route → new canonical route. Explicit so none silently becomes Today.
    today: 'today', sleep: 'sleep', training: 'training', health: 'health', readiness: 'readiness',
    checkin: 'review/latest', plan: 'plan', 'plan-setup': 'plan/commitments', settings: 'settings',
    onboarding: 'onboarding', strength: 'strength', help: 'help',
  };
  const METRIC_IDS = Object.keys(window.RaringData.metrics);
  function validRoute(route) {
    if (ROOTS.includes(route) || DETAIL_DOMAINS.includes(route) || route === 'onboarding') return true;
    if (['explore', 'gear', 'trends', 'reviews', 'settings', 'help', 'adapt', 'surfaces', 'legacy', 'plan/commitments', 'health/weight', 'review/latest'].includes(route)) return true;
    if (/^sleep\/(tonight|patterns|naps|stages|timing)$/.test(route)) return true;
    if (route === 'training/performance' || route === 'strength/muscles') return true;
    if (/^workout\/[\w-]+\/evidence$/.test(route)) return validRoute(route.replace('/evidence', ''));
    if (/^adapt\/(pause|limits|routines|lighter)$/.test(route)) return true;
    if (/^health\/journal$/.test(route)) return true;
    if (/^settings\/(export|journal)$/.test(route)) return true;
    if (/^workout\/a-\d+$/.test(route) && window.RaringData.activities.concat(window.RaringData.extraActivity).some(a => `workout/${a.id}` === route)) return true;
    if (/^metric\/\w+$/.test(route)) return METRIC_IDS.includes(route.slice(7));
    if (/^strength\/[\w-]+$/.test(route)) return window.RaringData.exercises.some(e => `strength/${e.id}` === route);
    if (/^intention\/\w+$/.test(route)) return true;
    if (/^scheduled\/[\w-]+$/.test(route)) return window.RaringData.schedule.some(s => `scheduled/${s.id}` === route);
    if (/^review\/\d{4}-W\d{2}$/.test(route)) return true;
    if (/^help\/[\w-]+$/.test(route)) return route.slice(5) in window.RaringData.helpPages;
    return false;
  }
  function canonical(raw) {
    let route = raw.replace(/^#/, '');
    try { route = decodeURIComponent(route); } catch { return {route: 'today', invalid: raw}; }
    if (route === '' ) return {route: 'today'};
    if (route in LEGACY && LEGACY[route] !== route) return {route: resolveAlias(LEGACY[route]), alias: route};
    route = resolveAlias(route);
    if (validRoute(route)) return {route};
    return {route: 'today', invalid: route};
  }
  function resolveAlias(route) {
    if (route === 'review/latest') return `review/${latestReviewWeek()}`;
    return route;
  }
  function latestReviewWeek() {
    const ctx = R.derive(state);
    // The review for a week appears on its Sunday; otherwise the previous week's.
    const dow = new Date(ctx.calendarDate + 'T00:00:00Z').getUTCDay();
    return dow === 0 ? R.weekId(ctx.calendarDate) : R.weekId(R.addDays(ctx.weekFrom, -1));
  }
  const METRIC_PARENT = id => ({readiness: 'readiness', sleepScore: 'sleep'})[id] || window.RaringData.metrics[id].domain;
  function parentOf(route) {
    if (route === 'sleep' || route === 'readiness' || route === 'explore' || route === 'adapt' || route === 'settings') return 'today';
    if (['training', 'strength', 'activities', 'health', 'trends', 'help'].includes(route)) return 'explore';
    if (route === 'gear') return 'training';
    if (route === 'reviews' || route.startsWith('review/')) return 'progress';
    if (route.startsWith('sleep/')) return 'sleep';
    if (route.startsWith('adapt/')) return 'adapt';
    if (route.startsWith('health/')) return 'health';
    if (route.startsWith('settings/')) return 'settings';
    if (route.startsWith('help/')) return 'help';
    if (route.startsWith('training/')) return 'training';
    if (route.startsWith('workout/')) return route.endsWith('/evidence') ? route.replace('/evidence', '') : 'activities';
    if (route.startsWith('metric/')) return METRIC_PARENT(route.slice(7));
    if (route.startsWith('strength/')) return 'strength';
    if (route.startsWith('intention/') || route.startsWith('scheduled/') || route === 'plan/commitments') return 'plan';
    return 'today';
  }
  const rootTrail = r => r === 'today' ? ['today'] : ['today', r];
  const rootOf = trail => ROOTS.includes(trail[1]) ? trail[1] : 'today';
  function defaultTrail(route) {
    if (route === 'surfaces' || route === 'legacy') return ['today', route];
    if (ROOTS.includes(route)) return rootTrail(route);
    return [...defaultTrail(parentOf(route)), route];
  }
  function nextTrail(route, from) {
    if (ROOTS.includes(route)) return rootTrail(route);
    const i = from.indexOf(route);
    if (i >= 0) return from.slice(0, i + 1);
    if (route === 'explore') return [...rootTrail(rootOf(from)), 'explore'];
    if (route === 'settings') return [...from.filter(r => r !== 'settings' && !r.startsWith('settings/')), 'settings'];
    return [...from, route];
  }

  // ---------- history-backed trail (mirrors the app's navigation.ts approach) ----------
  const KEY = '__raringExperience';
  let trail = [], desired = [], processing = false, waiting = null, sheet = null, ui = {}, restoreScroll = null, notice = null;
  const same = (a, b) => a.join('\0') === b.join('\0');
  const entry = (t, extra = {}) => ({[KEY]: {trail: t, ...extra}});
  const owned = () => { const s = history.state && history.state[KEY]; return s && Array.isArray(s.trail) && s.trail.at(-1) === location.hash.slice(1) ? s : null; };
  const go = n => new Promise(resolve => { waiting = resolve; history.go(n); });

  function stashUi() {
    const s = history.state && history.state[KEY];
    if (s) history.replaceState({[KEY]: {...s, ui: {...ui, scroll: window.scrollY}}}, '', location.hash);
  }
  async function apply() {
    if (processing) return;
    processing = true;
    try {
      while (!same(trail, desired)) {
        if (sheet) { await go(-1); sheet = null; continue; }
        const next = desired;
        let common = 0;
        while (common < trail.length && common < next.length && trail[common] === next[common]) common++;
        if (common === 0) common = 1;
        if (common < trail.length) {
          await go(common - trail.length);
          const s = owned();
          trail = s ? s.trail : trail.slice(0, common);
          ui = s && s.ui ? s.ui : {};
          restoreScroll = ui.scroll ?? 0;
          continue;
        }
        stashUi();
        for (const page of next.slice(common)) {
          trail = [...trail, page];
          history.pushState(entry(trail), '', `#${page}`);
        }
        ui = {};
        restoreScroll = null;
      }
    } finally { processing = false; }
    leaveEffects();
    render();
  }
  function navigate(route) {
    const c = canonical(route);
    desired = nextTrail(c.route, desired);
    void apply();
  }
  function external(raw, label) {
    const c = canonical(raw);
    notice = c.invalid ? `That link (“${c.invalid}”) isn’t a Raring page any more, so Today opened instead.` : c.alias ? `Old link #${c.alias} opened its current page.` : label || null;
    desired = defaultTrail(c.route);
    void apply();
  }
  function back() {
    if (sheet) { history.back(); return; }
    if (desired.length <= 1) return;
    desired = desired.slice(0, -1);
    void apply();
  }
  function openSheet(name, arg) {
    if (sheet) { sheet = {name, arg}; history.replaceState({[KEY]: {...history.state[KEY], sheet}}, '', location.hash); render(); return; }
    stashUi();
    sheet = {name, arg};
    history.pushState({[KEY]: {...history.state[KEY], sheet}}, '', location.hash);
    render();
    requestAnimationFrame(() => document.querySelector('.sheet [data-autofocus], .sheet h2')?.focus());
  }
  const closeSheet = () => { if (sheet) history.back(); };

  // A followed or typed fragment link adds a browser entry we don't own. Rewind it,
  // then build the page's canonical trail so Back can't revisit an obsolete entry.
  let externalTarget = null;
  function foreign() {
    if (externalTarget) return;
    const c = canonical(location.hash);
    notice = c.invalid ? `That link (“${c.invalid}”) isn’t a Raring page any more, so Today opened instead.` : c.alias ? `Old link #${c.alias} opened its current page.` : null;
    externalTarget = defaultTrail(c.route);
    history.go(-1);
  }
  window.addEventListener('popstate', () => {
    if (waiting) { const r = waiting; waiting = null; r(); return; }
    if (externalTarget) {
      const t = externalTarget; externalTarget = null;
      const s = owned();
      if (s) { trail = s.trail; sheet = s.sheet || null; }
      desired = t; void apply();
      return;
    }
    const s = owned();
    if (s) {
      const wasSheet = !!sheet;
      trail = desired = s.trail; sheet = s.sheet || null; ui = s.ui || {};
      restoreScroll = wasSheet && !sheet ? null : ui.scroll ?? 0;
      leaveEffects(); render(!wasSheet || !!sheet);
    } else foreign();
  });
  window.addEventListener('hashchange', () => { if (!owned() && !processing && !waiting && !externalTarget) foreign(); });
  document.addEventListener('visibilitychange', () => { if (document.hidden) background(); });

  // Any route change closes entries; backgrounding also removes the rendered rows.
  let journalRoute = null;
  function leaveEffects() {
    const top = trail.at(-1) || 'today';
    if (state.journal.open && journalRoute !== top) { state.journal.open = false; save(); }
    journalRoute = top;
  }
  function background() {
    if (state.journal.open) { state.journal.open = false; save(); render(true); }
  }

  // ---------- state helpers used by pages ----------
  let toastText = '';
  function toast(text) {
    toastText = text;
    const el = document.getElementById('toast');
    if (el) { el.textContent = text; el.classList.add('on'); clearTimeout(toast.t); toast.t = setTimeout(() => el.classList.remove('on'), 3200); }
  }
  function set(patch, message) {
    Object.assign(state, typeof patch === 'function' ? patch(state) || {} : patch);
    save();
    render(true);
    if (message) toast(state.offline ? `${message} · saved on this phone (offline)` : message);
  }

  // ---------- rendering ----------
  function render(keepScroll) {
    const P = window.RaringPages;
    const route = trail.at(-1) || 'today';
    journalRoute = route;
    const ctx = R.derive(state);
    const env = {state, ctx, route, trail, ui, sheet, notice, rootOf: rootOf(trail), parentLabel: parentLabel(), R};
    const scrollY = window.scrollY;
    document.getElementById('app').innerHTML = P.shell(env);
    document.title = `${P.title(route, env)} · Raring prototype`;
    if (restoreScroll != null) { window.scrollTo(0, restoreScroll); restoreScroll = null; }
    else if (keepScroll) window.scrollTo(0, scrollY);
    else {
      const anchor = route.includes('/') ? document.getElementById(`sec-${route.split('/')[1]}`) : null;
      if (anchor && !route.startsWith('metric/') && !route.startsWith('workout/')) anchor.scrollIntoView({block: 'start'});
      else window.scrollTo(0, 0);
    }
    if (ctx.lead.celebrate && !state.celebrated[ctx.lead.celebrate] && route === 'today') {
      state.celebrated[ctx.lead.celebrate] = new Date().toISOString(); save();
    }
    if (!keepScroll && !sheet) document.querySelector('main h1')?.focus({preventScroll: true});
  }
  function parentLabel() {
    const p = trail.at(-2);
    return p ? window.RaringPages.title(p, {state, ctx: R.derive(state)}) : 'Today';
  }

  // ---------- events ----------
  document.addEventListener('click', e => {
    const el = e.target.closest('[data-go],[data-sheet],[data-act],[data-ext]');
    if (!el || el.disabled) return;
    e.preventDefault();
    const dlg = el.closest('dialog');
    if (dlg && (el.dataset.ext || el.dataset.go)) dlg.close();
    if (el.dataset.go) { if (sheet) { const to = el.dataset.go; sheetThen(() => navigate(to)); } else navigate(el.dataset.go); }
    else if (el.dataset.sheet) openSheet(el.dataset.sheet, el.dataset.arg);
    else if (el.dataset.ext) { const to = el.dataset.ext; sheetThen(() => external(to, el.dataset.label)); }
    else window.RaringActions[el.dataset.act]?.(el.dataset.arg, el);
  });
  document.addEventListener('change', e => {
    const el = e.target.closest('[data-change]');
    if (el) window.RaringActions[el.dataset.change]?.(el.value, el);
  });
  document.addEventListener('input', e => {
    const el = e.target.closest('[data-input]');
    if (el) window.RaringActions[el.dataset.input]?.(el.value, el);
  });
  document.addEventListener('keydown', e => {
    if (e.key === 'Escape' && document.getElementById('scenario')?.open) return;
    if (e.key === 'Escape' && sheet) closeSheet();
    if (e.key === 'Tab' && sheet) {
      const nodes = [...document.querySelectorAll('.sheet button:not(:disabled), .sheet input, .sheet select, .sheet textarea')];
      const first = nodes[0], last = nodes.at(-1);
      if (e.shiftKey && (document.activeElement === first || document.activeElement.tagName === 'H2')) { e.preventDefault(); last?.focus(); }
      else if (!e.shiftKey && document.activeElement === last) { e.preventDefault(); first?.focus(); }
    }
  });
  function sheetThen(fn) {
    if (!sheet) return fn();
    const after = () => { window.removeEventListener('popstate', after); setTimeout(fn, 0); };
    window.addEventListener('popstate', after);
    history.back();
  }

  window.RaringApp = {
    get state() { return state; }, set, toast, navigate, external, back, openSheet, closeSheet, sheetThen,
    reset(profile, extra) { state = fresh(profile || state.profile, extra); save(); notice = null; external('today'); },
    applyCase(c) { state = fresh(c.setup.profile || 'simple', c.setup); save(); notice = `Case ${c.id}: ${c.title}`; external(c.start || 'today'); },
    background() { background(); render(true); toast('Resumed: the app came back from the background. Your choices are unchanged.'); },
    sync(kind) {
      if (state.offline) { toast('Offline: sync waits for a connection. Your local choices already apply.'); return; }
      if (kind === 'exception') state.vitalsException = true;
      save(); render(true); toast(kind === 'exception' ? 'Synced: a new notice arrived; dismissed suggestions stay dismissed.' : 'Synced: nothing you chose has changed.');
    },
    clearNotice() { notice = null; render(true); },
    _ui(next) { ui = next; render(true); },
    openScenario() {
      const d = document.getElementById('scenario');
      d.innerHTML = window.RaringPages.scenarioBody({state});
      d.showModal();
    },
    defaultTrail, canonical, LEGACY, latestReviewWeek,
  };

  // ---------- start ----------
  function start() {
    const params = new URLSearchParams(location.search);
    if (params.get('text')) document.documentElement.style.setProperty('--text-scale', params.get('text'));
    // Media queries ignore the root font size, so measure how many rem fit across instead.
    const fit = () => { const rem = parseFloat(getComputedStyle(document.documentElement).fontSize); document.documentElement.classList.toggle('narrow-text', innerWidth / rem < 24); };
    fit(); window.addEventListener('resize', fit);
    const caseId = params.get('case');
    if (caseId) {
      const c = window.RaringCases.find(x => x.id === caseId);
      params.delete('case');
      const q = params.toString();
      history.replaceState(null, '', `${location.pathname}${q ? `?${q}` : ''}${location.hash}`);
      if (c) {
        state = fresh(c.setup.profile || 'simple', c.setup); save(); notice = `Case ${c.id}: ${c.title}`;
        const t = defaultTrail(canonical(c.start || 'today').route);
        startWith(t);
        return;
      }
    }
    const s = owned();
    if (s) { trail = desired = s.trail; ui = s.ui || {}; sheet = s.sheet || null; restoreScroll = ui.scroll ?? 0; render(); return; }
    const c = canonical(location.hash || '#today');
    if (c.alias) notice = `Old link #${c.alias} opened its current page.`;
    if (c.invalid) notice = `That link (“${c.invalid}”) isn’t a Raring page any more, so Today opened instead.`;
    startWith(defaultTrail(c.route));
  }
  function startWith(t) {
    trail = [t[0]];
    history.replaceState(entry(trail), '', `#${t[0]}`);
    for (const page of t.slice(1)) { trail = [...trail, page]; history.pushState(entry(trail), '', `#${page}`); }
    desired = trail;
    render();
  }
  window.addEventListener('DOMContentLoaded', start);
})();
