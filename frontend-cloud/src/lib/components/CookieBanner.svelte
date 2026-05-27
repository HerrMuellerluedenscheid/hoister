<script lang="ts">
	import { analyticsConsent } from '$lib/consent.svelte';

	const visible = $derived(analyticsConsent.value === 'unknown');
</script>

{#if visible}
	<div
		class="fixed right-0 bottom-0 left-0 z-50 border-t border-zinc-800 bg-zinc-950 px-6 py-4"
	>
		<div
			class="mx-auto flex max-w-4xl flex-col items-start gap-3 sm:flex-row sm:items-center sm:justify-between"
		>
			<p class="text-sm text-zinc-400">
				We use strictly necessary cookies for authentication. With your permission we'd also like to
				use PostHog to anonymously measure how the dashboard is used so we can improve it. See our <a
					href="/datenschutz"
					class="text-indigo-400 underline hover:text-indigo-300">privacy policy</a
				> for details.
			</p>
			<div class="flex shrink-0 gap-2">
				<button
					onclick={() => analyticsConsent.decline()}
					class="rounded-lg border border-zinc-700 px-4 py-1.5 text-sm font-medium text-zinc-400 transition hover:border-zinc-500 hover:text-zinc-100"
				>
					Decline
				</button>
				<button
					onclick={() => analyticsConsent.accept()}
					class="rounded-lg bg-indigo-500 px-4 py-1.5 text-sm font-medium text-white transition hover:bg-indigo-400"
				>
					Accept
				</button>
			</div>
		</div>
	</div>
{/if}
