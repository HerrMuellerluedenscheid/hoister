<script lang="ts">
	import type { Deployment } from '../../bindings/Deployment';
	import RedactedText from './RedactedText.svelte';

	let { data }: { data: Deployment[] } = $props();

	// Track which deployments have their captured failed-container logs expanded.
	let expanded = $state<Set<bigint>>(new Set());

	function toggle(id: bigint) {
		const next = new Set(expanded);
		if (next.has(id)) next.delete(id);
		else next.add(id);
		expanded = next;
	}
</script>

{#if data.length === 0}
	<div class="rounded-xl border border-line-subtle bg-card px-4 py-6 text-center text-ink-muted">
		No deployments found.
	</div>
{:else}
	<div class="overflow-x-auto rounded-xl border border-line">
		<table class="min-w-full bg-card">
			<thead class="sticky top-0 z-10 bg-element">
				<tr>
					<th
						class="border-b border-line-subtle px-6 py-3 text-left text-xs font-medium tracking-wider text-ink-muted uppercase"
					>
						Host
					</th>
					<th
						class="border-b border-line-subtle px-6 py-3 text-left text-xs font-medium tracking-wider text-ink-muted uppercase"
					>
						Project | Service
					</th>
					<th
						class="border-b border-line-subtle px-6 py-3 text-left text-xs font-medium tracking-wider text-ink-muted uppercase"
					>
						Status
					</th>
					<th
						class="border-b border-line-subtle px-6 py-3 text-left text-xs font-medium tracking-wider text-ink-muted uppercase"
					>
						Date
					</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-line">
				{#each data as item}
					<tr class="transition-colors hover:bg-element/50">
						<td class="px-6 py-4 text-sm whitespace-nowrap text-ink-muted">
							{item.hostname}
						</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-ink">
							<p>{item.project_name} | {item.service_name}</p>
							<p class="font-mono text-xs text-ink-faint">
								{item.digest.replace('sha256:', '').slice(0, 12)}
							</p>
						</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap">
							<div class="flex items-center gap-2">
								{#if item.status === 'Pending'}
									<span class="h-2 w-2 rounded-full bg-yellow-400"></span>
									<span class="text-ink-secondary">Pending</span>
								{:else if item.status === 'Started'}
									<span class="h-2 w-2 rounded-full bg-blue-400"></span>
									<span class="text-ink-secondary">Started</span>
								{:else if item.status === 'Success'}
									<span class="h-2 w-2 rounded-full bg-success"></span>
									<span class="font-medium text-success">Success</span>
								{:else if item.status === 'Failed'}
									<span class="h-2 w-2 rounded-full bg-error"></span>
									<span class="font-medium text-error">Failed</span>
								{:else if item.status === 'RollbackFinished'}
									<span class="h-2 w-2 rounded-full bg-brand-accent"></span>
									<span class="font-medium text-brand-accent">Rolled Back</span>
								{:else if item.status === 'NoUpdate'}
									<span class="h-2 w-2 rounded-full bg-line-active"></span>
									<span class="text-ink-faint">No Update</span>
								{:else}
									<span class="text-ink-muted">{item.status}</span>
								{/if}
							</div>
							{#if item.logs}
								<button
									type="button"
									onclick={() => toggle(item.id)}
									class="mt-2 text-xs text-brand-accent hover:text-brand-light"
								>
									{expanded.has(item.id) ? 'Hide logs' : 'View logs'}
								</button>
							{/if}
						</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-ink-faint">
							{item.created_at}
						</td>
					</tr>
					{#if item.logs && expanded.has(item.id)}
						<tr class="bg-canvas/60">
							<td colspan="4" class="px-6 pb-4">
								<p class="mb-2 text-xs text-ink-faint">
									Failed container logs (tail). Secrets matching known sensitive env-var values are
									redacted.
								</p>
								<pre
									class="max-h-96 overflow-auto rounded-lg bg-black p-4 font-mono text-xs leading-relaxed whitespace-pre-wrap text-ink-code"><RedactedText
										text={item.logs}
									/></pre>
							</td>
						</tr>
					{/if}
				{/each}
			</tbody>
		</table>
	</div>
{/if}
