<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import Deployments from '$lib/components/Deployments.svelte';
	import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();

	const container = $derived(data.inspections?.container_inspections);
	const deployments = $derived(data.deployments.slice(0, 8));

	const metricPoints = $derived(data.metrics?.points ?? []);
	const cpuSeries = $derived(
		metricPoints.map((p) => ({ t: Date.parse(p.recorded_at), v: p.cpu_pct }))
	);
	const memSeries = $derived(
		metricPoints.map((p) => ({ t: Date.parse(p.recorded_at), v: p.mem_bytes }))
	);
	// Cap the memory chart at the container's limit when one is set, so the
	// graph reflects headroom rather than auto-scaling to the peak.
	const memLimit = $derived(
		metricPoints.length > 0 ? metricPoints[metricPoints.length - 1].mem_limit_bytes : 0
	);

	function formatBytes(bytes: number): string {
		if (bytes <= 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
		return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	const hostname = $derived(data.inspections?.hostname);
	const service_name = $derived(data.inspections?.service_name);
	const project_name = $derived(data.inspections?.project_name);
	const last_updated = $derived(data.inspections?.last_updated);
	const last_logs = $derived(data.inspections?.last_logs);

	let now = $state(Date.now());
	let refreshInterval: ReturnType<typeof setInterval>;

	const stale = $derived(last_updated ? now - new Date(last_updated).getTime() > 60_000 : false);

	onMount(() => {
		refreshInterval = setInterval(() => {
			now = Date.now();
			invalidateAll();
		}, 10_000);
	});

	onDestroy(() => clearInterval(refreshInterval));

	function formatDate(s: string | undefined): string {
		if (!s) return '—';
		const d = new Date(s);
		if (isNaN(d.getTime())) return '—';
		return d.toLocaleString();
	}

	function statusClass(status: string | undefined): string {
		return status === 'running'
			? 'bg-emerald-500/15 text-emerald-300 border-emerald-500/30'
			: status === 'exited' || status === 'dead'
				? 'bg-red-500/15 text-red-300 border-red-500/30'
				: status === 'paused'
					? 'bg-yellow-500/15 text-yellow-300 border-yellow-500/30'
					: status === 'restarting'
						? 'bg-blue-500/15 text-blue-300 border-blue-500/30'
						: 'bg-zinc-700/40 text-zinc-300 border-zinc-600/40';
	}
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-6xl space-y-6">
		{#if data.error || !container}
			<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
				{data.error ?? 'Container not found.'}
			</div>
		{:else}
			<!-- Header -->
			<div>
				<h1 class="mb-1 text-2xl font-bold">
					<span class="text-zinc-400">{project_name}</span>
					<span class="px-2 text-zinc-600">/</span>
					<span>{service_name}</span>
				</h1>
				<p class="text-xs text-zinc-500">Host: {hostname}</p>
				<p class="font-mono text-xs text-zinc-500">{container.Id}</p>
				<p class="text-xs text-zinc-600">Last updated: {formatDate(last_updated)}</p>
			</div>

			{#if stale}
				<div
					class="rounded-xl border border-amber-700 bg-amber-950/40 px-4 py-3 text-sm text-amber-300"
				>
					<p class="font-semibold">Stale data</p>
					<p>This container has not reported in over a minute.</p>
				</div>
			{/if}

			<!-- Status -->
			<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
				<h2 class="mb-4 text-base font-semibold text-zinc-200">Status</h2>
				<div class="grid grid-cols-2 gap-4 md:grid-cols-4">
					<div>
						<span class="text-xs text-zinc-500">State</span>
						<p class="mt-1">
							<span
								class="inline-flex rounded-full border px-2 py-0.5 text-xs font-medium capitalize {statusClass(
									container.State?.Status
								)}"
							>
								{container.State?.Status ?? 'unknown'}
							</span>
						</p>
					</div>
					<div>
						<span class="text-xs text-zinc-500">Exit code</span>
						<p class="mt-1 font-mono text-sm">{container.State?.ExitCode ?? '—'}</p>
					</div>
					<div>
						<span class="text-xs text-zinc-500">Restart count</span>
						<p class="mt-1 font-mono text-sm">{container.RestartCount ?? 0}</p>
					</div>
					<div>
						<span class="text-xs text-zinc-500">OOM killed</span>
						<p class="mt-1 text-sm">{container.State?.OOMKilled ? 'Yes' : 'No'}</p>
					</div>
				</div>
				<div class="mt-4 grid grid-cols-1 gap-4 border-t border-zinc-800 pt-4 md:grid-cols-2">
					<div>
						<span class="text-xs text-zinc-500">Created</span>
						<p class="mt-1 text-sm">{formatDate(container.Created)}</p>
					</div>
					<div>
						<span class="text-xs text-zinc-500">Started</span>
						<p class="mt-1 text-sm">{formatDate(container.State?.StartedAt)}</p>
					</div>
					<div>
						<span class="text-xs text-zinc-500">Finished</span>
						<p class="mt-1 text-sm">{formatDate(container.State?.FinishedAt)}</p>
					</div>
					<div>
						<span class="text-xs text-zinc-500">PID</span>
						<p class="mt-1 font-mono text-sm">{container.State?.Pid ?? '—'}</p>
					</div>
				</div>
			</section>

			<!-- Resource usage -->
			<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
				<h2 class="mb-1 text-base font-semibold text-zinc-200">Resource usage (last 7 days)</h2>
				{#if metricPoints.length > 0}
					<p class="mb-4 text-xs text-zinc-500">Sampled roughly once a minute by the agent.</p>
					<div class="grid grid-cols-1 gap-4 lg:grid-cols-2">
						<TimeSeriesChart
							points={cpuSeries}
							label="CPU"
							color="#818cf8"
							formatValue={(v) => `${v.toFixed(1)}%`}
						/>
						<TimeSeriesChart
							points={memSeries}
							label="Memory"
							color="#34d399"
							formatValue={formatBytes}
							max={memLimit > 0 ? memLimit : undefined}
						/>
					</div>
				{:else}
					<p class="mt-2 text-sm text-zinc-500">
						No metrics recorded yet. Enable <code class="rounded bg-zinc-800 px-1 py-0.5 font-mono"
							>HOISTER_REPORT_METRICS=true</code
						> on the agent to collect CPU and memory usage over time.
					</p>
				{/if}
			</section>

			<!-- Exit reason -->
			{#if container.State?.Error || (container.State?.Status && container.State.Status !== 'running' && container.State.Status !== 'created')}
				<section class="rounded-xl border border-red-700 bg-red-950/40 p-5">
					<h2 class="mb-3 text-base font-semibold text-red-300">Exit reason</h2>
					{#if container.State.Error}
						<div class="mb-3">
							<span class="text-xs text-red-400">Docker error</span>
							<p class="mt-1 font-mono text-sm break-all text-red-200">{container.State.Error}</p>
						</div>
					{/if}
					<div class="grid grid-cols-1 gap-3 text-sm md:grid-cols-3">
						<div>
							<span class="text-xs text-red-400">Exit code</span>
							<p class="font-mono">{container.State.ExitCode ?? '—'}</p>
						</div>
						<div>
							<span class="text-xs text-red-400">OOM killed</span>
							<p>{container.State.OOMKilled ? 'Yes' : 'No'}</p>
						</div>
						<div>
							<span class="text-xs text-red-400">Finished at</span>
							<p>{formatDate(container.State.FinishedAt)}</p>
						</div>
					</div>
					{#if !container.State.Error && container.State.ExitCode !== 0}
						<p class="mt-3 text-xs text-red-400">
							Docker did not report a startup error, so the container process exited on its own.
							Enable <code class="rounded bg-zinc-800 px-1 py-0.5 font-mono"
								>HOISTER_REPORT_LOGS=true</code
							>
							on the agent to forward log tails for crashed containers.
						</p>
					{/if}
				</section>
			{/if}

			<!-- Container logs (only present if agent has HOISTER_REPORT_LOGS=true and the container is in a non-running state) -->
			{#if last_logs}
				<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
					<h2 class="mb-1 text-base font-semibold text-zinc-200">Container logs (tail)</h2>
					<p class="mb-3 text-xs text-zinc-500">
						Last lines captured by the agent because the container is not running. Secrets matching
						known sensitive env-var values are redacted.
					</p>
					<pre
						class="max-h-96 overflow-auto rounded-lg bg-black p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap text-zinc-200">{last_logs}</pre>
				</section>
			{/if}

			<!-- Health -->
			{#if container.State?.Health}
				<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
					<h2 class="mb-2 text-base font-semibold text-zinc-200">Health check</h2>
					<div class="mb-4 flex items-center gap-3 text-sm">
						<span
							class="inline-flex rounded-full border px-2 py-0.5 text-xs font-medium {container
								.State.Health.Status === 'healthy'
								? 'border-emerald-500/40 bg-emerald-500/15 text-emerald-300'
								: container.State.Health.Status === 'unhealthy'
									? 'border-red-500/40 bg-red-500/15 text-red-300'
									: 'border-zinc-600/40 bg-zinc-700/40 text-zinc-300'}"
						>
							{container.State.Health.Status}
						</span>
						<span class="text-xs text-zinc-500"
							>Failing streak:
							<span class="font-mono">{container.State.Health.FailingStreak ?? 0}</span></span
						>
					</div>
					{#if container.State.Health.Log && container.State.Health.Log.length > 0}
						<h3 class="mb-2 text-xs font-medium text-zinc-400">Recent probes</h3>
						<div class="space-y-2">
							{#each container.State.Health.Log.slice(-3).reverse() as probe}
								<div
									class="rounded-lg border p-3 {probe.ExitCode === 0
										? 'border-emerald-500/30 bg-emerald-500/5'
										: 'border-red-500/30 bg-red-500/5'}"
								>
									<div class="mb-1 flex justify-between text-xs text-zinc-500">
										<span>{formatDate(probe.End)}</span>
										<span>exit {probe.ExitCode}</span>
									</div>
									{#if probe.Output}
										<pre
											class="overflow-x-auto rounded bg-black/40 p-2 font-mono text-xs whitespace-pre-wrap text-zinc-200">{probe.Output}</pre>
									{/if}
								</div>
							{/each}
						</div>
					{/if}
				</section>
			{/if}

			<!-- Recent deployments -->
			<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
				<h2 class="mb-3 text-base font-semibold text-zinc-200">Recent deployments</h2>
				<Deployments data={deployments} />
			</section>

			<!-- Configuration -->
			<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
				<h2 class="mb-3 text-base font-semibold text-zinc-200">Configuration</h2>
				<dl class="space-y-3 text-sm">
					<div>
						<dt class="text-xs text-zinc-500">Image</dt>
						<dd class="mt-1 font-mono break-all text-zinc-200">{container.Config?.Image ?? '—'}</dd>
					</div>
					<div>
						<dt class="text-xs text-zinc-500">Hostname</dt>
						<dd class="mt-1 font-mono text-zinc-200">{container.Config?.Hostname ?? '—'}</dd>
					</div>
					<div>
						<dt class="text-xs text-zinc-500">Working directory</dt>
						<dd class="mt-1 font-mono text-zinc-200">{container.Config?.WorkingDir || '—'}</dd>
					</div>
					<div>
						<dt class="text-xs text-zinc-500">Command</dt>
						<dd
							class="mt-1 rounded-lg bg-zinc-950 p-3 font-mono text-xs whitespace-pre-wrap text-zinc-200"
						>
							{container.Config?.Cmd ? container.Config.Cmd.join(' ') : '—'}
						</dd>
					</div>
				</dl>
			</section>

			<!-- Environment Variables -->
			{#if container.Config?.Env && container.Config.Env.length > 0}
				<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
					<h2 class="mb-3 text-base font-semibold text-zinc-200">Environment variables</h2>
					<div class="divide-y divide-zinc-800">
						{#each container.Config.Env as env}
							{#if env.includes('=')}
								{@const [key, ...valueParts] = env.split('=')}
								{@const value = valueParts.join('=')}
								<div class="flex flex-col gap-0.5 py-2 sm:flex-row sm:items-start sm:gap-3">
									<span class="font-mono text-xs break-all text-zinc-500 sm:w-64 sm:flex-shrink-0"
										>{key}</span
									>
									<span class="font-mono text-xs break-all text-zinc-200">{value}</span>
								</div>
							{/if}
						{/each}
					</div>
				</section>
			{/if}

			<!-- Network -->
			{#if container.NetworkSettings?.Networks}
				<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
					<h2 class="mb-3 text-base font-semibold text-zinc-200">Networks</h2>
					<div class="space-y-3">
						{#each Object.entries(container.NetworkSettings.Networks) as [networkName, network]}
							{@const net = network as {
								IPAddress?: string;
								Gateway?: string;
								MacAddress?: string;
							}}
							<div class="rounded-lg border border-zinc-800 p-3">
								<h3 class="mb-2 text-sm font-medium text-zinc-300">{networkName}</h3>
								<div class="grid grid-cols-1 gap-3 text-xs md:grid-cols-3">
									<div>
										<span class="text-zinc-500">IP</span>
										<p class="font-mono text-zinc-200">{net.IPAddress || '—'}</p>
									</div>
									<div>
										<span class="text-zinc-500">Gateway</span>
										<p class="font-mono text-zinc-200">{net.Gateway || '—'}</p>
									</div>
									<div>
										<span class="text-zinc-500">MAC</span>
										<p class="font-mono text-zinc-200">{net.MacAddress || '—'}</p>
									</div>
								</div>
							</div>
						{/each}
					</div>
				</section>
			{/if}

			<!-- Mounts -->
			{#if container.Mounts && container.Mounts.length > 0}
				<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
					<h2 class="mb-3 text-base font-semibold text-zinc-200">Mounts</h2>
					<div class="space-y-2">
						{#each container.Mounts as mount}
							<div class="rounded-lg border border-zinc-800 p-3">
								<div class="mb-2 flex items-center justify-between text-xs">
									<span
										class="inline-flex rounded-full border border-indigo-500/40 bg-indigo-500/15 px-2 py-0.5 text-indigo-300"
									>
										{mount.Type}
									</span>
									<span class="text-zinc-500">{mount.RW ? 'Read/Write' : 'Read-Only'}</span>
								</div>
								<dl class="space-y-1 text-xs">
									<div class="flex gap-2">
										<dt class="w-20 text-zinc-500">Source:</dt>
										<dd class="font-mono break-all text-zinc-200">{mount.Source}</dd>
									</div>
									<div class="flex gap-2">
										<dt class="w-20 text-zinc-500">Destination:</dt>
										<dd class="font-mono text-zinc-200">{mount.Destination}</dd>
									</div>
								</dl>
							</div>
						{/each}
					</div>
				</section>
			{/if}

			<!-- Labels -->
			{#if container.Config?.Labels && Object.keys(container.Config.Labels).length > 0}
				<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
					<h2 class="mb-3 text-base font-semibold text-zinc-200">Labels</h2>
					<div class="divide-y divide-zinc-800">
						{#each Object.entries(container.Config.Labels) as [key, value]}
							<div class="flex flex-col gap-0.5 py-2 sm:flex-row sm:items-start sm:gap-3">
								<span class="font-mono text-xs break-all text-zinc-500 sm:w-80 sm:flex-shrink-0"
									>{key}</span
								>
								<span class="font-mono text-xs break-all text-zinc-200">{value}</span>
							</div>
						{/each}
					</div>
				</section>
			{/if}

			<!-- Host config summary -->
			<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
				<h2 class="mb-3 text-base font-semibold text-zinc-200">Host configuration</h2>
				<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
					<div>
						<span class="text-xs text-zinc-500">Memory limit</span>
						<p class="mt-1 font-mono text-sm text-zinc-200">
							{container.HostConfig?.Memory === 0
								? 'Unlimited'
								: `${container.HostConfig?.Memory} bytes`}
						</p>
					</div>
					<div>
						<span class="text-xs text-zinc-500">CPU shares</span>
						<p class="mt-1 font-mono text-sm text-zinc-200">
							{container.HostConfig?.CpuShares || 'Default'}
						</p>
					</div>
					<div>
						<span class="text-xs text-zinc-500">Restart policy</span>
						<p class="mt-1 text-sm text-zinc-200">
							{container.HostConfig?.RestartPolicy?.Name ?? '—'}
						</p>
					</div>
				</div>
			</section>
		{/if}
	</div>
</div>
