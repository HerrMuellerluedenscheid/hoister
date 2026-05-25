<script lang="ts">
	import { browser } from '$app/environment';

	let visible = $state(browser && !localStorage.getItem('cookie-consent'));

	function accept() {
		localStorage.setItem('cookie-consent', '1');
		visible = false;
	}
</script>

{#if visible}
	<div
		class="fixed right-0 bottom-0 left-0 z-50 border-t border-zinc-800 bg-zinc-950 px-6 py-4"
	>
		<div
			class="mx-auto flex max-w-4xl flex-col items-start gap-3 sm:flex-row sm:items-center sm:justify-between"
		>
			<p class="text-sm text-zinc-400">
				This site uses strictly necessary cookies for authentication. No tracking or advertising
				cookies are used. See our <a
					href="/datenschutz"
					class="text-indigo-400 underline hover:text-indigo-300">privacy policy</a
				> for details.
			</p>
			<button
				onclick={accept}
				class="shrink-0 rounded-lg border border-zinc-700 px-4 py-1.5 text-sm font-medium text-zinc-300 transition hover:border-zinc-500 hover:text-zinc-100"
			>
				OK
			</button>
		</div>
	</div>
{/if}
