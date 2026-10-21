export const tabs = ['today', 'plan', 'training', 'health', 'progress'] as const;

const metrics = new Set([
  'bodyBattery', 'stress', 'heartRate', 'restingHr', 'sleepHr', 'hrv', 'respiration', 'spo2', 'skinTemp',
  'vo2max', 'endurance', 'hill', 'runningTolerance', 'acuteLoad', 'fitnessAge',
]);
const routes = new Set([...tabs, 'sleep', 'sleep/last', 'sleep/tonight', 'readiness', 'settings', 'onboarding', 'strength', 'help',
  'profile', 'gear', 'plan/commitments', 'review/latest', 'reviews', 'health/weight', 'progress/milestones']);
// Earlier routes keep working; each opens its current page, never a silent Today.
export const aliases: Record<string, string> = {checkin: 'review/latest', 'plan-setup': 'plan/commitments', explore: 'training', 'metric/weight': 'health/weight', activities: 'training'};
export const helpPages = ['strength-start', 'strength-sets', 'strength-muscles', 'strength-watches'] as const;
const trainingMetrics = new Set(['vo2max', 'endurance', 'hill', 'runningTolerance', 'acuteLoad']);

/** A real calendar date, so a mistyped fragment opens Today rather than an empty page. */
export function isoDate(value: string): boolean {
  if (!/^\d{4}-\d{2}-\d{2}$/.test(value)) return false;
  const date = new Date(value + 'T12:00:00Z');
  return Number.isFinite(date.getTime()) && date.toISOString().slice(0, 10) === value;
}
/** One opened past day or workout: the route that names its own date. */
export function datedDetail(route: string): { page: 'sleep' | 'health' | 'stress' | 'heartRate'; date: string } | null {
  const sleep = /^sleep\/(\d{4}-\d{2}-\d{2})$/.exec(route);
  if (sleep && isoDate(sleep[1])) return { page: 'sleep', date: sleep[1] };
  const health = /^health\/(\d{4}-\d{2}-\d{2})$/.exec(route);
  if (health && isoDate(health[1])) return { page: 'health', date: health[1] };
  const metric = /^metric\/(stress|heartRate)\/(\d{4}-\d{2}-\d{2})$/.exec(route);
  if (metric && isoDate(metric[2])) return { page: metric[1] as 'stress' | 'heartRate', date: metric[2] };
  return null;
}

/** A route this app has a page for; anything else is null. */
export function knownRoute(hashOrRoute: string): string | null {
  let route = hashOrRoute.startsWith('#') ? hashOrRoute.slice(1) : hashOrRoute;
  try { route = decodeURIComponent(route); } catch { return null; }
  route = aliases[route] ?? route;
  if (routes.has(route)) return route;
  if (datedDetail(route)) return route;
  if (/^workout\/[\w-]+$/.test(route)) return route;
  if (/^review\/\d{4}-\d{2}-\d{2}$/.test(route)) return route;
  if (route.startsWith('metric/') && metrics.has(route.slice(7))) return route;
  if (route.startsWith('help/') && (helpPages as readonly string[]).includes(route.slice(5))) return route;
  return null;
}

export function normalizeRoute(hashOrRoute: string): string {
  return knownRoute(hashOrRoute) ?? 'today';
}

/** The tab a trail belongs to, ignoring Today. */
function section(trail: string[]): string | undefined {
  return trail.findLast(route => (tabs as readonly string[]).includes(route) && route !== 'today');
}
/** Pages that open over whatever asked for them, without stacking twice. */
function over(route: string, fromTrail: string[], skip: (page: string) => boolean): string[] {
  const caller = fromTrail.filter(page => !skip(page) && page !== 'onboarding');
  return [...(caller[0] === 'today' ? caller : ['today']), route];
}

