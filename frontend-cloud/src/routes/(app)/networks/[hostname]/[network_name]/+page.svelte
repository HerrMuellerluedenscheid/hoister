<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import type { PageProps } from './$types';
	import type { ContainerStateResponse } from '../../../../../bindings/ContainerStateResponse';

	let { data }: PageProps = $props();

	let refreshInterval: ReturnType<typeof setInterval>;

	onMount(() => {
		refreshInterval = setInterval(() => invalidateAll(), 10_000);
	});

	onDestroy(() => clearInterval(refreshInterval));

	const members = $derived(
		[...data.members].sort((a, b) => a.service_name.localeCompare(b.service_name))
	);

	function status(inspection: ContainerStateResponse): string {
		return inspection.container_inspections?.State?.Status ?? 'unknown';
	}

	function ipOnNetwork(inspection: ContainerStateResponse): string {
		const net = inspection.container_inspections?.NetworkSettings?.Networks?.[data.networkName];
		return net?.IPAddress || '—';
	}

	function statusClass(s: string): string {
		return s === 'running'
			? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30'
			: s === 'exited' || s === 'dead'
				? 'bg-red-500/15 text-red-300 border-red-500/30'
				: s === 'paused'
					? 'bg-yellow-500/15 text-yellow-300 border-yellow-500/30'
					: s === 'restarting'
						? 'bg-blue-500/15 text-blue-300 border-blue-500/30'
						: 'bg-zinc-700/40 text-zinc-300 border-zinc-600/40';
	}
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-6xl space-y-6">
		<div>
			<a href="/containers" class="text-xs text-zinc-500 hover:text-zinc-300">← Containers</a>
			<h1 class="mt-2 text-2xl font-bold">
				<span class="text-zinc-400">Network</span>
				<span class="px-2 text-zinc-600">/</span>
				<span class="font-mono break-all">{data.networkName}</span>
			</h1>
			<p class="text-xs text-zinc-500">Host: {data.hostname}</p>
		</div>

		{#if data.error}
			<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
				<span class="font-medium">Error:</span>
				{data.error}
			</div>
		{/if}

		<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
			<h2 class="mb-4 text-base font-semibold text-zinc-200">
				Services on this network
				<span class="ml-1 text-sm font-normal text-zinc-500">({members.length})</span>
			</h2>

			{#if members.length === 0}
				<p class="text-sm text-zinc-500">
					No other reporting services are connected to this network.
				</p>
			{:else}
				<div class="grid grid-cols-[repeat(auto-fit,minmax(280px,1fr))] gap-3">
					{#each members as member (member.project_name + '/' + member.service_name)}
						<a
							href="/containers/{encodeURIComponent(member.hostname)}/{encodeURIComponent(
								member.project_name
							)}/{encodeURIComponent(member.service_name)}"
							class="block rounded-lg border border-zinc-800 bg-zinc-950/40 p-4 transition hover:border-zinc-700 hover:bg-zinc-900"
						>
							<div class="mb-2 flex items-start justify-between gap-3">
								<h3 class="text-sm font-semibold break-all text-zinc-100">
									{member.service_name}
								</h3>
								<span
									class="rounded-full border px-2 py-0.5 text-[10px] font-medium capitalize {statusClass(
										status(member)
									)}"
								>
									{status(member)}
								</span>
							</div>
							<dl class="space-y-1 text-xs text-zinc-400">
								<div class="flex gap-1">
									<dt class="text-zinc-500">Project:</dt>
									<dd class="break-all">{member.project_name}</dd>
								</div>
								<div class="flex gap-1">
									<dt class="text-zinc-500">IP:</dt>
									<dd class="font-mono">{ipOnNetwork(member)}</dd>
								</div>
							</dl>
						</a>
					{/each}
				</div>
			{/if}
		</section>
	</div>
</div>
