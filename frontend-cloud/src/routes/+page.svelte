<script lang="ts">
	import { Show, SignInButton, UserButton } from 'svelte-clerk';
	import SEO from '$lib/components/SEO.svelte';
	import { SITE_URL } from '$lib/seo';

	const description =
		'Hoister automatically updates your running Docker containers when a new image is pushed — with built-in health-check rollback. A self-hostable Watchtower alternative for Docker Compose.';

	// SoftwareApplication schema so search engines understand this is a
	// self-hostable developer tool and can show rich results.
	const jsonLd = {
		'@context': 'https://schema.org',
		'@type': 'SoftwareApplication',
		name: 'Hoister',
		applicationCategory: 'DeveloperApplication',
		operatingSystem: 'Docker',
		description,
		url: SITE_URL,
		offers: { '@type': 'Offer', price: '0', priceCurrency: 'USD' },
		sameAs: ['https://github.com/HerrMuellerluedenscheid/hoister']
	};

	const composeSnippet = `services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      HOISTER_CONTROLLER_TOKEN: "hst_<paste-your-token-here>"
      HOISTER_HOSTNAME: "<this-host-name>"
      # Opt in to forwarding container log tails for crashed containers:
      # HOISTER_REPORT_LOGS: "true"`;

	let copied = $state(false);
	async function copySnippet() {
		await navigator.clipboard.writeText(composeSnippet);
		copied = true;
		setTimeout(() => (copied = false), 2000);
	}
</script>

<SEO
	title="Hoister — Automatic Docker container updates with rollback"
	{description}
	path="/"
	{jsonLd}
/>

