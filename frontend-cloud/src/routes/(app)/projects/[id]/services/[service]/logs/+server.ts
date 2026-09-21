import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getProjectServiceLogs, requestProjectServiceLogs } from '$lib/api/projects';

/**
 * Trigger an on-demand log request: the controller broadcasts an SSE event to
 * the project owner's agents, and the agent on the project's host ships its
 * log tail back (only if it runs with HOISTER_REPORT_LOGS=true). The browser
 * then polls GET.
 */
export const POST: RequestHandler = async ({ locals, params }) => {
	const auth = locals.auth();
	if (!auth.userId) throw error(401, 'Not authenticated');

	const ok = await requestProjectServiceLogs(auth.userId, params.id, params.service);
	return json({ ok });
};

/**
 * Read the latest forwarded log tail for this service from the controller's
 * ephemeral store. Returns `null` until the agent has answered (or the entry
 * has expired); these logs are never persisted.
 */
export const GET: RequestHandler = async ({ locals, params }) => {
	const auth = locals.auth();
	if (!auth.userId) throw error(401, 'Not authenticated');

	const logs = await getProjectServiceLogs(auth.userId, params.id, params.service);
	return json(logs);
};
