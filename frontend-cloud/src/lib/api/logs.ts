import { env } from '$env/dynamic/private';
import type { ApiResponse } from '../../bindings/ApiResponse';
import type { ContainerLogsResponse } from '../../bindings/ContainerLogsResponse';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

/**
 * Ask the agent on `hostname` to ship the current log tail for one service.
 * Fire-and-forget: the controller broadcasts an SSE request to the agent, which
 * only answers when it was started with HOISTER_REPORT_LOGS=true. The logs are
 * then read back with `getContainerLogs`. Returns false when the controller
 * could not accept the request.
 */
export async function requestContainerLogs(
	userId: string,
	hostname: string,
	projectName: string,
	serviceName: string
): Promise<boolean> {
	if (!BACKEND_URL) return false;
	const response = await fetch(
		`${BACKEND_URL}/container/logs/${encodeURIComponent(hostname)}/${encodeURIComponent(projectName)}/${encodeURIComponent(serviceName)}/request`,
		{ method: 'POST', headers: backendHeaders(userId) }
	);
	return response.ok;
}

/**
 * Read the most recently forwarded log tail for one service. Returns null until
 * the agent has answered the request (controller 404) or after the entry
 * expires from the controller's in-memory store.
 */
export async function getContainerLogs(
	userId: string,
	hostname: string,
	projectName: string,
	serviceName: string
): Promise<ContainerLogsResponse | null> {
	if (!BACKEND_URL) return null;
	const response = await fetch(
		`${BACKEND_URL}/container/logs/${encodeURIComponent(hostname)}/${encodeURIComponent(projectName)}/${encodeURIComponent(serviceName)}`,
		{ headers: backendHeaders(userId) }
	);
	if (!response.ok) return null;
	const body = (await response.json()) as ApiResponse<ContainerLogsResponse>;
	return body.data ?? null;
}