<div class="flex min-h-screen flex-col bg-zinc-950 text-zinc-100">
	<!-- Nav -->
	<header class="flex items-center justify-between px-8 py-5">
		<div class="flex items-center gap-2">
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
		</div>

		<nav class="flex items-center gap-4">
			<a
				href="https://docs.hoister.io"
				target="_blank"
				rel="noopener noreferrer"
				class="text-sm font-medium text-zinc-300 transition hover:text-zinc-100"
			>
				Docs
			</a>
			<Show when="signed-out">
				{#snippet children()}
					<SignInButton mode="modal">
						{#snippet children()}
							<button
								class="rounded-lg border bg-indigo-300 border-zinc-700 px-4 py-1.5 text-sm font-medium text-zinc-800 transition hover:border-zinc-500 hover:text-zinc-100"
							>
								Sign in
							</button>
						{/snippet}
					</SignInButton>
				{/snippet}
			</Show>
			<Show when="signed-in">
				{#snippet children()}
					<a
						href="/dashboard"
						class="rounded-lg bg-indigo-600 px-4 py-1.5 text-sm font-medium text-white transition hover:bg-indigo-500"
					>
						Dashboard
					</a>
					<UserButton />
				{/snippet}
			</Show>
		</nav>
	</header>

	<!-- Hero -->
	<main class="flex flex-1 flex-col items-center justify-center px-8 text-center">
		<div
			class="mb-4 inline-flex items-center gap-2 rounded-full border border-indigo-500/30 bg-indigo-500/10 px-3 py-1 text-xs font-medium text-indigo-300"
		>
			<span class="h-1.5 w-1.5 rounded-full bg-indigo-400"></span>
			Automated container deployments
		</div>

		<h1 class="mb-5 max-w-2xl text-5xl font-bold tracking-tight text-white">
			Deploy Docker images<br />
			<span class="text-indigo-400">without the toil</span>
		</h1>

		<p class="mb-10 max-w-lg text-base leading-relaxed text-zinc-400">
			Hoister watches your container registry and automatically updates running containers when a
			new image is pushed — with built-in rollback if something goes wrong.
		</p>

		<Show when="signed-in">
			{#snippet children()}
				<a
					href="/dashboard"
					class="rounded-xl bg-indigo-600 px-6 py-3 text-sm font-semibold text-white shadow-lg transition hover:bg-indigo-500 active:scale-95"
				>
					Go to Dashboard →
				</a>
			{/snippet}
		</Show>
	</main>

	<!-- Feature strip -->
	<section class="mx-auto mb-20 grid max-w-3xl grid-cols-1 gap-6 px-8 sm:grid-cols-3">
		{#each [{ icon: '🔄', title: 'Auto-updates', desc: 'Detects new image digests and rolls out without any manual steps.' }, { icon: '↩️', title: 'Auto-rollback', desc: 'Health check fails? Hoister restores the previous container automatically.' }, { icon: '🔔', title: 'Notifications', desc: 'Get alerts via Slack, Telegram, Discord, Teams, Mattermost, Gotify and more on every deploy.' }] as feature}
			<div class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
				<div class="mb-2 text-2xl">{feature.icon}</div>
				<div class="mb-1 font-semibold text-zinc-100">{feature.title}</div>
				<div class="text-sm leading-relaxed text-zinc-400">{feature.desc}</div>
			</div>
		{/each}
	</section>

	<!-- Connect your stack -->
	<section class="mx-auto mb-20 w-full max-w-3xl px-8">
		<div class="rounded-2xl border border-zinc-800 bg-zinc-900 p-6">
			<h2 class="mb-1 text-lg font-semibold text-zinc-100">Connect a Docker Compose stack</h2>
			<p class="mb-4 text-sm text-zinc-400">
				Sign in to <a href="/tokens" class="text-indigo-400 hover:text-indigo-300">/tokens</a> to
				mint an agent token, then add this service to your existing
				<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono text-xs">docker-compose.yaml</code>:
			</p>
			<div class="relative">
				<pre
					class="overflow-x-auto rounded-lg bg-zinc-950 p-4 font-mono text-xs leading-relaxed text-zinc-200">{composeSnippet}</pre>
				<button
					type="button"
					onclick={copySnippet}
					class="absolute top-3 right-3 rounded-md border border-zinc-700 bg-zinc-800 px-3 py-1 text-xs text-zinc-300 transition hover:border-zinc-500 hover:text-zinc-100"
				>
					{copied ? 'Copied!' : 'Copy'}
				</button>
			</div>
			<p class="mt-3 text-xs text-zinc-500">
				The <code class="rounded bg-zinc-800 px-1 py-0.5 font-mono">HOISTER_CONTROLLER_URL</code>
				override will become unnecessary once the public
				<code class="rounded bg-zinc-800 px-1 py-0.5 font-mono">emrius11/hoister:latest</code>
				image is bumped to the cloud-aware build.
			</p>
		</div>
	</section>

	<!-- Watchtower comparison -->
	<section class="mx-auto mb-20 w-full max-w-3xl px-8">
		<a
				href="https://docs.hoister.io/watchtower-alternative/"
				target="_blank"
				rel="noopener noreferrer"
				class="flex flex-col items-start justify-between gap-3 rounded-2xl border border-zinc-800 bg-zinc-900 p-6 transition hover:border-zinc-600 sm:flex-row sm:items-center"
		>
			<div>
				<h2 class="text-lg font-semibold text-zinc-100">Coming from Watchtower?</h2>
				<p class="mt-1 text-sm text-zinc-400">
					Hoister auto-updates containers the same way — but rolls back and restores volumes when an
					update fails its health check.
				</p>
			</div>
			<span class="shrink-0 text-sm font-medium text-indigo-400">Compare →</span>
		</a>
	</section>

	<!-- Contribute / bounty -->
	<section class="mx-auto mb-20 w-full max-w-3xl px-8">
		<div class="rounded-2xl border border-indigo-500/30 bg-indigo-500/10 p-6 text-center sm:p-8">
			<h2 class="mb-2 text-lg font-semibold text-zinc-100">
				Got an idea for a handy feature? Found a vulnerability?
			</h2>
			<p class="mx-auto mb-5 max-w-xl text-sm leading-relaxed text-zinc-300">
				Open an issue or send a report and get up to 3 years of Pro subscription for free.
			</p>
			<div class="flex flex-col items-center justify-center gap-3 sm:flex-row">
				<a
					href="https://github.com/HerrMuellerluedenscheid/hoister/issues/new"
					target="_blank"
					rel="noopener noreferrer"
					class="rounded-xl bg-white px-5 py-2.5 text-sm font-semibold text-zinc-900 transition hover:bg-zinc-100 active:scale-95"
				>
					Open an issue
				</a>
				<a
					href="https://github.com/HerrMuellerluedenscheid/hoister/security/advisories/new"
					target="_blank"
					rel="noopener noreferrer"
					class="rounded-xl border border-zinc-700 bg-zinc-900 px-5 py-2.5 text-sm font-semibold text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-800 active:scale-95"
				>
					Report a vulnerability
				</a>
			</div>
		</div>
	</section>

	<footer class="border-t border-zinc-800 py-5 text-center text-xs text-zinc-600">
		Hoister — open source on
		<a
			href="https://github.com/HerrMuellerluedenscheid/hoister"
			target="_blank"
			rel="noopener noreferrer"
			class="hover:text-zinc-400">GitHub</a
		>
		·
		<a
			href="https://docs.hoister.io"
			target="_blank"
			rel="noopener noreferrer"
			class="hover:text-zinc-400">Docs</a
		>
		·
		<a href="/impressum" class="hover:text-zinc-400">Impressum</a> ·
		<a href="/datenschutz" class="hover:text-zinc-400">Datenschutz</a>
	</footer>
</div>
