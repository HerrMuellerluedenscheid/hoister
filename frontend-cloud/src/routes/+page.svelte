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
    image: hoister/hoister:latest
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

<div class="relative flex min-h-screen flex-col bg-canvas text-ink">
	<!-- Decorative background blobs -->
	<div class="pointer-events-none absolute inset-0 overflow-hidden" aria-hidden="true">
		<div class="absolute top-0 left-1/2 h-[600px] w-[800px] -translate-x-1/2 -translate-y-1/4 rounded-full bg-brand/10 blur-[130px]"></div>
		<div class="absolute top-[6%] -right-36 h-[500px] w-[500px] rounded-full bg-brand-light/12 blur-[110px]"></div>
		<div class="absolute top-[55%] -left-32 h-[420px] w-[420px] rounded-full bg-success/8 blur-[100px]"></div>
	</div>

	<!-- Nav -->
	<header class="flex items-center justify-between px-8 py-5">
		<div class="flex items-center gap-2">
			<svg
				class="h-7 w-7 text-brand-accent"
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
				class="text-sm font-medium text-ink-secondary transition hover:text-ink"
			>
				Docs
			</a>
			<Show when="signed-out">
				{#snippet children()}
					<SignInButton mode="modal">
						{#snippet children()}
							<button
								class="rounded-lg border border-line-subtle bg-brand-light px-4 py-1.5 text-sm font-medium text-ink-inverse transition hover:border-line-active hover:text-ink"
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
						class="rounded-lg bg-brand px-4 py-1.5 text-sm font-medium text-stone-100 transition hover:bg-brand-hover"
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
			class="mb-4 inline-flex items-center gap-2 rounded-full border border-brand-hover/30 bg-brand-hover/10 px-3 py-1 text-xs font-medium text-brand-light"
		>
			<span class="h-1.5 w-1.5 rounded-full bg-brand-accent"></span>
			Automated container deployments
		</div>

		<h1 class="mb-5 max-w-2xl text-5xl font-bold tracking-tight text-ink">
			Deploy Docker images<br />
			<span class="text-brand-accent">without the toil</span>
		</h1>

		<p class="mb-10 max-w-lg text-base leading-relaxed text-ink-muted">
			Hoister watches your container registry and automatically updates running containers when a
			new image is pushed — with built-in rollback if something goes wrong.
		</p>

		<Show when="signed-in">
			{#snippet children()}
				<a
					href="/dashboard"
					class="rounded-xl bg-brand px-6 pt-3 pb-3 mb-5 text-sm font-semibold text-stone-100 shadow-lg transition hover:bg-brand-hover active:scale-95"
				>
					Go to Dashboard →
				</a>
			{/snippet}
		</Show>
	</main>

	<!-- Feature strip -->
	<section class="mx-auto mb-20 grid max-w-3xl grid-cols-1 gap-6 px-8 sm:grid-cols-3">
		{#each [{ icon: '🔄', title: 'Auto-updates', desc: 'Detects new image digests and rolls out without any manual steps.' }, { icon: '↩️', title: 'Auto-rollback', desc: 'Health check fails? Hoister restores the previous container automatically.' }, { icon: '🔔', title: 'Notifications', desc: 'Get alerts via Slack, Telegram, Discord, Teams, Mattermost, Gotify and more on every deploy.' }] as feature}
			<div class="rounded-xl border border-line bg-card p-5">
				<div class="mb-2 text-2xl">{feature.icon}</div>
				<div class="mb-1 font-semibold text-ink">{feature.title}</div>
				<div class="text-sm leading-relaxed text-ink-muted">{feature.desc}</div>
			</div>
		{/each}
	</section>

	<!-- Connect your stack -->
	<section class="mx-auto mb-20 w-full max-w-3xl px-8">
		<div class="rounded-2xl border border-line bg-card p-6">
			<h2 class="mb-1 text-lg font-semibold text-ink">Connect a Docker Compose stack</h2>
			<p class="mb-4 text-sm text-ink-muted">
				Sign in to <a href="/tokens" class="text-brand-accent hover:text-brand-light">/tokens</a> to
				mint an agent token, then add this service to your existing
				<code class="rounded bg-element px-1 py-0.5 font-mono text-xs">docker-compose.yaml</code>:
			</p>
			<div class="relative">
				<pre
					class="overflow-x-auto rounded-lg bg-canvas p-4 font-mono text-xs leading-relaxed text-ink-code">{composeSnippet}</pre>
				<button
					type="button"
					onclick={copySnippet}
					class="absolute top-3 right-3 rounded-md border border-line-subtle bg-element px-3 py-1 text-xs text-ink-secondary transition hover:border-line-active hover:text-ink"
				>
					{copied ? 'Copied!' : 'Copy'}
				</button>
			</div>
			<p class="mt-3 text-xs text-ink-faint">
				The <code class="rounded bg-element px-1 py-0.5 font-mono">HOISTER_CONTROLLER_URL</code>
				override will become unnecessary once the public
				<code class="rounded bg-element px-1 py-0.5 font-mono">hoister/hoister:latest</code>
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
				class="flex flex-col items-start justify-between gap-3 rounded-2xl border border-line bg-card p-6 transition hover:border-line-subtle sm:flex-row sm:items-center"
		>
			<div>
				<h2 class="text-lg font-semibold text-ink">Coming from Watchtower?</h2>
				<p class="mt-1 text-sm text-ink-muted">
					Hoister auto-updates containers the same way — but rolls back and restores volumes when an
					update fails its health check.
				</p>
			</div>
			<span class="shrink-0 text-sm font-medium text-brand-accent">Compare →</span>
		</a>
	</section>

	<!-- Contribute / bounty -->
	<section class="mx-auto mb-20 w-full max-w-3xl px-8">
		<div class="rounded-2xl border border-brand-hover/30 bg-brand-hover/10 p-6 text-center sm:p-8">
			<h2 class="mb-2 text-lg font-semibold text-ink">
				Got an idea for a handy feature? Found a vulnerability?
			</h2>
			<p class="mx-auto mb-5 max-w-xl text-sm leading-relaxed text-ink-secondary">
				Open an issue or send a report and get up to 3 years of Pro subscription for free.
			</p>
			<div class="flex flex-col items-center justify-center gap-3 sm:flex-row">
				<a
					href="https://github.com/HerrMuellerluedenscheid/hoister/issues/new"
					target="_blank"
					rel="noopener noreferrer"
					class="rounded-xl border border-line bg-element px-5 py-2.5 text-sm font-semibold text-ink transition hover:bg-line hover:border-line-subtle active:scale-95"
				>
					Open an issue
				</a>
				<a
					href="https://github.com/HerrMuellerluedenscheid/hoister/security/advisories/new"
					target="_blank"
					rel="noopener noreferrer"
					class="rounded-xl border border-line-subtle bg-card px-5 py-2.5 text-sm font-semibold text-ink-code transition hover:border-line-active hover:bg-element active:scale-95"
				>
					Report a vulnerability
				</a>
			</div>
		</div>
	</section>

	<footer class="border-t border-line py-5 text-center text-xs text-ink-ghost">
		Hoister — open source on
		<a
			href="https://github.com/HerrMuellerluedenscheid/hoister"
			target="_blank"
			rel="noopener noreferrer"
			class="hover:text-ink-muted">GitHub</a
		>
		·
		<a
			href="https://docs.hoister.io"
			target="_blank"
			rel="noopener noreferrer"
			class="hover:text-ink-muted">Docs</a
		>
		·
		<a href="/impressum" class="hover:text-ink-muted">Impressum</a> ·
		<a href="/datenschutz" class="hover:text-ink-muted">Datenschutz</a>
	</footer>
</div>
