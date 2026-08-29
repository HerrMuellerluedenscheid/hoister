<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageProps } from './$types';
	import type { AlertEvent, AlertEventKind, AlertMetric, AlertRule } from '$lib/api/alerts';

	let { data, form }: PageProps = $props();

	const METRICS: Array<{ value: AlertMetric; label: string; unit: string }> = [
		{ value: 'cpu_pct', label: 'CPU', unit: '%' },
		{ value: 'mem_pct', label: 'Memory (% of limit)', unit: '%' },
		{ value: 'mem_bytes', label: 'Memory (absolute)', unit: 'MiB' }
	];

	let metric = $state<AlertMetric>('cpu_pct');
	let creating = $state(false);
	let busyId = $state<string | null>(null);

	const unit = $derived(METRICS.find((m) => m.value === metric)?.unit ?? '%');

	function metricLabel(m: AlertMetric): string {
		return METRICS.find((entry) => entry.value === m)?.label ?? m;
	}

	function formatThreshold(rule: AlertRule): string {
		if (rule.metric === 'mem_bytes') {
			return `≥ ${(rule.threshold / (1024 * 1024)).toLocaleString(undefined, { maximumFractionDigits: 1 })} MiB`;
		}
		return `≥ ${rule.threshold}%`;
	}

	function formatDuration(seconds: number): string {
		if (seconds === 0) return '0s';
		const h = Math.floor(seconds / 3600);
		const m = Math.floor((seconds % 3600) / 60);
		const s = seconds % 60;
		return [h ? `${h}h` : '', m ? `${m}m` : '', s ? `${s}s` : ''].filter(Boolean).join(' ');
	}

	function formatScope(rule: AlertRule): string {
		const parts = [rule.hostname, rule.project, rule.service].filter(Boolean);
		return parts.length > 0 ? parts.join(' / ') : 'all containers';
	}

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return isNaN(d.getTime()) ? iso : d.toLocaleString();
	}

	const EVENT_KINDS: Record<AlertEventKind, { label: string; classes: string }> = {
		fired: {
			label: 'Fired',
			classes: 'border-error-border bg-error-bg text-error'
		},
		still_firing: {
			label: 'Still firing',
			classes: 'border-warning-border bg-warning-bg text-warning'
		},
		resolved: {
			label: 'Resolved',
			classes: 'border-success-border bg-success-bg text-success'
		}
	};

	/** A recorded reading or threshold, in the unit its metric is measured in. */
	function formatValue(m: AlertMetric, value: number): string {
		if (m !== 'mem_bytes') return `${value.toFixed(1)}%`;
		const units = ['B', 'KiB', 'MiB', 'GiB', 'TiB'];
		let scaled = Math.max(value, 0);
		let step = 0;
		while (scaled >= 1024 && step < units.length - 1) {
			scaled /= 1024;
			step++;
		}
		return `${scaled.toFixed(step === 0 ? 0 : 1)} ${units[step]}`;
	}

	/** "3 minutes ago" — the absolute timestamp stays in the row's title. */
	function formatRelative(iso: string): string {
		const then = new Date(iso).getTime();
		if (isNaN(then)) return iso;
		const seconds = Math.round((Date.now() - then) / 1000);
		if (seconds < 60) return 'just now';
		const steps: Array<[Intl.RelativeTimeFormatUnit, number]> = [
			['minute', 60],
			['hour', 3600],
			['day', 86400],
			['month', 2592000],
			['year', 31536000]
		];
		const format = new Intl.RelativeTimeFormat(undefined, { numeric: 'auto' });
		let chosen: [Intl.RelativeTimeFormatUnit, number] = steps[0];
		for (const step of steps) {
			if (seconds >= step[1]) chosen = step;
		}
		return format.format(-Math.round(seconds / chosen[1]), chosen[0]);
	}

	function eventScope(event: AlertEvent): string {
		return `${event.hostname} / ${event.project} / ${event.service}`;
	}

	function kindClasses(event: AlertEvent): string {
		return EVENT_KINDS[event.kind].classes;
	}

	/** The rule's condition as it stood when the alert triggered. */
	function eventCondition(event: AlertEvent): string {
		const threshold = `threshold ${formatValue(event.metric, event.threshold)}`;
		if (event.for_seconds === 0) return threshold;
		return `${threshold} over ${formatDuration(event.for_seconds)}`;
	}

	function containerHref(event: AlertEvent): string {
		const path = [event.hostname, event.project, event.service].map(encodeURIComponent);
		return `/containers/${path.join('/')}`;
	}
</script>

