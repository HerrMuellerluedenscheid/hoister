import { getCollection } from 'astro:content';
import { OGImageRoute } from 'astro-og-canvas';

// One generated 1200x630 social card per docs page, keyed by the same slug the
// route middleware uses (see src/plugins/og-middleware.ts). The index entry has
// an empty id, so it is normalised to "index".
const entries = await getCollection('docs');

const pages = Object.fromEntries(
	entries.map(({ id, data }) => [id || 'index', data])
);

export const { getStaticPaths, GET } = await OGImageRoute({
	param: 'slug',
	pages,
	getImageOptions: (_path, page: (typeof pages)[string]) => ({
		title: page.title,
		description: page.description ?? 'Automatic Docker container updates with rollback.',
		bgGradient: [[9, 9, 11]],
		border: { color: [129, 140, 248], width: 8, side: 'inline-start' },
		padding: 80,
		font: {
			title: { color: [250, 250, 250], weight: 'Bold', size: 64 },
			description: { color: [161, 161, 170], weight: 'Normal', size: 32 }
		},
		fonts: [
			'https://api.fontsource.org/v1/fonts/inter/latin-400-normal.ttf',
			'https://api.fontsource.org/v1/fonts/inter/latin-700-normal.ttf'
		]
	})
});
