import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { listAccessibleDeployments } from '$lib/api/projects';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	// Across owned and shared projects alike.
	try {
		const deployments = await listAccessibleDeployments(auth.userId);
		return { deployments, deploymentsError: null };
	} catch (e) {
		console.error('[deployments] fetch failed:', e);
		return { deployments: [], deploymentsError: 'Failed to connect to backend' };
	}
};
