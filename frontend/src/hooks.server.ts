import { redirect } from '@sveltejs/kit';
import type { Handle } from '@sveltejs/kit';

export const handle: Handle = async ({ event, resolve }) => {
    const session = event.cookies.get('session');

    // Allow access to login page
    if (event.url.pathname === '/login') {
        return resolve(event);
    }

    // Check if authenticated
    if (session !== 'authenticated') {
        throw redirect(303, '/login');
    }

    return resolve(event);
};