/** The trail lists browser entries from root to the current page, not visits in chronological order. */
export function routeTrail(target: string, fromTrail: string[] = []): string[] {
  const route = normalizeRoute(target);
  if (route === 'onboarding') return ['onboarding'];
  if (route === 'today') return ['today'];
  if ((tabs as readonly string[]).includes(route)) return ['today', route];
  // Opening the page already shown keeps its trail.
  if (fromTrail.length > 1 && fromTrail.at(-1) === route) return [...fromTrail];
  if (route === 'readiness') return ['today', route];
  // Last night and Tonight are tabs of one page: switching replaces, never stacks.
  if (route === 'sleep' || route === 'sleep/last' || route === 'sleep/tonight') return section(fromTrail) === 'health' ? ['today', 'health', route] : ['today', route];
  // A dated night is its own page under whichever page or tab opened it.
  if (route.startsWith('sleep/')) {
    const last = fromTrail.at(-1);
    if (last?.startsWith('sleep') || last?.startsWith('health/')) return [...fromTrail, route];
    const tab = section(fromTrail);
    return tab ? ['today', tab, route] : ['today', route];
  }
  if (route === 'health/weight') return ['today','health',route];
  // A dated Health day returns to its caller's section, or to Health when deep-linked.
  if (route.startsWith('health/')) {
    const tab = section(fromTrail);
    return tab && tab !== 'health' ? ['today', tab, route] : ['today', 'health', route];
  }
  // A dated metric opens under the day page that asked for it.
  if (route.startsWith('metric/') && datedDetail(route)) {
    const last = fromTrail.at(-1);
    // The stress calendar opens a day over the live stress page, then moves between days in place.
    if (route.startsWith('metric/stress/') && last?.startsWith('metric/stress/')) return [...fromTrail.slice(0, -1), route];
    if (route.startsWith('metric/stress/') && last === 'metric/stress') return [...fromTrail, route];
    if (last?.startsWith('health/') || last?.startsWith('sleep')) return [...fromTrail, route];
    const tab = section(fromTrail);
    return tab ? ['today', tab, route] : ['today', 'health', route];
  }
  if (route === 'plan/commitments') return fromTrail.at(-1)?.startsWith('review/') ? [...fromTrail, route] : ['today', 'plan', route];
  if (route === 'reviews' || route === 'progress/milestones') return ['today', 'progress', route];
  if (route.startsWith('review/')) return fromTrail.at(-1) === 'reviews' ? [...fromTrail, route] : ['today', 'progress', route];
  if (route === 'strength') return ['today', 'training', route];
  if (route === 'profile') return over(route, fromTrail, page => page === 'profile' || page === 'settings' || page === 'gear');
  if (route === 'gear') return fromTrail.at(-1) === 'profile' ? [...fromTrail, route] : ['today', 'training', route];
  if (route === 'settings') {
    if (fromTrail.at(-1) === 'profile') return [...fromTrail, route];
    const caller = fromTrail.at(-1) === 'settings' ? fromTrail.slice(0, -1) : fromTrail;
    return [...(caller[0] === 'today' ? caller : ['today']), route];
  }
  if (route === 'help' || route.startsWith('help/')) {
    if (route !== 'help' && fromTrail.at(-1) === 'help') return [...fromTrail, route];
    // Help opens over the page that asked for it; reopening it does not stack.
    return over(route, fromTrail, page => page === 'help' || page.startsWith('help/'));
  }
  if (route.startsWith('workout/')) {
    const last = fromTrail.at(-1);
    if (last === 'strength') return ['today', 'training', last, route];
    const tab = section(fromTrail);
    if (tab) return ['today', tab, route];
    return fromTrail.some(page => page === 'today') ? ['today', route] : ['today', 'training', route];
  }
  const metric = route.slice(7);
  const tab = section(fromTrail);
  if (fromTrail.at(-1)?.startsWith('sleep')) return [...fromTrail, route];
  const parent = tab ?? (fromTrail[0] === 'today' || metric === 'bodyBattery' || metric === 'stress' ? 'today' :
    trainingMetrics.has(metric) ? 'training' : 'health');
  return parent === 'today' ? ['today', route] : ['today', parent, route];
}

const stateKey = '__raringNavigation';

function ownedTrail(state: unknown, hash: string): string[] | null {
  if (!state || typeof state !== 'object') return null;
  const trail = (state as Record<string, unknown>)[stateKey];
  if (!Array.isArray(trail) || trail.length === 0 || !trail.every(page => typeof page === 'string')) return null;
  if (trail.at(-1) !== normalizeRoute(hash) || hash !== `#${trail.at(-1)}`) return null;
  // Reject stale/corrupt state instead of adopting an unrelated browser entry.
  for (let i = 0; i < trail.length; i++) {
    if (routeTrail(trail[i], trail.slice(0, i)).join('\0') !== trail.slice(0, i + 1).join('\0')) return null;
  }
  return trail;
}

