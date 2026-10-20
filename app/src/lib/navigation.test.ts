import { afterEach, describe, expect, it, vi } from 'vitest';
import { createNavigation, knownRoute, normalizeRoute, routeTrail, tabs } from './navigation';

describe('routes', () => {
  it('accepts known routes and rejects invalid or malformed fragments', () => {
    expect(tabs).toEqual(['today', 'plan', 'training', 'health', 'progress']);
    for (const route of [...tabs, 'sleep', 'readiness', 'settings', 'onboarding', 'strength', 'help', 'help/strength-sets',
      'activities', 'profile', 'gear', 'plan/commitments', 'review/latest', 'reviews', 'review/2025-10-13',
      'workout/123_abc-2', 'sleep/2025-10-14', 'health/2025-10-14', 'metric/stress/2025-10-14',
      'metric/heartRate/2025-10-14', ...['bodyBattery', 'stress', 'restingHr', 'hrv', 'respiration', 'spo2',
        'skinTemp', 'vo2max', 'endurance', 'hill', 'runningTolerance'].map(id => `metric/${id}`)]) {
      expect(normalizeRoute(`#${route}`)).toBe(route);
    }
    for (const route of ['#metric/unknown', '#workout/', '#workout/a/b', '#%ZZ', '#review/2025-1-1',
      '#sleep/other', '#sleep/2025-1-1', '#sleep/2025-13-01', '#health/2025-10-32', '#metric/bodyBattery/2025-10-14',
      '#metric/stress/2025-1-1', '#https://bad.example', '#onboarding/next', '#help/unknown', '#strength/x']) {
      expect(normalizeRoute(route)).toBe('today');
      expect(knownRoute(route)).toBeNull();
    }
  });

  it('keeps every earlier route as an alias for its current page', () => {
    expect(normalizeRoute('#checkin')).toBe('review/latest');
    expect(normalizeRoute('#plan-setup')).toBe('plan/commitments');
    expect(normalizeRoute('#explore')).toBe('training');
    expect(normalizeRoute('#metric/weight')).toBe('health/weight');
    expect(routeTrail('metric/weight')).toEqual(['today', 'health', 'health/weight']);
    for (const native of ['today', 'readiness', 'sleep', 'metric/bodyBattery', 'metric/stress', 'health', 'training']) {
      expect(normalizeRoute(native)).toBe(native);
    }
    expect(routeTrail('checkin')).toEqual(['today', 'progress', 'review/latest']);
    expect(routeTrail('plan-setup', ['today', 'health'])).toEqual(['today', 'plan', 'plan/commitments']);
    expect(routeTrail('plan-setup', ['today', 'progress', 'review/latest'])).toEqual(['today', 'progress', 'review/latest', 'plan/commitments']);
  });

  it('builds logical parents rather than visit order', () => {
    expect(routeTrail('health', ['today', 'sleep'])).toEqual(['today', 'health']);
    expect(routeTrail('readiness', ['today', 'training'])).toEqual(['today', 'readiness']);
    expect(routeTrail('plan', ['today', 'health'])).toEqual(['today', 'plan']);
    expect(routeTrail('sleep', ['today', 'health'])).toEqual(['today', 'health', 'sleep']);
    expect(routeTrail('sleep', ['today', 'plan'])).toEqual(['today', 'sleep']);
    // Sleep's tabs replace each other and keep their caller's section.
    expect(routeTrail('sleep/tonight', ['today', 'sleep/last'])).toEqual(['today', 'sleep/tonight']);
    expect(routeTrail('sleep/last', ['today', 'health', 'sleep/tonight'])).toEqual(['today', 'health', 'sleep/last']);
    expect(routeTrail('metric/hrv', ['today', 'sleep/last'])).toEqual(['today', 'sleep/last', 'metric/hrv']);
    expect(knownRoute('sleep/tonight')).toBe('sleep/tonight');
    expect(knownRoute('sleep/naps')).toBeNull();
    expect(routeTrail('workout/7')).toEqual(['today', 'training', 'activities', 'workout/7']);
    expect(routeTrail('workout/7', ['today'])).toEqual(['today', 'workout/7']);
    expect(routeTrail('workout/7', ['today', 'training'])).toEqual(['today', 'training', 'workout/7']);
    expect(routeTrail('workout/7', ['today', 'plan'])).toEqual(['today', 'plan', 'workout/7']);
    expect(routeTrail('workout/7', ['today', 'training', 'activities'])).toEqual(['today', 'training', 'activities', 'workout/7']);
    expect(routeTrail('settings', ['today', 'training', 'workout/7'])).toEqual(['today', 'training', 'workout/7', 'settings']);
    expect(routeTrail('settings', ['today', 'settings'])).toEqual(['today', 'settings']);
    expect(routeTrail('settings')).toEqual(['today', 'settings']);
    expect(routeTrail('profile', ['today', 'plan'])).toEqual(['today', 'plan', 'profile']);
    expect(routeTrail('settings', ['today', 'plan', 'profile'])).toEqual(['today', 'plan', 'profile', 'settings']);
    expect(routeTrail('gear', ['today', 'profile'])).toEqual(['today', 'profile', 'gear']);
    expect(routeTrail('gear')).toEqual(['today', 'training', 'gear']);
    expect(routeTrail('review/2025-10-13', ['today'])).toEqual(['today', 'progress', 'review/2025-10-13']);
    expect(routeTrail('review/2025-10-13', ['today', 'progress', 'reviews'])).toEqual(['today', 'progress', 'reviews', 'review/2025-10-13']);
    expect(routeTrail('metric/hrv')).toEqual(['today', 'health', 'metric/hrv']);
    expect(routeTrail('metric/hrv', ['today'])).toEqual(['today', 'metric/hrv']);
    expect(routeTrail('metric/hrv', ['today', 'progress'])).toEqual(['today', 'progress', 'metric/hrv']);
    expect(routeTrail('metric/bodyBattery')).toEqual(['today', 'metric/bodyBattery']);
    expect(routeTrail('metric/stress')).toEqual(['today', 'metric/stress']);
    expect(routeTrail('metric/stress', ['today', 'health'])).toEqual(['today', 'health', 'metric/stress']);
    expect(routeTrail('metric/vo2max')).toEqual(['today', 'training', 'metric/vo2max']);
    expect(routeTrail('metric/hrv', ['today', 'sleep'])).toEqual(['today', 'sleep', 'metric/hrv']);
    // A dated day or night is its own page and Back returns to the page that opened it.
    expect(routeTrail('sleep/2025-10-14')).toEqual(['today', 'sleep/2025-10-14']);
    expect(routeTrail('sleep/2025-10-14', ['today', 'health'])).toEqual(['today', 'health', 'sleep/2025-10-14']);
    expect(routeTrail('sleep/2025-10-14', ['today', 'sleep'])).toEqual(['today', 'sleep', 'sleep/2025-10-14']);
    expect(routeTrail('sleep/2025-10-14', ['today', 'progress', 'health/2025-10-01'])).toEqual(['today', 'progress', 'health/2025-10-01', 'sleep/2025-10-14']);
    expect(routeTrail('sleep/2025-10-14', ['today', 'progress'])).toEqual(['today', 'progress', 'sleep/2025-10-14']);
    expect(routeTrail('sleep/last', ['today', 'progress', 'sleep/2025-10-14'])).toEqual(['today', 'sleep/last']);
    expect(routeTrail('health/2025-10-14')).toEqual(['today', 'health', 'health/2025-10-14']);
    expect(routeTrail('health/2025-10-14', ['today', 'progress'])).toEqual(['today', 'progress', 'health/2025-10-14']);
    expect(routeTrail('health/2025-10-14', ['today', 'health'])).toEqual(['today', 'health', 'health/2025-10-14']);
    expect(routeTrail('metric/stress/2025-10-14', ['today', 'progress', 'health/2025-10-14'])).toEqual(['today', 'progress', 'health/2025-10-14', 'metric/stress/2025-10-14']);
    expect(routeTrail('metric/stress/2025-10-14', ['today', 'health'])).toEqual(['today', 'health', 'metric/stress/2025-10-14']);
    expect(routeTrail('metric/stress/2025-10-14')).toEqual(['today', 'health', 'metric/stress/2025-10-14']);
    expect(routeTrail('metric/heartRate/2025-10-14', ['today', 'sleep/2025-10-14'])).toEqual(['today', 'sleep/2025-10-14', 'metric/heartRate/2025-10-14']);
    expect(routeTrail('onboarding', ['today', 'settings'])).toEqual(['onboarding']);
  });

  it('nests strength under Training and help over its caller', () => {
    expect(routeTrail('strength', ['today'])).toEqual(['today', 'training', 'strength']);
    expect(routeTrail('workout/7', ['today', 'training', 'strength'])).toEqual(['today', 'training', 'strength', 'workout/7']);
    expect(routeTrail('help', ['today', 'training', 'strength'])).toEqual(['today', 'training', 'strength', 'help']);
    expect(routeTrail('help/strength-sets', ['today', 'training', 'strength', 'help'])).toEqual(['today', 'training', 'strength', 'help', 'help/strength-sets']);
    expect(routeTrail('help/strength-sets', ['today', 'training', 'workout/7'])).toEqual(['today', 'training', 'workout/7', 'help/strength-sets']);
    expect(routeTrail('help', ['today', 'help', 'help/strength-sets'])).toEqual(['today', 'help']);
    expect(routeTrail('help')).toEqual(['today', 'help']);
  });
});

