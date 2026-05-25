<script lang="ts">
	import { UserButton } from 'svelte-clerk';
	import Deployments from '$lib/components/Deployments.svelte';

	let { data } = $props();

	let copied = $state(false);
	let manuallyShown = $state(false);
	let manuallyHidden = $state(false);
	const showSnippet = $derived(
		(data.agentToken != null) &&
		((data.agentToken.is_new && !manuallyHidden) || manuallyShown)
	);

	const snippet = $derived(
		data.agentToken
			? `services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_CONTROLLER_URL: "https://your-controller:3033"
      HOISTER_API_SECRET: "${data.agentToken.token}"`
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
		<UserButton />
	</header>

	<main class="mx-auto max-w-4xl space-y-10 px-8 py-10">

		{#if data.agentToken}
			{#if data.agentToken.is_new || showSnippet}
				<!-- First-time setup banner -->
				<section class="rounded-xl border border-indigo-500/40 bg-indigo-500/10 p-6">
					<div class="mb-1 flex items-center justify-between">
						<h2 class="text-lg font-semibold text-indigo-300">Connect your agent</h2>
						{#if !data.agentToken.is_new}
							<button
								onclick={() => { manuallyHidden = true; manuallyShown = false; }}
								class="text-xs text-zinc-500 hover:text-zinc-300"
							>
								Dismiss
							</button>
						{/if}
					</div>
					<p class="mb-4 text-sm text-zinc-400">
						Add the <code class="rounded bg-zinc-800 px-1 py-0.5 text-zinc-200">hoister</code> service
						to your
						<code class="rounded bg-zinc-800 px-1 py-0.5 text-zinc-200">docker-compose.yaml</code>
						and replace <code class="rounded bg-zinc-800 px-1 py-0.5 text-zinc-200"
							>your-controller</code
						> with your controller's hostname or IP.
					</p>
					<div class="relative">
						<pre
							class="overflow-x-auto rounded-lg bg-zinc-900 p-4 text-sm leading-relaxed text-zinc-200">{snippet}</pre>
						<button
							onclick={copySnippet}
							class="absolute right-3 top-3 rounded-md border border-zinc-700 bg-zinc-800 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-500 hover:text-zinc-100"
						>
							{copied ? 'Copied!' : 'Copy'}
						</button>
					</div>
				</section>
			{:else}
				<!-- Compact token row for returning users -->
				<div class="flex items-center justify-between rounded-xl border border-zinc-800 bg-zinc-900 px-5 py-3">
					<div class="flex items-center gap-3 text-sm text-zinc-400">
						<span class="h-2 w-2 rounded-full bg-emerald-400"></span>
						Agent connected
						<code class="font-mono text-xs text-zinc-500">{data.agentToken.token.slice(0, 16)}…</code>
					</div>
					<button
						onclick={() => { manuallyShown = true; manuallyHidden = false; }}
						class="text-xs text-zinc-500 hover:text-zinc-300"
					>
						Show setup snippet
					</button>
				</div>
			{/if}
		{/if}

		{#if data.tokenError}
			<div class="rounded-xl border border-yellow-800 bg-yellow-950/40 px-4 py-3 text-sm text-yellow-400">
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
