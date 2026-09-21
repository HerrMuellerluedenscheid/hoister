import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { acceptInvitation, declineInvitation } from '$lib/api/projects';
import { listTokens } from '$lib/api/tokens';
import { startProCheckout } from '$lib/server/checkout';

export const load: PageServerLoad = async ({ locals, parent }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	// Projects and invitations are loaded (and claimed) by the app layout.
	const { projects, projectsError, invitations } = await parent();

	// A user with nothing to look at yet — no project, no token, no
	// invitation to answer — belongs in onboarding. Someone who was invited
	// stays here to accept.
	if (!projectsError && projects.length === 0 && invitations.length === 0) {
		const tokens = await listTokens(auth.userId).catch((e) => {
			console.error('[projects] token check failed:', e);
			return null;
		});
		if (tokens && tokens.length === 0) throw redirect(303, '/tokens');
	}

	return {};
};

function invitationId(form: FormData): string {
	const raw = form.get('invitation_id');
	return typeof raw === 'string' ? raw.trim() : '';
}

export const actions: Actions = {
	accept: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const id = invitationId(await request.formData());
		if (!id) return fail(400, { invitationError: 'Invalid invitation' });
		const result = await acceptInvitation(auth.userId, id);
		if (!result.ok) return fail(400, { invitationError: result.error });
		throw redirect(303, `/projects/${result.data}`);
	},

	decline: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const id = invitationId(await request.formData());
		if (!id) return fail(400, { invitationError: 'Invalid invitation' });
		const result = await declineInvitation(auth.userId, id);
		if (!result.ok) return fail(400, { invitationError: result.error });
		return { declined: id };
	},

	upgrade: async ({ locals, url }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const result = await startProCheckout(auth.userId, url.origin);
		if (!result.ok) return fail(result.status, { upgradeError: result.error });
		throw redirect(303, result.url);
	}
};
