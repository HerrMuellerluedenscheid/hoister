<script lang="ts">
	import Deployments from '$lib/components/Deployments.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<div class="space-y-10 px-8 py-10">
	<h1 class="text-2xl font-bold">Dashboard</h1>

	{#if data.tokenCount === 0}
		<section class="rounded-xl border border-indigo-500/40 bg-indigo-500/10 p-5">
			<div class="flex items-center justify-between gap-4">
				<div>
					<h2 class="text-base font-semibold text-indigo-300">No agent connected yet</h2>
					<p class="mt-1 text-sm text-zinc-300">
						Create your first agent token to start reporting container state to hoister.io.
					</p>
				</div>
				<a
					href="/tokens"
					class="rounded-md bg-indigo-500 px-4 py-2 text-sm font-semibold text-white hover:bg-indigo-400"
				>
					Create token
				</a>
			</div>
		</section>
	{/if}

	<section>
		<h2 class="mb-4 text-lg font-semibold text-zinc-200">Recent deployments</h2>

		{#if data.deploymentsError}
			<div class="mb-4 rounded-xl border border-red-800 bg-red-950 px-4 py-3 text-red-400">
				{data.deploymentsError}
			</div>
		{/if}

		<Deployments data={data.deployments} />
	</section>
</div>
