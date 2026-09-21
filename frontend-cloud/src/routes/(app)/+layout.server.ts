import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { getMe } from '$lib/api/me';
import type { PlanStatus } from '$lib/api/me';
import { listProjects, listReceivedInvitations } from '$lib/api/projects';
import { claimPendingInvitations, lookupPeople, type Person } from '$lib/server/sharing';
import type { ProjectSummaryResponse } from '../../bindings/ProjectSummaryResponse';
import type { ReceivedInvitationResponse } from '../../bindings/ReceivedInvitationResponse';

export const load: LayoutServerLoad = async ({ locals, cookies, url, untrack }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	// Before listing invitations, so a freshly signed-up invitee sees the
	// invitation they signed up for on their very first page. `untrack` keeps
	// this load from re-running on every navigation just because it looked at
	// the URL.
	const secure = untrack(() => url.protocol === 'https:');
	await claimPendingInvitations(auth.userId, cookies, secure);

	const [meResult, projectsResult, invitationsResult] = await Promise.allSettled([
		getMe(auth.userId),
		listProjects(auth.userId),
		listReceivedInvitations(auth.userId)
	]);

	let me: PlanStatus | null = null;
	let meError: string | null = null;
	if (meResult.status === 'fulfilled') {
		me = meResult.value;
	} else {
		console.error('[layout] /me load failed:', meResult.reason);
		meError = 'Failed to load plan';
	}

	let projects: ProjectSummaryResponse[] = [];
	let projectsError: string | null = null;
	if (projectsResult.status === 'fulfilled') {
		projects = projectsResult.value;
	} else {
		console.error('[layout] projects load failed:', projectsResult.reason);
		projectsError = 'Failed to connect to the controller';
	}

	let invitations: ReceivedInvitationResponse[] = [];
	if (invitationsResult.status === 'fulfilled') {
		invitations = invitationsResult.value;
	} else {
		console.error('[layout] invitations load failed:', invitationsResult.reason);
	}
	// Only ask Clerk when there is someone to name.
	const inviters: Record<string, Person> =
		invitations.length > 0 ? await lookupPeople(invitations.map((i) => i.invited_by)) : {};

	return { me, meError, projects, projectsError, invitations, inviters };
};
