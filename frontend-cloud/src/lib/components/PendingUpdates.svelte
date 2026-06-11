<script lang="ts">
	import { enhance } from '$app/forms';
	import type { PendingUpdate } from '$lib/api/pendingUpdates';

	let { updates, compact = false }: { updates: PendingUpdate[]; compact?: boolean } = $props();

	// Track in-flight deploys, keyed by host/project/service, so the button can
	// show progress and disable itself.
	let applying = $state<Set<string>>(new Set());

	function updateKey(u: { hostname: string; project_name: string; service_name: string }): string {
		return `${u.hostname}/${u.project_name}/${u.service_name}`;
	}

	// The Deploy control posts to the hosting page's `?/apply` form action.
	function deployForm(key: string) {
		return () => {
			applying = new Set([...applying, key]);
			return async ({ update: updatePage }: { update: () => Promise<void> }) => {
				await updatePage();
				applying = new Set([...applying].filter((k) => k !== key));
			};
		};
	}
</script>

{#if updates.length > 0}
	{#if compact}
		<!-- Compact banner: used on the container detail view, typically one row. -->
		<div class="space-y-2 rounded-xl border border-warning-border bg-warning-bg px-4 py-3">
			{#each updates as update (updateKey(update))}
				{@const key = updateKey(update)}
				<div class="flex flex-wrap items-center justify-between gap-3">
					<div class="text-sm">
						<p class="font-semibold text-warning">Update available</p>
						<p class="font-mono text-xs break-all text-warning">
							{update.image_name} · {update.new_digest.slice(0, 20)}…
						</p>
					</div>
					<form method="POST" action="?/apply" use:enhance={deployForm(key)}>
						<input type="hidden" name="hostname" value={update.hostname} />
						<input type="hidden" name="project_name" value={update.project_name} />
						<input type="hidden" name="service_name" value={update.service_name} />
						<button
							type="submit"
							disabled={applying.has(key)}
							class="rounded-md bg-amber-500 px-3 py-1.5 text-xs font-semibold text-amber-950 transition hover:bg-amber-400 disabled:opacity-50"
						>
							{applying.has(key) ? 'Deploying…' : 'Deploy'}
						</button>
					</form>
				</div>
			{/each}
		</div>
	{:else}
		<section>
			<h2 class="mb-3 text-base font-semibold text-warning">Pending updates</h2>
			<div class="overflow-x-auto rounded-xl border border-warning-border/50 bg-warning-bg">
				<table class="min-w-full divide-y divide-warning-border text-sm">
					<thead class="bg-warning-bg text-xs tracking-wider text-warning uppercase">
						<tr>
							<th class="px-4 py-2 text-left font-medium">Host</th>
							<th class="px-4 py-2 text-left font-medium">Service</th>
							<th class="px-4 py-2 text-left font-medium">Image</th>
							<th class="px-4 py-2 text-left font-medium">New digest</th>
							<th class="px-4 py-2 text-left font-medium">Detected</th>
							<th class="px-4 py-2"></th>
						</tr>
					</thead>
					<tbody class="divide-y divide-warning-border">
						{#each updates as update (updateKey(update))}
							{@const key = updateKey(update)}
							<tr class="text-ink-secondary">
								<td class="px-4 py-2 break-all">{update.hostname}</td>
								<td class="px-4 py-2 font-medium break-all text-ink">{update.service_name}</td>
								<td class="px-4 py-2 font-mono text-xs break-all">{update.image_name}</td>
								<td class="px-4 py-2 font-mono text-xs text-ink-faint"
									>{update.new_digest.slice(0, 20)}…</td
								>
								<td class="px-4 py-2 text-xs text-ink-faint"
									>{new Date(update.detected_at).toLocaleString()}</td
								>
								<td class="px-4 py-2">
									<form method="POST" action="?/apply" use:enhance={deployForm(key)}>
										<input type="hidden" name="hostname" value={update.hostname} />
										<input type="hidden" name="project_name" value={update.project_name} />
										<input type="hidden" name="service_name" value={update.service_name} />
										<button
											type="submit"
											disabled={applying.has(key)}
											class="rounded-md bg-amber-500 px-3 py-1 text-xs font-semibold text-amber-950 transition hover:bg-amber-400 disabled:opacity-50"
										>
											{applying.has(key) ? 'Deploying…' : 'Deploy'}
										</button>
									</form>
								</td>
							</tr>
						{/each}
					</tbody>
				</table>
			</div>
		</section>
	{/if}
{/if}
