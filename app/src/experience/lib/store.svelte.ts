// Prototype shell state: the scenario, caller-aware navigation backed by browser
// history, sheets and toasts. Only prototype choices persist, in this browser.
import {derive, latestReviewWeek, type Ctx, type Profile, type State} from './rules';
import {activityById, EXERCISES, SCHEDULE} from './world';
import {METRICS} from './metrics';

const STORE = 'raring-experience-prototype-v2';
const KEY = '__raringExperience';

export const base = (): State => ({
  profile: 'simple', day: '2026-11-18', clock: 'day', offline: false, follow: true, irregular: false,
  rest: null, pause: null, pauses: [], limits: {exclude: [], note: '', routines: {}}, rehab: {items: [], done: {}},
  intentions: [], dismissed: {}, journal: {consent: false, open: false, downloaded: false}, winddown: {},
  revisions: [], ownCommitments: null, ambitionsAdded: [], celebrated: {}, seen: {}, quiet: false,
  vitalsException: false, extraStrength: false, napCase: 'same', calendar: 'complete', notifyBedtime: false,
  permission: 'not-asked', channels: {morning: true, weekly: true, metrics: true, vitals: true, training: false, status: false},
  signedOut: false, dst: false, returning: false, noRunning: false, savedSleepTarget: false, winddownIntention: false,
  navVariant: 'tabs', reviewDraft: null,
});
const PROFILE_DEFAULTS: Record<Profile, Partial<State>> = {
  simple: {}, sleep: {napCase: 'same'}, sparse: {pauses: [{since: '2026-10-19', until: '2026-10-25', review: null}]},
  mixed: {
    intentions: [
      {id: 'i0', sport: 'strength', date: '2026-11-16', time: '07:00', link: null, history: [['Sun 15 Nov', 'Added for Mon 16 Nov, 07:00']]},
      {id: 'i1', sport: 'strength', date: '2026-11-18', time: '18:30', link: null, history: [['Sun 15 Nov', 'Added for Wed 18 Nov, 18:30']]},
      {id: 'i2', sport: 'cycling', date: '2026-11-20', time: null, link: null, history: [['Mon 16 Nov', 'Added for Fri 20 Nov, any time']]},
    ],
    savedSleepTarget: true, pauses: [{since: '2026-10-19', until: '2026-10-25', review: null}],
  },
};
export const fresh = (profile: Profile, extra: Partial<State> = {}): State => ({...base(), ...structuredClone(PROFILE_DEFAULTS[profile] || {}), profile, ...extra});

// Old route → current canonical route. Explicit, so none silently becomes Today.
export const LEGACY: Record<string, string> = {
  today: 'today', sleep: 'sleep', training: 'training', health: 'health', readiness: 'readiness',
  checkin: 'review/latest', plan: 'plan', 'plan-setup': 'plan/commitments', settings: 'settings',
  onboarding: 'onboarding', strength: 'strength', help: 'help',
};
const DETAIL_DOMAINS = ['sleep', 'readiness', 'training', 'strength', 'activities', 'health'];
const HELP_PAGES = ['strength-start', 'strength-sets', 'strength-muscles', 'strength-watches', 'sync', 'adapt', 'acute-load', 'legend'];

export type Sheet = {name: string, arg?: string} | null;
export type Ui = Record<string, unknown> & {scroll?: number};

class Store {
  state = $state<State>(fresh('simple'));
  trail = $state<string[]>(['today']);
  sheet = $state<Sheet>(null);
  ui = $state<Ui>({});
  notice = $state<string | null>(null);
  toastText = $state('');
  toastOn = $state(false);
  version = $state(0);
  scroller: HTMLElement | null = null;
  private desired: string[] = ['today'];
  private processing = false;
  private waiting: (() => void) | null = null;
  private externalTarget: string[] | null = null;
  private restoreScroll: number | null = null;
  private toastTimer = 0;

  ctx = $derived.by<Ctx>(() => { void this.version; return derive(this.state); });
  route = $derived(this.trail.at(-1) ?? 'today');
  roots = $derived(this.state.navVariant === 'tabs' ? ['today', 'plan', 'progress', 'explore'] : ['today', 'plan', 'progress']);
  root = $derived(this.roots.includes(this.trail[1]) ? this.trail[1] : this.roots.includes(this.trail[0]) && this.trail.length === 1 ? this.trail[0] : 'today');

