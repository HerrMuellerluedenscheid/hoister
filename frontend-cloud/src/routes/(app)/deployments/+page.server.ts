import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getDeployments } from '$lib/api/deployments';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	try {
		const { deployments } = await getDeployments(auth.userId);
		return { deployments, deploymentsError: null };
	} catch (e) {
		console.error('[deployments] fetch failed:', e);
		return { deployments: [], deploymentsError: 'Failed to connect to backend' };
	}
};
