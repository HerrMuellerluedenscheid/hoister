<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let comment = $state('');
	let creating = $state(false);
	let deletingId = $state<string | null>(null);
	let confirmingId = $state<string | null>(null);

	// First-run onboarding: a fresh account has no tokens yet, so we nudge the
	// user straight at the create button.
	const isFirstRun = $derived(data.tokens.length === 0);

	const justCreated = $derived(form?.created ?? null);

	// Show the setup modal once per freshly-minted token. Keyed on the token id
	// so dismissing it doesn't immediately re-open on the next reactive tick.
	let modalOpen = $state(false);
	let shownTokenId = $state<string | null>(null);
	$effect(() => {
		const id = justCreated?.id ?? null;
		if (id && id !== shownTokenId) {
			modalOpen = true;
			shownTokenId = id;
		}
	});

	// Ready-to-paste agent service, with the freshly-issued token injected.
	const composeSnippet = $derived(`services:
  hoister:
    image: hoister/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_CONTROLLER_TOKEN: "${justCreated?.token ?? ''}"
      HOISTER_HOSTNAME: "<this-host-name>"`);

	// The label that opts a container into Hoister management.
	const labelSnippet = `services:
  my-app:
    image: ghcr.io/acme/app:latest
    labels:
      - "hoister.enable=true"`;

	let copiedKey = $state<string | null>(null);
	async function copy(key: string, text: string) {
		await navigator.clipboard.writeText(text);
		copiedKey = key;
		setTimeout(() => {
			if (copiedKey === key) copiedKey = null;
		}, 2000);
	}

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return isNaN(d.getTime()) ? iso : d.toLocaleString();
	}
</script>

<svelte:window
	onkeydown={(e) => {
		if (e.key === 'Escape') modalOpen = false;
	}}
/>

