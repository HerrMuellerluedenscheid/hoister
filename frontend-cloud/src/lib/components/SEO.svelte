<script lang="ts">
	import { page } from '$app/state';
	import { SITE_URL } from '$lib/seo';

	interface Props {
		title: string;
		description: string;
		/** Path-only canonical (e.g. "/"); defaults to the current route. */
		path?: string;
		/** Absolute or root-relative OG image; defaults to the shared card. */
		image?: string;
		/** og:type — "website" for pages, "article" for posts. */
		type?: string;
		/** Discourage indexing (auth-gated / utility pages). */
		noindex?: boolean;
		/** Optional JSON-LD object rendered as a <script type="application/ld+json">. */
		jsonLd?: Record<string, unknown>;
	}

	let {
		title,
		description,
		path = page.url.pathname,
		image = '/og.png',
		type = 'website',
		noindex = false,
		jsonLd
	}: Props = $props();

	const canonical = $derived(`${SITE_URL}${path}`);
	const imageUrl = $derived(image.startsWith('http') ? image : `${SITE_URL}${image}`);
</script>

<svelte:head>
	<title>{title}</title>
	<meta name="description" content={description} />
	<link rel="canonical" href={canonical} />
	{#if noindex}
		<meta name="robots" content="noindex, nofollow" />
	{/if}

	<!-- Open Graph -->
	<meta property="og:type" content={type} />
	<meta property="og:site_name" content="Hoister" />
	<meta property="og:title" content={title} />
	<meta property="og:description" content={description} />
	<meta property="og:url" content={canonical} />
	<meta property="og:image" content={imageUrl} />
	<meta property="og:image:width" content="1200" />
	<meta property="og:image:height" content="630" />

	<!-- Twitter -->
	<meta name="twitter:card" content="summary_large_image" />
	<meta name="twitter:title" content={title} />
	<meta name="twitter:description" content={description} />
	<meta name="twitter:image" content={imageUrl} />

	{#if jsonLd}
		{@html `<script type="application/ld+json">${JSON.stringify(jsonLd)}<\/script>`}
	{/if}
</svelte:head>
