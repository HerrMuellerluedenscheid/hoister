import type { RequestHandler } from './$types';
import { SITE_URL } from '$lib/seo';

// Public, indexable routes only. Auth-gated app pages (/containers, /tokens, …)
// and the legal pages (/impressum, /datenschutz) are noindex and excluded.
const ROUTES = ['/'];

// Served dynamically by adapter-node rather than prerendered: the payload is
// tiny and on-demand rendering avoids the build-time prerender step (which
// aborts under bun in the production image build).
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
