import type { PageServerLoad } from './$types';
import { getInspections } from '$lib/api/inspect';

export const load: PageServerLoad = async () => {
  try {
    return await getInspections();
  } catch (err) {
    console.error('Failed to load deployments:', err);

    // Otherwise, return empty data with error message
    return {
      deployments: [],
      error: 'Failed to connect to backend service'
    };
  }
};
