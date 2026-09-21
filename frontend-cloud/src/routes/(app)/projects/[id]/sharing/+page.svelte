<script lang="ts">
	import { enhance } from '$app/forms';
	import { parseTimestamp } from '$lib/projectHealth';
	import type { PageProps } from './$types';

	let { data, form }: PageProps = $props();

	const project = $derived(data.detail.project);
	const isOwner = $derived(project.role === 'owner');

	let inviting = $state(false);
	let busyId = $state<string | null>(null);

	function displayName(userId: string): string {
		const p = data.people[userId];
		return p?.name ?? p?.email ?? 'Unknown user';
	}

	function initials(userId: string): string {
		const p = data.people[userId];
		const source = (p?.name || p?.email || '?').trim();
		const parts = source.split(/[\s@._-]+/).filter(Boolean);
		return ((parts[0]?.[0] ?? '?') + (parts[1]?.[0] ?? '')).toUpperCase();
	}

	function formatDate(raw: string): string {
		const t = parseTimestamp(raw);
		return isNaN(t) ? raw : new Date(t).toLocaleDateString();
	}

	function busyForm(id: string) {
		return () => {
			busyId = id;
			return async ({ update }: { update: () => Promise<void> }) => {
				await update();
				busyId = null;
			};
		};
	}
</script>

