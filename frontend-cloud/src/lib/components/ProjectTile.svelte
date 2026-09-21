<script lang="ts">
	import type { ProjectSummaryResponse } from '../../bindings/ProjectSummaryResponse';
	import {
		HEALTH_BADGE,
		HEALTH_DOT,
		HEALTH_LABEL,
		formatBytes,
		projectHealth,
		serviceHealth,
		timeAgo
	} from '$lib/projectHealth';

	let { project, now }: { project: ProjectSummaryResponse; now: number } = $props();

	/** Service chips shown before collapsing the rest into "+N". */
	const MAX_CHIPS = 8;

	const health = $derived(projectHealth(project, now));
	const running = $derived(project.services.filter((s) => s.status === 'running').length);
	const chips = $derived(project.services.slice(0, MAX_CHIPS));
	const hiddenChips = $derived(project.services.length - chips.length);
	const deployment = $derived(project.latest_deployment);
	const metrics = $derived(project.metrics);

	const SERVICE_DOT = {
		healthy: 'bg-success',
		degraded: 'bg-warning',
		down: 'bg-error'
	} as const;

	const DEPLOYMENT_STATUS: Record<string, { label: string; cls: string }> = {
		Success: { label: 'Deployed', cls: 'text-success' },
		Failed: { label: 'Failed', cls: 'text-error' },
		RollbackFinished: { label: 'Rolled back', cls: 'text-brand-accent' },
		Started: { label: 'Deploying', cls: 'text-ink-secondary' },
		Pending: { label: 'Pending', cls: 'text-ink-secondary' }
	};
</script>

<a
	href="/projects/{project.id}"
	class="group flex flex-col rounded-xl border border-line bg-card p-5 transition hover:border-line-active hover:shadow-sm"
>
	<!-- Header -->
	<div class="flex items-start justify-between gap-3">
		<div class="min-w-0">
			<h3 class="truncate text-base font-semibold text-ink group-hover:text-brand-accent">
				{project.name}
			</h3>
			<p class="truncate text-xs text-ink-faint">{project.hostname}</p>
		</div>
		<div class="flex shrink-0 items-center gap-1.5">
			{#if project.role === 'member'}
				<span
					class="rounded-full border border-line-subtle px-2 py-0.5 text-[10px] font-medium text-ink-muted"
					title="Shared with you by the project owner"
				>
					Shared
				</span>
			{/if}
			<span
				class="inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-xs font-medium {HEALTH_BADGE[
					health
				]}"
			>
				<span class="h-1.5 w-1.5 rounded-full {HEALTH_DOT[health]}"></span>
				{HEALTH_LABEL[health]}
			</span>
		</div>
	</div>

	<!-- Services -->
	<div class="mt-4">
		<p class="mb-1.5 text-[11px] tracking-wider text-ink-faint uppercase">
			Services
			{#if project.services.length > 0}
				<span class="normal-case">· {running}/{project.services.length} running</span>
			{/if}
		</p>
		{#if project.services.length === 0}
			<p class="text-xs text-ink-faint">No container state reported yet.</p>
		{:else}
			<ul class="flex flex-wrap gap-1.5">
				{#each chips as service (service.name)}
					<li
						class="inline-flex max-w-full items-center gap-1.5 rounded-md border border-line bg-canvas px-2 py-0.5 text-xs text-ink-secondary"
						title="{service.name}: {service.status ?? 'unknown'}{service.health
							? ` (${service.health})`
							: ''}"
					>
						<span class="h-1.5 w-1.5 shrink-0 rounded-full {SERVICE_DOT[serviceHealth(service)]}"
						></span>
						<span class="truncate">{service.name}</span>
					</li>
				{/each}
				{#if hiddenChips > 0}
					<li class="px-1 py-0.5 text-xs text-ink-faint">+{hiddenChips} more</li>
				{/if}
			</ul>
		{/if}
	</div>

	<!-- Latest rollout + resources -->
	<dl class="mt-4 grid grid-cols-2 gap-3 border-t border-line pt-4 text-xs">
		<div class="col-span-2 min-w-0">
			<dt class="text-ink-faint">Latest update</dt>
			<dd class="mt-0.5 truncate">
				{#if deployment}
					{@const status = DEPLOYMENT_STATUS[deployment.status] ?? {
						label: deployment.status,
						cls: 'text-ink-secondary'
					}}
					<span class="font-medium {status.cls}">{status.label}</span>
					<span class="text-ink-secondary">{deployment.service_name}</span>
					<span class="font-mono text-ink-faint"
						>{deployment.digest.replace('sha256:', '').slice(0, 12)}</span
					>
					<span class="text-ink-faint">· {timeAgo(deployment.created_at, now)}</span>
				{:else}
					<span class="text-ink-faint">No rollouts yet</span>
				{/if}
			</dd>
		</div>
		<div>
			<dt class="text-ink-faint">CPU</dt>
			<dd class="mt-0.5 font-mono text-ink-secondary">
				{metrics ? `${metrics.cpu_pct.toFixed(1)} %` : '—'}
			</dd>
		</div>
		<div>
			<dt class="text-ink-faint">Memory</dt>
			<dd class="mt-0.5 font-mono text-ink-secondary">
				{#if metrics}
					{formatBytes(metrics.mem_bytes)}
				{:else}
					—
				{/if}
			</dd>
		</div>
	</dl>

	<!-- Footer -->
	<div class="mt-4 flex flex-wrap items-center gap-x-3 gap-y-1 text-xs text-ink-faint">
		<span class={health === 'stale' ? 'text-warning' : ''}>
			{#if project.last_updated}
				{health === 'stale' ? 'Last report' : 'Updated'} {timeAgo(project.last_updated, now)}
			{:else}
				Never reported
			{/if}
		</span>
		{#if project.pending_updates > 0}
			<span
				class="rounded-full border border-warning-border bg-warning-bg px-2 py-0.5 font-medium text-warning"
			>
				{project.pending_updates} update{project.pending_updates === 1 ? '' : 's'} available
			</span>
		{/if}
		<span class="ml-auto flex items-center gap-3">
			<span title="People with access" class="inline-flex items-center gap-1">
				<svg
					class="h-3.5 w-3.5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M17 20h5v-2a4 4 0 00-5.4-3.7M9 20H2v-2a4 4 0 015.4-3.7M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a6 6 0 00-6 6h12a6 6 0 00-6-6z"
					/>
				</svg>
				{project.member_count + 1}
			</span>
			<span title="Project notifiers" class="inline-flex items-center gap-1">
				<svg
					class="h-3.5 w-3.5"
					fill="none"
					viewBox="0 0 24 24"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M15 17h5l-1.4-1.4A2 2 0 0118 14.2V11a6 6 0 00-4-5.7V5a2 2 0 10-4 0v.3A6 6 0 006 11v3.2c0 .5-.2 1-.6 1.4L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
					/>
				</svg>
				{project.notifier_count}
			</span>
		</span>
	</div>
</a>
