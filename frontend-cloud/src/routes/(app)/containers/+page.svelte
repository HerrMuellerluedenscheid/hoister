<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import InspectionCard from '$lib/components/InspectionCard.svelte';
	import PendingUpdates from '$lib/components/PendingUpdates.svelte';
	import type { PageProps } from './$types';
	import type { ContainerStateResponse } from '../../../bindings/ContainerStateResponse';

	let { data, form }: PageProps = $props();

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

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-7xl space-y-8">
		<div class="flex items-baseline justify-between gap-4">
			<h1 class="text-2xl font-bold">Containers</h1>
			{#if data.me}
				{@const max = data.me.limits.max_projects}
				{@const used = data.me.usage.projects}
				<div class="text-sm text-zinc-400">
					{#if max === null}
						{used} project{used === 1 ? '' : 's'} <span class="text-indigo-400">(Pro)</span>
					{:else}
						<span class:text-amber-400={used >= max}>
							{used} / {max} projects
						</span>
						{#if used >= max}
							·
							<a href="/settings/plan" class="text-indigo-400 underline hover:text-indigo-300"
								>Upgrade</a
							>
						{/if}
					{/if}
				</div>
			{/if}
		</div>

		{#if data.me && data.me.plan === 'free' && data.me.limits.max_projects !== null && data.me.usage.projects >= data.me.limits.max_projects}
			<div
				class="rounded-xl border border-amber-800/50 bg-amber-950/30 px-4 py-3 text-sm text-amber-300"
			>
				You've reached the Free plan limit of {data.me.limits.max_projects} compose projects. Any new
				project reported by an agent will be rejected until you
				<a href="/settings/plan" class="font-medium underline hover:text-amber-200"
					>upgrade to Pro</a
				>.
			</div>
		{/if}

		{#if data.error}
			<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
				<span class="font-medium">Error:</span>
				{data.error}
			</div>
		{/if}

		{#if form?.applyError}
			<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
				<span class="font-medium">Apply failed:</span>
				{form.applyError}
			</div>
		{/if}

		<PendingUpdates updates={data.pendingUpdates} />

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
	</div>
</div>
