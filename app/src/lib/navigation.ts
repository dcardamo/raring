export const tabs = ['today', 'sleep', 'training', 'health'] as const;

const metrics = new Set([
  'bodyBattery', 'restingHr', 'hrv', 'respiration', 'spo2', 'skinTemp',
  'vo2max', 'endurance', 'hill', 'runningTolerance',
]);
const routes = new Set([...tabs, 'readiness', 'checkin', 'settings', 'onboarding']);
const trainingMetrics = new Set(['vo2max', 'endurance', 'hill', 'runningTolerance']);

export function normalizeRoute(hashOrRoute: string): string {
  let route = hashOrRoute.startsWith('#') ? hashOrRoute.slice(1) : hashOrRoute;
  try { route = decodeURIComponent(route); } catch { return 'today'; }
  if (routes.has(route)) return route;
  if (/^workout\/[\w-]+$/.test(route)) return route;
  if (route.startsWith('metric/') && metrics.has(route.slice(7))) return route;
  return 'today';
}

function section(trail: string[]): string | undefined {
  return trail.findLast(route => (tabs as readonly string[]).includes(route) && route !== 'today');
}

/** The trail lists browser entries from root to the current page, not visits in chronological order. */
export function routeTrail(target: string, fromTrail: string[] = []): string[] {
  const route = normalizeRoute(target);
  if (route === 'onboarding') return ['onboarding'];
  if (route === 'today') return ['today'];
  if ((tabs as readonly string[]).includes(route)) return ['today', route];
  if (route === 'readiness' || route === 'checkin') return ['today', route];
  if (route === 'settings') {
    const caller = fromTrail.at(-1) === 'settings' ? fromTrail.slice(0, -1) : fromTrail;
    return [...(caller[0] === 'today' ? caller : ['today']), route];
  }
  if (route.startsWith('workout/')) {
    const parent = section(fromTrail) === 'training' ? 'training' :
      fromTrail.some(page => page === 'today') ? 'today' : 'training';
    return parent === 'today' ? ['today', route] : ['today', 'training', route];
  }
  const metric = route.slice(7);
  const parent = section(fromTrail) ?? (metric === 'bodyBattery' ? 'today' :
    trainingMetrics.has(metric) ? 'training' : 'health');
  return parent === 'today' ? ['today', route] : ['today', parent, route];
}

const stateKey = '__pepometerNavigation';

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
          // Push truncates a stale forward branch; the old login root cannot be
          // removed from browser history, but native Back exits at either root.
          window.history.pushState(state(trail), '', url(trail[0]));
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
