import tailwindcss from '@tailwindcss/vite';
import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
	plugins: [tailwindcss(), sveltekit()],
	// `stripe` is CJS and gets externalised by default, but the adapter-node
	// Docker image ships only the bundled `build/` dir (no node_modules), so an
	// external import fails at runtime. Force it to be bundled in.
	ssr: { noExternal: ['stripe'] }
});
