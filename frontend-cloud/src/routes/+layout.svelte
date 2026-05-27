<script lang="ts">
	import './layout.css';
	import { ClerkProvider } from 'svelte-clerk';
	import { env } from '$env/dynamic/public';
	import CookieBanner from '$lib/components/CookieBanner.svelte';
	import { afterNavigate } from '$app/navigation';
	import { analyticsConsent } from '$lib/consent.svelte';
	import { initPostHog, capturePageView, shutdownPostHog } from '$lib/posthog';

	let { children } = $props();

	$effect(() => {
		if (analyticsConsent.value === 'accepted') {
			initPostHog();
		} else if (analyticsConsent.value === 'declined') {
			shutdownPostHog();
		}
	});

	afterNavigate((nav) => {
		if (analyticsConsent.value !== 'accepted') return;
		if (!nav.to) return;
		capturePageView(nav.to.url.toString());
	});
</script>

<ClerkProvider publishableKey={env.PUBLIC_CLERK_PUBLISHABLE_KEY}>
	{@render children()}
	<CookieBanner />
</ClerkProvider>
