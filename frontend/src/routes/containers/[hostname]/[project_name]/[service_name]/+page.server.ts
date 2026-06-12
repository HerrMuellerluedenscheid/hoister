import type { PageServerLoad } from '../../../../../../.svelte-kit/types/src/routes';
import { getContainerInspection } from '$lib/api/inspectContainer';
import { getServiceMetrics } from '$lib/api/metrics';
import type { ContainerStateResponse } from '../../../../../bindings/ContainerStateResponse';
import type { Deployment } from '../../../../../bindings/Deployment';
import type { MetricPointResponse } from '../../../../../bindings/MetricPointResponse';
import { getDeploymentsByServiceName } from '$lib/api/deploymentsByImage';

export type ContainerPageData = {
  inspections: ContainerStateResponse;
  deployments: [Deployment];
  metrics: MetricPointResponse[];
  error: string | null;
};

export const load: PageServerLoad = async ({ params }) => {
  try {
    const [inspectionsResponse, deployments, metricsResponse] = await Promise.all([
      getContainerInspection(params.hostname, params.project_name, params.service_name),
      getDeploymentsByServiceName(params.project_name, params.service_name),
      // Metrics are best-effort: a failure here shouldn't blank the whole page.
      getServiceMetrics(params.hostname, params.project_name, params.service_name).catch(() => null)
    ]);
    return {
      inspections: inspectionsResponse.data,
      deployments: deployments.data,
      metrics: metricsResponse?.data?.points ?? [],
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
      metrics: [],
      error: 'Failed to connect to backend service'
    };
  }
};
