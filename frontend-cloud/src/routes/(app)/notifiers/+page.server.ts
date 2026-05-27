import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import {
	createNotifier,
	deleteNotifier,
	listNotifiers,
	setNotifierEnabled,
	type NotifierConfig
} from '$lib/api/notifiers';

function requireString(form: FormData, key: string): string | null {
	const raw = form.get(key);
	if (typeof raw !== 'string') return null;
	const trimmed = raw.trim();
	return trimmed.length > 0 ? trimmed : null;
}

function parseConfig(form: FormData): NotifierConfig | { error: string } {
	const kind = form.get('kind');
	switch (kind) {
		case 'slack': {
			const webhook = requireString(form, 'webhook');
			const channel = requireString(form, 'channel');
			if (!webhook || !channel) return { error: 'Slack: webhook and channel are required' };
			return { kind: 'slack', webhook, channel };
		}
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
		case 'gotify': {
			const server = requireString(form, 'server');
			const token = requireString(form, 'token');
			if (!server || !token) return { error: 'Gotify: server URL and token are required' };
			return { kind: 'gotify', server, token };
		}
		case 'email': {
			const smtp_server = requireString(form, 'smtp_server');
			const smtp_user = requireString(form, 'smtp_user');
			const smtp_password = requireString(form, 'smtp_password');
			const recipient = requireString(form, 'recipient');
			const from = requireString(form, 'from');
			if (!smtp_server || !smtp_user || !smtp_password || !recipient) {
				return {
					error: 'Email: smtp_server, smtp_user, smtp_password and recipient are required'
				};
			}
			return { kind: 'email', smtp_server, smtp_user, smtp_password, recipient, from };
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
			const created = await createNotifier(auth.userId, parsed);
			return { created };
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
	}
};
