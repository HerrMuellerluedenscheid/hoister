import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export type NotifierKind =
	| 'slack'
	| 'telegram'
	| 'discord'
	| 'discord_webhook'
	| 'teams'
	| 'gotify'
	| 'email';

export type NotifierConfig =
	| { kind: 'slack'; webhook: string; channel: string }
	| { kind: 'telegram'; bot_token: string; chat_id: number }
	| { kind: 'discord'; bot_token: string; channel_id: number }
	| { kind: 'discord_webhook'; webhook: string }
	| { kind: 'teams'; webhook: string }
	| { kind: 'gotify'; server: string; token: string }
	| { kind: 'email'; recipient: string };

/**
 * What the controller actually returns. Secret-bearing fields (webhook,
 * bot_token, smtp_password, gotify token) never leave the controller —
 * only `*_set: true` markers do, so an XSS sink in the dashboard can't
 * walk away with the customer's credentials.
 */
export type NotifierSummaryConfig =
	| { kind: 'slack'; channel: string; webhook_set: boolean }
	| { kind: 'telegram'; chat_id: number; bot_token_set: boolean }
	| { kind: 'discord'; channel_id: number; bot_token_set: boolean }
	| { kind: 'discord_webhook'; webhook_set: boolean }
	| { kind: 'teams'; webhook_set: boolean }
	| { kind: 'gotify'; server_host: string; token_set: boolean }
	| { kind: 'email'; recipient: string };

export interface Notifier {
	id: number;
	kind: NotifierKind;
	config: NotifierSummaryConfig;
	enabled: boolean;
	created_at: string;
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

export async function listNotifiers(userId: string): Promise<Notifier[]> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/notifiers`, {
		headers: backendHeaders(userId)
	});
	return unwrap<Notifier[]>(response, 'list notifiers');
}

export type CreateNotifierResult =
	| { ok: true; notifier: Notifier }
	| { ok: false; upgradeRequired: string };

export async function createNotifier(
	userId: string,
	config: NotifierConfig
): Promise<CreateNotifierResult> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const headers = { ...backendHeaders(userId), 'Content-Type': 'application/json' };
	const response = await fetch(`${BACKEND_URL}/notifiers`, {
		method: 'POST',
		headers,
		body: JSON.stringify(config)
	});
	if (response.status === 402) {
		const body = (await response.json().catch(() => ({}))) as { error?: string };
		return { ok: false, upgradeRequired: body.error ?? 'Upgrade required' };
	}
	const notifier = await unwrap<Notifier>(response, 'create notifier');
	return { ok: true, notifier };
}

export async function deleteNotifier(userId: string, notifierId: number): Promise<boolean> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/notifiers/${notifierId}`, {
		method: 'DELETE',
		headers: backendHeaders(userId)
	});
	if (response.status === 404) return false;
	if (!response.ok) throw error(response.status, 'Failed to delete notifier');
	return true;
}

export type TestNotifierResult = { ok: true } | { ok: false; error: string };

export async function testNotifier(userId: string, notifierId: number): Promise<TestNotifierResult> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/notifiers/${notifierId}/test`, {
		method: 'POST',
		headers: backendHeaders(userId)
	});
	if (response.status === 204) return { ok: true };
	if (response.status === 404) return { ok: false, error: 'Notifier not found' };
	const body = (await response.json().catch(() => ({}))) as { error?: string };
	return { ok: false, error: body.error ?? `Test failed (HTTP ${response.status})` };
}

export async function setNotifierEnabled(
	userId: string,
	notifierId: number,
	enabled: boolean
): Promise<boolean> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const headers = { ...backendHeaders(userId), 'Content-Type': 'application/json' };
	const response = await fetch(`${BACKEND_URL}/notifiers/${notifierId}/enabled`, {
		method: 'PATCH',
		headers,
		body: JSON.stringify({ enabled })
	});
	if (response.status === 404) return false;
	if (!response.ok) throw error(response.status, 'Failed to toggle notifier');
	return true;
}
