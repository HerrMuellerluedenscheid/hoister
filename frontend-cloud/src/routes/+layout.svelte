<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import { ClerkProvider } from 'svelte-clerk';
	import { PUBLIC_CLERK_PUBLISHABLE_KEY } from '$env/static/public';
	import CookieBanner from '$lib/components/CookieBanner.svelte';

	let { children } = $props();
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<ClerkProvider publishableKey={PUBLIC_CLERK_PUBLISHABLE_KEY}>
	{@render children()}
	<CookieBanner />
</ClerkProvider>
