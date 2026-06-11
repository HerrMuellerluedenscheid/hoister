<script lang="ts">
	import { analyticsConsent } from '$lib/consent.svelte';

	const visible = $derived(analyticsConsent.value === 'unknown');
</script>

{#if visible}
	<div
		class="fixed right-0 bottom-0 left-0 z-50 border-t border-line bg-canvas px-6 py-4"
	>
		<div
			class="mx-auto flex max-w-4xl flex-col items-start gap-3 sm:flex-row sm:items-center sm:justify-between"
		>
			<p class="text-sm text-ink-muted">
				We use strictly necessary cookies for authentication. With your permission we'd also like to
				use PostHog to anonymously measure how the dashboard is used so we can improve it. See our <a
					href="/datenschutz"
					class="text-brand-accent underline hover:text-brand-light">privacy policy</a
				> for details.
			</p>
			<div class="flex shrink-0 gap-2">
				<button
					onclick={() => analyticsConsent.decline()}
					class="rounded-lg border border-line-subtle px-4 py-1.5 text-sm font-medium text-ink-muted transition hover:border-line-active hover:text-ink"
				>
					Decline
				</button>
				<button
					onclick={() => analyticsConsent.accept()}
					class="rounded-lg bg-brand-hover px-4 py-1.5 text-sm font-medium text-white transition hover:bg-brand-accent"
				>
					Accept
				</button>
			</div>
		</div>
	</div>
{/if}
