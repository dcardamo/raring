import { describe, expect, it } from 'vitest';
import { shouldSync, SYNC_INTERVAL_MS } from './freshness';
import type { AppStatus } from './generated/AppStatus';

const now = Date.parse('2026-10-16T14:00:00Z');
const status: AppStatus = { signedIn: true, syncing: false, lastSync: null, historyDays: 30, lastError: null };

describe('foreground freshness', () => {
  it('refreshes missing and stale data, but not a recent background sync', () => {
    expect(shouldSync(status, null, now)).toBe(true);
    expect(shouldSync({ ...status, lastSync: 'invalid' }, null, now)).toBe(true);
    expect(shouldSync({ ...status, lastSync: new Date(now - SYNC_INTERVAL_MS).toISOString() }, null, now)).toBe(true);
    expect(shouldSync({ ...status, lastSync: new Date(now - 60_000).toISOString() }, null, now)).toBe(false);
  });
  it('does not sync signed-out accounts or overlap a running sync', () => {
    expect(shouldSync(null, null, now)).toBe(false);
    expect(shouldSync({ ...status, signedIn: false }, null, now)).toBe(false);
    expect(shouldSync({ ...status, syncing: true }, null, now)).toBe(false);
  });
  it('throttles failed attempts and repeated resume events too', () => {
    expect(shouldSync(status, now - 60_000, now)).toBe(false);
    expect(shouldSync(status, now - SYNC_INTERVAL_MS, now)).toBe(true);
  });
  it('refreshes across midnight, without treating the day as a cache key', () => {
    const midnight = Date.parse('2026-10-17T00:02:00Z');
    expect(shouldSync({ ...status, lastSync: '2026-10-16T23:00:00Z' }, null, midnight)).toBe(true);
  });
});
