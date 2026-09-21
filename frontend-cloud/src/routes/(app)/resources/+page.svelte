<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import ServiceMetricsCharts from '$lib/components/ServiceMetricsCharts.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	// Metrics are sampled ~once a minute, so a slower refresh than the live
	// container views is plenty and keeps the fan-out of time-series requests light.
	let refreshInterval: ReturnType<typeof setInterval>;
	onMount(() => {
		refreshInterval = setInterval(() => invalidateAll(), 30_000);
	});
	onDestroy(() => clearInterval(refreshInterval));
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-7xl space-y-10">
		<div>
			<h1 class="text-2xl font-bold">Resources</h1>
			<p class="mt-1 text-sm text-ink-muted">
				CPU, memory, network and disk I/O across your containers, including projects shared with you
				(last 7 days).
			</p>
		</div>

		{#if data.error}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				{data.error}
			</div>
		{/if}

		{#if data.groups.length === 0 && !data.error}
			<div class="rounded-xl border border-line bg-card px-5 py-4 text-sm text-ink-muted">
				No metrics recorded yet. Metrics are collected by default; make sure an agent is connected
				and that reporting isn't disabled with
				<code class="rounded bg-element px-1 py-0.5 font-mono text-xs"
					>HOISTER_REPORT_METRICS=false</code
				>.
			</div>
		{/if}

		{#each data.groups as group (group.project.id)}
			{@const projectHref = `/projects/${group.project.id}`}
			<section class="space-y-6">
				<h2 class="flex flex-wrap items-baseline gap-2 text-lg font-semibold text-ink-secondary">
					<a href={projectHref} class="hover:text-ink">{group.project.name}</a>
					<span class="text-sm font-normal text-ink-faint">{group.project.hostname}</span>
					{#if group.project.role === 'member'}
						<span
							class="rounded-full border border-line-subtle px-2 py-0.5 text-xs font-medium text-ink-muted"
						>
							Shared with you
						</span>
					{/if}
				</h2>

				{#each group.services as service (service.service_name)}
					<div class="space-y-3">
						<div class="border-b border-line pb-2">
							<a
								href="{projectHref}/services/{encodeURIComponent(service.service_name)}"
								class="text-sm font-medium text-ink-muted transition hover:text-ink"
							>
								{service.service_name}
							</a>
						</div>
						<ServiceMetricsCharts points={service.points} />
					</div>
				{/each}
			</section>
		{/each}
	</div>
</div>
