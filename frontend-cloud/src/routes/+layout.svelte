<script lang="ts">
	import './layout.css';
	import { ClerkProvider } from 'svelte-clerk';
	import { env } from '$env/dynamic/public';
	import CookieBanner from '$lib/components/CookieBanner.svelte';

	let { children } = $props();
</script>

<ClerkProvider publishableKey={env.PUBLIC_CLERK_PUBLISHABLE_KEY}>
	{@render children()}
	<CookieBanner />
</ClerkProvider>
