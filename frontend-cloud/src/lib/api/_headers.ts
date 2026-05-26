import { env } from '$env/dynamic/private';

/**
 * Build the headers every controller request needs:
 *   - X-User-Id   resolved from Clerk; identifies the tenant on the controller.
 *   - X-Internal-Auth (optional) shared secret the controller's internal
 *     router requires when bound to a non-loopback interface.
 *
 * Centralised so a forgotten header doesn't accidentally produce an
 * unauthenticated request.
 */
export function backendHeaders(userId: string): Record<string, string> {
	const headers: Record<string, string> = { 'X-User-Id': userId };
	if (env.HOISTER_CONTROLLER_INTERNAL_SECRET) {
		headers['X-Internal-Auth'] = env.HOISTER_CONTROLLER_INTERNAL_SECRET;
	}
	return headers;
}
