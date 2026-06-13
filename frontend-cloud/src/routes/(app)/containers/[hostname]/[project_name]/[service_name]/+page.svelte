<script lang="ts">
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import Deployments from '$lib/components/Deployments.svelte';
	import PendingUpdates from '$lib/components/PendingUpdates.svelte';
	import RedactedText from '$lib/components/RedactedText.svelte';
	import TimeSeriesChart from '$lib/components/TimeSeriesChart.svelte';
	import type { MetricPointResponse } from '../../../../../../bindings/MetricPointResponse';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

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

	// Network and disk are cumulative byte counters since container start, so a
	// raw plot just ramps upward. Differentiate against the previous sample to
	// recover throughput in bytes/second; the first sample has no predecessor
	// and a counter that goes backwards (container restart) is clamped to 0.
	function rateSeries(
		points: MetricPointResponse[],
		field: (p: MetricPointResponse) => number
	): { t: number; v: number }[] {
		const out: { t: number; v: number }[] = [];
		for (let i = 1; i < points.length; i++) {
			const t = Date.parse(points[i].recorded_at);
			const seconds = (t - Date.parse(points[i - 1].recorded_at)) / 1000;
			if (seconds <= 0) continue;
			const delta = field(points[i]) - field(points[i - 1]);
			out.push({ t, v: delta > 0 ? delta / seconds : 0 });
		}
		return out;
	}

	const netSeries = $derived([
		{ label: 'RX', color: '#34d399', points: rateSeries(metricPoints, (p) => p.net_rx_bytes) },
		{ label: 'TX', color: '#fbbf24', points: rateSeries(metricPoints, (p) => p.net_tx_bytes) }
	]);
	const diskSeries = $derived([
		{ label: 'Read', color: '#38bdf8', points: rateSeries(metricPoints, (p) => p.disk_read_bytes) },
		{
			label: 'Write',
			color: '#f472b6',
			points: rateSeries(metricPoints, (p) => p.disk_write_bytes)
		}
	]);

	function formatBytes(bytes: number): string {
		if (bytes <= 0) return '0 B';
		const units = ['B', 'KB', 'MB', 'GB', 'TB'];
		const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
		return `${(bytes / Math.pow(1024, i)).toFixed(i === 0 ? 0 : 1)} ${units[i]}`;
	}

	function formatRate(bytesPerSecond: number): string {
		return `${formatBytes(bytesPerSecond)}/s`;
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
			? 'bg-success-bg text-success border-success-border'
			: status === 'exited' || status === 'dead'
				? 'bg-error-bg text-error border-error-border'
				: status === 'paused'
					? 'bg-yellow-500/15 text-yellow-300 border-yellow-500/30'
					: status === 'restarting'
						? 'bg-blue-500/15 text-blue-300 border-blue-500/30'
						: 'bg-line-subtle/40 text-ink-secondary border-line-subtle/40';
	}
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-6xl space-y-6">
		{#if data.error || !container}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				{data.error ?? 'Container not found.'}
			</div>
		{:else}
			<!-- Header -->
			<div>
				<h1 class="mb-1 text-2xl font-bold">
					<span class="text-ink-muted">{project_name}</span>
					<span class="px-2 text-ink-ghost">/</span>
					<span>{service_name}</span>
				</h1>
				<p class="text-xs text-ink-faint">Host: {hostname}</p>
				<p class="font-mono text-xs text-ink-faint">{container.Id}</p>
				<p class="text-xs text-ink-ghost">Last updated: {formatDate(last_updated)}</p>
			</div>

			{#if form?.applyError}
				<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
					<span class="font-medium">Deploy failed:</span>
					{form.applyError}
				</div>
			{/if}

			<PendingUpdates updates={data.pendingUpdate} compact />

			{#if stale}
				<div
					class="rounded-xl border border-warning-border bg-warning-bg px-4 py-3 text-sm text-warning"
				>
					<p class="font-semibold">Stale data</p>
					<p>This container has not reported in over a minute.</p>
				</div>
			{/if}

			<!-- Status -->
			<section class="rounded-xl border border-line bg-card p-5">
				<h2 class="mb-4 text-base font-semibold text-ink-code">Status</h2>
				<div class="grid grid-cols-2 gap-4 md:grid-cols-4">
					<div>
						<span class="text-xs text-ink-faint">State</span>
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
						<span class="text-xs text-ink-faint">Exit code</span>
						<p class="mt-1 font-mono text-sm">{container.State?.ExitCode ?? '—'}</p>
					</div>
					<div>
						<span class="text-xs text-ink-faint">Restart count</span>
						<p class="mt-1 font-mono text-sm">{container.RestartCount ?? 0}</p>
					</div>
					<div>
						<span class="text-xs text-ink-faint">OOM killed</span>
						<p class="mt-1 text-sm">{container.State?.OOMKilled ? 'Yes' : 'No'}</p>
					</div>
				</div>
				<div class="mt-4 grid grid-cols-1 gap-4 border-t border-line pt-4 md:grid-cols-2">
					<div>
						<span class="text-xs text-ink-faint">Created</span>
						<p class="mt-1 text-sm">{formatDate(container.Created)}</p>
					</div>
					<div>
						<span class="text-xs text-ink-faint">Started</span>
						<p class="mt-1 text-sm">{formatDate(container.State?.StartedAt)}</p>
					</div>
					<div>
						<span class="text-xs text-ink-faint">Finished</span>
						<p class="mt-1 text-sm">{formatDate(container.State?.FinishedAt)}</p>
					</div>
					<div>
						<span class="text-xs text-ink-faint">PID</span>
						<p class="mt-1 font-mono text-sm">{container.State?.Pid ?? '—'}</p>
					</div>
				</div>
			</section>

			<!-- Resource usage -->
			<section class="rounded-xl border border-line bg-card p-5">
				<h2 class="mb-1 text-base font-semibold text-ink-code">Resource usage (last 7 days)</h2>
				{#if metricPoints.length > 0}
					<p class="mb-4 text-xs text-ink-faint">Sampled roughly once a minute by the agent.</p>
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
						<TimeSeriesChart series={netSeries} label="Network" formatValue={formatRate} />
						<TimeSeriesChart series={diskSeries} label="Disk I/O" formatValue={formatRate} />
					</div>
				{:else}
					<p class="mt-2 text-sm text-ink-faint">
						No metrics recorded yet. Metrics are collected by default; make sure the agent can reach
						the controller and that this container isn't disabled with <code
							class="rounded bg-element px-1 py-0.5 font-mono">HOISTER_REPORT_METRICS=false</code
						>.
					</p>
				{/if}
			</section>

			<!-- Exit reason -->
			{#if container.State?.Error || (container.State?.Status && container.State.Status !== 'running' && container.State.Status !== 'created')}
				<section class="rounded-xl border border-error-border bg-error-bg p-5">
					<h2 class="mb-3 text-base font-semibold text-error">Exit reason</h2>
					{#if container.State.Error}
						<div class="mb-3">
							<span class="text-xs text-error">Docker error</span>
							<p class="mt-1 font-mono text-sm break-all text-error">{container.State.Error}</p>
						</div>
					{/if}
					<div class="grid grid-cols-1 gap-3 text-sm md:grid-cols-3">
						<div>
							<span class="text-xs text-error">Exit code</span>
							<p class="font-mono">{container.State.ExitCode ?? '—'}</p>
						</div>
						<div>
							<span class="text-xs text-error">OOM killed</span>
							<p>{container.State.OOMKilled ? 'Yes' : 'No'}</p>
						</div>
						<div>
							<span class="text-xs text-error">Finished at</span>
							<p>{formatDate(container.State.FinishedAt)}</p>
						</div>
					</div>
					{#if !container.State.Error && container.State.ExitCode !== 0}
						<p class="mt-3 text-xs text-error">
							Docker did not report a startup error, so the container process exited on its own.
							Enable <code class="rounded bg-element px-1 py-0.5 font-mono"
								>HOISTER_REPORT_LOGS=true</code
							>
							on the agent to forward log tails for crashed containers.
						</p>
					{/if}
				</section>
			{/if}

			<!-- Container logs (only present if agent has HOISTER_REPORT_LOGS=true and the container is in a non-running state) -->
			{#if last_logs}
				<section class="rounded-xl border border-line bg-card p-5">
					<h2 class="mb-1 text-base font-semibold text-ink-code">Container logs (tail)</h2>
					<p class="mb-3 text-xs text-ink-faint">
						Last lines captured by the agent because the container is not running. Secrets matching
						known sensitive env-var values are redacted.
					</p>
					<pre
						class="max-h-96 overflow-auto rounded-lg bg-black p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap text-ink-code"><RedactedText
							text={last_logs}
						/></pre>
				</section>
			{/if}

			<!-- Health -->
			{#if container.State?.Health}
				<section class="rounded-xl border border-line bg-card p-5">
					<h2 class="mb-2 text-base font-semibold text-ink-code">Health check</h2>
					<div class="mb-4 flex items-center gap-3 text-sm">
						<span
							class="inline-flex rounded-full border px-2 py-0.5 text-xs font-medium {container
								.State.Health.Status === 'healthy'
								? 'border-success-border bg-success-bg text-success'
								: container.State.Health.Status === 'unhealthy'
									? 'border-error-border bg-error-bg text-error'
									: 'border-line-subtle/40 bg-line-subtle/40 text-ink-secondary'}"
						>
							{container.State.Health.Status}
						</span>
						<span class="text-xs text-ink-faint"
							>Failing streak:
							<span class="font-mono">{container.State.Health.FailingStreak ?? 0}</span></span
						>
					</div>
					{#if container.State.Health.Log && container.State.Health.Log.length > 0}
						<p class="text-xs text-ink-faint">
							Last checked: <span class="text-ink-secondary"
								>{formatDate(
									container.State.Health.Log[container.State.Health.Log.length - 1].End
								)}</span
							>
						</p>
					{/if}
				</section>
			{/if}

			<!-- Recent deployments -->
			<section class="rounded-xl border border-line bg-card p-5">
				<h2 class="mb-3 text-base font-semibold text-ink-code">Recent deployments</h2>
				<div class="max-h-96 overflow-y-auto">
					<Deployments data={deployments} />
				</div>
			</section>

			<!-- Configuration -->
			<section class="rounded-xl border border-line bg-card p-5">
				<h2 class="mb-3 text-base font-semibold text-ink-code">Configuration</h2>
				<dl class="space-y-3 text-sm">
					<div>
						<dt class="text-xs text-ink-faint">Image</dt>
						<dd class="mt-1 font-mono break-all text-ink-code">{container.Config?.Image ?? '—'}</dd>
					</div>
					<div>
						<dt class="text-xs text-ink-faint">Hostname</dt>
						<dd class="mt-1 font-mono text-ink-code">{container.Config?.Hostname ?? '—'}</dd>
					</div>
					<div>
						<dt class="text-xs text-ink-faint">Working directory</dt>
						<dd class="mt-1 font-mono text-ink-code">{container.Config?.WorkingDir || '—'}</dd>
					</div>
					<div>
						<dt class="text-xs text-ink-faint">Command</dt>
						<dd
							class="mt-1 rounded-lg bg-canvas p-3 font-mono text-xs whitespace-pre-wrap text-ink-code"
						>
							{container.Config?.Cmd ? container.Config.Cmd.join(' ') : '—'}
						</dd>
					</div>
				</dl>
			</section>

			<!-- Environment Variables -->
			{#if container.Config?.Env && container.Config.Env.length > 0}
				<section class="rounded-xl border border-line bg-card p-5">
					<h2 class="mb-3 text-base font-semibold text-ink-code">Environment variables</h2>
					<div class="divide-y divide-line">
						{#each container.Config.Env as env}
							{#if env.includes('=')}
								{@const [key, ...valueParts] = env.split('=')}
								{@const value = valueParts.join('=')}
								<div class="flex flex-col gap-0.5 py-2 sm:flex-row sm:items-start sm:gap-3">
									<span class="font-mono text-xs break-all text-ink-faint sm:w-64 sm:flex-shrink-0"
										>{key}</span
									>
									<span class="font-mono text-xs break-all text-ink-code"
										><RedactedText text={value} /></span
									>
								</div>
							{/if}
						{/each}
					</div>
				</section>
			{/if}

			<!-- Network -->
			{#if container.NetworkSettings?.Networks}
				<section class="rounded-xl border border-line bg-card p-5">
					<h2 class="mb-3 text-base font-semibold text-ink-code">Networks</h2>
					<div class="space-y-3">
						{#each Object.entries(container.NetworkSettings.Networks) as [networkName, network]}
							{@const net = network as {
								IPAddress?: string;
								Gateway?: string;
								MacAddress?: string;
							}}
							<div class="rounded-lg border border-line p-3">
								<a
									href="/networks/{encodeURIComponent(hostname ?? '')}/{encodeURIComponent(
										networkName
									)}"
									class="mb-2 inline-flex items-center gap-1 text-sm font-medium text-brand-light hover:text-brand-light hover:underline"
									title="View services on this network"
								>
									{networkName}
									<span aria-hidden="true">→</span>
								</a>
								<div class="grid grid-cols-1 gap-3 text-xs md:grid-cols-3">
									<div>
										<span class="text-ink-faint">IP</span>
										<p class="font-mono text-ink-code">{net.IPAddress || '—'}</p>
									</div>
									<div>
										<span class="text-ink-faint">Gateway</span>
										<p class="font-mono text-ink-code">{net.Gateway || '—'}</p>
									</div>
									<div>
										<span class="text-ink-faint">MAC</span>
										<p class="font-mono text-ink-code">{net.MacAddress || '—'}</p>
									</div>
								</div>
							</div>
						{/each}
					</div>
				</section>
			{/if}

			<!-- Mounts -->
			{#if container.Mounts && container.Mounts.length > 0}
				<section class="rounded-xl border border-line bg-card p-5">
					<h2 class="mb-3 text-base font-semibold text-ink-code">Mounts</h2>
					<div class="space-y-2">
						{#each container.Mounts as mount}
							<div class="rounded-lg border border-line p-3">
								<div class="mb-2 flex items-center justify-between text-xs">
									<span
										class="inline-flex rounded-full border border-brand-hover/40 bg-brand-hover/15 px-2 py-0.5 text-brand-light"
									>
										{mount.Type}
									</span>
									<span class="text-ink-faint">{mount.RW ? 'Read/Write' : 'Read-Only'}</span>
								</div>
								<dl class="space-y-1 text-xs">
									<div class="flex gap-2">
										<dt class="w-20 text-ink-faint">Source:</dt>
										<dd class="font-mono break-all text-ink-code">{mount.Source}</dd>
									</div>
									<div class="flex gap-2">
										<dt class="w-20 text-ink-faint">Destination:</dt>
										<dd class="font-mono text-ink-code">{mount.Destination}</dd>
									</div>
								</dl>
							</div>
						{/each}
					</div>
				</section>
			{/if}

			<!-- Labels -->
			{#if container.Config?.Labels && Object.keys(container.Config.Labels).length > 0}
				<section class="rounded-xl border border-line bg-card p-5">
					<h2 class="mb-3 text-base font-semibold text-ink-code">Labels</h2>
					<div class="divide-y divide-line">
						{#each Object.entries(container.Config.Labels) as [key, value]}
							<div class="flex flex-col gap-0.5 py-2 sm:flex-row sm:items-start sm:gap-3">
								<span class="font-mono text-xs break-all text-ink-faint sm:w-80 sm:flex-shrink-0"
									>{key}</span
								>
								<span class="font-mono text-xs break-all text-ink-code">{value}</span>
							</div>
						{/each}
					</div>
				</section>
			{/if}

			<!-- Host config summary -->
			<section class="rounded-xl border border-line bg-card p-5">
				<h2 class="mb-3 text-base font-semibold text-ink-code">Host configuration</h2>
				<div class="grid grid-cols-1 gap-4 md:grid-cols-3">
					<div>
						<span class="text-xs text-ink-faint">Memory limit</span>
						<p class="mt-1 font-mono text-sm text-ink-code">
							{container.HostConfig?.Memory === 0
								? 'Unlimited'
								: `${container.HostConfig?.Memory} bytes`}
						</p>
					</div>
					<div>
						<span class="text-xs text-ink-faint">CPU shares</span>
						<p class="mt-1 font-mono text-sm text-ink-code">
							{container.HostConfig?.CpuShares || 'Default'}
						</p>
					</div>
					<div>
						<span class="text-xs text-ink-faint">Restart policy</span>
						<p class="mt-1 text-sm text-ink-code">
							{container.HostConfig?.RestartPolicy?.Name ?? '—'}
						</p>
					</div>
				</div>
			</section>
		{/if}
	</div>
</div>
