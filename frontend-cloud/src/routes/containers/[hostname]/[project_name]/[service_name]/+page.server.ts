import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getContainerInspection } from '$lib/api/inspect';
import { getDeploymentsByServiceName } from '$lib/api/deployments';

export const load: PageServerLoad = async ({ locals, params }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	try {
		const [inspectionResponse, deployments] = await Promise.all([
			getContainerInspection(
				auth.userId,
				params.hostname,
				params.project_name,
				params.service_name
			),
			getDeploymentsByServiceName(auth.userId, params.project_name, params.service_name)
		]);
		return {
			inspections: inspectionResponse.data,
			deployments: deployments.deployments,
			error: null
		};
	} catch (err) {
		console.error('[container detail] load failed:', err);
		if (err && typeof err === 'object' && 'status' in err) throw err;
		return {
			inspections: null,
			deployments: [],
			error: 'Failed to connect to the controller'
		};
	}
};
