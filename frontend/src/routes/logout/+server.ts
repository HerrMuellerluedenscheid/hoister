import { redirect } from '@sveltejs/kit';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ cookies }) => {
    cookies.delete('session', { path: '/' });
    throw redirect(303, '/login');
};
