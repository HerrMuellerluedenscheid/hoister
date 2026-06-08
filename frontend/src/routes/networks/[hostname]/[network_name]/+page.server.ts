import type { PageServerLoad } from './$types';
import { getInspections } from '$lib/api/inspect';

export const load: PageServerLoad = async ({ params }) => {
  try {
    const { inspections } = await getInspections();
    // Docker networks are host-scoped, so a network is identified by host +
    // name. Keep every service whose inspect data lists this network.
    const members = inspections.filter(
      (i) =>
        i.hostname === params.hostname &&
        i.container_inspections?.NetworkSettings?.Networks &&
        params.network_name in i.container_inspections.NetworkSettings.Networks
    );
    return {
      hostname: params.hostname,
      networkName: params.network_name,
      members,
      error: null
    };
  } catch (err) {
    console.error('Failed to load network members:', err);
    return {
      hostname: params.hostname,
      networkName: params.network_name,
      members: [],
      error: 'Failed to connect to backend service'
    };
  }
};