export function createNavigation(onChange: (trail: string[]) => void): {
  start(): void;
  navigate(to: string): void;
  back(): boolean;
  destroy(): void;
} {
  let trail: string[] = [];
  let desired: string[] = [];
  let started = false;
  let processing = false;
  let waiting: (() => void) | null = null;
  let cancelled = false;
  let lastEmitted: string[] | null = null;
  let externalTarget: string[] | null = null;

  const url = (route: string) => `#${route}`;
  const state = (next: string[]) => ({ [stateKey]: next });
  const same = (a: string[], b: string[]) => a.join('\0') === b.join('\0');
  const emit = () => {
    if (!lastEmitted || !same(lastEmitted, trail)) {
      lastEmitted = [...trail];
      onChange([...trail]);
    }
  };

  async function apply() {
    if (processing || cancelled) return;
    processing = true;
    try {
      while (!same(trail, desired) && !cancelled) {
        const next = desired;
        let common = 0;
        while (common < trail.length && common < next.length && trail[common] === next[common]) common++;
        // Changing login state changes the root; never traverse outside our first entry.
        if (common === 0) common = 1;
        if (common < trail.length) {
          // Traversal is asynchronous. Do not show each transient parent as it pops.
          await new Promise<void>(resolve => {
            waiting = resolve;
            window.history.go(common - trail.length);
          });
          if (cancelled) break;
          trail = ownedTrail(window.history.state, window.location.hash) ?? routeTrail(window.location.hash);
          continue;
        }
        if (trail[0] !== next[0]) {
          trail = [next[0]];
          // Login changes the root, not its depth: Back must not reopen onboarding.
          window.history.replaceState(state(trail), '', url(trail[0]));
        }
        for (const page of next.slice(common)) {
          const prefix = [...trail, page];
          window.history.pushState(state(prefix), '', url(page));
          trail = prefix;
        }
      }
      if (!cancelled) emit();
    } finally {
      processing = false;
      if (!cancelled && !same(trail, desired)) void apply();
    }
  }

  function onPopstate() {
    if (externalTarget) {
      const target = externalTarget;
      externalTarget = null;
      trail = ownedTrail(window.history.state, window.location.hash) ?? trail;
      desired = target;
      void apply();
      return;
    }
    if (waiting) {
      const resolve = waiting;
      waiting = null;
      resolve();
      return;
    }
    const restored = ownedTrail(window.history.state, window.location.hash);
    if (restored) {
      trail = restored;
      desired = restored;
      emit();
    } else {
      // A non-owned browser entry (e.g. an external fragment link) is a new request.
      external();
    }
  }

  function external() {
    const route = normalizeRoute(window.location.hash);
    if (ownedTrail(window.history.state, window.location.hash)) return;
    // A fragment assignment already added a browser entry. Rewind it before building
    // the canonical branch so Back cannot revisit an obsolete section.
    if (externalTarget) return;
    externalTarget = routeTrail(route, desired);
    window.history.go(-1);
  }

  function onHashchange() {
    if (externalTarget) return;
    if (!ownedTrail(window.history.state, window.location.hash)) external();
  }

  return {
    start() {
      if (started || cancelled) return;
      started = true;
      window.addEventListener('popstate', onPopstate);
      window.addEventListener('hashchange', onHashchange);
      const restored = ownedTrail(window.history.state, window.location.hash);
      if (restored) {
        trail = desired = restored;
        emit();
      } else {
        const initial = routeTrail(window.location.hash);
        trail = desired = [initial[0]];
        window.history.replaceState(state(trail), '', url(trail[0]));
        if (initial.length > 1) {
          for (const page of initial.slice(1)) {
            trail = [...trail, page];
            window.history.pushState(state(trail), '', url(page));
          }
          desired = trail;
        }
        emit();
      }
    },
    navigate(to) {
      if (!started || cancelled) return;
      desired = routeTrail(to, desired);
      void apply();
    },
    back() {
      if (!started || cancelled || desired.length <= 1) return false;
      desired = desired.slice(0, -1);
      void apply();
      return true;
    },
    destroy() {
      cancelled = true;
      window.removeEventListener('popstate', onPopstate);
      window.removeEventListener('hashchange', onHashchange);
      waiting?.();
      waiting = null;
    },
  };
}