<div class="space-y-8 px-4 py-6 sm:px-8 sm:py-10">
	<header class="flex items-start justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Agent tokens</h1>
			<p class="mt-1 text-sm text-ink-muted">
				Each Hoister agent connects to <code
					class="rounded bg-element px-1 py-0.5 font-mono text-xs">api.hoister.io</code
				>
				with one of these tokens. Treat them like passwords — they're shown in plaintext exactly once
				at creation.
			</p>
		</div>
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

	<!-- Create form -->
	<section
		class="rounded-xl border bg-card p-5 transition-colors {isFirstRun
			? 'border-brand-hover/50'
			: 'border-line'}"
	>
		{#if isFirstRun}
			<p class="mb-3 flex items-center gap-2 text-sm font-medium text-brand-light">
				<span
					class="flex h-5 w-5 items-center justify-center rounded-full bg-brand-hover text-xs text-white"
					>1</span
				>
				Start here — create a token to connect your first host.
			</p>
		{:else}
			<h2 class="mb-3 text-base font-semibold text-ink-code">Create new token</h2>
		{/if}
		<form
			method="POST"
			action="?/create"
			use:enhance={() => {
				creating = true;
				return async ({ update }) => {
					await update({ reset: true });
					comment = '';
					creating = false;
				};
			}}
			class="flex flex-col gap-3 sm:flex-row"
		>
			<input
				type="text"
				name="comment"
				bind:value={comment}
				maxlength="120"
				placeholder="Optional label, e.g. the hostname"
				class="flex-1 rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink placeholder:text-ink-faint focus:border-brand-accent focus:outline-none"
			/>
			<button
				type="submit"
				disabled={creating}
				class="rounded-md bg-brand-hover px-4 py-2 text-sm font-semibold text-white transition hover:bg-brand-accent disabled:opacity-50 {isFirstRun &&
				!creating
					? 'flash'
					: ''}"
			>
				{creating ? 'Creating…' : 'Create token'}
			</button>
		</form>
	</section>

	<!-- Existing tokens -->
	<section>
		<h2 class="mb-3 text-base font-semibold text-ink-code">
			Your tokens ({data.tokens.length})
		</h2>

		{#if data.tokens.length === 0}
			<div class="rounded-xl border border-line bg-card px-5 py-4 text-sm text-ink-muted">
				No tokens yet. Create one above to connect your first agent.
			</div>
		{:else}
			<div class="overflow-x-auto rounded-xl border border-line">
				<table class="min-w-full divide-y divide-line text-sm">
					<thead class="bg-card text-xs tracking-wider text-ink-muted uppercase">
						<tr>
							<th class="px-4 py-2 text-left font-medium">Prefix</th>
							<th class="px-4 py-2 text-left font-medium">Comment</th>
							<th class="px-4 py-2 text-left font-medium">Created</th>
							<th class="px-4 py-2"></th>
						</tr>
					</thead>
					<tbody class="divide-y divide-line bg-canvas">
						{#each data.tokens as token (token.id)}
							<tr class="text-ink-secondary">
								<td class="px-4 py-3 font-mono text-xs">{token.token_prefix}…</td>
								<td class="px-4 py-3 break-all">
									{#if token.comment}
										{token.comment}
									{:else}
										<span class="text-ink-ghost">—</span>
									{/if}
								</td>
								<td class="px-4 py-3 text-xs text-ink-faint">{formatDate(token.created_at)}</td>
								<td class="px-4 py-3 text-right">
									{#if confirmingId === token.id}
										<div class="flex items-center justify-end gap-2">
											<span class="text-xs text-ink-muted">Delete this token?</span>
											<form
												method="POST"
												action="?/delete"
												use:enhance={() => {
													deletingId = token.id;
													confirmingId = null;
													return async ({ update }) => {
														await update();
														deletingId = null;
													};
												}}
											>
												<input type="hidden" name="id" value={token.id} />
												<button
													type="submit"
													disabled={deletingId === token.id}
													class="rounded-md border border-error-border bg-error-bg px-3 py-1 text-xs font-medium text-error transition hover:opacity-80 disabled:opacity-50"
												>
													{deletingId === token.id ? 'Deleting…' : 'Confirm'}
												</button>
											</form>
											<button
												type="button"
												onclick={() => (confirmingId = null)}
												class="rounded-md border border-line-subtle px-3 py-1 text-xs font-medium text-ink-secondary transition hover:border-line-active"
											>
												Cancel
											</button>
										</div>
									{:else}
										<button
											type="button"
											disabled={deletingId === token.id}
											onclick={() => (confirmingId = token.id)}
											class="rounded-md border border-error-border px-3 py-1 text-xs font-medium text-error transition hover:bg-error-bg disabled:opacity-50"
										>
											{deletingId === token.id ? 'Deleting…' : 'Delete'}
										</button>
									{/if}
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		{/if}
	</section>
</div>

<!-- Setup modal: shown once when a token is freshly minted. Carries the only
     plaintext view of the token plus the exact copy-paste to get running. -->
{#if modalOpen && justCreated?.token}
	<div
		class="fixed inset-0 z-50 flex items-start justify-center overflow-y-auto bg-black/60 p-4 sm:items-center"
		role="dialog"
		aria-modal="true"
		aria-labelledby="token-modal-title"
	>
		<!-- Backdrop click closes -->
		<button
			type="button"
			aria-label="Close"
			class="absolute inset-0 h-full w-full cursor-default"
			onclick={() => (modalOpen = false)}
		></button>

		<div
			class="relative z-10 my-4 w-full max-w-2xl space-y-5 rounded-2xl border border-line bg-card p-6 shadow-2xl"
		>
			<div class="flex items-start justify-between gap-4">
				<div>
					<h2 id="token-modal-title" class="text-lg font-semibold text-ink">
						Token created — connect your host
					</h2>
					<p class="mt-1 text-sm text-ink-muted">
						Copy the token now — this is the only time it's shown in full.
					</p>
				</div>
				<button
					type="button"
					onclick={() => (modalOpen = false)}
					aria-label="Close"
					class="-mt-1 -mr-1 rounded-md p-1 text-ink-muted transition hover:bg-element hover:text-ink"
				>
					<svg
						class="h-5 w-5"
						fill="none"
						viewBox="0 0 24 24"
						stroke="currentColor"
						stroke-width="2"
					>
						<path stroke-linecap="round" stroke-linejoin="round" d="M6 6l12 12M18 6L6 18" />
					</svg>
				</button>
			</div>

			<!-- 1. The token itself -->
			<div>
				<div class="mb-1.5 flex items-center gap-2 text-sm font-medium text-ink-secondary">
					<span
						class="flex h-5 w-5 items-center justify-center rounded-full bg-brand-hover text-xs text-white"
						>1</span
					>
					Your agent token
				</div>
				<div class="relative">
					<pre
						class="overflow-x-auto rounded-lg bg-canvas p-4 pr-16 font-mono text-sm break-all whitespace-pre-wrap text-ink">{justCreated.token}</pre>
					<button
						type="button"
						onclick={() => copy('token', justCreated.token ?? '')}
						class="absolute top-3 right-3 rounded-md border border-line-subtle bg-element px-3 py-1 text-xs text-ink-secondary transition hover:border-line-active hover:text-ink"
					>
						{copiedKey === 'token' ? 'Copied!' : 'Copy'}
					</button>
				</div>
			</div>

			<!-- 2. Add the agent to the stack -->
			<div>
				<div class="mb-1.5 flex items-center gap-2 text-sm font-medium text-ink-secondary">
					<span
						class="flex h-5 w-5 items-center justify-center rounded-full bg-brand-hover text-xs text-white"
						>2</span
					>
					Add the Hoister agent to your
					<code class="rounded bg-element px-1 py-0.5 font-mono text-xs">docker-compose.yaml</code>
				</div>
				<div class="relative">
					<pre
						class="overflow-x-auto rounded-lg bg-canvas p-4 pr-16 font-mono text-xs leading-relaxed text-ink-code">{composeSnippet}</pre>
					<button
						type="button"
						onclick={() => copy('compose', composeSnippet)}
						class="absolute top-3 right-3 rounded-md border border-line-subtle bg-element px-3 py-1 text-xs text-ink-secondary transition hover:border-line-active hover:text-ink"
					>
						{copiedKey === 'compose' ? 'Copied!' : 'Copy'}
					</button>
				</div>
			</div>

			<!-- 3. Label the containers to monitor -->
			<div>
				<div class="mb-1.5 flex items-center gap-2 text-sm font-medium text-ink-secondary">
					<span
						class="flex h-5 w-5 items-center justify-center rounded-full bg-brand-hover text-xs text-white"
						>3</span
					>
					Label each container you want managed
				</div>
				<p class="mb-2 text-xs text-ink-muted">
					Hoister only touches containers carrying the
					<code class="rounded bg-element px-1 py-0.5 font-mono text-xs">hoister.enable=true</code>
					label. Add it to every service you want auto-updated:
				</p>
				<div class="relative">
					<pre
						class="overflow-x-auto rounded-lg bg-canvas p-4 pr-16 font-mono text-xs leading-relaxed text-ink-code">{labelSnippet}</pre>
					<button
						type="button"
						onclick={() => copy('label', labelSnippet)}
						class="absolute top-3 right-3 rounded-md border border-line-subtle bg-element px-3 py-1 text-xs text-ink-secondary transition hover:border-line-active hover:text-ink"
					>
						{copiedKey === 'label' ? 'Copied!' : 'Copy'}
					</button>
				</div>
			</div>

			<div class="flex items-center justify-between gap-4 border-t border-line pt-4">
				<a
					href="https://docs.hoister.io/guides/getting-started/"
					target="_blank"
					rel="noopener noreferrer"
					class="text-sm text-brand-accent transition hover:text-brand-light"
				>
					Full setup guide →
				</a>
				<button
					type="button"
					onclick={() => (modalOpen = false)}
					class="rounded-md bg-brand-hover px-4 py-2 text-sm font-semibold text-white transition hover:bg-brand-accent"
				>
					Done
				</button>
			</div>
		</div>
	</div>
{/if}

<style>
	/* Draw the eye to the create button on first run. */
	:global(.flash) {
		animation: hoister-flash 1.6s ease-in-out infinite;
	}
	@keyframes hoister-flash {
		0%,
		100% {
			box-shadow: 0 0 0 0 color-mix(in srgb, var(--color-brand-accent) 60%, transparent);
		}
		50% {
			box-shadow: 0 0 0 7px color-mix(in srgb, var(--color-brand-accent) 0%, transparent);
		}
	}
	@media (prefers-reduced-motion: reduce) {
		:global(.flash) {
			animation: none;
			box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-brand-accent) 45%, transparent);
		}
	}
</style>
