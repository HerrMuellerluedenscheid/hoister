import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import { deleteProject, getInspections } from '$lib/api/inspect';
import { applyActionFromForm, getPendingUpdates } from '$lib/api/pendingUpdates';
import { listTokens } from '$lib/api/tokens';
import { startProCheckout } from '$lib/server/checkout';

export const load: PageServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	const [inspectionsResult, pendingUpdates] = await Promise.allSettled([
		getInspections(auth.userId),
		getPendingUpdates(auth.userId)
	]);

	if (inspectionsResult.status === 'rejected') {
		console.error('[containers] inspections fetch failed:', inspectionsResult.reason);
	}

	// A user with no token yet has nothing to do here — send them to onboarding.
	// Only pay for the token lookup when there are no containers to show; an
	// established user with a connected agent always has at least one inspection.
	if (
		inspectionsResult.status === 'fulfilled' &&
		inspectionsResult.value.inspections.length === 0
	) {
		const tokens = await listTokens(auth.userId).catch((e) => {
			console.error('[containers] token check failed:', e);
			return null;
		});
		if (tokens && tokens.length === 0) throw redirect(303, '/tokens');
	}

	return {
		inspections:
			inspectionsResult.status === 'fulfilled' ? inspectionsResult.value.inspections : [],
		error: inspectionsResult.status === 'rejected' ? 'Failed to connect to the controller' : null,
		pendingUpdates: pendingUpdates.status === 'fulfilled' ? pendingUpdates.value : []
	};
};

export const actions: Actions = {
	apply: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const result = await applyActionFromForm(auth.userId, await request.formData());
		if (!result.ok) return fail(result.status, { applyError: result.error });
		return { applied: result.applied };
	},

	deleteProject: async ({ locals, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const form = await request.formData();
		const hostname = form.get('hostname');
		const projectName = form.get('project_name');
		if (
			typeof hostname !== 'string' ||
			typeof projectName !== 'string' ||
			!hostname ||
			!projectName
		) {
			return fail(400, { deleteError: 'Invalid project' });
		}

		try {
			const ok = await deleteProject(auth.userId, hostname, projectName);
			if (!ok) return fail(404, { deleteError: 'Project not found — it may already be gone.' });
			return { deletedProject: projectName };
		} catch (e) {
			console.error('[containers] delete project failed:', e);
			return fail(500, { deleteError: 'Failed to delete project' });
		}
	},

	upgrade: async ({ locals, url }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const result = await startProCheckout(auth.userId, url.origin);
		if (!result.ok) return fail(result.status, { upgradeError: result.error });
		throw redirect(303, result.url);
	}
};
