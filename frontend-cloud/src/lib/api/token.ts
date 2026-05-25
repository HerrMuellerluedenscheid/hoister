import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export type TokenResponse = {
	token: string;
	user_id: string;
	is_new: boolean;
};

export async function getOrCreateToken(userId: string): Promise<TokenResponse> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');

	const response = await fetch(`${BACKEND_URL}/token`, {
		headers: { 'X-User-Id': userId }
	});

	if (!response.ok) throw error(response.status, 'Failed to retrieve agent token');

	const result = await response.json();
	return result.data as TokenResponse;
}
