import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getInspections } from '$lib/api/inspect';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	try {
		return await getInspections(auth.userId);
	} catch (err) {
		console.error('[containers] failed to load inspections:', err);
		return {
			inspections: [],
			error: 'Failed to connect to the controller'
		};
	}
};
