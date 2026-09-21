import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import {
	applyProjectUpdate,
	getProjectDeployments,
	getProjectPendingUpdates,
	getProjectService,
	getProjectServiceMetrics
} from '$lib/api/projects';

export const load: PageServerLoad = async ({ locals, params, parent }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');
	const userId = auth.userId;

	const { projects } = await parent();
	const project = projects.find((p) => p.id === params.id);
	if (!project) throw error(404, 'Project not found');

	try {
		const [inspections, deployments, metrics, pendingUpdates] = await Promise.all([
			getProjectService(userId, params.id, params.service),
			getProjectDeployments(userId, params.id, params.service),
			// Metrics are opt-in on the agent; don't fail the page when absent.
			getProjectServiceMetrics(userId, params.id, params.service).catch((e) => {
				console.error('[service detail] metrics load failed:', e);
				return null;
			}),
			getProjectPendingUpdates(userId, params.id)
		]);
		return {
			project,
			inspections,
			deployments,
			metrics,
			// Only this service's pending update is relevant here.
			pendingUpdate: pendingUpdates.filter((u) => u.service_name === params.service),
			error: null
		};
	} catch (err) {
		console.error('[service detail] load failed:', err);
		if (err && typeof err === 'object' && 'status' in err) throw err;
		return {
			project,
			inspections: null,
			deployments: [],
			metrics: null,
			pendingUpdate: [],
			error: 'Failed to connect to the controller'
		};
	}
};

export const actions: Actions = {
	apply: async ({ locals, params }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const ok = await applyProjectUpdate(auth.userId, params.id, params.service);
		if (!ok) return fail(502, { applyError: 'Controller rejected the deploy request' });
		return { applied: params.service };
	}
};
