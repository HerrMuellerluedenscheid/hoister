import { fail, redirect } from '@sveltejs/kit';
import type { Actions } from './$types';
import { config } from "$lib/server/config";
import bcrypt from 'bcryptjs';

const USERNAME = process.env.HOISTER_AUTH_USERNAME;
const PASSWORD = process.env.HOISTER_AUTH_PASSWORD;

// Detect if password is a bcrypt hash
function isBcryptHash(password: string): boolean {
  return password.startsWith('$2a$') || password.startsWith('$2b$') || password.startsWith('$2y$');
}

// Verify password against stored credential
async function verifyPassword(inputPassword: string, storedPassword: string): Promise<boolean> {
  if (isBcryptHash(storedPassword)) {
    return await bcrypt.compare(inputPassword, storedPassword);
  } else {
    console.warn('⚠️  WARNING: HOISTER_AUTH_PASSWORD is stored in plain text!');
    console.warn('⚠️  For better security, use a bcrypt hashed password instead.');
    console.warn('⚠️  Generate one e.g. with: node -e "console.log(require(\'bcryptjs\').hashSync(\'your-password\', 10))"');
    return inputPassword === storedPassword;
  }
}

export const actions = {
  default: async ({ request, cookies }) => {
    const data = await request.formData();
    const username = data.get('username');
    const password = data.get('password');

    if (USERNAME === undefined || PASSWORD === undefined) {
      return fail(500, { error: 'Authentication is not configured on the server.' });
    }

    if (typeof username !== 'string' || typeof password !== 'string') {
      return fail(400, { error: 'Invalid input' });
    }

    // Check username and password
    if (username === USERNAME && await verifyPassword(password, PASSWORD)) {
      cookies.set('session', 'authenticated', {
        path: '/',
        httpOnly: true,
        secure: config.useSecureCookies,
        sameSite: config.cookieSameSite,
        maxAge: config.sessionMaxAge
      });
      console.debug('User logged in:', username);
      throw redirect(303, '/containers');
    }

    return fail(401, { error: 'Invalid credentials' });
  }
} satisfies Actions;
