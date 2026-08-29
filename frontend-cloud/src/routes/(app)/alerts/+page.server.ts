import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import {
	createAlertRule,
	deleteAlertRule,
	listAlertHistory,
	listAlertRules,
	setAlertRuleEnabled,
	type AlertHistory,
	type AlertMetric,
	type CreateAlertRuleBody
} from '$lib/api/alerts';

const EMPTY_HISTORY: AlertHistory = { events: [], new_since: null, new_count: 0 };

const METRICS: AlertMetric[] = ['cpu_pct', 'mem_pct', 'mem_bytes'];

function optionalString(form: FormData, key: string): string | undefined {
	const raw = form.get(key);
	if (typeof raw !== 'string') return undefined;
	const trimmed = raw.trim();
	return trimmed.length > 0 ? trimmed : undefined;
}

function parseRule(form: FormData): CreateAlertRuleBody | { error: string } {
	const metric = form.get('metric');
	if (typeof metric !== 'string' || !METRICS.includes(metric as AlertMetric)) {
		return { error: `Unknown metric: ${String(metric)}` };
	}

	const threshold = Number(form.get('threshold'));
	if (!Number.isFinite(threshold) || threshold < 0) {
		return { error: 'Threshold must be a non-negative number' };
	}

	// Durations are entered in minutes; the API wants seconds.
	const forMinutes = Number(form.get('for_minutes'));
	if (!Number.isFinite(forMinutes) || forMinutes < 0) {
		return { error: 'Sustained duration must be a non-negative number of minutes' };
	}
	const cooldownRaw = form.get('cooldown_minutes');
	const cooldownMinutes = cooldownRaw === null || cooldownRaw === '' ? 0 : Number(cooldownRaw);
	if (!Number.isFinite(cooldownMinutes) || cooldownMinutes < 0) {
		return { error: 'Cooldown must be a non-negative number of minutes' };
	}

	// mem_bytes thresholds are entered in MiB for sanity.
	const apiThreshold = metric === 'mem_bytes' ? Math.round(threshold * 1024 * 1024) : threshold;

	return {
		metric: metric as AlertMetric,
		threshold: apiThreshold,
		for_seconds: Math.round(forMinutes * 60),
		cooldown_seconds: Math.round(cooldownMinutes * 60),
		hostname: optionalString(form, 'hostname'),
		project: optionalString(form, 'project'),
		service: optionalString(form, 'service')
	};
}

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	// The session id is what tells the controller whether this is a new login,
	// and so which history entries to flag as new. Rules and history are
	// independent — one failing must not blank out the other.
	const [rules, history] = await Promise.all([
		listAlertRules(auth.userId).catch((e) => {
			console.error('[alerts] list failed:', e);
			return null;
		}),
		listAlertHistory(auth.userId, auth.sessionId ?? '').catch((e) => {
			console.error('[alerts] history failed:', e);
			return null;
		})
	]);

	return {
		rules: rules ?? [],
		error: rules === null ? 'Failed to load alert rules from the controller' : null,
		history: history ?? EMPTY_HISTORY,
		historyError: history === null ? 'Failed to load alert history from the controller' : null
	};
};

export const actions: Actions = {
	create: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const parsed = parseRule(form);
		if ('error' in parsed) return fail(400, { createError: parsed.error });

		try {
			const result = await createAlertRule(auth.userId, parsed);
			if (!result.ok) return fail(400, { createError: result.error });
			return { created: result.rule };
		} catch (e) {
			console.error('[alerts] create failed:', e);
			return fail(500, { createError: 'Failed to create alert rule' });
		}
	},
	delete: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const raw = form.get('id');
		const id = typeof raw === 'string' ? raw.trim() : '';
		if (!id) return fail(400, { deleteError: 'Invalid alert rule id' });

		try {
			const ok = await deleteAlertRule(auth.userId, id);
			if (!ok) return fail(404, { deleteError: 'Alert rule not found' });
			return { deletedId: id };
		} catch (e) {
			console.error('[alerts] delete failed:', e);
			return fail(500, { deleteError: 'Failed to delete alert rule' });
		}
	},
	toggle: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const raw = form.get('id');
		const id = typeof raw === 'string' ? raw.trim() : '';
		if (!id) return fail(400, { toggleError: 'Invalid alert rule id' });
		const enabled = form.get('enabled') === 'true';

		try {
			const ok = await setAlertRuleEnabled(auth.userId, id, enabled);
			if (!ok) return fail(404, { toggleError: 'Alert rule not found' });
			return { toggledId: id, toggledTo: enabled };
		} catch (e) {
			console.error('[alerts] toggle failed:', e);
			return fail(500, { toggleError: 'Failed to toggle alert rule' });
		}
	}
};
