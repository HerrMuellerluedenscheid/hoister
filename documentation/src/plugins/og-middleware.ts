import { defineRouteMiddleware } from '@astrojs/starlight/route-data';

// Point each docs page at its generated social card (src/pages/og/[...slug].png.ts)
// and ensure a large-image Twitter card. Keyed by the page id, normalised so the
// index entry (empty id) maps to "index", matching the OG route.
export const onRequest = defineRouteMiddleware((context) => {
	const { head, entry } = context.locals.starlightRoute;
	const slug = entry.id || 'index';
	const ogImageUrl = new URL(`/og/${slug}.png`, context.site).href;

	head.push(
		{ tag: 'meta', attrs: { property: 'og:image', content: ogImageUrl } },
		{ tag: 'meta', attrs: { property: 'og:image:width', content: '1200' } },
		{ tag: 'meta', attrs: { property: 'og:image:height', content: '630' } },
		{ tag: 'meta', attrs: { name: 'twitter:card', content: 'summary_large_image' } },
		{ tag: 'meta', attrs: { name: 'twitter:image', content: ogImageUrl } }
	);
});
