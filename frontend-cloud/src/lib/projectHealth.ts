import type { ProjectSummaryResponse } from '../bindings/ProjectSummaryResponse';
import type { ProjectServiceSummary } from '../bindings/ProjectServiceSummary';
import { isStale } from './staleness';

/**
 * Overall state of a project, worst first:
 *  - `stale`      the agent stopped reporting, so nothing below is current
 *  - `down`       a service exited, died or fails its health check
 *  - `degraded`   a service is restarting, paused, starting up or just created
 *  - `healthy`    every service runs (and passes its health check, if any)
 *  - `unknown`    the project has never reported any container state
 */
export type ProjectHealth = 'stale' | 'down' | 'degraded' | 'healthy' | 'unknown';

export function serviceHealth(s: ProjectServiceSummary): 'down' | 'degraded' | 'healthy' {
	if (s.status === 'exited' || s.status === 'dead' || s.health === 'unhealthy') return 'down';
	if (s.status !== 'running' || s.health === 'starting') return 'degraded';
	return 'healthy';
}

export function projectHealth(
	p: ProjectSummaryResponse,
	nowMs: number = Date.now()
): ProjectHealth {
	if (!p.last_updated || p.services.length === 0) return 'unknown';
	if (isStale(p.last_updated, nowMs)) return 'stale';
	const states = p.services.map(serviceHealth);
	if (states.includes('down')) return 'down';
	if (states.includes('degraded')) return 'degraded';
	return 'healthy';
}

export const HEALTH_LABEL: Record<ProjectHealth, string> = {
	healthy: 'Healthy',
	degraded: 'Degraded',
	down: 'Down',
	stale: 'Not reporting',
	unknown: 'No data'
};

/** Small status dot. */
export const HEALTH_DOT: Record<ProjectHealth, string> = {
	healthy: 'bg-success',
	degraded: 'bg-warning',
	down: 'bg-error',
	stale: 'bg-line-active',
	unknown: 'bg-line-subtle'
};

/** Pill with border, background and text colour. */
export const HEALTH_BADGE: Record<ProjectHealth, string> = {
	healthy: 'border-success-border bg-success-bg text-success',
	degraded: 'border-warning-border bg-warning-bg text-warning',
	down: 'border-error-border bg-error-bg text-error',
	stale: 'border-line-subtle bg-element text-ink-muted',
	unknown: 'border-line bg-element text-ink-faint'
};

export function formatBytes(bytes: number): string {
	if (bytes < 1024) return `${bytes} B`;
	const units = ['KiB', 'MiB', 'GiB', 'TiB'];
	let value = bytes / 1024;
	let unit = 0;
	while (value >= 1024 && unit < units.length - 1) {
		value /= 1024;
		unit++;
	}
	return `${value.toFixed(value >= 10 ? 0 : 1)} ${units[unit]}`;
}

/**
 * Parse a controller timestamp. Besides RFC 3339 these arrive as SQLite's
 * `CURRENT_TIMESTAMP` (`2026-09-21 10:00:00`, UTC without a zone) or Postgres'
 * `timestamptz::text` (`2026-09-21 10:00:00.123+00`), neither of which
 * `Date` parses reliably.
 */
export function parseTimestamp(raw: string): number {
	let s = raw.trim().replace(' ', 'T');
	if (/[+-]\d{2}$/.test(s)) s += ':00';
	else if (!/(Z|[+-]\d{2}:?\d{2})$/i.test(s)) s += 'Z';
	return new Date(s).getTime();
}

/** "3m ago", "5h ago", "2d ago" — coarse, for tiles and lists. */
export function timeAgo(date: string, nowMs: number = Date.now()): string {
	const then = parseTimestamp(date);
	if (isNaN(then)) return date;
	const s = Math.max(0, Math.floor((nowMs - then) / 1000));
	if (s < 60) return `${s}s ago`;
	const m = Math.floor(s / 60);
	if (m < 60) return `${m}m ago`;
	const h = Math.floor(m / 60);
	if (h < 48) return `${h}h ago`;
	return `${Math.floor(h / 24)}d ago`;
}
