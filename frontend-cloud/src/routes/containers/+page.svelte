<script lang="ts">
	import { UserButton } from 'svelte-clerk';
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import InspectionCard from '$lib/components/InspectionCard.svelte';
	import type { PageProps } from './$types';
	import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';

	let { data }: PageProps = $props();

	let refreshInterval: ReturnType<typeof setInterval>;

	onMount(() => {
		refreshInterval = setInterval(() => invalidateAll(), 10_000);
	});

	onDestroy(() => clearInterval(refreshInterval));

	function groupByHostname(
		inspections: ContainerStateResponse[]
	): Map<string, ContainerStateResponse[]> {
		const grouped = new Map<string, ContainerStateResponse[]>();
		for (const inspection of inspections) {
			const host = inspection.hostname;
			if (!grouped.has(host)) grouped.set(host, []);
			grouped.get(host)!.push(inspection);
		}
		return grouped;
	}

	const grouped = $derived(groupByHostname(data.inspections));
</script>

<div class="min-h-screen bg-zinc-950 text-zinc-100">
	<header class="flex items-center justify-between border-b border-zinc-800 px-8 py-5">
		<div class="flex items-center gap-4">
			<a href="/dashboard" class="font-semibold tracking-tight hover:text-indigo-300">Hoister</a>
			<nav class="flex gap-3 text-sm text-zinc-400">
				<a href="/dashboard" class="hover:text-zinc-100">Dashboard</a>
				<a href="/containers" class="text-zinc-100">Containers</a>
			</nav>
		</div>
		<UserButton />
	</header>

	<main class="mx-auto max-w-7xl space-y-8 px-8 py-10">
		<h1 class="text-2xl font-bold">Containers</h1>

		{#if data.error}
			<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
				<span class="font-medium">Error:</span>
				{data.error}
			</div>
		{/if}

		{#if data.inspections.length === 0 && !data.error}
			<div class="rounded-xl border border-zinc-800 bg-zinc-900 px-5 py-4 text-sm text-zinc-400">
				No containers reporting yet. Make sure an agent is connected and labelled with
				<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">hoister.enable=true</code>.
			</div>
		{/if}

		{#each [...grouped] as [hostname, inspections] (hostname)}
			<section>
				<h2 class="mb-4 text-lg font-semibold text-zinc-300">{hostname}</h2>
				<div class="grid grid-cols-[repeat(auto-fit,minmax(320px,1fr))] gap-4">
					{#each inspections as inspection_data (inspection_data.service_name)}
						<InspectionCard {inspection_data} />
					{/each}
				</div>
			</section>
		{/each}
	</main>
</div>
