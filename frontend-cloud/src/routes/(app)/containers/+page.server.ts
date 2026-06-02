import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { getInspections } from '$lib/api/inspect';
import { applyActionFromForm, getPendingUpdates } from '$lib/api/pendingUpdates';

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

		const result = await applyActionFromForm(auth.userId, await request.formData());
		if (!result.ok) return fail(result.status, { applyError: result.error });
		return { applied: result.applied };
	}
};
