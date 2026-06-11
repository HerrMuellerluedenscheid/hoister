<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let comment = $state('');
	let creating = $state(false);
	let deletingId = $state<string | null>(null);
	let copied = $state(false);

	const justCreated = $derived(form?.created ?? null);

	async function copyPlaintext() {
		if (!justCreated?.token) return;
		await navigator.clipboard.writeText(justCreated.token);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}

	function formatDate(iso: string): string {
		const d = new Date(iso);
		return isNaN(d.getTime()) ? iso : d.toLocaleString();
	}
</script>

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

	<!-- Freshly-issued token panel: show plaintext exactly once. -->
	{#if justCreated?.token}
		<section class="rounded-xl border border-brand-hover/40 bg-brand-hover/10 p-5">
			<h2 class="mb-1 text-base font-semibold text-brand-light">New token issued</h2>
			<p class="mb-4 text-sm text-ink-secondary">
				Copy it now — this is the only time the full token will be shown. After you leave this page
				only the prefix (<code class="rounded bg-element px-1 py-0.5 font-mono text-xs">
					{justCreated.token_prefix}
				</code>) remains visible.
			</p>
			<div class="relative">
				<pre
					class="overflow-x-auto rounded-lg bg-card p-4 font-mono text-sm break-all whitespace-pre-wrap text-ink">{justCreated.token}</pre>
				<button
					type="button"
					onclick={copyPlaintext}
					class="absolute top-3 right-3 rounded-md border border-line-subtle bg-element px-3 py-1 text-xs text-ink-secondary transition hover:border-line-active hover:text-ink"
				>
					{copied ? 'Copied!' : 'Copy'}
				</button>
			</div>
		</section>
	{/if}

	<!-- Create form -->
	<section class="rounded-xl border border-line bg-card p-5">
		<h2 class="mb-3 text-base font-semibold text-ink-code">Create new token</h2>
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
				class="rounded-md bg-brand-hover px-4 py-2 text-sm font-semibold text-white transition hover:bg-brand-accent disabled:opacity-50"
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
									<form
										method="POST"
										action="?/delete"
										use:enhance={() => {
											deletingId = token.id;
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
											class="rounded-md border border-error-border px-3 py-1 text-xs font-medium text-error transition hover:bg-error-bg disabled:opacity-50"
										>
											{deletingId === token.id ? 'Deleting…' : 'Delete'}
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
