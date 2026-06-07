<script lang="ts">
	import { enhance } from '$app/forms';
	import { page } from '$app/state';
	import { env } from '$env/dynamic/public';
	import type { PageProps } from './$types';
	import type { NotifierKind } from '$lib/api/notifiers';

	let { data, form }: PageProps = $props();

	// Slack is added via OAuth ("Add to Slack"), not the manual form below.
	const ALL_KINDS: Array<{ value: NotifierKind; label: string }> = [
		{ value: 'telegram', label: 'Telegram' },
		{ value: 'discord', label: 'Discord (bot)' },
		{ value: 'discord_webhook', label: 'Discord (webhook)' },
		{ value: 'teams', label: 'Microsoft Teams' },
		{ value: 'gotify', label: 'Gotify' },
		{ value: 'email', label: 'Email' },
		{ value: 'ntfy', label: 'ntfy' },
		{ value: 'pushover', label: 'Pushover' },
		{ value: 'matrix', label: 'Matrix' },
		{ value: 'mattermost', label: 'Mattermost' },
		{ value: 'rocketchat', label: 'Rocket.Chat' },
		{ value: 'google_chat', label: 'Google Chat' },
		{ value: 'webhook', label: 'Webhook' }
	];
	const allowed = $derived<Set<NotifierKind>>(
		new Set(data.me?.limits.allowed_notifier_kinds ?? ['slack', ...ALL_KINDS.map((k) => k.value)])
	);
	const isFree = $derived((data.me?.plan ?? 'free') === 'free');

	const slackEnabled = Boolean(env.PUBLIC_SLACK_CLIENT_ID);
	const slackAllowed = $derived(allowed.has('slack'));
	const slackStatus = $derived(page.url.searchParams.get('slack'));

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
				return n.config.channel;
			case 'telegram':
				return `chat ${n.config.chat_id}`;
			case 'discord':
				return `channel ${n.config.channel_id}`;
			case 'discord_webhook':
				return n.config.webhook_set ? 'webhook' : '—';
			case 'teams':
				return n.config.webhook_set ? 'webhook' : '—';
			case 'gotify':
				return n.config.server_host;
			case 'email':
				return n.config.recipient;
			case 'ntfy':
				return `${n.config.topic} @ ${n.config.server_host}`;
			case 'pushover':
				return n.config.device ? `device ${n.config.device}` : 'pushover';
			case 'matrix':
				return `${n.config.room_id} @ ${n.config.homeserver_host}`;
			case 'mattermost':
				return n.config.channel
					? `${n.config.channel} @ ${n.config.webhook_host}`
					: n.config.webhook_host;
			case 'rocketchat':
				return n.config.channel
					? `${n.config.channel} @ ${n.config.webhook_host}`
					: n.config.webhook_host;
			case 'google_chat':
				return n.config.webhook_host;
			case 'webhook':
				return n.config.url_host;
		}
	}
</script>