type Entry = { state: unknown; hash: string };
class Browser {
  entries: Entry[];
  index = 0;
  listeners = new Map<string, Set<() => void>>();
  constructor(hash = '') { this.entries = [{ state: null, hash }]; }
  get location() { return { hash: this.entries[this.index].hash }; }
  history = {
    get state() { return browser.entries[browser.index].state; },
    pushState: (state: unknown, _title: string, url: string) => {
      this.entries.splice(this.index + 1);
      this.entries.push({ state, hash: url });
      this.index++;
    },
    replaceState: (state: unknown, _title: string, url: string) => {
      this.entries[this.index] = { state, hash: url };
    },
    go: (offset: number) => {
      const next = this.index + offset;
      if (next < 0 || next >= this.entries.length || next === this.index) return;
      queueMicrotask(() => {
        const old = this.location.hash;
        this.index = next;
        this.fire('popstate');
        if (old !== this.location.hash) this.fire('hashchange');
      });
    },
  };
  addEventListener = (name: string, cb: () => void) => {
    if (!this.listeners.has(name)) this.listeners.set(name, new Set());
    this.listeners.get(name)!.add(cb);
  };
  removeEventListener = (name: string, cb: () => void) => { this.listeners.get(name)?.delete(cb); };
  fire(name: string) { for (const cb of [...(this.listeners.get(name) ?? [])]) cb(); }
  link(hash: string) {
    this.entries.splice(this.index + 1);
    this.entries.push({ state: null, hash });
    this.index++;
    this.fire('hashchange');
  }
  hashes() { return this.entries.map(entry => entry.hash); }
}
let browser: Browser;
afterEach(() => vi.unstubAllGlobals());
const tick = async () => { for (let i = 0; i < 12; i++) await Promise.resolve(); };

