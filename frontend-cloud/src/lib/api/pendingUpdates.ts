import { env } from '$env/dynamic/private';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export interface PendingUpdate {
	hostname: string;
	project_name: string;
	service_name: string;
	image_name: string;
	new_digest: string;
	detected_at: string;
}

export async function getPendingUpdates(userId: string): Promise<PendingUpdate[]> {
	if (!BACKEND_URL) return [];
	const response = await fetch(`${BACKEND_URL}/pending-updates`, {
		headers: backendHeaders(userId)
	});
	if (!response.ok) return [];
	return (await response.json()) as PendingUpdate[];
}

export async function applyUpdate(
	userId: string,
	hostname: string,
	projectName: string,
	serviceName: string
): Promise<boolean> {
	if (!BACKEND_URL) return false;
	const response = await fetch(
		`${BACKEND_URL}/pending-updates/${encodeURIComponent(hostname)}/${encodeURIComponent(projectName)}/${encodeURIComponent(serviceName)}/apply`,
		{
			method: 'POST',
			headers: backendHeaders(userId)
		}
	);
	return response.ok;
}

export type ApplyActionResult =
	| { ok: true; applied: { hostname: string; project_name: string; service_name: string } }
	| { ok: false; status: number; error: string };

/**
 * Shared body for the `apply` SvelteKit form action used by the dashboard,
 * containers list, and container detail pages. Validates the form fields and
 * triggers the deployment on the controller.
 */
export async function applyActionFromForm(
	userId: string,
	formData: FormData
): Promise<ApplyActionResult> {
	const hostname = formData.get('hostname');
	const project_name = formData.get('project_name');
	const service_name = formData.get('service_name');

	if (
		typeof hostname !== 'string' ||
		typeof project_name !== 'string' ||
		typeof service_name !== 'string'
	) {
		return { ok: false, status: 400, error: 'Missing hostname / project / service' };
	}

	const applied = await applyUpdate(userId, hostname, project_name, service_name);
	if (!applied) {
		return { ok: false, status: 502, error: 'Controller rejected the deploy request' };
	}
	return { ok: true, applied: { hostname, project_name, service_name } };
}
