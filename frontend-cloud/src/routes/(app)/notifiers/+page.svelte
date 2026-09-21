<script lang="ts">
	import NotifierManager from '$lib/components/NotifierManager.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
</script>

<div class="space-y-8 px-4 py-6 sm:px-8 sm:py-10">
	<header>
		<h1 class="text-2xl font-bold">Notifiers</h1>
		<p class="mt-1 text-sm text-ink-muted">
			Get pinged when one of your agents reports a deployment, a rollback, or a pending update.
			Configure as many channels as you like — every enabled notifier receives every event.
		</p>
	</header>

	<NotifierManager
		notifiers={data.notifiers}
		me={data.me}
		{form}
		error={data.error}
		slackHref="/slack/oauth/start"
	/>
</div>
