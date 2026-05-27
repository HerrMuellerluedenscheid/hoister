import { redirect } from '@sveltejs/kit';
import type { LayoutServerLoad } from './$types';
import { getMe } from '$lib/api/me';
import type { PlanStatus } from '$lib/api/me';

export const load: LayoutServerLoad = async ({ locals }) => {
	const auth = locals.auth();
	if (!auth.userId) throw redirect(303, '/');

	let me: PlanStatus | null = null;
	let meError: string | null = null;
	try {
		me = await getMe(auth.userId);
	} catch (e) {
		console.error('[layout] /me load failed:', e);
		meError = 'Failed to load plan';
	}

	return { me, meError };
};
