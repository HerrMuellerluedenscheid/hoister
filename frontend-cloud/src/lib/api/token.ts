import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export type TokenResponse = {
	/**
	 * Plaintext token. Only set when the controller just minted a fresh one
	 * (first call to /token or after /token/rotate). Returning users get
	 * `null` because the controller only stores SHA-256 hashes — the
	 * original plaintext is unrecoverable. Call `rotateToken` to obtain a
	 * new one.
	 */
	token: string | null;
	user_id: string;
	is_new: boolean;
};

export async function getOrCreateToken(userId: string): Promise<TokenResponse> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');

	const response = await fetch(`${BACKEND_URL}/token`, {
		headers: backendHeaders(userId)
	});

	if (!response.ok) throw error(response.status, 'Failed to retrieve agent token');

	const result = await response.json();
	return result.data as TokenResponse;
}

export async function rotateToken(userId: string): Promise<TokenResponse> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');

	const response = await fetch(`${BACKEND_URL}/token/rotate`, {
		method: 'POST',
		headers: backendHeaders(userId)
	});

	if (!response.ok) throw error(response.status, 'Failed to rotate agent token');

	const result = await response.json();
	return result.data as TokenResponse;
}
