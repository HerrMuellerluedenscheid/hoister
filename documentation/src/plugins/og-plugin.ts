import type { StarlightPlugin } from '@astrojs/starlight/types';

// Registers the route middleware that injects per-page og:image / twitter:image
// tags pointing at the generated social cards.
export function ogPlugin(): StarlightPlugin {
	return {
		name: 'hoister-og-images',
		hooks: {
			'config:setup'({ addRouteMiddleware }) {
				addRouteMiddleware({ entrypoint: './src/plugins/og-middleware.ts' });
			}
		}
	};
}
