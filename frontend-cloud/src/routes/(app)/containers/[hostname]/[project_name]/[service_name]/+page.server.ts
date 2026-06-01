import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getContainerInspection } from '$lib/api/inspect';
import { getDeploymentsByServiceName } from '$lib/api/deployments';
import { getServiceMetrics } from '$lib/api/metrics';

export const load: PageServerLoad = async ({ locals, params }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	try {
		const [inspectionResponse, deployments, metrics] = await Promise.all([
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
			})
		]);
		return {
			inspections: inspectionResponse.data,
			deployments: deployments.deployments,
			metrics,
			error: null
		};
	} catch (err) {
		console.error('[container detail] load failed:', err);
		if (err && typeof err === 'object' && 'status' in err) throw err;
		return {
			inspections: null,
			deployments: [],
			metrics: null,
			error: 'Failed to connect to the controller'
		};
	}
};
