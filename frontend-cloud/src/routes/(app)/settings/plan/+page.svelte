<script lang="ts">
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
	const me = $derived(data.me);
</script>

<div class="space-y-8 px-4 py-6 sm:px-8 sm:py-10">
	<header>
		<h1 class="text-2xl font-bold">Plan</h1>
		<p class="mt-1 text-sm text-zinc-400">
			Your current Hoister plan and the limits that apply to it.
		</p>
	</header>

	{#if data.checkout === 'success'}
		<div
			class="rounded-xl border border-emerald-800 bg-emerald-950/30 px-4 py-3 text-sm text-emerald-300"
		>
			Subscription started — welcome to Pro! It can take a moment for the status to update here.
		</div>
	{:else if data.checkout === 'cancelled'}
		<div class="rounded-xl border border-amber-800 bg-amber-950/30 px-4 py-3 text-sm text-amber-300">
			Checkout cancelled — no charge was made.
		</div>
	{/if}

	{#if form?.error}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			{form.error}
		</div>
	{/if}

	{#if data.meError}
		<div class="rounded-xl border border-red-800 bg-red-950/40 px-4 py-3 text-sm text-red-400">
			{data.meError}
		</div>
	{:else if me}
		<section class="rounded-xl border border-zinc-800 bg-zinc-900 p-5">
			<div class="flex items-center justify-between gap-4">
				<div>
					<div class="text-xs tracking-wider text-zinc-400 uppercase">Current plan</div>
					<div class="mt-1 text-xl font-semibold text-zinc-100 capitalize">{me.plan}</div>
					{#if me.plan === 'pro' && data.subscriptionStatus}
						<div class="mt-1 text-xs text-zinc-500">
							Subscription status: <span class="text-zinc-300">{data.subscriptionStatus}</span>
						</div>
					{/if}
				</div>

				{#if me.plan === 'free'}
					<form method="POST" action="?/upgrade">
						<button
							type="submit"
							disabled={!data.stripeReady}
							title={data.stripeReady ? '' : 'Billing is not configured yet'}
							class="rounded-md bg-indigo-500 px-4 py-2 text-sm font-semibold text-white transition hover:bg-indigo-400 disabled:cursor-not-allowed disabled:opacity-50"
						>
							Upgrade to Pro
						</button>
					</form>
				{:else}
					<form method="POST" action="?/manage">
						<button
							type="submit"
							disabled={!data.stripeReady || !data.hasCustomer}
							class="rounded-md border border-zinc-700 px-4 py-2 text-sm font-medium text-zinc-200 transition hover:bg-zinc-800 disabled:cursor-not-allowed disabled:opacity-50"
						>
							Manage subscription
						</button>
					</form>
				{/if}
			</div>
		</section>

		<section>
			<h2 class="mb-3 text-base font-semibold text-zinc-200">Usage</h2>
			<div class="overflow-x-auto rounded-xl border border-zinc-800">
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
			</section>
		{/if}
	{/if}
</div>
