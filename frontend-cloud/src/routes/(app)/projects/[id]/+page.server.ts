import { error, fail, redirect } from '@sveltejs/kit';
import type { Actions, PageServerLoad } from './$types';
import {
	applyProjectUpdate,
	deleteProjectById,
	getProject,
	getProjectDeployments,
	getProjectPendingUpdates,
	getProjectServices,
	removeProjectMember
} from '$lib/api/projects';
import { lookupPeople } from '$lib/server/sharing';

export const load: PageServerLoad = async ({ locals, params }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');
	const userId = auth.userId;

	const detail = await getProject(userId, params.id);
	if (!detail) throw error(404, 'Project not found');

	const [services, pendingUpdates, deployments, people] = await Promise.all([
		getProjectServices(userId, params.id).catch((e) => {
			console.error('[project] services load failed:', e);
			return null;
		}),
		getProjectPendingUpdates(userId, params.id),
		getProjectDeployments(userId, params.id).catch((e) => {
			console.error('[project] deployments load failed:', e);
			return [];
		}),
		lookupPeople(detail.members.map((m) => m.user_id))
	]);

	return {
		detail,
		services: services ?? [],
		servicesError: services === null ? 'Failed to load services from the controller' : null,
		pendingUpdates,
		deployments,
		people,
		userId
	};
};

export const actions: Actions = {
	apply: async ({ locals, params, request }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const service = (await request.formData()).get('service_name');
		if (typeof service !== 'string' || !service) {
			return fail(400, { applyError: 'Missing service' });
		}
		const ok = await applyProjectUpdate(auth.userId, params.id, service);
		if (!ok) return fail(502, { applyError: 'Controller rejected the deploy request' });
		return { applied: service };
	},

	deleteProject: async ({ locals, params }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		try {
			await deleteProjectById(auth.userId, params.id);
		} catch (e) {
			console.error('[project] delete failed:', e);
			return fail(500, { deleteError: 'Failed to delete project' });
		}
		throw redirect(303, '/projects');
	},

	leave: async ({ locals, params }) => {
		const auth = locals.auth();
		if (!auth.userId) throw error(401, 'Not authenticated');

		const result = await removeProjectMember(auth.userId, params.id, auth.userId);
		if (!result.ok) return fail(400, { leaveError: result.error });
		throw redirect(303, '/projects');
	}
};
