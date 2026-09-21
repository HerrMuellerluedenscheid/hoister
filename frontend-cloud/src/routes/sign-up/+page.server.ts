import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

// Someone who already has a session (e.g. opened an invitation link while
// signed in) has nothing to sign up for; the app claims invitations itself.
export const load: PageServerLoad = ({ locals }) => {
	if (locals.auth().userId) throw redirect(303, '/projects');
	return {};
};
