import { redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { listNotifiers } from '$lib/api/notifiers';
import { notifierActions } from '$lib/server/notifierActions';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	try {
		const notifiers = await listNotifiers(auth.userId);
		return { notifiers, error: null };
	} catch (e) {
		console.error('[notifiers] list failed:', e);
		return { notifiers: [], error: 'Failed to load notifiers from the controller' };
	}
};

export const actions: Actions = notifierActions(() => undefined);
