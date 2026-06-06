import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import {
	createNotifier,
	deleteNotifier,
	listNotifiers,
	setNotifierEnabled,
	testNotifier,
	type NotifierConfig
} from '$lib/api/notifiers';

function requireString(form: FormData, key: string): string | null {
	const raw = form.get(key);
	if (typeof raw !== 'string') return null;
	const trimmed = raw.trim();
	return trimmed.length > 0 ? trimmed : null;
}

function parseConfig(form: FormData): NotifierConfig | { error: string } {
	// Slack is created via the OAuth "Add to Slack" flow (/slack/oauth/*),
	// not this manual form — so there is intentionally no `slack` case here.
	const kind = form.get('kind');
	switch (kind) {
		case 'telegram': {
			const bot_token = requireString(form, 'bot_token');
			const chat_raw = requireString(form, 'chat_id');
			const chat_id = chat_raw == null ? NaN : Number(chat_raw);
			if (!bot_token || !Number.isFinite(chat_id) || chat_id < 0) {
				return { error: 'Telegram: bot token and non-negative chat id are required' };
			}
			return { kind: 'telegram', bot_token, chat_id };
		}
		case 'discord': {
			const bot_token = requireString(form, 'bot_token');
			const channel_raw = requireString(form, 'channel_id');
			const channel_id = channel_raw == null ? NaN : Number(channel_raw);
			if (!bot_token || !Number.isFinite(channel_id) || channel_id < 0) {
				return { error: 'Discord: bot token and channel id are required' };
			}
			return { kind: 'discord', bot_token, channel_id };
		}
		case 'discord_webhook': {
			const webhook = requireString(form, 'webhook');
			if (!webhook) return { error: 'Discord webhook: a webhook URL is required' };
			return { kind: 'discord_webhook', webhook };
		}
		case 'teams': {
			const webhook = requireString(form, 'webhook');
			if (!webhook) return { error: 'Teams: a webhook URL is required' };
			return { kind: 'teams', webhook };
		}
		case 'gotify': {
			const server = requireString(form, 'server');
			const token = requireString(form, 'token');
			if (!server || !token) return { error: 'Gotify: server URL and token are required' };
			return { kind: 'gotify', server, token };
		}
		case 'email': {
			const recipient = requireString(form, 'recipient');
			if (!recipient) {
				return { error: 'Email: a recipient address is required' };
			}
			return { kind: 'email', recipient };
		}
		case 'ntfy': {
			const server = requireString(form, 'server');
			const topic = requireString(form, 'topic');
			if (!server || !topic) return { error: 'ntfy: server URL and topic are required' };
			const access_token = requireString(form, 'access_token');
			return access_token
				? { kind: 'ntfy', server, topic, access_token }
				: { kind: 'ntfy', server, topic };
		}
		case 'pushover': {
			const token = requireString(form, 'token');
			const user = requireString(form, 'user');
			if (!token || !user) {
				return { error: 'Pushover: application token and user/group key are required' };
			}
			const device = requireString(form, 'device');
			return device ? { kind: 'pushover', token, user, device } : { kind: 'pushover', token, user };
		}
		case 'matrix': {
			const homeserver = requireString(form, 'homeserver');
			const access_token = requireString(form, 'access_token');
			const room_id = requireString(form, 'room_id');
			if (!homeserver || !access_token || !room_id) {
				return { error: 'Matrix: homeserver URL, access token and room id are required' };
			}
			return { kind: 'matrix', homeserver, access_token, room_id };
		}
		case 'webhook': {
			const url = requireString(form, 'url');
			if (!url) return { error: 'Webhook: a URL is required' };
			// Optional headers entered one per line as `Name: value`.
			const headersRaw = form.get('headers');
			const headers: Record<string, string> = {};
			if (typeof headersRaw === 'string' && headersRaw.trim()) {
				for (const line of headersRaw.split('\n')) {
					const trimmed = line.trim();
					if (!trimmed) continue;
					const idx = trimmed.indexOf(':');
					if (idx === -1) return { error: `Webhook header must be "Name: value": ${trimmed}` };
					const name = trimmed.slice(0, idx).trim();
					const value = trimmed.slice(idx + 1).trim();
					if (!name) return { error: 'Webhook header name cannot be empty' };
					headers[name] = value;
				}
			}
			return Object.keys(headers).length > 0
				? { kind: 'webhook', url, headers }
				: { kind: 'webhook', url };
		}
		default:
			return { error: `Unknown notifier kind: ${String(kind)}` };
	}
}

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	try {
		const notifiers = await listNotifiers(auth.userId);
		return { notifiers, error: null };
	} catch (e) {
		console.error('[notifiers] list failed:', e);
		return { notifiers: [], error: 'Failed to load notifiers from the controller' };
	}
};

export const actions: Actions = {
	create: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const parsed = parseConfig(form);
		if ('error' in parsed) return fail(400, { createError: parsed.error });

		try {
			const result = await createNotifier(auth.userId, parsed);
			if (!result.ok) {
				return fail(402, { createError: result.upgradeRequired });
			}
			return { created: result.notifier };
		} catch (e) {
			console.error('[notifiers] create failed:', e);
			return fail(500, { createError: 'Failed to create notifier' });
		}
	},
	delete: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const raw = form.get('id');
		const id = typeof raw === 'string' ? Number.parseInt(raw, 10) : NaN;
		if (!Number.isFinite(id)) return fail(400, { deleteError: 'Invalid notifier id' });

		try {
			const ok = await deleteNotifier(auth.userId, id);
			if (!ok) return fail(404, { deleteError: 'Notifier not found' });
			return { deletedId: id };
		} catch (e) {
			console.error('[notifiers] delete failed:', e);
			return fail(500, { deleteError: 'Failed to delete notifier' });
		}
	},
	toggle: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const raw = form.get('id');
		const id = typeof raw === 'string' ? Number.parseInt(raw, 10) : NaN;
		if (!Number.isFinite(id)) return fail(400, { toggleError: 'Invalid notifier id' });
		const enabled = form.get('enabled') === 'true';

		try {
			const ok = await setNotifierEnabled(auth.userId, id, enabled);
			if (!ok) return fail(404, { toggleError: 'Notifier not found' });
			return { toggledId: id, toggledTo: enabled };
		} catch (e) {
			console.error('[notifiers] toggle failed:', e);
			return fail(500, { toggleError: 'Failed to toggle notifier' });
		}
	},
	test: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const raw = form.get('id');
		const id = typeof raw === 'string' ? Number.parseInt(raw, 10) : NaN;
		if (!Number.isFinite(id)) return fail(400, { testError: 'Invalid notifier id' });

		try {
			const result = await testNotifier(auth.userId, id);
			if (!result.ok) return fail(502, { testedId: id, testError: result.error });
			return { testedId: id };
		} catch (e) {
			console.error('[notifiers] test failed:', e);
			return fail(500, { testError: 'Failed to send test message' });
		}
	}
};
