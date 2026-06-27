import { error, json } from '@sveltejs/kit';
import type { RequestHandler } from './$types';
import { getContainerLogs, requestContainerLogs } from '$lib/api/logs';

/**
 * Trigger an on-demand log request: the controller broadcasts an SSE event to
 * the user's agents, and the agent on the matching host ships its log tail back
 * (only if it runs with HOISTER_REPORT_LOGS=true). The browser then polls GET.
 */
export const POST: RequestHandler = async ({ locals, params }) => {
	const auth = locals.auth();
	if (!auth.userId) throw error(401, 'Not authenticated');

	const ok = await requestContainerLogs(
		auth.userId,
		params.hostname,
		params.project_name,
		params.service_name
	);
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

	const logs = await getContainerLogs(
		auth.userId,
		params.hostname,
		params.project_name,
		params.service_name
	);
	return json(logs);
};
