import type { PageServerLoad } from './$types';
import { getContainerInspection } from '$lib/api/inspectContainer';
import type { ContainerStateResponse } from '../../../../bindings/ContainerStateResponse';
import type { Deployment } from '../../../../bindings/Deployment';
import { getDeploymentsByServiceName } from '$lib/api/deploymentsByImage';

export type ContainerPageData = {
  inspections: ContainerStateResponse;
  deployments: [Deployment];
  error: string | null;
};

export const load: PageServerLoad = async ({ params }) => {
  try {
    const inspectionsResponse = await getContainerInspection(
      params.project_name,
      params.service_name
    );
    const deployments = await getDeploymentsByServiceName(params.project_name, params.service_name);
    return {
      inspections: inspectionsResponse.data,
      deployments: deployments.data,
      error: null
    };
  } catch (err) {
    console.error('Failed to load deployments and inspections:', err);

    // If it's already a SvelteKit error, re-throw it
    if (err && typeof err === 'object' && 'status' in err) {
      throw err;
    }

    // Otherwise, return empty data with error message
    return {
      inspections: [],
      deployments: [],
      error: 'Failed to connect to backend service'
    };
  }
};