<div class="space-y-8 px-4 py-6 sm:px-8 sm:py-10">
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
	{#if form?.testError}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			<span class="font-medium">Test failed:</span>
			{form.testError}
		</div>
	{:else if form?.testedId}
		<div
			class="rounded-xl border border-emerald-800 bg-emerald-950/30 px-4 py-3 text-sm text-emerald-300"
		>
			Test message sent. Check the channel — if it didn't arrive, the credentials may still be wrong
			even though the dispatcher accepted them.
		</div>
	{/if}

	{#if slackStatus === 'connected'}
		<div
			class="rounded-xl border border-emerald-800 bg-emerald-950/30 px-4 py-3 text-sm text-emerald-300"
		>
			Slack connected — deployment events will post to the channel you chose.
		</div>
	{:else if slackStatus === 'denied'}
		<div
			class="rounded-xl border border-amber-800 bg-amber-950/30 px-4 py-3 text-sm text-amber-300"
		>
			Slack authorization was cancelled.
		</div>
	{:else if slackStatus === 'upgrade'}
		<div
			class="rounded-xl border border-amber-800 bg-amber-950/30 px-4 py-3 text-sm text-amber-300"
		>
			Slack notifiers require the Pro plan.
			<a href="/settings/plan" class="underline hover:text-amber-200">Upgrade to enable.</a>
		</div>
	{:else if slackStatus === 'error'}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			Couldn't connect Slack. Please try again.
		</div>
	{/if}

	{#if slackEnabled}
		<!-- Slack: installed via OAuth, not the manual form below. -->
		<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
			<h2 class="mb-1 text-base font-semibold text-zinc-200">Slack</h2>
			<p class="mb-3 text-sm text-zinc-400">
				Install Hoister into a Slack channel in two clicks — pick a channel and we'll post
				deployment events there. No webhook URLs to copy.
			</p>
			{#if slackAllowed}
				<a
					href="/slack/oauth/start"
					data-sveltekit-reload
					class="inline-flex items-center gap-2 rounded-md bg-[#4A154B] px-4 py-2 text-sm font-semibold text-white transition hover:bg-[#611f64]"
				>
					<svg class="h-4 w-4" viewBox="0 0 122.8 122.8" aria-hidden="true">
						<path
							d="M25.8 77.6c0 7.1-5.8 12.9-12.9 12.9S0 84.7 0 77.6s5.8-12.9 12.9-12.9h12.9zm6.5 0c0-7.1 5.8-12.9 12.9-12.9s12.9 5.8 12.9 12.9v32.3c0 7.1-5.8 12.9-12.9 12.9s-12.9-5.8-12.9-12.9z"
							fill="#36C5F0"
						/>
						<path
							d="M45.2 25.8c-7.1 0-12.9-5.8-12.9-12.9S38.1 0 45.2 0s12.9 5.8 12.9 12.9v12.9zm0 6.5c7.1 0 12.9 5.8 12.9 12.9s-5.8 12.9-12.9 12.9H12.9C5.8 58.1 0 52.3 0 45.2s5.8-12.9 12.9-12.9z"
							fill="#2EB67D"
						/>
						<path
							d="M97 45.2c0-7.1 5.8-12.9 12.9-12.9s12.9 5.8 12.9 12.9-5.8 12.9-12.9 12.9H97zm-6.5 0c0 7.1-5.8 12.9-12.9 12.9s-12.9-5.8-12.9-12.9V12.9C64.7 5.8 70.5 0 77.6 0s12.9 5.8 12.9 12.9z"
							fill="#ECB22E"
						/>
						<path
							d="M77.6 97c7.1 0 12.9 5.8 12.9 12.9s-5.8 12.9-12.9 12.9-12.9-5.8-12.9-12.9V97zm0-6.5c-7.1 0-12.9-5.8-12.9-12.9s5.8-12.9 12.9-12.9h32.3c7.1 0 12.9 5.8 12.9 12.9s-5.8 12.9-12.9 12.9z"
							fill="#E01E5A"
						/>
					</svg>
					Add to Slack
				</a>
			{:else}
				<p class="text-xs text-amber-400">
					Slack notifiers require the Pro plan.
					<a href="/settings/plan" class="underline hover:text-amber-300">Upgrade to enable.</a>
				</p>
			{/if}
		</section>
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
			class="ph-no-capture space-y-3"
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

			{#if kind === 'telegram'}
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
			{:else if kind === 'discord_webhook'}
				<div class="space-y-2">
					<input
						type="url"
						name="webhook"
						required
						placeholder="https://discord.com/api/webhooks/…"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<p class="text-xs text-zinc-500">
						In Discord: Channel Settings → Integrations → Webhooks → New Webhook → Copy Webhook URL.
						No bot needed; messages post as the webhook.
					</p>
				</div>
			{:else if kind === 'teams'}
				<div class="space-y-2">
					<input
						type="url"
						name="webhook"
						required
						placeholder="https://….webhook.office.com/webhookb2/…"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<p class="text-xs text-zinc-500">
						In Teams: channel → ⋯ → Workflows → "Post to a channel when a webhook request is
						received" → copy the generated URL. No app registration needed; messages post as the
						webhook.
					</p>
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
				<div class="space-y-2">
					<input
						type="email"
						name="recipient"
						required
						placeholder="you@example.com"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<p class="text-xs text-zinc-500">
						Deployment alerts are sent to this address from Hoister's mail server — no SMTP
						credentials needed.
					</p>
				</div>
			{:else if kind === 'ntfy'}
				<div class="space-y-3">
					<div class="grid gap-3 sm:grid-cols-2">
						<input
							type="url"
							name="server"
							required
							placeholder="https://ntfy.sh"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
						<input
							type="text"
							name="topic"
							required
							placeholder="Topic"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
					</div>
					<input
						type="text"
						name="access_token"
						placeholder="Access token (optional, for protected topics)"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<p class="text-xs text-zinc-500">
						Server must be reachable over https. Anyone who knows an unprotected topic can read it —
						treat it like a weak secret.
					</p>
				</div>
			{:else if kind === 'pushover'}
				<div class="space-y-3">
					<div class="grid gap-3 sm:grid-cols-2">
						<input
							type="text"
							name="token"
							required
							placeholder="Application API token"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
						<input
							type="text"
							name="user"
							required
							placeholder="User or group key"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
					</div>
					<input
						type="text"
						name="device"
						placeholder="Device name (optional — defaults to all devices)"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
				</div>
			{:else if kind === 'matrix'}
				<div class="space-y-3">
					<input
						type="url"
						name="homeserver"
						required
						placeholder="https://matrix.org"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<div class="grid gap-3 sm:grid-cols-2">
						<input
							type="text"
							name="access_token"
							required
							placeholder="Access token"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
						<input
							type="text"
							name="room_id"
							required
							placeholder="!roomid:matrix.org"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
					</div>
					<p class="text-xs text-zinc-500">
						Use a bot/user access token that has already joined the target room. Homeserver must be
						reachable over https.
					</p>
				</div>
			{:else if kind === 'mattermost'}
				<div class="space-y-3">
					<input
						type="url"
						name="webhook"
						required
						placeholder="https://mattermost.example.com/hooks/…"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<div class="grid gap-3 sm:grid-cols-2">
						<input
							type="text"
							name="channel"
							placeholder="Channel override (optional, e.g. town-square)"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
						<input
							type="text"
							name="username"
							placeholder="Display name (optional)"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
					</div>
					<p class="text-xs text-zinc-500">
						In Mattermost: Integrations → Incoming Webhooks → Add. Server must be reachable over
						https. A channel override only works if the webhook allows it.
					</p>
				</div>
			{:else if kind === 'rocketchat'}
				<div class="space-y-3">
					<input
						type="url"
						name="webhook"
						required
						placeholder="https://rocketchat.example.com/hooks/…"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<div class="grid gap-3 sm:grid-cols-2">
						<input
							type="text"
							name="channel"
							placeholder="Channel override (optional, e.g. #general)"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
						<input
							type="text"
							name="alias"
							placeholder="Alias / display name (optional)"
							class="rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
						/>
					</div>
					<p class="text-xs text-zinc-500">
						In Rocket.Chat: Administration → Integrations → New → Incoming. Server must be reachable
						over https.
					</p>
				</div>
			{:else if kind === 'google_chat'}
				<div class="space-y-2">
					<input
						type="url"
						name="webhook"
						required
						placeholder="https://chat.googleapis.com/v1/spaces/…?key=…&token=…"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<p class="text-xs text-zinc-500">
						In Google Chat: open the space → Apps & integrations → Webhooks → Add. Copy the full URL
						including the key/token — it's the secret that authorizes posting.
					</p>
				</div>
			{:else if kind === 'webhook'}
				<div class="space-y-2">
					<input
						type="url"
						name="url"
						required
						placeholder="https://example.com/hooks/hoister"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500"
					/>
					<textarea
						name="headers"
						rows="2"
						placeholder="Optional headers, one per line — e.g. Authorization: Bearer xxxxx"
						class="w-full rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 font-mono text-xs text-zinc-100 placeholder:text-zinc-500"
					></textarea>
					<p class="text-xs text-zinc-500">
						Hoister POSTs each event as JSON. The endpoint must be https and resolve to a public
						address.
					</p>
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
			<div class="overflow-x-auto rounded-xl border border-zinc-800">
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
									<div class="flex justify-end gap-2">
										<form
											method="POST"
											action="?/test"
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
												class="rounded-md border border-zinc-700 px-3 py-1 text-xs font-medium text-zinc-300 transition hover:bg-zinc-800 disabled:opacity-50"
											>
												{busyId === n.id ? 'Sending…' : 'Test'}
											</button>
										</form>
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
									</div>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</div>
