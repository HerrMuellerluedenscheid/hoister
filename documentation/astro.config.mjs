// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://docs.hoister.io',
	integrations: [
		starlight({
			title: 'Hoister',
			social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/HerrMuellerluedenscheid/hoister' }],
			sidebar: [
				{
					label: 'Guides',
					items: [
						// Each item here is one entry in the navigation menu.
						{ label: 'Getting Started', slug: 'guides/getting-started' },
						{ label: 'Notifications', slug: 'guides/notifications' },
						{ label: 'Registries', slug: 'guides/registries' },
						{ label: 'Dashboard', slug: 'guides/frontend' },
						{ label: 'Multi-host setup', slug: 'guides/multi-host' },
						{ label: 'TLS encryption', slug: 'guides/tls' },
					],
				},
				{
					label: 'Reference',
					autogenerate: { directory: 'reference' },
				},
			],
			customCss: [
				'./src/styles/custom.css',
			],
		}),
	],
});
