<script lang="ts">
	import Deployments from '$lib/components/Deployments.svelte';
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-7xl space-y-8">
		<h1 class="text-2xl font-bold">Deployments</h1>

		{#if data.deploymentsError}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				{data.deploymentsError}
			</div>
		{/if}

		<Deployments data={data.deployments} linkToContainer />
	</div>
</div>