describe('browser navigation', () => {
  it('replaces sibling branches and returns to Today, not previously visited tabs', async () => {
    browser = new Browser();
    vi.stubGlobal('window', browser);
    const changes: string[][] = [];
    const nav = createNavigation(trail => changes.push(trail));
    nav.start();
    nav.navigate('sleep');
    nav.navigate('training');
    await tick();
    expect(browser.hashes()).toEqual(['#today', '#training']);
    nav.navigate('workout/7');
    nav.navigate('settings');
    await tick();
    expect(browser.hashes()).toEqual(['#today', '#training', '#workout/7', '#settings']);
    expect(nav.back()).toBe(true);
    await tick();
    expect(changes.at(-1)).toEqual(['today', 'training', 'workout/7']);
    expect(nav.back()).toBe(true);
    await tick();
    expect(changes.at(-1)).toEqual(['today', 'training']);
    nav.navigate('health');
    await tick();
    expect(browser.hashes()).toEqual(['#today', '#health']);
    expect(changes.at(-1)).toEqual(['today', 'health']);
    expect(nav.back()).toBe(true);
    await tick();
    expect(nav.back()).toBe(false);
    expect(changes.at(-1)).toEqual(['today']);
    nav.destroy();
  });

  it('loads direct links with ancestors, preserves state on reload, and ignores repeat clicks', async () => {
    browser = new Browser('#workout/42');
    vi.stubGlobal('window', browser);
    const first = createNavigation(() => {});
    first.start();
    expect(browser.hashes()).toEqual(['#today', '#training', '#activities', '#workout/42']);
    first.destroy();
    const changes: string[][] = [];
    const nav = createNavigation(trail => changes.push(trail));
    nav.start();
    nav.navigate('workout/42');
    await tick();
    expect(browser.hashes()).toEqual(['#today', '#training', '#activities', '#workout/42']);
    expect(changes).toEqual([['today', 'training', 'activities', 'workout/42']]);
    nav.destroy();
  });

  it('coalesces rapid Back and tab changes without showing transient parents', async () => {
    browser = new Browser('#metric/hrv');
    vi.stubGlobal('window', browser);
    const changes: string[][] = [];
    const nav = createNavigation(trail => changes.push(trail));
    nav.start();
    nav.navigate('training');
    nav.navigate('sleep');
    await tick();
    expect(browser.hashes()).toEqual(['#today', '#sleep']);
    expect(changes).toEqual([['today', 'health', 'metric/hrv'], ['today', 'sleep']]);
    nav.navigate('health');
    expect(nav.back()).toBe(true);
    await tick();
    expect(changes.at(-1)).toEqual(['today']);
    nav.destroy();
  });

  it('restores owned entries on browser Back without revisiting old tabs', async () => {
    browser = new Browser();
    vi.stubGlobal('window', browser);
    const changes: string[][] = [];
    const nav = createNavigation(trail => changes.push(trail));
    nav.start();
    nav.navigate('training');
    nav.navigate('workout/7');
    browser.history.go(-1);
    await tick();
    expect(changes.at(-1)).toEqual(['today', 'training']);
    browser.history.go(-1);
    await tick();
    expect(changes.at(-1)).toEqual(['today']);
    nav.destroy();
  });

  it('rebases external hash links without a duplicate refresh', async () => {
    browser = new Browser();
    vi.stubGlobal('window', browser);
    const changes: string[][] = [];
    const nav = createNavigation(trail => changes.push(trail));
    nav.start();
    browser.link('#sleep');
    await tick();
    expect(browser.hashes()).toEqual(['#today', '#sleep']);
    expect(changes).toEqual([['today'], ['today', 'sleep']]);
    nav.destroy();
  });

  it('treats onboarding as a root when login state changes', async () => {
    browser = new Browser('#onboarding');
    vi.stubGlobal('window', browser);
    const nav = createNavigation(() => {});
    nav.start();
    expect(nav.back()).toBe(false);
    nav.navigate('today');
    await tick();
    expect(browser.hashes()).toEqual(['#today']);
    nav.navigate('sleep');
    nav.navigate('onboarding');
    await tick();
    expect(browser.location.hash).toBe('#onboarding');
    expect(browser.index).toBe(0);
    expect(nav.back()).toBe(false);
    nav.destroy();
  });
});
