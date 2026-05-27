<script lang="ts">
	import type { PageProps } from './$types';

	let { data }: PageProps = $props();
	const me = $derived(data.me);
</script>

<div class="space-y-8 px-8 py-10">
	<header>
		<h1 class="text-2xl font-bold">Plan</h1>
		<p class="mt-1 text-sm text-zinc-400">
			Your current Hoister plan and the limits that apply to it.
		</p>
	</header>

	{#if data.meError}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			{data.meError}
		</div>
	{:else if me}
		<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
			<div class="flex items-center justify-between">
				<div>
					<div class="text-xs tracking-wider text-zinc-400 uppercase">Current plan</div>
					<div class="mt-1 text-xl font-semibold capitalize text-zinc-100">{me.plan}</div>
				</div>
				{#if me.plan === 'free'}
					<button
						type="button"
						disabled
						title="Stripe checkout not wired yet"
						class="rounded-md bg-indigo-500 px-4 py-2 text-sm font-semibold text-white opacity-50"
					>
						Upgrade to Pro
					</button>
				{:else}
					<span class="rounded-full bg-indigo-500/15 px-3 py-1 text-xs font-medium text-indigo-300">
						Pro
					</span>
				{/if}
			</div>
		</section>

		<section>
			<h2 class="mb-3 text-base font-semibold text-zinc-200">Usage</h2>
			<div class="overflow-hidden rounded-xl border border-zinc-800">
				<table class="min-w-full divide-y divide-zinc-800 text-sm">
					<tbody class="divide-y divide-zinc-800 bg-zinc-950">
						<tr class="text-zinc-300">
							<td class="px-4 py-3 text-zinc-400">Compose projects</td>
							<td class="px-4 py-3">
								{me.usage.projects} /
								{me.limits.max_projects === null ? 'unlimited' : me.limits.max_projects}
							</td>
						</tr>
						<tr class="text-zinc-300">
							<td class="px-4 py-3 text-zinc-400">Notifier kinds available</td>
							<td class="px-4 py-3">
								{me.limits.allowed_notifier_kinds.join(', ')}
							</td>
						</tr>
						<tr class="text-zinc-300">
							<td class="px-4 py-3 text-zinc-400">Notifiers configured</td>
							<td class="px-4 py-3">
								{#if Object.keys(me.usage.notifiers_by_kind).length === 0}
									<span class="text-zinc-600">—</span>
								{:else}
									{Object.entries(me.usage.notifiers_by_kind)
										.map(([k, v]) => `${v} ${k}`)
										.join(', ')}
								{/if}
							</td>
						</tr>
					</tbody>
				</table>
			</div>
		</section>

		{#if me.plan === 'free'}
			<section class="rounded-xl border border-indigo-500/30 bg-indigo-500/5 p-5">
				<h2 class="text-base font-semibold text-indigo-300">What Pro unlocks</h2>
				<ul class="mt-3 list-disc space-y-1 pl-5 text-sm text-zinc-300">
					<li>Unlimited compose projects (Free is capped at 2)</li>
					<li>Slack, Gotify, and Email notifiers</li>
					<li>Same controller, same agent — just lift the caps</li>
				</ul>
				<p class="mt-3 text-xs text-zinc-500">
					Billing through Stripe is not wired yet. Upgrade button will activate once it lands.
				</p>
			</section>
		{/if}
	{/if}
</div>
