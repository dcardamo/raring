import { afterEach, describe, expect, it, vi } from 'vitest';
import { createNavigation, normalizeRoute, routeTrail, tabs } from './navigation';

describe('routes', () => {
  it('accepts known routes and rejects invalid or malformed fragments', () => {
    expect(tabs).toEqual(['today', 'sleep', 'training', 'health']);
    for (const route of [...tabs, 'readiness', 'checkin', 'settings', 'onboarding',
      'workout/123_abc-2', ...['bodyBattery', 'restingHr', 'hrv', 'respiration', 'spo2',
        'skinTemp', 'vo2max', 'endurance', 'hill', 'runningTolerance'].map(id => `metric/${id}`)]) {
      expect(normalizeRoute(`#${route}`)).toBe(route);
    }
    for (const route of ['#metric/unknown', '#workout/', '#workout/a/b', '#%ZZ',
      '#sleep/other', '#https://bad.example', '#onboarding/next']) {
      expect(normalizeRoute(route)).toBe('today');
    }
  });

  it('builds logical parents rather than visit order', () => {
    expect(routeTrail('health', ['today', 'sleep'])).toEqual(['today', 'health']);
    expect(routeTrail('readiness', ['today', 'training'])).toEqual(['today', 'readiness']);
    expect(routeTrail('checkin')).toEqual(['today', 'checkin']);
    expect(routeTrail('workout/7')).toEqual(['today', 'training', 'workout/7']);
    expect(routeTrail('workout/7', ['today'])).toEqual(['today', 'workout/7']);
    expect(routeTrail('workout/7', ['today', 'training'])).toEqual(['today', 'training', 'workout/7']);
    expect(routeTrail('settings', ['today', 'training', 'workout/7'])).toEqual(['today', 'training', 'workout/7', 'settings']);
    expect(routeTrail('settings', ['today', 'settings'])).toEqual(['today', 'settings']);
    expect(routeTrail('settings')).toEqual(['today', 'settings']);
    expect(routeTrail('metric/hrv')).toEqual(['today', 'health', 'metric/hrv']);
    expect(routeTrail('metric/hrv', ['today'])).toEqual(['today', 'metric/hrv']);
    expect(routeTrail('metric/bodyBattery')).toEqual(['today', 'metric/bodyBattery']);
    expect(routeTrail('metric/vo2max')).toEqual(['today', 'training', 'metric/vo2max']);
    expect(routeTrail('metric/hrv', ['today', 'sleep'])).toEqual(['today', 'sleep', 'metric/hrv']);
    expect(routeTrail('onboarding', ['today', 'settings'])).toEqual(['onboarding']);
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
    expect(browser.hashes()).toEqual(['#today', '#training', '#workout/42']);
    first.destroy();
    const changes: string[][] = [];
    const nav = createNavigation(trail => changes.push(trail));
    nav.start();
    nav.navigate('workout/42');
    await tick();
    expect(browser.hashes()).toEqual(['#today', '#training', '#workout/42']);
    expect(changes).toEqual([['today', 'training', 'workout/42']]);
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
