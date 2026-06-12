import type { PageServerLoad } from './$types';
import { getInspections } from '$lib/api/inspect';
import { getPendingUpdates } from '$lib/api/pendingUpdates';
import { getLatestMetrics } from '$lib/api/metrics';

export const load: PageServerLoad = async () => {
  try {
    const [inspections, pendingUpdates, latestMetrics] = await Promise.all([
      getInspections(),
      getPendingUpdates(),
      getLatestMetrics()
    ]);
    return { ...inspections, pendingUpdates, latestMetrics };
  } catch (err) {
    console.error('Failed to load deployments:', err);

    return {
      inspections: [],
      pendingUpdates: [],
      latestMetrics: [],
      error: 'Failed to connect to backend service'
    };
  }
};
