<script lang="ts">
	import SEO from '$lib/components/SEO.svelte';
</script>

<SEO
	title="Impressum — Hoister"
	description="Legal notice and provider identification for Hoister."
	path="/impressum"
	noindex
/>

<div class="flex min-h-screen flex-col bg-zinc-950 text-zinc-100">
	<header class="flex items-center justify-between px-8 py-5">
		<a href="/" class="flex items-center gap-2">
			<svg
				class="h-7 w-7 text-indigo-400"
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
			<span class="text-lg font-semibold tracking-tight">Hoister</span>
		</a>
	</header>

	<main class="mx-auto w-full max-w-2xl flex-1 px-8 py-12">
		<h1 class="mb-8 text-3xl font-bold tracking-tight">Impressum</h1>

		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				Angaben gemäß § 5 TMG
			</h2>
			<div class="space-y-1 text-zinc-300">
				<p class="font-semibold">Vector &amp; Veneer UG (haftungsbeschränkt)</p>
				<p>Rubensstr. 108</p>
				<p>12157 Berlin</p>
				<p>Deutschland</p>
			</div>
			<div class="mt-3 space-y-1 text-zinc-300">
				<p class="text-zinc-400">Vertreten durch:</p>
				<p>Marius Kriegerowski (Geschäftsführer)</p>
			</div>
		</section>

		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">Kontakt</h2>
			<div class="space-y-1 text-zinc-300">
				<p>
					E-Mail: <a
						href="mailto:info@vectorandveneer.com"
						class="text-indigo-400 hover:text-indigo-300">info@vectorandveneer.com</a
					>
				</p>
			</div>
		</section>

		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				Hinweis Handelsregister &amp; USt-IdNr.
			</h2>
			<div class="space-y-1 text-zinc-300">
				<p>Registergericht: Amtsgericht Charlottenburg</p>
				<p>Registernummer: HRB 286900 B</p>
			</div>
			<p class="mt-3 text-zinc-400">
				Eine Umsatzsteuer-Identifikationsnummer gemäß § 27a UStG wurde bislang nicht erteilt.
			</p>
		</section>

		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				Verantwortlich für den Inhalt (§ 55 Abs. 2 RStV)
			</h2>
			<div class="space-y-1 text-zinc-300">
				<p>Marius Kriegerowski</p>
				<p>Rubensstr. 108</p>
				<p>12157 Berlin</p>
			</div>
		</section>

		<section class="mb-8">
			<h2 class="mb-3 text-sm font-semibold tracking-widest text-zinc-500 uppercase">
				Datenschutz
			</h2>
			<p class="text-zinc-400">
				Informationen zur Verarbeitung personenbezogener Daten finden Sie in unserer <a
					href="/datenschutz"
					class="text-indigo-400 hover:text-indigo-300">Datenschutzerklärung</a
				>.
			</p>
		</section>
	</main>

	<footer class="border-t border-zinc-800 py-5 text-center text-xs text-zinc-600">
		Hoister — open source on GitHub ·
		<a href="/impressum" class="hover:text-zinc-400">Impressum</a> ·
		<a href="/datenschutz" class="hover:text-zinc-400">Datenschutz</a>
	</footer>
</div>
