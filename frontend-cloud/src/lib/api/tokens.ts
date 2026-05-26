import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { backendHeaders } from './_headers';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

export interface ApiToken {
	id: number;
	user_id: string;
	/** Plaintext token. Only present when the controller just minted it. */
	token?: string;
	/** `hst_<first 8 hex>` — stable, displayable identifier. */
	token_prefix: string;
	comment: string | null;
	created_at: string;
}

interface ApiResponse<T> {
	success: boolean;
	data: T | null;
	error: string | null;
}

async function unwrap<T>(response: Response, what: string): Promise<T> {
	if (!response.ok) throw error(response.status, `Failed to ${what}`);
	const result = (await response.json()) as ApiResponse<T>;
	if (!result.success || result.error || result.data == null) {
		throw error(500, result.error || `Failed to ${what}`);
	}
	return result.data;
}

export async function listTokens(userId: string): Promise<ApiToken[]> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/tokens`, {
		headers: backendHeaders(userId)
	});
	return unwrap<ApiToken[]>(response, 'list tokens');
}

export async function createToken(userId: string, comment: string | null): Promise<ApiToken> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const headers = { ...backendHeaders(userId), 'Content-Type': 'application/json' };
	const response = await fetch(`${BACKEND_URL}/tokens`, {
		method: 'POST',
		headers,
		body: JSON.stringify({ comment: comment?.trim() || null })
	});
	return unwrap<ApiToken>(response, 'create token');
}

export async function deleteToken(userId: string, tokenId: number): Promise<boolean> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/tokens/${tokenId}`, {
		method: 'DELETE',
		headers: backendHeaders(userId)
	});
	if (response.status === 404) return false;
	if (!response.ok) throw error(response.status, 'Failed to delete token');
	return true;
}
