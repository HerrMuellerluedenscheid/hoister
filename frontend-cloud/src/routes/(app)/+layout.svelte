<script lang="ts">
	import { page } from '$app/state';
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

	function isActive(href: string): boolean {
		const path = page.url.pathname;
		return path === href || path.startsWith(href + '/');
	}

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

<div class="flex min-h-screen bg-zinc-950 text-zinc-100">
	<aside class="flex w-56 flex-shrink-0 flex-col border-r border-zinc-800 bg-zinc-950 px-4 py-5">
		<a href="/dashboard" class="mb-8 flex items-center gap-2 font-semibold tracking-tight">
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
			<UserButton />
		</div>
	</aside>

	<main class="min-w-0 flex-1">
		{@render children()}
	</main>
</div>
