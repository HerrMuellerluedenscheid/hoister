import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { backendHeaders } from './_headers';
import type { Plan } from './me';

const BACKEND_URL = env.HOISTER_CONTROLLER_URL;

/**
 * Set a user's plan on the controller — the authoritative plan store that
 * gates notifier kinds and project limits. Called from the Stripe webhook
 * after the signature has been verified. The target user is passed as
 * `X-User-Id` (via `backendHeaders`), so the controller never trusts a
 * user id from a request body.
 */
export async function setPlan(userId: string, plan: Plan): Promise<void> {
	if (!BACKEND_URL) throw error(500, 'Backend URL not configured');
	const response = await fetch(`${BACKEND_URL}/billing/plan`, {
		method: 'POST',
		headers: { ...backendHeaders(userId), 'Content-Type': 'application/json' },
		body: JSON.stringify({ plan })
	});
	if (response.status !== 204 && !response.ok) {
		throw error(response.status, `Failed to set plan for ${userId}`);
	}
}
