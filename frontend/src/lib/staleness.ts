// How long a container state may go without an update before the UI flags it as
// stale. The agent reports on its own schedule, so the threshold has to leave
// room for one missed report plus transport delay — otherwise every service
// blinks stale between reports.
export const STALE_AFTER_MS = 75_000;

/** True when `dateString` is older than the staleness threshold. */
export function isStale(dateString: string, nowMs: number = Date.now()): boolean {
  return nowMs - new Date(dateString).getTime() > STALE_AFTER_MS;
}
