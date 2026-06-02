import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { getDeployments } from '$lib/api/deployments';
import { listTokens } from '$lib/api/tokens';
import { getLatestMetrics } from '$lib/api/metrics';
import { applyActionFromForm, getPendingUpdates } from '$lib/api/pendingUpdates';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const [deploymentsResult, tokensResult, metricsResult, pendingResult] = await Promise.allSettled([
		getDeployments(auth.userId),
		listTokens(auth.userId),
		getLatestMetrics(auth.userId),
		getPendingUpdates(auth.userId)
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
	if (pendingResult.status === 'rejected') {
		console.error('[dashboard] pending updates fetch failed:', pendingResult.reason);
	}

	return {
		deployments:
			deploymentsResult.status === 'fulfilled' ? deploymentsResult.value.deployments : [],
		deploymentsError:
			deploymentsResult.status === 'rejected' ? 'Failed to connect to backend' : null,
		tokenCount: tokensResult.status === 'fulfilled' ? tokensResult.value.length : null,
		latestMetrics: metricsResult.status === 'fulfilled' ? metricsResult.value : [],
		pendingUpdates: pendingResult.status === 'fulfilled' ? pendingResult.value : []
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
