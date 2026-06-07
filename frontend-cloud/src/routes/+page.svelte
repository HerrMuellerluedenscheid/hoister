<script lang="ts">
	import { Show, SignInButton, UserButton } from 'svelte-clerk';

	const composeSnippet = `services:
  hoister:
    image: emrius11/hoister:latest
    volumes:
      - /var/run/docker.sock:/var/run/docker.sock
    security_opt:
      - no-new-privileges:true
    environment:
      # Until the public :latest image picks up the new default,
      # set the controller URL explicitly:
      HOISTER_CONTROLLER_URL: "https://api.hoister.io"
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
								class="rounded-lg border border-zinc-700 px-4 py-1.5 text-sm font-medium text-zinc-300 transition hover:border-zinc-500 hover:text-zinc-100"
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

		<Show when="signed-out">
			{#snippet children()}
				<div class="flex flex-col items-center gap-4 sm:flex-row">
					<SignInButton mode="modal">
						{#snippet children()}
							<button
								class="flex items-center gap-2.5 rounded-xl bg-white px-6 py-3 text-sm font-semibold text-zinc-900 shadow-lg transition hover:bg-zinc-100 active:scale-95"
							>
								<svg class="h-4 w-4" viewBox="0 0 24 24" fill="currentColor">
									<path
										d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0024 12c0-6.63-5.37-12-12-12z"
									/>
								</svg>
								Continue with GitHub
							</button>
						{/snippet}
					</SignInButton>

					<SignInButton mode="modal">
						{#snippet children()}
							<button
								class="flex items-center gap-2.5 rounded-xl border border-zinc-700 bg-zinc-900 px-6 py-3 text-sm font-semibold text-zinc-200 transition hover:border-zinc-500 hover:bg-zinc-800 active:scale-95"
							>
								<svg class="h-4 w-4" viewBox="0 0 24 24">
									<path
										d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
										fill="#4285F4"
									/>
									<path
										d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
										fill="#34A853"
									/>
									<path
										d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l3.66-2.84z"
										fill="#FBBC05"
									/>
									<path
										d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
										fill="#EA4335"
									/>
								</svg>
								Continue with Google
							</button>
						{/snippet}
					</SignInButton>
				</div>
			{/snippet}
		</Show>

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
