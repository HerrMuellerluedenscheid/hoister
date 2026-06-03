<script lang="ts">
	import { REDACTION_MARKER } from '$lib/redaction';

	// Renders a string where the agent's redaction marker is shown as a small
	// "redacted" badge instead of literal `***REDACTED***`. Works for a value
	// that is entirely a secret (env vars) and for markers embedded in log text.
	let { text }: { text: string } = $props();

	let segments = $derived(text.split(REDACTION_MARKER));
</script>

{#each segments as segment, i}{segment}{#if i < segments.length - 1}<span
			class="inline-flex items-center gap-1 rounded border border-amber-500/40 bg-amber-500/10 px-1.5 align-middle font-sans text-[0.65rem] font-medium tracking-wide text-amber-300 uppercase select-none"
			title="Hidden by the agent before it left the host"
			><svg class="h-3 w-3" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true"
				><path
					fill-rule="evenodd"
					d="M10 1a4 4 0 00-4 4v2H5a2 2 0 00-2 2v6a2 2 0 002 2h10a2 2 0 002-2V9a2 2 0 00-2-2h-1V5a4 4 0 00-4-4zm2 6V5a2 2 0 10-4 0v2h4z"
					clip-rule="evenodd"
				/></svg
			>redacted</span
		>{/if}{/each}
