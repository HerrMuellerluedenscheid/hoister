<script lang="ts">
	import { onDestroy, onMount } from 'svelte';
	import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';

	const { inspection_data }: { inspection_data: ContainerStateResponse } = $props();

	const inspection = $derived(inspection_data.container_inspections);
	const hoisterEnabled = $derived(inspection.Config?.Labels?.['hoister.enable'] === 'true');
	const hoisterBackupVolumes = $derived(
		inspection.Config?.Labels?.['hoister.backup-volumes'] === 'true'
	);

	let now = $state(Date.now());
	let interval: ReturnType<typeof setInterval>;

	onMount(() => {
		interval = setInterval(() => (now = Date.now()), 1000);
	});

	onDestroy(() => clearInterval(interval));

	const uptime = $derived(getUptime(inspection.State?.StartedAt, now));
	const lastUpdatedAgo = $derived(getTimeAgo(inspection_data.last_updated, now));
	const stale = $derived(isStale(inspection_data.last_updated, now));

	function getUptime(startedAt: string | undefined, nowMs: number): string {
		if (!startedAt) return '—';
		const diffMs = nowMs - new Date(startedAt).getTime();
		if (diffMs < 0) return '—';
		const s = Math.floor(diffMs / 1000);
		const m = Math.floor(s / 60);
		const h = Math.floor(m / 60);
		const d = Math.floor(h / 24);
		if (d > 0) return `${d}d ${h % 24}h`;
		if (h > 0) return `${h}h ${m % 60}m`;
		if (m > 0) return `${m}m ${s % 60}s`;
		return `${s}s`;
	}

	function isStale(dateString: string, nowMs: number): boolean {
		return nowMs - new Date(dateString).getTime() > 60_000;
	}

	function getTimeAgo(dateString: string, nowMs: number): string {
		const diffMs = nowMs - new Date(dateString).getTime();
		const s = Math.floor(diffMs / 1000);
		const m = Math.floor(s / 60);
		const h = Math.floor(m / 60);
		if (h > 0) return `${h}h ${m % 60}m ago`;
		if (m > 0) return `${m}m ${s % 60}s ago`;
		return `${s}s ago`;
	}

	const status = $derived(inspection.State?.Status ?? 'unknown');
	const statusClass = $derived(
		status === 'running'
			? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30'
			: status === 'exited' || status === 'dead'
				? 'bg-red-500/15 text-red-300 border-red-500/30'
				: status === 'paused'
					? 'bg-yellow-500/15 text-yellow-300 border-yellow-500/30'
					: status === 'restarting'
						? 'bg-blue-500/15 text-blue-300 border-blue-500/30'
						: 'bg-zinc-700/40 text-zinc-300 border-zinc-600/40'
	);
</script>

<a
	href="/containers/{inspection_data.hostname}/{inspection_data.project_name}/{inspection_data.service_name}"
	class="block rounded-xl border border-zinc-800 bg-zinc-900 p-5 transition hover:border-zinc-700 hover:bg-zinc-900/80 {stale
		? 'opacity-60'
		: ''}"
>
	<div class="mb-2 flex items-start justify-between gap-3">
		<h3 class="text-base font-semibold break-all text-zinc-100">{inspection_data.service_name}</h3>
		<span class="rounded-full border px-2 py-0.5 text-xs font-medium capitalize {statusClass}">
			{status}
		</span>
	</div>
	<dl class="space-y-1 text-xs text-zinc-400">
		<div class="flex gap-1">
			<dt class="text-zinc-500">Host:</dt>
			<dd class="break-all">{inspection_data.hostname}</dd>
		</div>
		<div class="flex gap-1">
			<dt class="text-zinc-500">Image:</dt>
			<dd class="font-mono break-all">{inspection.Config?.Image ?? '—'}</dd>
		</div>
		<div class="flex gap-1">
			<dt class="text-zinc-500">Uptime:</dt>
			<dd>{uptime}</dd>
		</div>
		<div class="flex gap-1 {stale ? 'text-amber-400' : ''}">
			<dt class="text-zinc-500">{stale ? 'Stale — last update:' : 'Updated:'}</dt>
			<dd>{lastUpdatedAgo}</dd>
		</div>
	</dl>
	<div class="mt-3 flex flex-wrap gap-1.5">
		{#if hoisterEnabled}
			<span
				class="inline-flex items-center rounded-full border border-emerald-500/40 px-2 py-0.5 text-[10px] text-emerald-300"
			>
				Hoister enabled
			</span>
		{/if}
		{#if hoisterBackupVolumes}
			<span
				class="inline-flex items-center rounded-full border border-emerald-500/40 px-2 py-0.5 text-[10px] text-emerald-300"
			>
				Backup volumes
			</span>
		{/if}
	</div>
</a>
