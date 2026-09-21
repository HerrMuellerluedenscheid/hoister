import { error, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { listNotifiers } from '$lib/api/notifiers';
import { notifierActions } from '$lib/server/notifierActions';

export const load: PageServerLoad = async ({ locals, params, parent }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const { projects } = await parent();
	const project = projects.find((p) => p.id === params.id);
	if (!project) throw error(404, 'Project not found');

	try {
		const notifiers = await listNotifiers(auth.userId, params.id);
		return { project, notifiers, error: null };
	} catch (e) {
		console.error('[project notifiers] list failed:', e);
		return { project, notifiers: [], error: 'Failed to load notifiers from the controller' };
	}
};

export const actions: Actions = notifierActions((event) => event.params.id);
