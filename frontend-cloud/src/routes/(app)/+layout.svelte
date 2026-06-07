<script lang="ts">
	import { page } from '$app/state';
	import { afterNavigate } from '$app/navigation';
	import { UserButton, useClerkContext } from 'svelte-clerk';
	import { analyticsConsent } from '$lib/consent.svelte';
	import { identifyUser, resetUser } from '$lib/posthog';

	let { children } = $props();

	const nav = [
		{ href: '/dashboard', label: 'Dashboard' },
		{ href: '/containers', label: 'Containers' },
		{ href: '/tokens', label: 'Tokens' },
		{ href: '/notifiers', label: 'Notifiers' },
		{ href: '/settings/plan', label: 'Plan' }
	];

	let mobileOpen = $state(false);

	function isActive(href: string): boolean {
		const path = page.url.pathname;
		return path === href || path.startsWith(href + '/');
	}

	// Close the drawer whenever navigation finishes (e.g. a nav link tap).
	afterNavigate(() => {
		mobileOpen = false;
	});

	const clerk = useClerkContext();
	let lastIdentified = $state<string | null>(null);
	$effect(() => {
		if (analyticsConsent.value !== 'accepted') return;
		const userId = clerk.auth.userId ?? null;
		if (userId && userId !== lastIdentified) {
			identifyUser(userId);
			lastIdentified = userId;
		} else if (!userId && lastIdentified) {
			resetUser();
			lastIdentified = null;
		}
	});
</script>

<svelte:head>
	<!-- Auth-gated app pages carry no SEO value and shouldn't be indexed. -->
	<meta name="robots" content="noindex, nofollow" />
</svelte:head>

<svelte:window
	onkeydown={(e) => {
		if (e.key === 'Escape') mobileOpen = false;
	}}
/>

<div class="flex min-h-screen flex-col bg-zinc-950 text-zinc-100 md:flex-row">
	<!-- Mobile top bar -->
	<header class="flex items-center justify-between border-b border-zinc-800 px-4 py-3 md:hidden">
		<button
			type="button"
			onclick={() => (mobileOpen = true)}
			aria-label="Open navigation"
			aria-expanded={mobileOpen}
			class="-ml-1 rounded-md p-2 text-zinc-300 transition hover:bg-zinc-900 hover:text-zinc-100"
		>
			<svg class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
				<path stroke-linecap="round" stroke-linejoin="round" d="M4 6h16M4 12h16M4 18h16" />
			</svg>
		</button>
		<a href="/dashboard" class="flex items-center gap-2 font-semibold tracking-tight">
			<svg
				class="h-6 w-6 text-indigo-400"
				viewBox="0 0 24 24"
				fill="none"
				stroke="currentColor"
				stroke-width="2"
			>
				<path
					stroke-linecap="round"
					stroke-linejoin="round"
					d="M5 10l1.5-4.5h11L19 10M5 10h14M5 10l-2 7h18l-2-7M9 17v2m6-2v2m-3-9v4"
				/>
			</svg>
			<span>Hoister</span>
		</a>
		<UserButton />
	</header>

	<!-- Backdrop (mobile, when drawer open) -->
	{#if mobileOpen}
		<button
			type="button"
			aria-label="Close navigation"
			onclick={() => (mobileOpen = false)}
			class="fixed inset-0 z-40 bg-black/60 md:hidden"
		></button>
	{/if}

	<!-- Sidebar: slide-in drawer on mobile, static column on md+ -->
	<aside
		class="fixed inset-y-0 left-0 z-50 flex w-64 flex-col border-r border-zinc-800 bg-zinc-950 px-4 py-5 transition-transform duration-200 ease-out md:static md:z-auto md:w-56 md:shrink-0 md:translate-x-0 md:transition-none {mobileOpen
			? 'translate-x-0'
			: '-translate-x-full'}"
	>
		<div class="mb-8 flex items-center justify-between">
			<a href="/dashboard" class="flex items-center gap-2 font-semibold tracking-tight">
				<svg
					class="h-6 w-6 text-indigo-400"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M5 10l1.5-4.5h11L19 10M5 10h14M5 10l-2 7h18l-2-7M9 17v2m6-2v2m-3-9v4"
					/>
				</svg>
				<span>Hoister</span>
			</a>
			<button
				type="button"
				onclick={() => (mobileOpen = false)}
				aria-label="Close navigation"
				class="rounded-md p-1 text-zinc-400 transition hover:bg-zinc-900 hover:text-zinc-100 md:hidden"
			>
				<svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
					<path stroke-linecap="round" stroke-linejoin="round" d="M6 6l12 12M18 6L6 18" />
				</svg>
			</button>
		</div>
		<nav class="space-y-1">
			{#each nav as item (item.href)}
				<a
					href={item.href}
					class="block rounded-md px-3 py-2 text-sm transition {isActive(item.href)
						? 'bg-zinc-800 text-zinc-100'
						: 'text-zinc-400 hover:bg-zinc-900 hover:text-zinc-100'}"
				>
					{item.label}
				</a>
			{/each}
		</nav>
		<div class="mt-auto border-t border-zinc-800 pt-4">
			<a
				href="https://github.com/HerrMuellerluedenscheid/hoister"
				target="_blank"
				rel="noopener noreferrer"
				class="mb-3 flex items-center gap-2 px-3 text-sm text-zinc-400 transition hover:text-zinc-100"
			>
				<svg class="h-5 w-5" fill="currentColor" viewBox="0 0 24 24" aria-hidden="true">
					<path
						fill-rule="evenodd"
						d="M12 2C6.477 2 2 6.484 2 12.017c0 4.425 2.865 8.18 6.839 9.504.5.092.682-.217.682-.483 0-.237-.008-.868-.013-1.703-2.782.605-3.369-1.343-3.369-1.343-.454-1.158-1.11-1.466-1.11-1.466-.908-.62.069-.608.069-.608 1.003.07 1.531 1.032 1.531 1.032.892 1.53 2.341 1.088 2.91.832.092-.647.35-1.088.636-1.338-2.22-.253-4.555-1.113-4.555-4.951 0-1.093.39-1.988 1.029-2.688-.103-.253-.446-1.272.098-2.65 0 0 .84-.27 2.75 1.026A9.564 9.564 0 0112 6.844c.85.004 1.705.115 2.504.337 1.909-1.296 2.747-1.027 2.747-1.027.546 1.379.202 2.398.1 2.651.64.7 1.028 1.595 1.028 2.688 0 3.848-2.339 4.695-4.566 4.943.359.309.678.92.678 1.855 0 1.338-.012 2.419-.012 2.747 0 .268.18.58.688.482A10.019 10.019 0 0022 12.017C22 6.484 17.522 2 12 2z"
						clip-rule="evenodd"
					/>
				</svg>
				<span>GitHub</span>
			</a>
			<div class="hidden md:block">
				<UserButton />
			</div>
		</div>
	</aside>

	<main class="min-w-0 flex-1">
		{@render children()}
	</main>
</div>
