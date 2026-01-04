import { error } from '@sveltejs/kit';
import type { Deployment } from '../../bindings/Deployment';
import { env } from '$env/dynamic/private';
import type { ProjectName } from '../../bindings/ProjectName';
import type { ServiceName } from '../../bindings/ServiceName';
import type { ApiResponse } from '../../bindings/ApiResponse';
import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export async function getDeploymentsByServiceName(
  project_name: ProjectName,
  service_name: ServiceName
): Promise<ApiResponse<Deployment>> {
  if (!BACKEND_URL) {
    throw error(500, 'Backend URL not configured');
  }
  const response = await fetch(`${BACKEND_URL}/deployments/${project_name}/${service_name}`);

  if (!response.ok) {
    throw error(response.status, 'Failed to load data from backend');
  }

  return await response.json();
}
