import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { deleteProjectInvitation, getProject, removeProjectMember } from '$lib/api/projects';
import { inviteToProject, lookupPeople } from '$lib/server/sharing';

export const load: PageServerLoad = async ({ locals, params }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const detail = await getProject(auth.userId, params.id);
	if (!detail) throw error(404, 'Project not found');

	const people = await lookupPeople([
		...detail.members.map((m) => m.user_id),
		...detail.members.flatMap((m) => (m.invited_by ? [m.invited_by] : [])),
		...detail.invitations.map((i) => i.invited_by),
		...detail.invitations.flatMap((i) => (i.user_id ? [i.user_id] : []))
	]);

	return { detail, people, userId: auth.userId };
};

function field(form: FormData, key: string): string {
	const raw = form.get(key);
	return typeof raw === 'string' ? raw.trim() : '';
}

export const actions: Actions = {
	invite: async ({ locals, params, request, url }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const email = field(await request.formData(), 'email');
		if (!email) return fail(400, { inviteError: 'Enter an email address.', email });

		const outcome = await inviteToProject({
			inviterId: auth.userId,
			projectId: params.id,
			rawEmail: email,
			origin: url.origin
		});
		if (outcome.kind === 'error') return fail(400, { inviteError: outcome.message, email });
		return { invite: outcome };
	},

	remove: async ({ locals, params, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const memberId = field(await request.formData(), 'member_id');
		if (!memberId) return fail(400, { memberError: 'Invalid member' });
		const result = await removeProjectMember(auth.userId, params.id, memberId);
		if (!result.ok) return fail(400, { memberError: result.error });
		return { removed: memberId };
	},

	revoke: async ({ locals, params, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const invitationId = field(await request.formData(), 'invitation_id');
		if (!invitationId) return fail(400, { memberError: 'Invalid invitation' });
		const result = await deleteProjectInvitation(auth.userId, params.id, invitationId);
		if (!result.ok) return fail(400, { memberError: result.error });
		return { revoked: invitationId };
	},

	leave: async ({ locals, params }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const result = await removeProjectMember(auth.userId, params.id, auth.userId);
		if (!result.ok) return fail(400, { memberError: result.error });
		throw redirect(303, '/projects');
	}
};
