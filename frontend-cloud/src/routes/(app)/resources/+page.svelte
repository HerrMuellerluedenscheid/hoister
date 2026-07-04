<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import ServiceMetricsCharts from '$lib/components/ServiceMetricsCharts.svelte';
	import type { ServiceMetricsResponse } from '../../../bindings/ServiceMetricsResponse';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Group services under their host so each host gets one labelled block.
	function groupByHost(services: ServiceMetricsResponse[]): Map<string, ServiceMetricsResponse[]> {
		const grouped = new Map<string, ServiceMetricsResponse[]>();
		for (const s of services) {
			if (!grouped.has(s.hostname)) grouped.set(s.hostname, []);
			grouped.get(s.hostname)!.push(s);
		}
		return grouped;
	}

	const grouped = $derived(groupByHost(data.services));

	// Metrics are sampled ~once a minute, so a slower refresh than the live
	// container views is plenty and keeps the fan-out of time-series requests light.
	let refreshInterval: ReturnType<typeof setInterval>;
	onMount(() => {
		refreshInterval = setInterval(() => invalidateAll(), 30_000);
	});
	onDestroy(() => clearInterval(refreshInterval));

	function containerHref(s: ServiceMetricsResponse): string {
		return `/containers/${encodeURIComponent(s.hostname)}/${encodeURIComponent(
			s.project_name
		)}/${encodeURIComponent(s.service_name)}`;
	}
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-7xl space-y-10">
		<div>
			<h1 class="text-2xl font-bold">Resources</h1>
			<p class="mt-1 text-sm text-ink-muted">
				CPU, memory, network and disk I/O across your containers (last 7 days).
			</p>
		</div>

		{#if data.error}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				{data.error}
			</div>
		{/if}

		{#if data.services.length === 0 && !data.error}
			<div class="rounded-xl border border-line bg-card px-5 py-4 text-sm text-ink-muted">
				No metrics recorded yet. Metrics are collected by default; make sure an agent is connected
				and that reporting isn't disabled with
				<code class="rounded bg-element px-1 py-0.5 font-mono text-xs">HOISTER_REPORT_METRICS=false</code
				>.
			</div>
		{/if}

		{#each [...grouped] as [hostname, services] (hostname)}
			<section class="space-y-6">
				<h2 class="text-lg font-semibold text-ink-secondary">{hostname}</h2>

				{#each services as service (service.project_name + '/' + service.service_name)}
					<div class="space-y-3">
						<div class="border-b border-line pb-2">
							<a
								href={containerHref(service)}
								class="text-sm font-medium text-ink-muted transition hover:text-ink"
							>
								<span class="text-ink-faint">{service.project_name}</span>
								<span class="px-1 text-ink-ghost">/</span>{service.service_name}
							</a>
						</div>

						{#if service.points.length > 0}
							<ServiceMetricsCharts points={service.points} />
						{:else}
							<p class="text-sm text-ink-faint">No metrics recorded yet for this container.</p>
						{/if}
					</div>
				{/each}
			</section>
		{/each}
	</div>
</div>
