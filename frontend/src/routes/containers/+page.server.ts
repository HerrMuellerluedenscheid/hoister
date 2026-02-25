import type { PageServerLoad } from './$types';
import { getInspections } from '$lib/api/inspect';
import { getPendingUpdates } from '$lib/api/pendingUpdates';

export const load: PageServerLoad = async () => {
  try {
    const [inspections, pendingUpdates] = await Promise.all([
      getInspections(),
      getPendingUpdates()
    ]);
    return { ...inspections, pendingUpdates };
  } catch (err) {
    console.error('Failed to load deployments:', err);

    return {
      inspections: [],
      pendingUpdates: [],
      error: 'Failed to connect to backend service'
    };
  }
};
