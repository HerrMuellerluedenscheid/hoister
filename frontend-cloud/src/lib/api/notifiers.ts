import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export type NotifierKind = 'slack' | 'telegram' | 'discord' | 'gotify' | 'email';

export type NotifierConfig =
	| { kind: 'slack'; webhook: string; channel: string }
	| { kind: 'telegram'; bot_token: string; chat_id: number }
	| { kind: 'discord'; bot_token: string; channel_id: number }
	| { kind: 'gotify'; server: string; token: string }
	| {
			kind: 'email';
			smtp_server: string;
			smtp_user: string;
			smtp_password: string;
			from: string | null;
			recipient: string;
	  };

export interface Notifier {
	id: number;
	user_id: string;
	kind: NotifierKind;
	config: NotifierConfig;
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
