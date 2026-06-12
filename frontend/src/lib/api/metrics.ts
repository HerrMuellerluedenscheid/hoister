import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { HostName } from '../../bindings/HostName';
import type { ProjectName } from '../../bindings/ProjectName';
import type { ServiceName } from '../../bindings/ServiceName';
import type { ApiResponse } from '../../bindings/ApiResponse';
import type { ServiceMetricsResponse } from '../../bindings/ServiceMetricsResponse';
import type { LatestMetricsResponse } from '../../bindings/LatestMetricsResponse';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

/**
 * Latest resource-usage sample per container, for the dashboard cards. Returns
 * an empty list rather than throwing so a metrics hiccup never blanks the
 * dashboard — the cards just render without figures.
 */
export async function getLatestMetrics(): Promise<LatestMetricsResponse> {
  if (!BACKEND_URL) {
    throw error(500, 'Backend URL not configured');
  }
  const response = await fetch(`${BACKEND_URL}/container/metrics`);
  if (!response.ok) {
    console.error(`Failed to load latest metrics: ${response.status}`);
    return [];
  }
  return await response.json();
}

/** Resource-usage time series for one service over the retention window. */
export async function getServiceMetrics(
  hostname: HostName,
  project_name: ProjectName,
  service_name: ServiceName
): Promise<ApiResponse<ServiceMetricsResponse>> {
  if (!BACKEND_URL) {
    throw error(500, 'Backend URL not configured');
  }
  const response = await fetch(
    `${BACKEND_URL}/container/metrics/${hostname}/${project_name}/${service_name}`
  );
  if (!response.ok) {
    throw error(response.status, 'Failed to load service metrics from backend');
  }
  return await response.json();
}
