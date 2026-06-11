<script lang="ts">
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();
	const me = $derived(data.me);
</script>

<div class="space-y-8 px-4 py-6 sm:px-8 sm:py-10">
	<header>
		<h1 class="text-2xl font-bold">Plan</h1>
		<p class="mt-1 text-sm text-ink-muted">
			Your current Hoister plan and the limits that apply to it.
		</p>
	</header>

	{#if data.checkout === 'success'}
		<div
			class="rounded-xl border border-success-border bg-success-bg px-4 py-3 text-sm text-success"
		>
			Subscription started — welcome to Pro! It can take a moment for the status to update here.
		</div>
	{:else if data.checkout === 'cancelled'}
		<div class="rounded-xl border border-warning-border bg-warning-bg px-4 py-3 text-sm text-warning">
			Checkout cancelled — no charge was made.
		</div>
	{/if}

	{#if form?.error}
		<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
			{form.error}
		</div>
	{/if}

	{#if data.meError}
		<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
			{data.meError}
		</div>
	{:else if me}
		<section class="rounded-xl border border-line bg-card p-5">
			<div class="flex items-center justify-between gap-4">
				<div>
					<div class="text-xs tracking-wider text-ink-muted uppercase">Current plan</div>
					<div class="mt-1 text-xl font-semibold text-ink capitalize">{me.plan}</div>
					{#if me.plan === 'pro' && data.subscriptionStatus}
						<div class="mt-1 text-xs text-ink-faint">
							Subscription status: <span class="text-ink-secondary">{data.subscriptionStatus}</span>
						</div>
					{/if}
				</div>

				{#if me.plan === 'free'}
					<form method="POST" action="?/upgrade">
						<button
							type="submit"
							disabled={!data.stripeReady}
							title={data.stripeReady ? '' : 'Billing is not configured yet'}
							class="rounded-md bg-brand-hover px-4 py-2 text-sm font-semibold text-white transition hover:bg-brand-accent disabled:cursor-not-allowed disabled:opacity-50"
						>
							Upgrade to Pro
						</button>
					</form>
				{:else}
					<form method="POST" action="?/manage">
						<button
							type="submit"
							disabled={!data.stripeReady || !data.hasCustomer}
							class="rounded-md border border-line-subtle px-4 py-2 text-sm font-medium text-ink-code transition hover:bg-element disabled:cursor-not-allowed disabled:opacity-50"
						>
							Manage subscription
						</button>
					</form>
				{/if}
			</div>
		</section>

		<section>
			<h2 class="mb-3 text-base font-semibold text-ink-code">Usage</h2>
			<div class="overflow-x-auto rounded-xl border border-line">
				<table class="min-w-full divide-y divide-line text-sm">
					<tbody class="divide-y divide-line bg-canvas">
						<tr class="text-ink-secondary">
							<td class="px-4 py-3 text-ink-muted">Compose projects</td>
							<td class="px-4 py-3">
								{me.usage.projects} /
								{me.limits.max_projects === null ? 'unlimited' : me.limits.max_projects}
							</td>
						</tr>
						<tr class="text-ink-secondary">
							<td class="px-4 py-3 text-ink-muted">Notifier kinds available</td>
							<td class="px-4 py-3">
								{me.limits.allowed_notifier_kinds.join(', ')}
							</td>
						</tr>
						<tr class="text-ink-secondary">
							<td class="px-4 py-3 text-ink-muted">Notifiers configured</td>
							<td class="px-4 py-3">
								{#if Object.keys(me.usage.notifiers_by_kind).length === 0}
									<span class="text-ink-ghost">—</span>
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
			<section class="rounded-xl border border-brand-hover/30 bg-brand-hover/5 p-5">
				<h2 class="text-base font-semibold text-brand-light">What Pro unlocks</h2>
				<ul class="mt-3 list-disc space-y-1 pl-5 text-sm text-ink-secondary">
					<li>Unlimited compose projects (Free is capped at 2)</li>
					<li>Slack, Gotify, and Email notifiers</li>
					<li>Same controller, same agent — just lift the caps</li>
				</ul>
			</section>
		{/if}
	{/if}
</div>
