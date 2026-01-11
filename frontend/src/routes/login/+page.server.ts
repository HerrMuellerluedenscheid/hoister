import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import {config} from "$lib/server/config";

const USERNAME = process.env.HOISTER_AUTH_USERNAME;
const PASSWORD = process.env.HOISTER_AUTH_PASSWORD;

export const actions = {
  default: async ({ request, cookies }) => {
    const data = await request.formData();
    const username = data.get('username');
    const password = data.get('password');
    if (USERNAME === undefined || PASSWORD === undefined) {
      return fail(500, { error: 'Authentication is not configured on the server.' });
    }

    if (username === USERNAME && password === PASSWORD) {
      cookies.set('session', 'authenticated', {
        path: '/',
        httpOnly: true,
        secure: config.useSecureCookies,
        sameSite: config.cookieSameSite,
        maxAge: config.sessionMaxAge
      });
      console.debug("logged in");
      throw redirect(303, '/containers');
    }

    return fail(401, { error: 'Invalid credentials' });
  }
} satisfies Actions;
