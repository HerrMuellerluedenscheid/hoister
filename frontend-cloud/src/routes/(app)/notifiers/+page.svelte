<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageProps } from './$types';
	import type { NotifierKind } from '$lib/api/notifiers';

	let { data, form }: PageProps = $props();

	const ALL_KINDS: Array<{ value: NotifierKind; label: string }> = [
		{ value: 'slack', label: 'Slack' },
		{ value: 'telegram', label: 'Telegram' },
		{ value: 'discord', label: 'Discord' },
		{ value: 'gotify', label: 'Gotify' },
		{ value: 'email', label: 'Email' }
	];
	const allowed = $derived<Set<NotifierKind>>(
		new Set(data.me?.limits.allowed_notifier_kinds ?? ALL_KINDS.map((k) => k.value))
	);
	const isFree = $derived((data.me?.plan ?? 'free') === 'free');

	let kind = $state<NotifierKind>('telegram');
	let creating = $state(false);
	let busyId = $state<number | null>(null);

	const selectedAllowed = $derived(allowed.has(kind));

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return isNaN(d.getTime()) ? iso : d.toLocaleString();
	}

	function summarise(n: (typeof data.notifiers)[number]): string {
		switch (n.config.kind) {
			case 'slack':
				return `${n.config.channel}`;
			case 'telegram':
				return `chat ${n.config.chat_id}`;
			case 'discord':
				return `channel ${n.config.channel_id}`;
			case 'gotify':
				return n.config.server;
			case 'email':
				return n.config.recipient;
		}
	}
</script>

