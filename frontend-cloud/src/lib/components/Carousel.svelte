<script lang="ts">
	const slides = [
		{
			src: '/screenshots/containers_overview.png',
			title: 'Containers overview',
			caption: 'See every monitored container across your projects, with image, uptime and update status at a glance.'
		},
		{
			src: '/screenshots/recent_deployments_table.png',
			title: 'Recent deployments',
			caption: 'A live feed of rollouts per host and service — success, failure and rollback, with the exact image digest.'
		},
		{
			src: '/screenshots/resource_usage.png',
			title: 'Resource usage',
			caption: 'Per-container CPU, memory, network and disk I/O sampled by the agent over the last 7 days.'
		},
		{
			src: '/screenshots/services_on_network.png',
			title: 'Services on the network',
			caption: 'Discover the other services sharing a Docker network, with their project and IP address.'
		},
		{
			src: '/screenshots/notification_meu.png',
			title: 'Notifications',
			caption: 'Wire up Slack, Discord, Gotify and more in a couple of clicks, then test them right from the dashboard.'
		}
	];

	let current = $state(0);
	let paused = $state(false);

	function go(i: number) {
		current = (i + slides.length) % slides.length;
	}

	$effect(() => {
		if (paused) return;
		const id = setInterval(() => go(current + 1), 5000);
		return () => clearInterval(id);
	});
</script>

<div
	class="relative w-full"
	role="group"
	aria-roledescription="carousel"
	aria-label="Hoister dashboard screenshots"
	onmouseenter={() => (paused = true)}
	onmouseleave={() => (paused = false)}
>
	<div class="overflow-hidden rounded-2xl border border-line bg-card shadow-xl">
		<div
			class="flex transition-transform duration-500 ease-out"
			style="transform: translateX(-{current * 100}%)"
		>
			{#each slides as slide (slide.src)}
				<div class="w-full shrink-0">
					<div class="bg-canvas p-3 sm:p-5">
						<img
							src={slide.src}
							alt={slide.title}
							loading="lazy"
							class="mx-auto max-h-[420px] w-full rounded-lg object-contain"
						/>
					</div>
					<div class="border-t border-line px-5 py-4">
						<div class="text-sm font-semibold text-ink">{slide.title}</div>
						<p class="mt-0.5 text-sm text-ink-muted">{slide.caption}</p>
					</div>
				</div>
			{/each}
		</div>
	</div>

	<!-- Prev / next -->
	<button
		type="button"
		aria-label="Previous screenshot"
		onclick={() => go(current - 1)}
		class="absolute top-1/2 left-3 -translate-y-1/2 rounded-full border border-line-subtle bg-card/80 p-2 text-ink-secondary backdrop-blur transition hover:border-line-active hover:text-ink"
	>
		<svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
			<path stroke-linecap="round" stroke-linejoin="round" d="M15 19l-7-7 7-7" />
		</svg>
	</button>
	<button
		type="button"
		aria-label="Next screenshot"
		onclick={() => go(current + 1)}
		class="absolute top-1/2 right-3 -translate-y-1/2 rounded-full border border-line-subtle bg-card/80 p-2 text-ink-secondary backdrop-blur transition hover:border-line-active hover:text-ink"
	>
		<svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
			<path stroke-linecap="round" stroke-linejoin="round" d="M9 5l7 7-7 7" />
		</svg>
	</button>

	<!-- Dots -->
	<div class="mt-4 flex items-center justify-center gap-2">
		{#each slides as slide, i (slide.src)}
			<button
				type="button"
				aria-label="Go to screenshot {i + 1}"
				aria-current={i === current}
				onclick={() => go(i)}
				class="h-2 rounded-full transition-all {i === current
					? 'w-6 bg-brand-accent'
					: 'w-2 bg-line hover:bg-line-active'}"
			></button>
		{/each}
	</div>
</div>
