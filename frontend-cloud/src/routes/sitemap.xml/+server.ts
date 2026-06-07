import type { RequestHandler } from './$types';
import { SITE_URL } from '$lib/seo';

// Public, indexable routes only. Auth-gated app pages (/dashboard, /tokens, …)
// and the legal pages (/impressum, /datenschutz) are noindex and excluded.
const ROUTES = ['/'];

export const prerender = true;

export const GET: RequestHandler = () => {
	const urls = ROUTES.map(
		(path) =>
			`	<url>\n		<loc>${SITE_URL}${path}</loc>\n		<changefreq>weekly</changefreq>\n	</url>`
	).join('\n');

	const body = `<?xml version="1.0" encoding="UTF-8"?>\n<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">\n${urls}\n</urlset>\n`;

	return new Response(body, {
		headers: {
			'Content-Type': 'application/xml',
			'Cache-Control': 'max-age=0, s-maxage=3600'
		}
	});
};
