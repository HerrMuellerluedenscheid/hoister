import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { getContainerInspection } from '$lib/api/inspect';
import { getDeploymentsByServiceName } from '$lib/api/deployments';
import { getServiceMetrics } from '$lib/api/metrics';
import { applyActionFromForm, getPendingUpdates } from '$lib/api/pendingUpdates';

export const load: PageServerLoad = async ({ locals, params }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	try {
		const [inspectionResponse, deployments, metrics, pendingUpdates] = await Promise.all([
			getContainerInspection(
				auth.userId,
				params.hostname,
				params.project_name,
				params.service_name
			),
			getDeploymentsByServiceName(auth.userId, params.project_name, params.service_name),
			// Metrics are opt-in on the agent; don't fail the page when absent.
			getServiceMetrics(
				auth.userId,
				params.hostname,
				params.project_name,
				params.service_name
			).catch((e) => {
				console.error('[container detail] metrics load failed:', e);
				return null;
			}),
			getPendingUpdates(auth.userId).catch((e) => {
				console.error('[container detail] pending updates load failed:', e);
				return [];
			})
		]);
		// Only this service's pending update is relevant on the detail page.
		const pendingUpdate = pendingUpdates.filter(
			(u) =>
				u.hostname === params.hostname &&
				u.project_name === params.project_name &&
				u.service_name === params.service_name
		);
		return {
			inspections: inspectionResponse.data,
			deployments: deployments.deployments,
			metrics,
			pendingUpdate,
			error: null
		};
	} catch (err) {
		console.error('[container detail] load failed:', err);
		if (err && typeof err === 'object' && 'status' in err) throw err;
		return {
			inspections: null,
			deployments: [],
			metrics: null,
			pendingUpdate: [],
			error: 'Failed to connect to the controller'
		};
	}
};

export const actions: Actions = {
	apply: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const result = await applyActionFromForm(auth.userId, await request.formData());
		if (!result.ok) return fail(result.status, { applyError: result.error });
		return { applied: result.applied };
	}
};
