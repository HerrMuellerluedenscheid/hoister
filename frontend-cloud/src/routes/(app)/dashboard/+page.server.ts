import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getDeployments } from '$lib/api/deployments';
import { listTokens } from '$lib/api/tokens';
import { getLatestMetrics } from '$lib/api/metrics';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const [deploymentsResult, tokensResult, metricsResult] = await Promise.allSettled([
		getDeployments(auth.userId),
		listTokens(auth.userId),
		getLatestMetrics(auth.userId)
	]);

	if (deploymentsResult.status === 'rejected') {
		console.error('[dashboard] deployments fetch failed:', deploymentsResult.reason);
	}
	if (tokensResult.status === 'rejected') {
		console.error('[dashboard] tokens fetch failed:', tokensResult.reason);
	}
	if (metricsResult.status === 'rejected') {
		console.error('[dashboard] metrics fetch failed:', metricsResult.reason);
	}

	return {
		deployments:
			deploymentsResult.status === 'fulfilled' ? deploymentsResult.value.deployments : [],
		deploymentsError:
			deploymentsResult.status === 'rejected' ? 'Failed to connect to backend' : null,
		tokenCount: tokensResult.status === 'fulfilled' ? tokensResult.value.length : null,
		latestMetrics: metricsResult.status === 'fulfilled' ? metricsResult.value : []
	};
};
