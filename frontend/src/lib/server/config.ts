const USE_HTTPS = process.env.USE_HTTPS === 'true';
const USE_SECURE_COOKIES = process.env.USE_SECURE_COOKIES === 'true' || USE_HTTPS;

export const config = {
  useHttps: USE_HTTPS,
  useSecureCookies: USE_SECURE_COOKIES,
  sessionMaxAge: parseInt(process.env.SESSION_MAX_AGE || '604800'), // Default 1 week in seconds
  cookieSameSite: (process.env.COOKIE_SAME_SITE || 'strict') as 'strict' | 'lax' | 'none'
};
