<script lang="ts">
	import { enhance } from '$app/forms';
	import { invalidateAll } from '$app/navigation';
	import { onDestroy, onMount } from 'svelte';
	import Deployments from '$lib/components/Deployments.svelte';
	import InspectionCard from '$lib/components/InspectionCard.svelte';
	import PendingUpdates from '$lib/components/PendingUpdates.svelte';
	import {
		HEALTH_BADGE,
		HEALTH_DOT,
		HEALTH_LABEL,
		formatBytes,
		projectHealth,
		timeAgo
	} from '$lib/projectHealth';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const project = $derived(data.detail.project);
	const isOwner = $derived(project.role === 'owner');
	const base = $derived(`/projects/${project.id}`);

	let now = $state(Date.now());
	let refreshInterval: ReturnType<typeof setInterval>;
	onMount(() => {
		refreshInterval = setInterval(() => {
			now = Date.now();
			invalidateAll();
		}, 10_000);
	});
	onDestroy(() => clearInterval(refreshInterval));

	const health = $derived(projectHealth(project, now));
	const running = $derived(project.services.filter((s) => s.status === 'running').length);

	function initials(name: string | null | undefined, fallback: string): string {
		const source = (name || fallback).trim();
		const parts = source.split(/[\s@._-]+/).filter(Boolean);
		return ((parts[0]?.[0] ?? '?') + (parts[1]?.[0] ?? '')).toUpperCase();
	}

	let confirming = $state(false);
	let working = $state(false);
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-7xl space-y-8">
		<!-- Header -->
		<div class="space-y-4">
			<nav class="text-xs text-ink-faint">
				<a href="/projects" class="hover:text-ink-secondary">Projects</a>
				<span class="px-1 text-ink-ghost">/</span>
				<span class="text-ink-muted">{project.name}</span>
			</nav>

			<div class="flex flex-wrap items-start justify-between gap-4">
				<div class="min-w-0">
					<div class="flex flex-wrap items-center gap-3">
						<h1 class="truncate text-2xl font-bold">{project.name}</h1>
						<span
							class="inline-flex items-center gap-1.5 rounded-full border px-2 py-0.5 text-xs font-medium {HEALTH_BADGE[
								health
							]}"
						>
							<span class="h-1.5 w-1.5 rounded-full {HEALTH_DOT[health]}"></span>
							{HEALTH_LABEL[health]}
						</span>
						{#if !isOwner}
							<span
								class="rounded-full border border-line-subtle px-2 py-0.5 text-xs font-medium text-ink-muted"
							>
								Shared with you
							</span>
						{/if}
					</div>
					<p class="mt-1 text-sm text-ink-muted">
						Host <span class="font-mono text-ink-secondary">{project.hostname}</span>
						{#if project.last_updated}
							· <span class={health === 'stale' ? 'text-warning' : ''}
								>{health === 'stale' ? 'last report' : 'updated'}
								{timeAgo(project.last_updated, now)}</span
							>
						{/if}
					</p>
				</div>

				<div class="flex flex-wrap items-center gap-2">
					<!-- Who has access -->
					<a
						href="{base}/sharing"
						class="mr-1 flex -space-x-2"
						title="{data.detail.members.length} {data.detail.members.length === 1
							? 'person has'
							: 'people have'} access"
					>
						{#each data.detail.members.slice(0, 5) as member (member.user_id)}
							{@const person = data.people[member.user_id]}
							{#if person?.imageUrl}
								<img
									src={person.imageUrl}
									alt={person.name ?? person.email ?? 'Member'}
									class="h-7 w-7 rounded-full border-2 border-canvas object-cover"
								/>
							{:else}
								<span
									class="flex h-7 w-7 items-center justify-center rounded-full border-2 border-canvas bg-element text-[10px] font-semibold text-ink-muted"
								>
									{initials(person?.name, person?.email ?? member.user_id)}
								</span>
							{/if}
						{/each}
						{#if data.detail.members.length > 5}
							<span
								class="flex h-7 w-7 items-center justify-center rounded-full border-2 border-canvas bg-element text-[10px] font-semibold text-ink-muted"
							>
								+{data.detail.members.length - 5}
							</span>
						{/if}
					</a>

					<a
						href="{base}/notifiers"
						class="inline-flex items-center gap-2 rounded-md border border-line-subtle bg-canvas px-3 py-1.5 text-sm font-medium text-ink-secondary transition hover:border-line-active hover:text-ink"
					>
						<svg
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M15 17h5l-1.4-1.4A2 2 0 0118 14.2V11a6 6 0 00-4-5.7V5a2 2 0 10-4 0v.3A6 6 0 006 11v3.2c0 .5-.2 1-.6 1.4L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9"
							/>
						</svg>
						Notifiers
						{#if project.notifier_count > 0}
							<span class="rounded-full bg-element px-1.5 text-xs text-ink-muted"
								>{project.notifier_count}</span
							>
						{/if}
					</a>
					<a
						href="{base}/sharing"
						class="inline-flex items-center gap-2 rounded-md bg-brand-hover px-3 py-1.5 text-sm font-semibold text-white transition hover:bg-brand-accent"
					>
						<svg
							class="h-4 w-4"
							fill="none"
							viewBox="0 0 24 24"
							stroke="currentColor"
							stroke-width="2"
						>
							<path
								stroke-linecap="round"
								stroke-linejoin="round"
								d="M18 9v3m0 0v3m0-3h3m-3 0h-3m-2-5a4 4 0 11-8 0 4 4 0 018 0zM3 20a6 6 0 0112 0v1H3v-1z"
							/>
						</svg>
						{isOwner ? 'Share' : 'Members'}
					</a>
				</div>
			</div>
		</div>

		{#if form?.applyError}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				<span class="font-medium">Deploy failed:</span>
				{form.applyError}
			</div>
		{/if}
		{#if form?.deleteError || form?.leaveError}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				{form.deleteError ?? form.leaveError}
			</div>
		{/if}

		<!-- At a glance -->
		<dl class="grid grid-cols-2 gap-3 md:grid-cols-4">
			<div class="rounded-xl border border-line bg-card px-4 py-3">
				<dt class="text-xs text-ink-faint">Services running</dt>
				<dd class="mt-1 text-lg font-semibold text-ink">
					{running}<span class="text-sm font-normal text-ink-faint">
						/ {project.services.length}</span
					>
				</dd>
			</div>
			<div class="rounded-xl border border-line bg-card px-4 py-3">
				<dt class="text-xs text-ink-faint">CPU</dt>
				<dd class="mt-1 font-mono text-lg font-semibold text-ink">
					{project.metrics ? `${project.metrics.cpu_pct.toFixed(1)} %` : '—'}
				</dd>
			</div>
			<div class="rounded-xl border border-line bg-card px-4 py-3">
				<dt class="text-xs text-ink-faint">Memory</dt>
				<dd class="mt-1 font-mono text-lg font-semibold text-ink">
					{project.metrics ? formatBytes(project.metrics.mem_bytes) : '—'}
				</dd>
			</div>
			<div class="rounded-xl border border-line bg-card px-4 py-3">
				<dt class="text-xs text-ink-faint">Latest update</dt>
				<dd class="mt-1 truncate text-sm text-ink">
					{#if project.latest_deployment}
						<span class="font-medium">{project.latest_deployment.service_name}</span>
						<span class="text-ink-faint"
							>· {project.latest_deployment.status === 'RollbackFinished'
								? 'rolled back'
								: project.latest_deployment.status.toLowerCase()}
							{timeAgo(project.latest_deployment.created_at, now)}</span
						>
					{:else}
						<span class="text-ink-faint">No rollouts yet</span>
					{/if}
				</dd>
			</div>
		</dl>

		<PendingUpdates updates={data.pendingUpdates} />

		<!-- Services -->
		<section class="space-y-3">
			<h2 class="text-base font-semibold text-ink-code">Services</h2>
			{#if data.servicesError}
				<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
					{data.servicesError}
				</div>
			{:else if data.services.length === 0}
				<div class="rounded-xl border border-line bg-card px-5 py-4 text-sm text-ink-muted">
					This project hasn't reported any container state yet.
				</div>
			{:else}
				<div class="grid grid-cols-[repeat(auto-fit,minmax(300px,1fr))] gap-4">
					{#each data.services as inspection_data (inspection_data.service_name)}
						<InspectionCard
							{inspection_data}
							href="{base}/services/{encodeURIComponent(inspection_data.service_name)}"
						/>
					{/each}
				</div>
			{/if}
		</section>

		<!-- Deployments -->
		<section class="space-y-3">
			<h2 class="text-base font-semibold text-ink-code">Recent deployments</h2>
			<div class="max-h-[32rem] overflow-y-auto">
				<Deployments
					data={data.deployments.slice(0, 20)}
					linkToContainer
					hrefFor={(d) => `${base}/services/${encodeURIComponent(d.service_name)}`}
				/>
			</div>
		</section>

		<!-- Danger zone -->
		<section class="rounded-xl border border-line bg-card p-5">
			{#if isOwner}
				<h2 class="text-base font-semibold text-ink-code">Delete project</h2>
				<p class="mt-1 text-sm text-ink-muted">
					Removes the project's state, metrics, deployment history, project notifiers and everyone's
					access to it, freeing a slot against your plan. If the agent still reports it, the project
					comes back — unshared and without history.
				</p>
			{:else}
				<h2 class="text-base font-semibold text-ink-code">Leave project</h2>
				<p class="mt-1 text-sm text-ink-muted">
					You'll lose access until the owner shares it with you again.
				</p>
			{/if}
			<form
				method="POST"
				action={isOwner ? '?/deleteProject' : '?/leave'}
				class="mt-3 flex items-center gap-2"
				use:enhance={() => {
					working = true;
					return async ({ update }) => {
						await update();
						working = false;
						confirming = false;
					};
				}}
			>
				{#if confirming}
					<span class="text-xs text-ink-muted">Are you sure?</span>
					<button
						type="submit"
						disabled={working}
						class="rounded-md bg-red-600 px-3 py-1.5 text-xs font-semibold text-white transition hover:bg-red-500 disabled:opacity-50"
					>
						{working ? 'Working…' : isOwner ? 'Delete project' : 'Leave project'}
					</button>
					<button
						type="button"
						onclick={() => (confirming = false)}
						disabled={working}
						class="rounded-md border border-line-subtle px-3 py-1.5 text-xs font-medium text-ink-secondary transition hover:bg-element disabled:opacity-50"
					>
						Cancel
					</button>
				{:else}
					<button
						type="button"
						onclick={() => (confirming = true)}
						class="rounded-md border border-error-border px-3 py-1.5 text-xs font-medium text-error transition hover:bg-error-bg"
					>
						{isOwner ? 'Delete project…' : 'Leave project…'}
					</button>
				{/if}
			</form>
		</section>
	</div>
</div>
