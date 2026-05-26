import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { getInspections } from '$lib/api/inspect';
import { applyUpdate, getPendingUpdates } from '$lib/api/pendingUpdates';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const [inspectionsResult, pendingUpdates] = await Promise.allSettled([
		getInspections(auth.userId),
		getPendingUpdates(auth.userId)
	]);

	if (inspectionsResult.status === 'rejected') {
		console.error('[containers] inspections fetch failed:', inspectionsResult.reason);
	}

	return {
		inspections:
			inspectionsResult.status === 'fulfilled' ? inspectionsResult.value.inspections : [],
		error: inspectionsResult.status === 'rejected' ? 'Failed to connect to the controller' : null,
		pendingUpdates: pendingUpdates.status === 'fulfilled' ? pendingUpdates.value : []
	};
};

export const actions: Actions = {
	apply: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const data = await request.formData();
		const hostname = data.get('hostname');
		const project_name = data.get('project_name');
		const service_name = data.get('service_name');

		if (
			typeof hostname !== 'string' ||
			typeof project_name !== 'string' ||
			typeof service_name !== 'string'
		) {
			return fail(400, { applyError: 'Missing hostname / project / service' });
		}

		const ok = await applyUpdate(auth.userId, hostname, project_name, service_name);
		if (!ok) return fail(502, { applyError: 'Controller rejected the apply request' });
		return { applied: { hostname, project_name, service_name } };
	}
};
