import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { listTokens } from '$lib/api/tokens';
import { startProCheckout } from '$lib/server/checkout';

export const load: PageServerLoad = async ({ locals, parent }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	// The project list is loaded (and invitations claimed) by the app layout.
	const { projects, projectsError } = await parent();

	// A user with no project and no token yet has nothing to see here — send
	// them to onboarding. Someone who was only invited to a project has a
	// project, so they stay.
	if (!projectsError && projects.length === 0) {
		const tokens = await listTokens(auth.userId).catch((e) => {
			console.error('[projects] token check failed:', e);
			return null;
		});
		if (tokens && tokens.length === 0) throw redirect(303, '/tokens');
	}

	return {};
};

export const actions: Actions = {
	upgrade: async ({ locals, url }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const result = await startProCheckout(auth.userId, url.origin);
		if (!result.ok) return fail(result.status, { upgradeError: result.error });
		throw redirect(303, result.url);
	}
};
