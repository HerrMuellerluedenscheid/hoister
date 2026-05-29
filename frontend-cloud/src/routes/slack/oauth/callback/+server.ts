import { redirect } from '@sveltejs/kit';
import { env as publicEnv } from '$env/dynamic/public';
import { env as privateEnv } from '$env/dynamic/private';
import { createNotifier } from '$lib/api/notifiers';
import type { RequestHandler } from './$types';

const STATE_COOKIE = 'slack_oauth_state';

/** Where Slack's `oauth.v2.access` lands the installed webhook. */
interface SlackOAuthResponse {
	ok: boolean;
	error?: string;
	incoming_webhook?: { url?: string; channel?: string };
}

function done(status: 'connected' | 'denied' | 'error' | 'upgrade'): never {
	throw redirect(303, `/notifiers?slack=${status}`);
}

/**
 * Completes the Slack "Add to Slack" flow: validates the CSRF `state`,
 * exchanges the one-time `code` for an incoming webhook, and persists it as a
 * Slack notifier for the signed-in user via the controller.
 */
export const GET: RequestHandler = async ({ locals, url, cookies, fetch }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	// Always clear the one-time state cookie, whatever the outcome.
	const expectedState = cookies.get(STATE_COOKIE);
	cookies.delete(STATE_COOKIE, { path: '/slack/oauth' });

	// User declined on Slack's consent screen, or Slack returned an error.
	if (url.searchParams.get('error')) done('denied');

	const code = url.searchParams.get('code');
	const state = url.searchParams.get('state');
	if (!code || !state || !expectedState || state !== expectedState) done('error');

	const clientId = publicEnv.PUBLIC_SLACK_CLIENT_ID;
	const clientSecret = privateEnv.SLACK_CLIENT_SECRET;
	if (!clientId || !clientSecret) done('error');

	let payload: SlackOAuthResponse;
	try {
		const resp = await fetch('https://slack.com/api/oauth.v2.access', {
			method: 'POST',
			headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
			body: new URLSearchParams({
				client_id: clientId,
				client_secret: clientSecret,
				code: code!,
				redirect_uri: `${url.origin}/slack/oauth/callback`
			})
		});
		payload = (await resp.json()) as SlackOAuthResponse;
	} catch (e) {
		console.error('[slack oauth] token exchange failed:', e);
		done('error');
	}

	const webhook = payload!.incoming_webhook?.url;
	const channel = payload!.incoming_webhook?.channel;
	if (!payload!.ok || !webhook || !channel) {
		console.error('[slack oauth] unexpected response:', payload!.error ?? 'no incoming_webhook');
		done('error');
	}

	try {
		const result = await createNotifier(auth.userId, { kind: 'slack', webhook, channel });
		if (!result.ok) done('upgrade');
	} catch (e) {
		console.error('[slack oauth] createNotifier failed:', e);
		done('error');
	}

	done('connected');
};
