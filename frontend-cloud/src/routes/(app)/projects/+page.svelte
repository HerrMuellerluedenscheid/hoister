<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import ProjectTile from '$lib/components/ProjectTile.svelte';
	import { projectHealth, timeAgo } from '$lib/projectHealth';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	let answering = $state<string | null>(null);

	function inviterName(userId: string): string {
		const p = data.inviters[userId];
		if (p?.name && p.email) return `${p.name} (${p.email})`;
		return p?.name ?? p?.email ?? 'Someone';
	}

	let now = $state(Date.now());
	let refreshInterval: ReturnType<typeof setInterval>;

	onMount(() => {
		refreshInterval = setInterval(() => {
			now = Date.now();
			invalidateAll();
		}, 10_000);
	});

	onDestroy(() => clearInterval(refreshInterval));

	const owned = $derived(data.projects.filter((p) => p.role === 'owner'));
	const shared = $derived(data.projects.filter((p) => p.role === 'member'));

	const attention = $derived(
		data.projects.filter((p) => {
			const h = projectHealth(p, now);
			return h === 'down' || h === 'stale';
		}).length
	);

	const max = $derived(data.me?.limits.max_projects ?? null);
	const atProjectLimit = $derived(
		data.me?.plan === 'free' && max !== null && data.me.usage.projects >= max
	);
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-7xl space-y-8">
		<div class="flex flex-wrap items-baseline justify-between gap-4">
			<div>
				<h1 class="text-2xl font-bold">Projects</h1>
				{#if data.projects.length > 0}
					<p class="mt-1 text-sm text-ink-muted">
						{data.projects.length} project{data.projects.length === 1 ? '' : 's'}
						{#if attention > 0}
							· <span class="font-medium text-error"
								>{attention} need{attention === 1 ? 's' : ''} attention</span
							>
						{:else}
							· all reporting
						{/if}
					</p>
				{/if}
			</div>
			{#if data.me}
				{@const used = data.me.usage.projects}
				<div class="text-sm text-ink-muted">
					{#if max === null}
						{used} project{used === 1 ? '' : 's'} <span class="text-brand-accent">(Pro)</span>
					{:else}
						<span class:text-warning={used >= max}>
							{used} / {max} projects
						</span>
					{/if}
				</div>
			{/if}
		</div>

		{#if data.invitations.length > 0}
			<section class="space-y-3" aria-labelledby="invitations-heading">
				<h2
					id="invitations-heading"
					class="text-sm font-semibold tracking-wider text-ink-muted uppercase"
				>
					Invitations
				</h2>
				{#if form?.invitationError}
					<div
						class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error"
					>
						{form.invitationError}
					</div>
				{/if}
				<ul class="space-y-2">
					{#each data.invitations as invitation (invitation.id)}
						<li
							class="flex flex-col gap-3 rounded-xl border border-brand-light/40 bg-brand-light/5 px-4 py-3 sm:flex-row sm:items-center sm:justify-between"
						>
							<div class="min-w-0 text-sm">
								<p class="text-ink">
									<span class="font-medium">{inviterName(invitation.invited_by)}</span>
									invited you to co-maintain
									<span class="font-semibold">{invitation.project_name}</span>
									<span class="text-ink-faint">on {invitation.hostname}</span>
								</p>
								<p class="mt-0.5 text-xs text-ink-faint">
									{timeAgo(invitation.created_at, now)} · you'll see its services, deployments and resource
									usage, can deploy pending updates and manage its notifiers
								</p>
							</div>
							<div class="flex shrink-0 gap-2">
								<form
									method="POST"
									action="?/decline"
									use:enhance={() => {
										answering = invitation.id;
										return async ({ update }) => {
											await update();
											answering = null;
										};
									}}
								>
									<input type="hidden" name="invitation_id" value={invitation.id} />
									<button
										type="submit"
										disabled={answering === invitation.id}
										class="rounded-md border border-line-subtle bg-canvas px-3 py-1.5 text-sm font-medium text-ink-secondary transition hover:bg-element disabled:opacity-50"
									>
										Decline
									</button>
								</form>
								<form
									method="POST"
									action="?/accept"
									use:enhance={() => {
										answering = invitation.id;
										return async ({ update }) => {
											await update();
											answering = null;
										};
									}}
								>
									<input type="hidden" name="invitation_id" value={invitation.id} />
									<button
										type="submit"
										disabled={answering === invitation.id}
										class="rounded-md bg-brand-hover px-3 py-1.5 text-sm font-semibold text-white transition hover:bg-brand-accent disabled:opacity-50"
									>
										{answering === invitation.id ? 'Working…' : 'Accept'}
									</button>
								</form>
							</div>
						</li>
					{/each}
				</ul>
			</section>
		{/if}

		{#if atProjectLimit && max !== null}
			<div
				class="flex flex-col gap-3 rounded-xl border border-warning-border/50 bg-warning-bg px-4 py-4 text-sm text-warning sm:flex-row sm:items-center sm:justify-between"
			>
				<p>
					To monitor more than {max} project{max === 1 ? '' : 's'}, upgrade to Pro. You can also
					delete a project from its page to free up a slot.
				</p>
				<form method="POST" action="?/upgrade" class="shrink-0">
					<button
						type="submit"
						class="rounded-md bg-brand-hover px-4 py-2 text-sm font-semibold whitespace-nowrap text-white transition hover:bg-brand-accent"
					>
						Upgrade to Pro
					</button>
				</form>
			</div>
		{/if}

		{#if form?.upgradeError}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				<span class="font-medium">Upgrade failed:</span>
				{form.upgradeError}
			</div>
		{/if}

		{#if data.projectsError}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				<span class="font-medium">Error:</span>
				{data.projectsError}
			</div>
		{/if}

		{#if data.projects.length === 0 && data.invitations.length === 0 && !data.projectsError}
			<div class="rounded-xl border border-line bg-card px-5 py-4 text-sm text-ink-muted">
				No projects reporting yet. Make sure an agent is connected and your containers are labelled
				with
				<code class="rounded bg-element px-1 py-0.5 font-mono text-xs">hoister.enable=true</code>.
			</div>
		{/if}

		{#if owned.length > 0}
			<section class="space-y-3">
				{#if shared.length > 0}
					<h2 class="text-sm font-semibold tracking-wider text-ink-muted uppercase">
						Your projects
					</h2>
				{/if}
				<div class="grid grid-cols-[repeat(auto-fill,minmax(300px,1fr))] gap-4">
					{#each owned as project (project.id)}
						<ProjectTile {project} {now} />
					{/each}
				</div>
			</section>
		{/if}

		{#if shared.length > 0}
			<section class="space-y-3">
				<h2 class="text-sm font-semibold tracking-wider text-ink-muted uppercase">
					Shared with you
				</h2>
				<div class="grid grid-cols-[repeat(auto-fill,minmax(300px,1fr))] gap-4">
					{#each shared as project (project.id)}
						<ProjectTile {project} {now} />
					{/each}
				</div>
			</section>
		{/if}
	</div>
</div>
