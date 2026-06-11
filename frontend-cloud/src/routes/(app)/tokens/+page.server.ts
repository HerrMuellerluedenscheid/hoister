import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { createToken, deleteToken, listTokens } from '$lib/api/tokens';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	try {
		const tokens = await listTokens(auth.userId);
		return { tokens, error: null };
	} catch (e) {
		console.error('[tokens] list failed:', e);
		return { tokens: [], error: 'Failed to load tokens from the controller' };
	}
};

export const actions: Actions = {
	create: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const raw = form.get('comment');
		const comment = typeof raw === 'string' && raw.trim().length > 0 ? raw.trim() : null;

		try {
			const created = await createToken(auth.userId, comment);
			return { created };
		} catch (e) {
			console.error('[tokens] create failed:', e);
			return fail(500, { createError: 'Failed to create token' });
		}
	},
	delete: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const id = form.get('id');
		if (typeof id !== 'string' || id.trim() === '') return fail(400, { deleteError: 'Invalid token id' });

		try {
			const ok = await deleteToken(auth.userId, id);
			if (!ok) return fail(404, { deleteError: 'Token not found' });
			return { deletedId: id };
		} catch (e) {
			console.error('[tokens] delete failed:', e);
			return fail(500, { deleteError: 'Failed to delete token' });
		}
	}
};