<div class="space-y-8 px-4 py-6 sm:px-8 sm:py-10">
	<header>
		<h1 class="text-2xl font-bold">Alerts</h1>
		<p class="mt-1 text-sm text-ink-muted">
			Get notified when a container's CPU or memory stays above a threshold. An alert only fires
			after the threshold has been exceeded for the sustained duration — a single spike stays
			silent — and you get a second notification once it recovers. Delivery uses your configured
			<a href="/notifiers" class="underline hover:text-ink">notifiers</a>.
		</p>
	</header>

	{#if data.error}
		<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
			{data.error}
		</div>
	{/if}

	{#if form?.createError}
		<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
			<span class="font-medium">Create failed:</span>
			{form.createError}
		</div>
	{/if}
	{#if form?.deleteError}
		<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
			<span class="font-medium">Delete failed:</span>
			{form.deleteError}
		</div>
	{/if}
	{#if form?.toggleError}
		<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
			<span class="font-medium">Toggle failed:</span>
			{form.toggleError}
		</div>
	{/if}

	{#if data.historyError}
		<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
			{data.historyError}
		</div>
	{/if}

	<!-- Alert history -->
	<section>
		<div class="mb-3 flex flex-wrap items-center gap-3">
			<h2 class="text-base font-semibold text-ink-code">
				Alert history ({data.history.events.length})
			</h2>
			{#if data.history.new_count > 0}
				<span
					class="rounded-full border border-brand-light bg-brand/10 px-2.5 py-0.5 text-xs font-medium text-brand-accent"
				>
					{data.history.new_count} new since your last login
				</span>
			{/if}
		</div>

		{#if data.history.events.length === 0}
			<div class="rounded-xl border border-line bg-card px-5 py-4 text-sm text-ink-muted">
				Nothing has triggered yet. Alerts show up here as soon as a rule fires, together with the
				host and service it fired on.
			</div>
		{:else}
			<div class="overflow-x-auto rounded-xl border border-line">
				<table class="min-w-full divide-y divide-line text-sm">
					<thead class="bg-card text-xs tracking-wider text-ink-muted uppercase">
						<tr>
							<th class="px-4 py-2 text-left font-medium">Triggered</th>
							<th class="px-4 py-2 text-left font-medium">Status</th>
							<th class="px-4 py-2 text-left font-medium">Reading</th>
							<th class="px-4 py-2 text-left font-medium">System</th>
						</tr>
					</thead>
					<tbody class="divide-y divide-line bg-canvas">
						{#each data.history.events as event (event.id)}
							<tr class="text-ink-secondary {event.is_new ? 'bg-brand/5' : ''}">
								<td
									class="border-l-2 px-4 py-3 whitespace-nowrap {event.is_new
										? 'border-brand-accent'
										: 'border-transparent'}"
								>
									<div class="flex items-center gap-2">
										{#if event.is_new}
											<span
												class="rounded-sm bg-brand-accent px-1.5 py-0.5 text-[10px] font-semibold tracking-wider text-white uppercase"
											>
												New
											</span>
										{/if}
										<span>{formatRelative(event.triggered_at)}</span>
									</div>
									<div class="mt-0.5 text-xs text-ink-faint">{formatDate(event.triggered_at)}</div>
								</td>
								<td class="px-4 py-3">
									<span class="rounded-full border px-2 py-0.5 text-xs font-medium {kindClasses(event)}">
										{EVENT_KINDS[event.kind].label}
									</span>
								</td>
								<td class="px-4 py-3">
									{metricLabel(event.metric)}
									<span class="font-medium text-ink">
										{formatValue(event.metric, event.value)}
									</span>
									<span class="text-ink-faint">({eventCondition(event)})</span>
								</td>
								<td class="px-4 py-3 break-all">
									<a href={containerHref(event)} class="underline hover:text-ink">
										{eventScope(event)}
									</a>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>

	<!-- Create form -->
	<section class="rounded-xl border border-line bg-card p-5">
		<h2 class="mb-3 text-base font-semibold text-ink-code">Add alert rule</h2>
		<form
			method="POST"
			action="?/create"
			use:enhance={() => {
				creating = true;
				return async ({ update }) => {
					await update({ reset: true });
					creating = false;
				};
			}}
			class="space-y-3"
		>
			<div class="grid gap-3 sm:grid-cols-2">
				<div>
					<label for="metric" class="mb-1 block text-xs tracking-wider text-ink-muted uppercase">
						Metric
					</label>
					<select
						id="metric"
						name="metric"
						bind:value={metric}
						class="w-full rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink"
					>
						{#each METRICS as m (m.value)}
							<option value={m.value}>{m.label}</option>
						{/each}
					</select>
				</div>
				<div>
					<label for="threshold" class="mb-1 block text-xs tracking-wider text-ink-muted uppercase">
						Threshold ({unit})
					</label>
					<input
						id="threshold"
						type="number"
						name="threshold"
						required
						min="0"
						step="any"
						placeholder={metric === 'mem_bytes' ? 'e.g. 512' : 'e.g. 80'}
						class="w-full rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
					/>
				</div>
			</div>

			<div class="grid gap-3 sm:grid-cols-2">
				<div>
					<label
						for="for_minutes"
						class="mb-1 block text-xs tracking-wider text-ink-muted uppercase"
					>
						Sustained for (minutes)
					</label>
					<input
						id="for_minutes"
						type="number"
						name="for_minutes"
						required
						min="0"
						step="1"
						value="5"
						class="w-full rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink"
					/>
					<p class="mt-1 text-xs text-ink-faint">
						The threshold must be exceeded for this long before the alert fires (and stay below it
						for the same time to resolve). 0 fires on a single sample.
					</p>
				</div>
				<div>
					<label
						for="cooldown_minutes"
						class="mb-1 block text-xs tracking-wider text-ink-muted uppercase"
					>
						Cooldown (minutes, optional)
					</label>
					<input
						id="cooldown_minutes"
						type="number"
						name="cooldown_minutes"
						min="0"
						step="1"
						placeholder="0 = notify once per episode"
						class="w-full rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
					/>
					<p class="mt-1 text-xs text-ink-faint">
						While the alert keeps firing, remind at most once per this interval.
					</p>
				</div>
			</div>

			<div>
				<p class="mb-1 text-xs tracking-wider text-ink-muted uppercase">Scope (optional)</p>
				<div class="grid gap-3 sm:grid-cols-3">
					<input
						type="text"
						name="hostname"
						placeholder="Host (any)"
						class="rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
					/>
					<input
						type="text"
						name="project"
						placeholder="Project (any)"
						class="rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
					/>
					<input
						type="text"
						name="service"
						placeholder="Service (any)"
						class="rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
					/>
				</div>
				<p class="mt-1 text-xs text-ink-faint">
					Leave empty to watch every container reporting metrics, or narrow the rule to one host,
					project, or service.
				</p>
			</div>

			<div>
				<button
					type="submit"
					disabled={creating}
					class="rounded-md bg-brand-hover px-4 py-2 text-sm font-semibold text-white transition hover:bg-brand-accent disabled:cursor-not-allowed disabled:opacity-50"
				>
					{creating ? 'Saving…' : 'Add alert rule'}
				</button>
			</div>
		</form>
	</section>

	<!-- Existing rules -->
	<section>
		<h2 class="mb-3 text-base font-semibold text-ink-code">
			Your alert rules ({data.rules.length})
		</h2>

		{#if data.rules.length === 0}
			<div class="rounded-xl border border-line bg-card px-5 py-4 text-sm text-ink-muted">
				No alert rules yet. Add one above — it applies to every agent that reports metrics.
			</div>
		{:else}
			<div class="overflow-x-auto rounded-xl border border-line">
				<table class="min-w-full divide-y divide-line text-sm">
					<thead class="bg-card text-xs tracking-wider text-ink-muted uppercase">
						<tr>
							<th class="px-4 py-2 text-left font-medium">Metric</th>
							<th class="px-4 py-2 text-left font-medium">Condition</th>
							<th class="px-4 py-2 text-left font-medium">Scope</th>
							<th class="px-4 py-2 text-left font-medium">Status</th>
							<th class="px-4 py-2 text-left font-medium">Created</th>
							<th class="px-4 py-2"></th>
						</tr>
					</thead>
					<tbody class="divide-y divide-line bg-canvas">
						{#each data.rules as rule (rule.id)}
							<tr class="text-ink-secondary">
								<td class="px-4 py-3 font-mono text-xs">{metricLabel(rule.metric)}</td>
								<td class="px-4 py-3">
									{formatThreshold(rule)} for {formatDuration(rule.for_seconds)}
									{#if rule.cooldown_seconds > 0}
										<span class="text-ink-faint">
											· remind every {formatDuration(rule.cooldown_seconds)}
										</span>
									{/if}
								</td>
								<td class="px-4 py-3 break-all">{formatScope(rule)}</td>
								<td class="px-4 py-3">
									<form
										method="POST"
										action="?/toggle"
										use:enhance={() => {
											busyId = rule.id;
											return async ({ update }) => {
												await update();
												busyId = null;
											};
										}}
									>
										<input type="hidden" name="id" value={rule.id} />
										<input type="hidden" name="enabled" value={(!rule.enabled).toString()} />
										<button
											type="submit"
											disabled={busyId === rule.id}
											class="rounded-md border px-3 py-1 text-xs font-medium transition disabled:opacity-50 {rule.enabled
												? 'border-success-border text-success hover:bg-success-bg'
												: 'border-line-subtle text-ink-muted hover:bg-element'}"
										>
											{rule.enabled ? 'Enabled' : 'Disabled'}
										</button>
									</form>
								</td>
								<td class="px-4 py-3 text-xs text-ink-faint">{formatDate(rule.created_at)}</td>
								<td class="px-4 py-3 text-right">
									<form
										method="POST"
										action="?/delete"
										use:enhance={() => {
											busyId = rule.id;
											return async ({ update }) => {
												await update();
												busyId = null;
											};
										}}
									>
										<input type="hidden" name="id" value={rule.id} />
										<button
											type="submit"
											disabled={busyId === rule.id}
											class="rounded-md border border-error-border px-3 py-1 text-xs font-medium text-error transition hover:bg-error-bg disabled:opacity-50"
										>
											{busyId === rule.id ? 'Working…' : 'Delete'}
										</button>
									</form>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</div>
