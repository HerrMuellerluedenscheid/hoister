import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { ProjectName } from '../../bindings/ProjectName';
import type { ServiceName } from '../../bindings/ServiceName';
import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';
import type { ApiResponse } from '../../bindings/ApiResponse';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export async function getContainerInspection(
  project_name: ProjectName,
  service_name: ServiceName
): Promise<ApiResponse<ContainerStateResponse>> {

  if (!BACKEND_URL) {
    throw error(500, 'Backend URL not configured');
  }

  const response = await fetch(`${BACKEND_URL}/container/state/${project_name}/${service_name}`);
  if (!response.ok) {
    throw error(response.status, 'Failed to load container state from backend');
  }

  return await response.json();
}
