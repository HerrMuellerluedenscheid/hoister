<script lang="ts">
	import NotifierManager from '$lib/components/NotifierManager.svelte';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const project = $derived(data.project);
	const isOwner = $derived(project.role === 'owner');
</script>

<div class="space-y-8 px-4 py-6 sm:px-8 sm:py-10">
	<header>
		<nav class="mb-2 text-xs text-ink-faint">
			<a href="/projects" class="hover:text-ink-secondary">Projects</a>
			<span class="px-1 text-ink-ghost">/</span>
			<a href="/projects/{project.id}" class="hover:text-ink-secondary">{project.name}</a>
		</nav>
		<h1 class="text-2xl font-bold">Notifiers for {project.name}</h1>
		<p class="mt-1 text-sm text-ink-muted">
			These channels receive deployments, rollbacks, pending updates and metric alerts of
			<span class="font-medium text-ink-secondary">{project.name}</span> only — on top of
			{isOwner ? 'your' : "the owner's"}
			<a href="/notifiers" class="underline hover:text-ink">account-wide notifiers</a>. Everyone
			with access to this project can see and manage them.
		</p>
		{#if !isOwner}
			<p class="mt-2 text-xs text-ink-faint">
				Which kinds are available depends on the project owner's plan.
			</p>
		{/if}
	</header>

	<!-- Members aren't limited by their own plan here: the controller applies
	     the owner's, so let it decide and surface its answer. -->
	<NotifierManager
		notifiers={data.notifiers}
		me={isOwner ? data.me : null}
		{form}
		error={data.error}
		slackHref="/slack/oauth/start?project={encodeURIComponent(project.id)}"
		listTitle="Project notifiers"
		emptyText="No project notifiers yet. Add one above to route this project's events to a dedicated channel."
	/>
</div>