  load() {
    try {
      const saved = JSON.parse(localStorage.getItem(STORE) || 'null');
      this.state = saved?.profile ? {...base(), ...saved} : fresh('simple');
    } catch { this.state = fresh('simple'); }
    // Explicit opening never survives a cold start.
    this.state.journal.open = false;
  }
  save() { localStorage.setItem(STORE, JSON.stringify(this.state)); }
  set(patch: Partial<State> | ((s: State) => Partial<State> | void), message?: string) {
    const next = typeof patch === 'function' ? patch(this.state) || {} : patch;
    Object.assign(this.state, next);
    this.save();
    if (message) this.toast(this.state.offline ? `${message} · saved on this phone (offline)` : message);
  }
  toast(text: string) {
    this.toastText = text; this.toastOn = true;
    clearTimeout(this.toastTimer);
    this.toastTimer = window.setTimeout(() => this.toastOn = false, 3200);
  }

  // ---------- routes ----------
  validRoute(route: string): boolean {
    if (['today', 'plan', 'progress', 'explore', 'onboarding', ...DETAIL_DOMAINS].includes(route)) return true;
    if (['gear', 'trends', 'reviews', 'settings', 'help', 'adapt', 'surfaces', 'legacy', 'compare', 'plan/commitments', 'health/weight', 'health/journal', 'strength/muscles', 'training/performance'].includes(route)) return true;
    if (/^sleep\/(tonight|patterns|naps|stages|timing)$/.test(route)) return true;
    if (/^adapt\/(pause|limits|routines|lighter)$/.test(route)) return true;
    if (/^settings\/(export|journal)$/.test(route)) return true;
    if (/^workout\/[\w-]+$/.test(route)) return !!activityById(route.slice(8));
    if (/^metric\/\w+$/.test(route)) return route.slice(7) in METRICS;
    if (/^strength\/[\w-]+$/.test(route)) return EXERCISES.some(e => `strength/${e.id}` === route);
    if (/^intention\/\w+$/.test(route)) return true;
    if (/^scheduled\/[\w-]+$/.test(route)) return SCHEDULE.some(s => `scheduled/${s.id}` === route);
    if (/^review\/\d{4}-W\d{2}$/.test(route)) return true;
    if (/^help\/[\w-]+$/.test(route)) return HELP_PAGES.includes(route.slice(5));
    return false;
  }
  canonical(raw: string): {route: string, alias?: string, invalid?: string} {
    let route = raw.replace(/^#/, '');
    try { route = decodeURIComponent(route); } catch { return {route: 'today', invalid: raw}; }
    if (route === '') return {route: 'today'};
    if (route in LEGACY && LEGACY[route] !== route) return {route: this.resolveAlias(LEGACY[route]), alias: route};
    route = this.resolveAlias(route);
    if (this.validRoute(route)) return {route};
    return {route: 'today', invalid: route};
  }
  resolveAlias(route: string) { return route === 'review/latest' ? `review/${latestReviewWeek(this.ctx)}` : route; }
  parentOf(route: string): string {
    if (['sleep', 'readiness', 'adapt', 'settings', 'compare'].includes(route)) return 'today';
    if (route === 'explore') return 'today';
    if (['training', 'strength', 'activities', 'health', 'trends', 'help'].includes(route)) return 'explore';
    if (route === 'gear') return 'training';
    if (route === 'reviews' || route.startsWith('review/')) return 'progress';
    if (route.startsWith('sleep/')) return 'sleep';
    if (route.startsWith('adapt/')) return 'adapt';
    if (route.startsWith('health/')) return 'health';
    if (route.startsWith('settings/')) return 'settings';
    if (route.startsWith('help/')) return 'help';
    if (route.startsWith('training/')) return 'training';
    if (route.startsWith('workout/')) return 'activities';
    if (route.startsWith('metric/')) return METRICS[route.slice(7)].parent;
    if (route.startsWith('strength/')) return 'strength';
    if (route.startsWith('intention/') || route.startsWith('scheduled/') || route === 'plan/commitments') return 'plan';
    return 'today';
  }
  rootTrail(r: string) { return r === 'today' ? ['today'] : ['today', r]; }
  rootOf(trail: string[]) { return this.roots.includes(trail[1]) ? trail[1] : 'today'; }
  defaultTrail(route: string): string[] {
    if (route === 'surfaces' || route === 'legacy') return ['today', route];
    if (this.roots.includes(route)) return this.rootTrail(route);
    return [...this.defaultTrail(this.parentOf(route)), route];
  }
  nextTrail(route: string, from: string[]) {
    if (this.roots.includes(route)) return this.rootTrail(route);
    const i = from.indexOf(route);
    if (i >= 0) return from.slice(0, i + 1);
    if (route === 'explore') return [...this.rootTrail(this.rootOf(from)), 'explore'];
    if (route === 'settings') return [...from.filter(r => r !== 'settings' && !r.startsWith('settings/')), 'settings'];
    return [...from, route];
  }

  // ---------- history-backed trail ----------
  private same = (a: string[], b: string[]) => a.join('\0') === b.join('\0');
  private entry = (t: string[], extra: Record<string, unknown> = {}) => ({[KEY]: {trail: t, ...extra}});
  private owned() {
    const s = history.state && history.state[KEY];
    return s && Array.isArray(s.trail) && s.trail.at(-1) === location.hash.slice(1) ? s as {trail: string[], ui?: Ui, sheet?: Sheet} : null;
  }
  private go(n: number) { return new Promise<void>(resolve => { this.waiting = resolve; history.go(n); }); }
  private stashUi() {
    const s = history.state && history.state[KEY];
    if (s) history.replaceState({[KEY]: {...s, ui: {...$state.snapshot(this.ui), scroll: this.scroller?.scrollTop ?? 0}}}, '', location.hash);
  }
  setUi(patch: Ui) {
    this.ui = {...this.ui, ...patch};
    const s = history.state && history.state[KEY];
    if (s) history.replaceState({[KEY]: {...s, ui: $state.snapshot(this.ui)}}, '', location.hash);
  }
  private async apply() {
    if (this.processing) return;
    this.processing = true;
    try {
      while (!this.same(this.trail, this.desired)) {
        if (this.sheet) { await this.go(-1); this.sheet = null; continue; }
        const next = this.desired;
        let common = 0;
        while (common < this.trail.length && common < next.length && this.trail[common] === next[common]) common++;
        if (common === 0) common = 1;
        if (common < this.trail.length) {
          await this.go(common - this.trail.length);
          const s = this.owned();
          this.trail = s ? s.trail : this.trail.slice(0, common);
          this.ui = s?.ui ?? {};
          this.restoreScroll = (this.ui.scroll as number) ?? 0;
          continue;
        }
        this.stashUi();
        let t = this.trail;
        for (const page of next.slice(common)) { t = [...t, page]; history.pushState(this.entry(t), '', `#${page}`); }
        this.trail = t;
        this.ui = {};
        this.restoreScroll = 0;
      }
    } finally { this.processing = false; }
    this.leaveEffects();
    this.afterRender();
  }
  navigate(route: string) {
    if (this.sheet) { this.sheetThen(() => this.navigate(route)); return; }
    const c = this.canonical(route);
    this.desired = this.nextTrail(c.route, this.desired);
    void this.apply();
  }
  external(raw: string, label?: string) {
    const c = this.canonical(raw);
    this.notice = c.invalid ? `That link (“${c.invalid}”) isn’t a Raring page any more, so Today opened instead.` : c.alias ? `Old link #${c.alias} opened its current page.` : label || null;
    this.desired = this.defaultTrail(c.route);
    if (this.sheet) { this.sheetThen(() => void this.apply()); return; }
    void this.apply();
  }
  back() {
    if (this.sheet) { history.back(); return; }
    if (this.desired.length <= 1) return;
    this.desired = this.desired.slice(0, -1);
    void this.apply();
  }
  openSheet(name: string, arg?: string) {
    if (this.sheet) { this.sheet = {name, arg}; history.replaceState({[KEY]: {...history.state[KEY], sheet: {name, arg}}}, '', location.hash); return; }
    this.stashUi();
    this.sheet = {name, arg};
    history.pushState({[KEY]: {...history.state[KEY], sheet: {name, arg}}}, '', location.hash);
  }
  closeSheet() { if (this.sheet) history.back(); }
  sheetThen(fn: () => void) {
    if (!this.sheet) { fn(); return; }
    const after = () => { window.removeEventListener('popstate', after); setTimeout(fn, 0); };
    window.addEventListener('popstate', after);
    history.back();
  }
  private foreign() {
    if (this.externalTarget) return;
    const c = this.canonical(location.hash);
    this.notice = c.invalid ? `That link (“${c.invalid}”) isn’t a Raring page any more, so Today opened instead.` : c.alias ? `Old link #${c.alias} opened its current page.` : null;
    this.externalTarget = this.defaultTrail(c.route);
    history.go(-1);
  }
  private onPop = () => {
    if (this.waiting) { const r = this.waiting; this.waiting = null; r(); return; }
    if (this.externalTarget) {
      const t = this.externalTarget; this.externalTarget = null;
      const s = this.owned();
      if (s) { this.trail = s.trail; this.sheet = s.sheet || null; }
      this.desired = t; void this.apply();
      return;
    }
    const s = this.owned();
    if (s) {
      const wasSheet = !!this.sheet;
      this.trail = this.desired = s.trail; this.sheet = s.sheet || null; this.ui = s.ui || {};
      this.restoreScroll = wasSheet && !this.sheet ? null : (this.ui.scroll as number) ?? 0;
      this.leaveEffects(); this.afterRender();
    } else this.foreign();
  };
  // Any route change closes journal entries; backgrounding also hides them.
  private journalRoute: string | null = null;
  private leaveEffects() {
    const top = this.trail.at(-1) || 'today';
    if (this.state.journal.open && this.journalRoute !== top) { this.state.journal.open = false; this.save(); }
    this.journalRoute = top;
    const lead = this.ctx.lead;
    if (top === 'today' && lead.celebrate && !this.state.celebrated[lead.celebrate]) {
      // Seen once; reopening never replays it.
      setTimeout(() => { this.state.celebrated[lead.celebrate!] = new Date().toISOString(); this.save(); }, 1600);
    }
  }
  background() { if (this.state.journal.open) { this.state.journal.open = false; this.save(); } }
  private afterRender() {
    const want = this.restoreScroll;
    this.restoreScroll = null;
    requestAnimationFrame(() => {
      if (!this.scroller) return;
      const r = this.route;
      const anchor = r.includes('/') && !r.startsWith('metric/') && !r.startsWith('workout/') ? document.getElementById(`sec-${r.split('/')[1]}`) : null;
      if (want === 0 && anchor) anchor.scrollIntoView({block: 'start'});
      else if (want != null) this.scroller.scrollTop = want;
      if (want === 0) (document.querySelector('main h1') as HTMLElement | null)?.focus({preventScroll: true});
    });
  }
  parentLabel = $derived.by(() => { const p = this.trail.at(-2); return p ? titleOf(p) : 'Today'; });

  start() {
    this.load();
    window.addEventListener('popstate', this.onPop);
    window.addEventListener('hashchange', () => { if (!this.owned() && !this.processing && !this.waiting && !this.externalTarget) this.foreign(); });
    document.addEventListener('visibilitychange', () => { if (document.hidden) this.background(); });
    const params = new URLSearchParams(location.search);
    if (params.get('text')) document.documentElement.style.setProperty('--text-scale', params.get('text'));
    const caseId = params.get('case');
    if (caseId) {
      params.delete('case');
      const q = params.toString();
      history.replaceState(null, '', `${location.pathname}${q ? `?${q}` : ''}${location.hash}`);
      if (applyCaseHook) { applyCaseHook(caseId, true); return; }
    }
    const s = this.owned();
    if (s) { this.trail = this.desired = s.trail; this.ui = s.ui || {}; this.sheet = s.sheet || null; this.restoreScroll = (this.ui.scroll as number) ?? 0; this.afterRender(); return; }
    const c = this.canonical(location.hash || '#today');
    if (c.alias) this.notice = `Old link #${c.alias} opened its current page.`;
    if (c.invalid) this.notice = `That link (“${c.invalid}”) isn’t a Raring page any more, so Today opened instead.`;
    this.startWith(this.defaultTrail(c.route));
  }
  startWith(t: string[]) {
    let trail = [t[0]];
    history.replaceState(this.entry(trail), '', `#${t[0]}`);
    for (const page of t.slice(1)) { trail = [...trail, page]; history.pushState(this.entry(trail), '', `#${page}`); }
    this.trail = this.desired = trail;
    this.sheet = null;
    this.ui = {};
    this.restoreScroll = 0;
    this.leaveEffects();
    this.afterRender();
  }
  reset(profile?: Profile, extra?: Partial<State>) {
    this.state = fresh(profile || this.state.profile, extra); this.save(); this.notice = null; this.external('today');
  }
}

let applyCaseHook: ((id: string, cold: boolean) => void) | null = null;
export const setCaseHook = (fn: (id: string, cold: boolean) => void) => { applyCaseHook = fn; };

const TITLES: Record<string, string> = {
  onboarding: 'Welcome', today: 'Today', plan: 'Plan', progress: 'Progress', explore: 'Explore', sleep: 'Sleep', readiness: 'Readiness', training: 'Training',
  strength: 'Strength', activities: 'Activities', health: 'Health', gear: 'Gear', trends: 'Trends', reviews: 'Weekly reviews',
  settings: 'Settings', help: 'Help', adapt: 'Adapt', surfaces: 'Status surfaces', legacy: 'Old links', compare: 'Compare', 'plan/commitments': 'Commitments',
  'health/weight': 'Weight', 'health/journal': 'Journal', 'sleep/patterns': 'Sleep patterns', 'strength/muscles': 'Muscles', 'training/performance': 'Performance',
};
export function titleOf(route: string): string {
  if (TITLES[route]) return TITLES[route];
  const [a, b] = route.split('/');
  if (a === 'workout') { const x = activityById(b); return x ? x.name : 'Workout'; }
  if (a === 'metric') return METRICS[b]?.name || 'Metric';
  if (a === 'strength') return EXERCISES.find(e => e.id === b)?.name || 'Exercise';
  if (a === 'intention') return 'Your plan';
  if (a === 'scheduled') return SCHEDULE.find(s => s.id === b)?.name ?? 'Garmin schedule';
  if (a === 'review') return `Week ${b.slice(6)}`;
  if (a === 'help') return 'Help';
  if (TITLES[a]) return TITLES[a];
  return route;
}

export const store = new Store();
