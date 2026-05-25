<script lang="ts">
	import { UserButton } from 'svelte-clerk';
	import { enhance } from '$app/forms';
	import Deployments from '$lib/components/Deployments.svelte';
	import type { TokenResponse } from '$lib/api/token';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let copied = $state(false);
	let manuallyShown = $state(false);
	let manuallyHidden = $state(false);
	let rotating = $state(false);

	// A freshly-rotated token (from a form action) takes priority over the
	// load-time token — `data.agentToken.token` is null for returning users
	// because the controller only stores the SHA-256 hash.
	const currentToken = $derived<TokenResponse | null>(form?.rotatedToken ?? data.agentToken);

	const hasPlaintext = $derived(currentToken != null && currentToken.token != null);

	// Show the snippet when:
	//  - the controller just minted the token (first call or after rotate), OR
	//  - the user explicitly clicked "Show snippet" AND we have a plaintext
	//    to fill in. We never reveal a stub `your-token` snippet for tokens
	//    whose plaintext we can't recover.
	const showSnippet = $derived(
		hasPlaintext && ((currentToken!.is_new && !manuallyHidden) || manuallyShown)
	);

	const snippet = $derived(
		hasPlaintext
			? `services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_CONTROLLER_TOKEN: "${currentToken!.token}"`
			: ''
	);

	async function copySnippet() {
		await navigator.clipboard.writeText(snippet);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<div class="min-h-screen bg-zinc-950 text-zinc-100">
	<header class="flex items-center justify-between border-b border-zinc-800 px-8 py-5">
		<div class="flex items-center gap-4">
			<div class="flex items-center gap-2">
				<svg
					class="h-6 w-6 text-indigo-400"
					viewBox="0 0 24 24"
					fill="none"
					stroke="currentColor"
					stroke-width="2"
				>
					<path
						stroke-linecap="round"
						stroke-linejoin="round"
						d="M5 10l1.5-4.5h11L19 10M5 10h14M5 10l-2 7h18l-2-7M9 17v2m6-2v2m-3-9v4"
					/>
				</svg>
				<span class="font-semibold tracking-tight">Hoister</span>
			</div>
			<nav class="flex gap-3 text-sm text-zinc-400">
				<a href="/dashboard" class="text-zinc-100">Dashboard</a>
				<a href="/containers" class="hover:text-zinc-100">Containers</a>
			</nav>
		</div>
		<UserButton />
	</header>

	<main class="mx-auto max-w-4xl space-y-10 px-8 py-10">
		{#if currentToken}
			{#if showSnippet}
				<!-- Snippet visible: first issue or after rotate -->
				<section class="rounded-xl border border-indigo-500/40 bg-indigo-500/10 p-6">
					<div class="mb-1 flex items-center justify-between">
						<h2 class="text-lg font-semibold text-indigo-300">Connect your agent</h2>
						<button
							onclick={() => {
								manuallyHidden = true;
								manuallyShown = false;
							}}
							class="text-xs text-zinc-500 hover:text-zinc-300"
						>
							Dismiss
						</button>
					</div>
					<p class="mb-4 text-sm text-zinc-400">
						Add the <code class="rounded bg-zinc-800 px-1 py-0.5 text-zinc-200">hoister</code>
						service to your
						<code class="rounded bg-zinc-800 px-1 py-0.5 text-zinc-200">docker-compose.yaml</code>.
						This token is shown <strong>once</strong> — save it now. Rotate if it leaks or you lose it.
					</p>
					<div class="relative">
						<pre
							class="overflow-x-auto rounded-lg bg-zinc-900 p-4 text-sm leading-relaxed text-zinc-200">{snippet}</pre>
						<button
							onclick={copySnippet}
							class="absolute top-3 right-3 rounded-md border border-zinc-700 bg-zinc-800 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-500 hover:text-zinc-100"
						>
							{copied ? 'Copied!' : 'Copy'}
						</button>
					</div>
				</section>
			{:else}
				<!-- Compact row: returning user with no plaintext available -->
				<div
					class="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-900 px-5 py-3"
				>
					<div class="flex items-center gap-3 text-sm text-zinc-400">
						<span class="h-2 w-2 rounded-full bg-emerald-400"></span>
						Agent token issued
						<span class="text-xs text-zinc-600"
							>(plaintext only shown once; rotate to get a new one)</span
						>
					</div>
					<form
						method="POST"
						action="?/rotateToken"
						use:enhance={() => {
							rotating = true;
							return async ({ update }) => {
								await update();
								rotating = false;
								manuallyShown = true;
								manuallyHidden = false;
							};
						}}
					>
						<button
							type="submit"
							disabled={rotating}
							class="rounded-md border border-zinc-700 bg-zinc-800 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-500 hover:text-zinc-100 disabled:opacity-50"
						>
							{rotating ? 'Rotating…' : 'Rotate token'}
						</button>
					</form>
				</div>
			{/if}
		{/if}

		{#if form?.rotateError}
			<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
				<span class="font-medium">Rotation failed:</span>
				{form.rotateError}
			</div>
		{/if}

		{#if data.tokenError}
			<div
				class="rounded-xl border border-yellow-800 bg-yellow-950/40 px-4 py-3 text-sm text-yellow-400"
			>
				<span class="font-medium">Agent token unavailable:</span>
				{data.tokenError}
			</div>
		{/if}

		<section>
			<h1 class="mb-4 text-2xl font-bold">Deployments</h1>

			{#if data.error}
				<div class="mb-4 rounded-xl border border-red-800 bg-red-950 px-4 py-3 text-red-400">
					{data.error}
				</div>
			{/if}

			<Deployments data={data.deployments} />
		</section>
	</main>
</div>
