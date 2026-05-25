import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getDeployments } from '$lib/api/deployments';
import { getOrCreateToken } from '$lib/api/token';
import type { TokenResponse } from '$lib/api/token';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const [deploymentsResult, tokenResult] = await Promise.allSettled([
		getDeployments(auth.userId),
		getOrCreateToken(auth.userId)
	]);

	if (deploymentsResult.status === 'rejected') {
		console.error('[dashboard] deployments fetch failed:', deploymentsResult.reason);
	}
	if (tokenResult.status === 'rejected') {
		console.error('[dashboard] token fetch failed:', tokenResult.reason);
	}

	return {
		deployments: deploymentsResult.status === 'fulfilled' ? deploymentsResult.value.deployments : [],
		error: deploymentsResult.status === 'rejected' ? 'Failed to connect to backend' : null,
		agentToken: tokenResult.status === 'fulfilled' ? (tokenResult.value as TokenResponse) : null,
		tokenError:
			tokenResult.status === 'rejected'
				? 'Could not retrieve agent token — check HOISTER_CONTROLLER_URL on the controller'
				: null
	};
};
