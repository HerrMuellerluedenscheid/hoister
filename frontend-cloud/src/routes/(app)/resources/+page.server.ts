import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getLatestMetrics, getServiceMetrics } from '$lib/api/metrics';
import type { ServiceMetricsResponse } from '../../../bindings/ServiceMetricsResponse';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');
	const userId = auth.userId;

	// The controller has no bulk time-series endpoint, so we first learn which
	// containers report metrics, then fan out one time-series request per
	// container. Kept parallel and fault-tolerant: a single slow or missing
	// series shouldn't sink the whole page.
	let latest;
	try {
		latest = await getLatestMetrics(userId);
	} catch (e) {
		console.error('[resources] latest metrics fetch failed:', e);
		return { services: [] as ServiceMetricsResponse[], error: 'Failed to connect to backend' };
	}

	const results = await Promise.allSettled(
		latest.map((m) => getServiceMetrics(userId, m.hostname, m.project_name, m.service_name))
	);

	const services: ServiceMetricsResponse[] = [];
	for (const r of results) {
		if (r.status === 'fulfilled') services.push(r.value);
		else console.error('[resources] service metrics fetch failed:', r.reason);
	}

	// Stable ordering so the grouped layout doesn't reshuffle between refreshes.
	services.sort(
		(a, b) =>
			a.hostname.localeCompare(b.hostname) ||
			a.project_name.localeCompare(b.project_name) ||
			a.service_name.localeCompare(b.service_name)
	);

	return { services, error: null };
};
