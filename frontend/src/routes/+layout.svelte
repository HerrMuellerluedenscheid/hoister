<script lang="ts">
	import '../app.css';
	import favicon from '$lib/assets/favicon.svg';
	import Navbar from "$lib/components/Navbar.svelte";
	import { page } from '$app/stores';

	let { children } = $props();
</script>

<svelte:head>
	<link rel="icon" href={favicon} />
</svelte:head>

{#if $page.url.pathname !== '/login'}
	<Navbar />
{/if}

{@render children?.()}
