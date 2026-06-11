<script lang="ts">
	import Deployments from '$lib/components/Deployments.svelte';
	import PendingUpdates from '$lib/components/PendingUpdates.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	// Busiest containers first so the panel surfaces what's under load.
	const metrics = $derived([...(data.latestMetrics ?? [])].sort((a, b) => b.cpu_pct - a.cpu_pct));

	function formatBytes(bytes: number): string {
		if (bytes <= 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
		return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	function memLabel(used: number, limit: number): string {
		return limit > 0 ? `${formatBytes(used)} / ${formatBytes(limit)}` : formatBytes(used);
	}
</script>

<div class="space-y-10 px-4 py-6 sm:px-8 sm:py-10">
	<h1 class="text-2xl font-bold">Dashboard</h1>

	{#if form?.applyError}
		<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
			<span class="font-medium">Deploy failed:</span>
			{form.applyError}
		</div>
	{/if}

	<PendingUpdates updates={data.pendingUpdates} />

	{#if data.tokenCount === 0}
		<section class="rounded-xl border border-brand-hover/40 bg-brand-hover/10 p-5">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h2 class="text-base font-semibold text-brand-light">No agent connected yet</h2>
					<p class="mt-1 text-sm text-ink-secondary">
						Create your first agent token to start reporting container state to hoister.io.
					</p>
				</div>
				<a
					href="/tokens"
					class="rounded-md bg-brand-hover px-4 py-2 text-sm font-semibold text-white hover:bg-brand-accent"
				>
					Create token
				</a>
			</div>
		</section>
	{/if}

	{#if metrics.length > 0}
		<section>
			<h2 class="mb-4 text-lg font-semibold text-ink-code">Resource usage</h2>
			<div class="overflow-hidden rounded-xl border border-line">
				<table class="w-full text-sm">
					<thead class="bg-card text-xs text-ink-faint">
						<tr>
							<th class="px-4 py-2 text-left font-medium">Container</th>
							<th class="px-4 py-2 text-left font-medium">Host</th>
							<th class="px-4 py-2 text-right font-medium">CPU</th>
							<th class="px-4 py-2 text-right font-medium">Memory</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-line">
						{#each metrics as m}
							<tr class="hover:bg-card/60">
								<td class="px-4 py-2">
									<a
										href="/containers/{encodeURIComponent(m.hostname)}/{encodeURIComponent(
											m.project_name
										)}/{encodeURIComponent(m.service_name)}"
										class="text-brand-accent hover:text-brand-light"
									>
										<span class="text-ink-faint">{m.project_name}</span>
										<span class="px-1 text-ink-ghost">/</span>{m.service_name}
									</a>
								</td>
								<td class="px-4 py-2 text-ink-muted">{m.hostname}</td>
								<td class="px-4 py-2 text-right font-mono">{m.cpu_pct.toFixed(1)}%</td>
								<td class="px-4 py-2 text-right font-mono text-ink-secondary">
									{memLabel(m.mem_bytes, m.mem_limit_bytes)}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</section>
	{/if}

	<section>
		<h2 class="mb-4 text-lg font-semibold text-ink-code">Recent deployments</h2>

		{#if data.deploymentsError}
			<div class="mb-4 rounded-xl border border-error-border bg-error-bg px-4 py-3 text-error">
				{data.deploymentsError}
			</div>
		{/if}

		<Deployments data={data.deployments} />
	</section>
</div>
