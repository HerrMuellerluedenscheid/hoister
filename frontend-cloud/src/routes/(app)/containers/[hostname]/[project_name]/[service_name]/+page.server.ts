import { error, redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

/**
 * Legacy host/project/service URL. Notifications and older bookmarks still
 * link here, so resolve it to the project-scoped service page — for projects
 * shared with the user too. Prefer the user's own project if a shared one has
 * the same host and name.
 */
export const load: PageServerLoad = async ({ params, parent }) => {
	const { projects } = await parent();
	const matches = projects.filter(
		(p) => p.hostname === params.hostname && p.name === params.project_name
	);
	const project = matches.find((p) => p.role === 'owner') ?? matches[0];
	if (!project) throw error(404, 'Project not found');
	throw redirect(
		302,
		`/projects/${project.id}/services/${encodeURIComponent(params.service_name)}`
	);
};