<div class="px-4 py-6 sm:px-8 sm:py-10">
	<div class="mx-auto max-w-3xl space-y-8">
		<header>
			<nav class="mb-2 text-xs text-ink-faint">
				<a href="/projects" class="hover:text-ink-secondary">Projects</a>
				<span class="px-1 text-ink-ghost">/</span>
				<a href="/projects/{project.id}" class="hover:text-ink-secondary">{project.name}</a>
			</nav>
			<h1 class="text-2xl font-bold">{isOwner ? 'Share' : 'Members of'} {project.name}</h1>
			<p class="mt-1 text-sm text-ink-muted">
				Co-maintainers see this project's services, deployments, resource usage and logs, can deploy
				pending updates, and manage its
				<a href="/projects/{project.id}/notifiers" class="underline hover:text-ink">notifiers</a>.
				Only the owner can invite or remove people and delete the project. Shared projects don't
				count against your co-maintainers' plans.
			</p>
		</header>

		{#if isOwner}
			<section class="rounded-xl border border-line bg-card p-5">
				<h2 class="text-base font-semibold text-ink-code">Invite someone</h2>
				<p class="mt-1 mb-3 text-sm text-ink-muted">
					People with a Hoister account get access right away. Everyone else gets an email inviting
					them to sign up, and access once they do.
				</p>
				<form
					method="POST"
					action="?/invite"
					class="ph-no-capture flex flex-col gap-2 sm:flex-row"
					use:enhance={() => {
						inviting = true;
						return async ({ update, result }) => {
							await update({ reset: result.type === 'success' });
							inviting = false;
						};
					}}
				>
					<label for="invite-email" class="sr-only">Email address</label>
					<input
						id="invite-email"
						type="email"
						name="email"
						required
						autocomplete="off"
						placeholder="colleague@example.com"
						value={form?.email ?? ''}
						class="min-w-0 flex-1 rounded-md border border-line-subtle bg-canvas px-3 py-2 text-sm text-ink placeholder:text-ink-faint"
					/>
					<button
						type="submit"
						disabled={inviting}
						class="rounded-md bg-brand-hover px-4 py-2 text-sm font-semibold text-white transition hover:bg-brand-accent disabled:opacity-50"
					>
						{inviting ? 'Inviting…' : 'Invite'}
					</button>
				</form>

				{#if form?.inviteError}
					<p class="mt-3 text-sm text-error">{form.inviteError}</p>
				{:else if form?.invite}
					{@const invite = form.invite}
					<p
						class="mt-3 text-sm {invite.kind === 'added' || invite.kind === 'invited'
							? 'text-success'
							: 'text-ink-muted'}"
					>
						{#if invite.kind === 'added'}
							{invite.name ?? invite.email} now has access to {project.name}.
						{:else if invite.kind === 'already-member'}
							{invite.email} already has access.
						{:else if invite.kind === 'invited'}
							Invitation sent to {invite.email}. They'll get access as soon as they sign up with
							this address.
						{:else if invite.kind === 'already-invited'}
							{invite.email} has already been invited. Revoke the invitation below to send a new one.
						{/if}
					</p>
				{/if}
			</section>
		{/if}

		{#if form?.memberError}
			<div class="rounded-xl border border-error-border bg-error-bg px-4 py-3 text-sm text-error">
				{form.memberError}
			</div>
		{/if}

		<!-- People with access -->
		<section>
			<h2 class="mb-3 text-base font-semibold text-ink-code">
				People with access ({data.detail.members.length})
			</h2>
			<ul class="divide-y divide-line overflow-hidden rounded-xl border border-line bg-card">
				{#each data.detail.members as member (member.user_id)}
					{@const person = data.people[member.user_id]}
					{@const isMe = member.user_id === data.userId}
					<li class="flex items-center gap-3 px-4 py-3">
						{#if person?.imageUrl}
							<img
								src={person.imageUrl}
								alt=""
								class="h-8 w-8 shrink-0 rounded-full object-cover"
							/>
						{:else}
							<span
								class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-element text-xs font-semibold text-ink-muted"
							>
								{initials(member.user_id)}
							</span>
						{/if}
						<div class="min-w-0 flex-1">
							<p class="truncate text-sm font-medium text-ink">
								{displayName(member.user_id)}
								{#if isMe}<span class="font-normal text-ink-faint">(you)</span>{/if}
							</p>
							<p class="truncate text-xs text-ink-faint">
								{#if person?.email && person.email !== person.name}{person.email} ·
								{/if}
								{#if member.role === 'owner'}
									owner
								{:else}
									added {formatDate(member.created_at)}{#if member.invited_by}
										by {displayName(member.invited_by)}{/if}
								{/if}
							</p>
						</div>
						<span
							class="shrink-0 rounded-full border px-2 py-0.5 text-xs font-medium {member.role ===
							'owner'
								? 'border-brand-light/40 bg-brand-light/10 text-brand-accent'
								: 'border-line-subtle text-ink-muted'}"
						>
							{member.role === 'owner' ? 'Owner' : 'Co-maintainer'}
						</span>
						{#if member.role === 'member' && (isOwner || isMe)}
							<form
								method="POST"
								action={isMe ? '?/leave' : '?/remove'}
								use:enhance={busyForm(member.user_id)}
							>
								<input type="hidden" name="member_id" value={member.user_id} />
								<button
									type="submit"
									disabled={busyId === member.user_id}
									class="rounded-md border border-error-border px-3 py-1 text-xs font-medium text-error transition hover:bg-error-bg disabled:opacity-50"
								>
									{busyId === member.user_id ? 'Working…' : isMe ? 'Leave' : 'Remove'}
								</button>
							</form>
						{/if}
					</li>
				{/each}
			</ul>
		</section>

		<!-- Pending invitations -->
		{#if data.detail.invitations.length > 0}
			<section>
				<h2 class="mb-3 text-base font-semibold text-ink-code">
					Pending invitations ({data.detail.invitations.length})
				</h2>
				<ul class="divide-y divide-line overflow-hidden rounded-xl border border-line bg-card">
					{#each data.detail.invitations as invitation (invitation.id)}
						<li class="flex items-center gap-3 px-4 py-3">
							<span
								class="flex h-8 w-8 shrink-0 items-center justify-center rounded-full border border-dashed border-line-active text-ink-faint"
								aria-hidden="true"
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
										d="M3 8l7.9 5.3a2 2 0 002.2 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z"
									/>
								</svg>
							</span>
							<div class="min-w-0 flex-1">
								<p class="truncate text-sm text-ink">{invitation.email}</p>
								<p class="text-xs text-ink-faint">
									invited {formatDate(invitation.created_at)} · waiting for sign-up
								</p>
							</div>
							{#if isOwner}
								<form method="POST" action="?/revoke" use:enhance={busyForm(invitation.id)}>
									<input type="hidden" name="invitation_id" value={invitation.id} />
									<button
										type="submit"
										disabled={busyId === invitation.id}
										class="rounded-md border border-line-subtle px-3 py-1 text-xs font-medium text-ink-secondary transition hover:bg-element disabled:opacity-50"
									>
										{busyId === invitation.id ? 'Revoking…' : 'Revoke'}
									</button>
								</form>
							{/if}
						</li>
					{/each}
				</ul>
			</section>
		{/if}
	</div>
</div>
