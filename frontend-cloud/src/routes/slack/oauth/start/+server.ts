import { error, redirect } from '@sveltejs/kit';
import { env } from '$env/dynamic/public';
import type { RequestHandler } from './$types';

// Not exported: SvelteKit +server.ts only allows HTTP-verb / `_`-prefixed
// exports. The callback route keeps its own copy of this name.
const STATE_COOKIE = 'slack_oauth_state';

/**
 * Kicks off the Slack "Add to Slack" OAuth flow. Generates a CSRF `state`,
 * stashes it in an httpOnly cookie, and redirects the user to Slack's consent
 * screen requesting the `incoming-webhook` scope. On approval Slack sends the
 * user back to /slack/oauth/callback with a one-time code.
 */
export const GET: RequestHandler = ({ locals, url, cookies }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const clientId = env.PUBLIC_SLACK_CLIENT_ID;
	if (!clientId) throw error(503, 'Slack integration is not configured');

	const state = crypto.randomUUID();
	cookies.set(STATE_COOKIE, state, {
		path: '/slack/oauth',
		httpOnly: true,
		sameSite: 'lax',
		secure: url.protocol === 'https:',
		maxAge: 600
	});

	const redirectUri = `${url.origin}/slack/oauth/callback`;
	const authorize = new URL('https://slack.com/oauth/v2/authorize');
	authorize.searchParams.set('client_id', clientId);
	authorize.searchParams.set('scope', 'incoming-webhook');
	authorize.searchParams.set('redirect_uri', redirectUri);
	authorize.searchParams.set('state', state);

	throw redirect(302, authorize.toString());
};