<div class="space-y-8 px-8 py-10">
	<header>
		<h1 class="text-2xl font-bold">Notifiers</h1>
		<p class="mt-1 text-sm text-zinc-400">
			Get pinged when one of your agents reports a deployment, a rollback, or a pending update.
			Configure as many channels as you like — every enabled notifier receives every event.
		</p>
	</header>

	{#if data.error}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			{data.error}
		</div>
	{/if}

	{#if form?.createError}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			<span class="font-medium">Create failed:</span>
			{form.createError}
		</div>
	{/if}
	{#if form?.deleteError}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			<span class="font-medium">Delete failed:</span>
			{form.deleteError}
		</div>
	{/if}
	{#if form?.toggleError}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			<span class="font-medium">Toggle failed:</span>
			{form.toggleError}
		</div>
	{/if}

	<!-- Create form -->
	<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
		<h2 class="mb-3 text-base font-semibold text-zinc-200">Add notifier</h2>
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
			<div>
				<label for="kind" class="mb-1 block text-xs tracking-wider text-zinc-400 uppercase">
					Kind
				</label>
				<select
					id="kind"
					name="kind"
					bind:value={kind}
					class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100"
				>
					{#each ALL_KINDS as k (k.value)}
						<option value={k.value} disabled={!allowed.has(k.value)}>
							{k.label}{allowed.has(k.value) ? '' : ' (Pro)'}
						</option>
					{/each}
				</select>
				{#if !selectedAllowed && isFree}
					<p class="mt-2 text-xs text-amber-400">
						This notifier requires the Pro plan. <a
							href="/settings/plan"
							class="underline hover:text-amber-300">Upgrade to enable.</a
						>
					</p>
				{/if}
			</div>

			{#if kind === 'slack'}
				<div class="grid gap-3 sm:grid-cols-2">
					<input
						type="url"
						name="webhook"
						required
						placeholder="https://hooks.slack.com/services/…"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<input
						type="text"
						name="channel"
						required
						placeholder="#deploys"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
				</div>
			{:else if kind === 'telegram'}
				<div class="grid gap-3 sm:grid-cols-2">
					<input
						type="text"
						name="bot_token"
						required
						placeholder="Bot token"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<input
						type="number"
						name="chat_id"
						required
						min="0"
						placeholder="Chat ID"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
				</div>
			{:else if kind === 'discord'}
				<div class="grid gap-3 sm:grid-cols-2">
					<input
						type="text"
						name="bot_token"
						required
						placeholder="Bot token"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<input
						type="number"
						name="channel_id"
						required
						min="0"
						placeholder="Channel ID"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
				</div>
			{:else if kind === 'gotify'}
				<div class="grid gap-3 sm:grid-cols-2">
					<input
						type="url"
						name="server"
						required
						placeholder="https://gotify.example.com"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<input
						type="text"
						name="token"
						required
						placeholder="Application token"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
				</div>
			{:else if kind === 'email'}
				<div class="grid gap-3 sm:grid-cols-2">
					<input
						type="text"
						name="smtp_server"
						required
						placeholder="smtp.example.com"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<input
						type="email"
						name="recipient"
						required
						placeholder="you@example.com"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<input
						type="text"
						name="smtp_user"
						required
						placeholder="SMTP user"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<input
						type="password"
						name="smtp_password"
						required
						placeholder="SMTP password"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<input
						type="text"
						name="from"
						placeholder="Sender display name (optional)"
						class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500 sm:col-span-2"
					/>
				</div>
			{/if}

			<div>
				<button
					type="submit"
					disabled={creating || !selectedAllowed}
					class="rounded-md bg-indigo-500 px-4 py-2 text-sm font-semibold text-white transition hover:bg-indigo-400 disabled:cursor-not-allowed disabled:opacity-50"
				>
					{creating ? 'Saving…' : 'Add notifier'}
				</button>
			</div>
		</form>
	</section>

	<!-- Existing notifiers -->
	<section>
		<h2 class="mb-3 text-base font-semibold text-zinc-200">
			Your notifiers ({data.notifiers.length})
		</h2>

		{#if data.notifiers.length === 0}
			<div class="rounded-xl border border-zinc-800 bg-zinc-900 px-5 py-4 text-sm text-zinc-400">
				No notifiers yet. Add one above and your next deployment will ping you.
			</div>
		{:else}
			<div class="overflow-hidden rounded-xl border border-zinc-800">
				<table class="min-w-full divide-y divide-zinc-800 text-sm">
					<thead class="bg-zinc-900 text-xs tracking-wider text-zinc-400 uppercase">
						<tr>
							<th class="px-4 py-2 text-left font-medium">Kind</th>
							<th class="px-4 py-2 text-left font-medium">Target</th>
							<th class="px-4 py-2 text-left font-medium">Status</th>
							<th class="px-4 py-2 text-left font-medium">Created</th>
							<th class="px-4 py-2"></th>
						</tr>
					</thead>
					<tbody class="divide-y divide-zinc-800 bg-zinc-950">
						{#each data.notifiers as n (n.id)}
							<tr class="text-zinc-300">
								<td class="px-4 py-3 font-mono text-xs uppercase">{n.kind}</td>
								<td class="px-4 py-3 break-all">{summarise(n)}</td>
								<td class="px-4 py-3">
									<form
										method="POST"
										action="?/toggle"
										use:enhance={() => {
											busyId = n.id;
											return async ({ update }) => {
												await update();
												busyId = null;
											};
										}}
									>
										<input type="hidden" name="id" value={n.id} />
										<input type="hidden" name="enabled" value={(!n.enabled).toString()} />
										<button
											type="submit"
											disabled={busyId === n.id}
											class="rounded-md border px-3 py-1 text-xs font-medium transition disabled:opacity-50 {n.enabled
												? 'border-emerald-500/40 text-emerald-300 hover:bg-emerald-500/15'
												: 'border-zinc-700 text-zinc-400 hover:bg-zinc-800'}"
										>
											{n.enabled ? 'Enabled' : 'Disabled'}
										</button>
									</form>
								</td>
								<td class="px-4 py-3 text-xs text-zinc-500">{formatDate(n.created_at)}</td>
								<td class="px-4 py-3 text-right">
									<form
										method="POST"
										action="?/delete"
										use:enhance={() => {
											busyId = n.id;
											return async ({ update }) => {
												await update();
												busyId = null;
											};
										}}
									>
										<input type="hidden" name="id" value={n.id} />
										<button
											type="submit"
											disabled={busyId === n.id}
											class="rounded-md border border-red-500/40 px-3 py-1 text-xs font-medium text-red-300 transition hover:bg-red-500/15 disabled:opacity-50"
										>
											{busyId === n.id ? 'Working…' : 'Delete'}
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
