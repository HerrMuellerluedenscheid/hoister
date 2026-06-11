import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { ContainerStateResponses } from '../../bindings/ContainerStateResponses';
import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';
import type { ApiResponse } from '../../bindings/ApiResponse';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export type Inspection = {
	inspections: ContainerStateResponses;
	error: string | null;
};

export async function getInspections(userId: string): Promise<Inspection> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');

	const response = await fetch(`${BACKEND_URL}/container/state`, {
		headers: backendHeaders(userId)
	});
	if (!response.ok) throw error(response.status, 'Failed to load container state from backend');

	return { inspections: await response.json(), error: null };
}

/**
 * Delete a single (host, project) from the controller — removes its container
 * state and persisted metrics, freeing a slot against the plan's project cap.
 * Returns `false` when the project no longer exists (404).
 */
export async function deleteProject(
	userId: string,
	hostname: string,
	project_name: string
): Promise<boolean> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');

	const response = await fetch(
		`${BACKEND_URL}/container/state/${encodeURIComponent(hostname)}/${encodeURIComponent(project_name)}`,
		{ method: 'DELETE', headers: backendHeaders(userId) }
	);
	if (response.status === 404) return false;
	if (!response.ok) throw error(response.status, 'Failed to delete project');
	return true;
}

export async function getContainerInspection(
	userId: string,
	hostname: string,
	project_name: string,
	service_name: string
): Promise<ApiResponse<ContainerStateResponse>> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');

	const response = await fetch(
		`${BACKEND_URL}/container/state/${encodeURIComponent(hostname)}/${encodeURIComponent(project_name)}/${encodeURIComponent(service_name)}`,
		{ headers: backendHeaders(userId) }
	);
	if (!response.ok) throw error(response.status, 'Failed to load container state from backend');

	return await response.json();
}
