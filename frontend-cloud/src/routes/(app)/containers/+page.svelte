<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import InspectionCard from '$lib/components/InspectionCard.svelte';
	import type { PageProps } from './$types';
	import type { ContainerStateResponse } from '../../../bindings/ContainerStateResponse';

	let { data, form }: PageProps = $props();

	let refreshInterval: ReturnType<typeof setInterval>;
	let applying = $state<Set<string>>(new Set());

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

	function updateKey(u: { hostname: string; project_name: string; service_name: string }): string {
		return `${u.hostname}/${u.project_name}/${u.service_name}`;
	}

	const grouped = $derived(groupByHostname(data.inspections));
</script>

<div class="px-8 py-10">
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
				<a href="/settings/plan" class="font-medium underline hover:text-amber-200">upgrade to Pro</a
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

		{#if data.pendingUpdates.length > 0}
			<section>
				<h2 class="mb-3 text-base font-semibold text-amber-300">Pending updates</h2>
				<div class="overflow-hidden rounded-xl border border-amber-800/50 bg-amber-950/20">
					<table class="min-w-full divide-y divide-amber-800/40 text-sm">
						<thead class="bg-amber-900/30 text-xs tracking-wider text-amber-300 uppercase">
							<tr>
								<th class="px-4 py-2 text-left font-medium">Host</th>
								<th class="px-4 py-2 text-left font-medium">Service</th>
								<th class="px-4 py-2 text-left font-medium">Image</th>
								<th class="px-4 py-2 text-left font-medium">New digest</th>
								<th class="px-4 py-2 text-left font-medium">Detected</th>
								<th class="px-4 py-2"></th>
							</tr>
						</thead>
						<tbody class="divide-y divide-amber-800/30">
							{#each data.pendingUpdates as update (updateKey(update))}
								{@const key = updateKey(update)}
								<tr class="text-zinc-300">
									<td class="px-4 py-2 break-all">{update.hostname}</td>
									<td class="px-4 py-2 font-medium break-all text-zinc-100"
										>{update.service_name}</td
									>
									<td class="px-4 py-2 font-mono text-xs break-all">{update.image_name}</td>
									<td class="px-4 py-2 font-mono text-xs text-zinc-500"
										>{update.new_digest.slice(0, 20)}…</td
									>
									<td class="px-4 py-2 text-xs text-zinc-500"
										>{new Date(update.detected_at).toLocaleString()}</td
									>
									<td class="px-4 py-2">
										<form
											method="POST"
											action="?/apply"
											use:enhance={() => {
												applying = new Set([...applying, key]);
												return async ({ update: updatePage }) => {
													await updatePage();
													applying = new Set([...applying].filter((k) => k !== key));
												};
											}}
										>
											<input type="hidden" name="hostname" value={update.hostname} />
											<input type="hidden" name="project_name" value={update.project_name} />
											<input type="hidden" name="service_name" value={update.service_name} />
											<button
												type="submit"
												disabled={applying.has(key)}
												class="rounded-md bg-amber-500 px-3 py-1 text-xs font-semibold text-amber-950 transition hover:bg-amber-400 disabled:opacity-50"
											>
												{applying.has(key) ? 'Applying…' : 'Apply'}
											</button>
										</form>
									</td>
								</tr>
							{/each}
						</tbody>
					</table>
				</div>
			</section>
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
	</div>
</div>
