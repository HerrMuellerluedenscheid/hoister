import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';

const USERNAME = process.env.HOISTER_AUTH_USERNAME;
const PASSWORD = process.env.HOISTER_AUTH_PASSWORD;

export const actions = {
  default: async ({ request, cookies }) => {
    console.debug("logging attempt");
    const data = await request.formData();
    const username = data.get('username');
    const password = data.get('password');
    if (USERNAME === undefined || PASSWORD === undefined) {
      return fail(500, { error: 'Authentication is not configured on the server.' });
    }

    if (username === USERNAME && password === PASSWORD) {
      console.debug("logged in");
      cookies.set('session', 'authenticated', {
        path: '/',
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'strict',
        maxAge: 60 * 60 * 24 * 7 // 1 week
      });

      throw redirect(303, '/containers');
    }

    return fail(401, { error: 'Invalid credentials' });
  }
} satisfies Actions;
