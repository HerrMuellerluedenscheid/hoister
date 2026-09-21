import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';
import { getProjectServiceMetrics } from '$lib/api/projects';
import type { ServiceMetricsResponse } from '../../../bindings/ServiceMetricsResponse';
import type { ProjectSummaryResponse } from '../../../bindings/ProjectSummaryResponse';

export interface ProjectResources {
	project: Pick<ProjectSummaryResponse, 'id' | 'name' | 'hostname' | 'role'>;
	services: ServiceMetricsResponse[];
}

export const load: PageServerLoad = async ({ locals, parent }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');
	const userId = auth.userId;

	// Owned and shared projects alike, as loaded by the app layout.
	const { projects, projectsError } = await parent();
	if (projectsError) return { groups: [] as ProjectResources[], error: projectsError };

	// The controller has no bulk time-series endpoint, so fan out one request
	// per service through the project-scoped endpoint (which works for shared
	// projects too). Kept parallel and fault-tolerant: a single slow or missing
	// series shouldn't sink the whole page.
	const targets = projects.flatMap((project) =>
		project.services.map((service) => ({ project, service: service.name }))
	);
	const results = await Promise.allSettled(
		targets.map((t) => getProjectServiceMetrics(userId, t.project.id, t.service))
	);

	const byProject = new Map<string, ProjectResources>();
	results.forEach((result, i) => {
		const { project } = targets[i];
		if (result.status === 'rejected') {
			console.error('[resources] service metrics fetch failed:', result.reason);
			return;
		}
		// Containers that don't report metrics would only add empty charts.
		if (result.value.points.length === 0) return;
		let group = byProject.get(project.id);
		if (!group) {
			const { id, name, hostname, role } = project;
			group = { project: { id, name, hostname, role }, services: [] };
			byProject.set(project.id, group);
		}
		group.services.push(result.value);
	});

	// `projects` is already ordered by name; keep that, and services by name.
	const groups = projects.flatMap((p) => byProject.get(p.id) ?? []);
	for (const g of groups) g.services.sort((a, b) => a.service_name.localeCompare(b.service_name));

	return { groups, error: null };
};
