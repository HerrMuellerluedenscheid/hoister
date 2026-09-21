import { redirect } from '@sveltejs/kit';
import type { PageServerLoad } from './$types';

// The project list moved to /projects; keep old bookmarks working.
export const load: PageServerLoad = () => {
	throw redirect(301, '/projects');
};
