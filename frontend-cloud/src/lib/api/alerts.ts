import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

/** Mirrors `hoister_shared::alerts::AlertMetric`. */
export type AlertMetric = 'cpu_pct' | 'mem_pct' | 'mem_bytes';

/** Mirrors the controller's `AlertRuleResponse`. */
export interface AlertRule {
	/** Controller-side UUID. Treat as an opaque string. */
	id: string;
	metric: AlertMetric;
	/** In the metric's unit: percent for `cpu_pct`/`mem_pct`, bytes for `mem_bytes`. */
	threshold: number;
	/** How long the threshold must be breached before the alert fires. */
	for_seconds: number;
	/** Minimum seconds between repeat notifications while firing; 0 = once per episode. */
	cooldown_seconds: number;
	/** Scope — null means "any". */
	hostname: string | null;
	project: string | null;
	service: string | null;
	enabled: boolean;
	created_at: string;
}

/** Mirrors `hoister_shared::alerts::AlertEventKind`. */
export type AlertEventKind = 'fired' | 'still_firing' | 'resolved';

/** Mirrors the controller's `AlertEventResponse`: one recorded transition. */
export interface AlertEvent {
	id: string;
	/** null once the rule behind this entry has been deleted. */
	rule_id: string | null;
	kind: AlertEventKind;
	metric: AlertMetric;
	/** In the metric's unit, as the rule read at the time it triggered. */
	threshold: number;
	/** The reading that produced the transition. */
	value: number;
	for_seconds: number;
	hostname: string;
	project: string;
	service: string;
	/** RFC 3339. */
	triggered_at: string;
	/** Whether this entry arrived since the previous login. */
	is_new: boolean;
}

/** Mirrors the controller's `AlertHistoryResponse`. */
export interface AlertHistory {
	/** Newest first. */
	events: AlertEvent[];
	/** Start of the previous login session; null on a first-ever login. */
	new_since: string | null;
	new_count: number;
}

/** Mirrors the controller's `CreateAlertRuleBody`. */
export interface CreateAlertRuleBody {
	metric: AlertMetric;
	threshold: number;
	for_seconds: number;
	cooldown_seconds: number;
	hostname?: string;
	project?: string;
	service?: string;
}

interface ApiResponse<T> {
	success: boolean;
	data: T | null;
	error: string | null;
}

async function unwrap<T>(response: Response, what: string): Promise<T> {
	if (!response.ok) throw error(response.status, `Failed to ${what}`);
	const result = (await response.json()) as ApiResponse<T>;
	if (!result.success || result.error || result.data == null) {
		throw error(500, result.error || `Failed to ${what}`);
	}
	return result.data;
}

export async function listAlertRules(userId: string): Promise<AlertRule[]> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/alerts`, {
		headers: backendHeaders(userId)
	});
	return unwrap<AlertRule[]>(response, 'list alert rules');
}

export type CreateAlertRuleResult =
	| { ok: true; rule: AlertRule }
	| { ok: false; error: string };

export async function createAlertRule(
	userId: string,
	body: CreateAlertRuleBody
): Promise<CreateAlertRuleResult> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const headers = { ...backendHeaders(userId), 'Content-Type': 'application/json' };
	const response = await fetch(`${BACKEND_URL}/alerts`, {
		method: 'POST',
		headers,
		body: JSON.stringify(body)
	});
	if (response.status === 400) {
		const result = (await response.json().catch(() => ({}))) as { error?: string };
		return { ok: false, error: result.error ?? 'Invalid alert rule' };
	}
	const rule = await unwrap<AlertRule>(response, 'create alert rule');
	return { ok: true, rule };
}

export async function deleteAlertRule(userId: string, ruleId: string): Promise<boolean> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/alerts/${ruleId}`, {
		method: 'DELETE',
		headers: backendHeaders(userId)
	});
	if (response.status === 404) return false;
	if (!response.ok) throw error(response.status, 'Failed to delete alert rule');
	return true;
}

export async function setAlertRuleEnabled(
	userId: string,
	ruleId: string,
	enabled: boolean
): Promise<boolean> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const headers = { ...backendHeaders(userId), 'Content-Type': 'application/json' };
	const response = await fetch(`${BACKEND_URL}/alerts/${ruleId}/enabled`, {
		method: 'PATCH',
		headers,
		body: JSON.stringify({ enabled })
	});
	if (response.status === 404) return false;
	if (!response.ok) throw error(response.status, 'Failed to toggle alert rule');
	return true;
}

/**
 * The alert history plus which entries arrived since the previous login.
 *
 * `sessionId` is the caller's current login session: the controller compares
 * it against the one it last saw to decide whether to move the "new since"
 * cutoff on, so reading the history is what marks the previous batch as seen.
 */
export async function listAlertHistory(
	userId: string,
	sessionId: string
): Promise<AlertHistory> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const query = new URLSearchParams({ session: sessionId });
	const response = await fetch(`${BACKEND_URL}/alerts/events?${query}`, {
		headers: backendHeaders(userId)
	});
	return unwrap<AlertHistory>(response, 'list alert history');
}
