import type { AppStatus } from './generated/AppStatus';

export const SYNC_INTERVAL_MS = 15 * 60 * 1000;
export const CACHE_POLL_MS = 30_000;

// Cache reads are cheap. Garmin requests (including failed attempts) are throttled.
export function shouldSync(status: AppStatus | null, lastAttempt: number | null, now = Date.now()): boolean {
  if (!status?.signedIn || status.syncing) return false;
  if (lastAttempt !== null && now - lastAttempt < SYNC_INTERVAL_MS) return false;
  const lastSync = status.lastSync ? Date.parse(status.lastSync) : NaN;
  return !Number.isFinite(lastSync) || now - lastSync >= SYNC_INTERVAL_MS;
}
