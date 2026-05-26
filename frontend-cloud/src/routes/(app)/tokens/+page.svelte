<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let comment = $state('');
	let creating = $state(false);
	let deletingId = $state<number | null>(null);
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

<div class="space-y-8 px-8 py-10">
	<header class="flex items-start justify-between gap-4">
		<div>
			<h1 class="text-2xl font-bold">Agent tokens</h1>
			<p class="mt-1 text-sm text-zinc-400">
				Each Hoister agent connects to <code
					class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">api.hoister.io</code
				>
				with one of these tokens. Treat them like passwords — they're shown in plaintext exactly once
				at creation.
			</p>
		</div>
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

	<!-- Freshly-issued token panel: show plaintext exactly once. -->
	{#if justCreated?.token}
		<section class="rounded-xl border border-indigo-500/40 bg-indigo-500/10 p-5">
			<h2 class="mb-1 text-base font-semibold text-indigo-300">New token issued</h2>
			<p class="mb-4 text-sm text-zinc-300">
				Copy it now — this is the only time the full token will be shown. After you leave this page
				only the prefix (<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">
					{justCreated.token_prefix}
				</code>) remains visible.
			</p>
			<div class="relative">
				<pre
					class="overflow-x-auto rounded-lg bg-zinc-900 p-4 font-mono text-sm break-all whitespace-pre-wrap text-zinc-100">{justCreated.token}</pre>
				<button
					type="button"
					onclick={copyPlaintext}
					class="absolute top-3 right-3 rounded-md border border-zinc-700 bg-zinc-800 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-500 hover:text-zinc-100"
				>
					{copied ? 'Copied!' : 'Copy'}
				</button>
			</div>
		</section>
	{/if}

	<!-- Create form -->
	<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
		<h2 class="mb-3 text-base font-semibold text-zinc-200">Create new token</h2>
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
				placeholder="Optional label, e.g. 'vectorandveneer'"
				class="flex-1 rounded-md border border-zinc-700 bg-zinc-950 px-3 py-2 text-sm text-zinc-100 placeholder:text-zinc-500 focus:border-indigo-400 focus:outline-none"
			/>
			<button
				type="submit"
				disabled={creating}
				class="rounded-md bg-indigo-500 px-4 py-2 text-sm font-semibold text-white transition hover:bg-indigo-400 disabled:opacity-50"
			>
				{creating ? 'Creating…' : 'Create token'}
			</button>
		</form>
	</section>

	<!-- Existing tokens -->
	<section>
		<h2 class="mb-3 text-base font-semibold text-zinc-200">
			Your tokens ({data.tokens.length})
		</h2>

		{#if data.tokens.length === 0}
			<div class="rounded-xl border border-zinc-800 bg-zinc-900 px-5 py-4 text-sm text-zinc-400">
				No tokens yet. Create one above to connect your first agent.
			</div>
		{:else}
			<div class="overflow-hidden rounded-xl border border-zinc-800">
				<table class="min-w-full divide-y divide-zinc-800 text-sm">
					<thead class="bg-zinc-900 text-xs tracking-wider text-zinc-400 uppercase">
						<tr>
							<th class="px-4 py-2 text-left font-medium">Prefix</th>
							<th class="px-4 py-2 text-left font-medium">Comment</th>
							<th class="px-4 py-2 text-left font-medium">Created</th>
							<th class="px-4 py-2"></th>
						</tr>
					</thead>
					<tbody class="divide-y divide-zinc-800 bg-zinc-950">
						{#each data.tokens as token (token.id)}
							<tr class="text-zinc-300">
								<td class="px-4 py-3 font-mono text-xs">{token.token_prefix}…</td>
								<td class="px-4 py-3 break-all">
									{#if token.comment}
										{token.comment}
									{:else}
										<span class="text-zinc-600">—</span>
									{/if}
								</td>
								<td class="px-4 py-3 text-xs text-zinc-500">{formatDate(token.created_at)}</td>
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
											class="rounded-md border border-red-500/40 px-3 py-1 text-xs font-medium text-red-300 transition hover:bg-red-500/15 disabled:opacity-50"
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
