import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import type { ApiResponse } from '../../bindings/ApiResponse';
import type { ServiceMetricsResponse } from '../../bindings/ServiceMetricsResponse';
import type { LatestMetricsResponse } from '../../bindings/LatestMetricsResponse';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

/**
 * Resource-usage time series for one service over the controller's retention
 * window. Returns an empty point list when the agent hasn't opted into
 * metrics (the endpoint always responds 200 with whatever it has).
 */
export async function getServiceMetrics(
	userId: string,
	hostname: string,
	project_name: string,
	service_name: string
): Promise<ServiceMetricsResponse> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');

	const response = await fetch(
		`${BACKEND_URL}/container/metrics/${encodeURIComponent(hostname)}/${encodeURIComponent(project_name)}/${encodeURIComponent(service_name)}`,
		{ headers: backendHeaders(userId) }
	);
	if (!response.ok) throw error(response.status, 'Failed to load metrics from backend');

	const result = (await response.json()) as ApiResponse<ServiceMetricsResponse>;
	if (!result.success || !result.data)
		throw error(500, result.error || 'Unknown error from backend');
	return result.data;
}

/** Latest sample per container, for the dashboard aggregate panel. */
export async function getLatestMetrics(userId: string): Promise<LatestMetricsResponse> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');

	const response = await fetch(`${BACKEND_URL}/container/metrics`, {
		headers: backendHeaders(userId)
	});
	if (!response.ok) throw error(response.status, 'Failed to load metrics from backend');

	return (await response.json()) as LatestMetricsResponse;
}
