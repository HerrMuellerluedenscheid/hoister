<script lang="ts">
	import type { Deployment } from '../../bindings/Deployment';

	let { data }: { data: Deployment[] } = $props();
</script>

{#if data.length === 0}
	<div class="rounded-xl border border-zinc-700 bg-zinc-900 px-4 py-6 text-center text-zinc-400">
		No deployments found.
	</div>
{:else}
	<div class="overflow-x-auto rounded-xl border border-zinc-800">
		<table class="min-w-full bg-zinc-900">
			<thead class="bg-zinc-800">
				<tr>
					<th
						class="border-b border-zinc-700 px-6 py-3 text-left text-xs font-medium tracking-wider text-zinc-400 uppercase"
					>
						Host
					</th>
					<th
						class="border-b border-zinc-700 px-6 py-3 text-left text-xs font-medium tracking-wider text-zinc-400 uppercase"
					>
						Project | Service
					</th>
					<th
						class="border-b border-zinc-700 px-6 py-3 text-left text-xs font-medium tracking-wider text-zinc-400 uppercase"
					>
						Status
					</th>
					<th
						class="border-b border-zinc-700 px-6 py-3 text-left text-xs font-medium tracking-wider text-zinc-400 uppercase"
					>
						Date
					</th>
				</tr>
			</thead>
			<tbody class="divide-y divide-zinc-800">
				{#each data as item}
					<tr class="transition-colors hover:bg-zinc-800/50">
						<td class="px-6 py-4 text-sm whitespace-nowrap text-zinc-400">
							{item.hostname}
						</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-zinc-100">
							<p>{item.project_name} | {item.service_name}</p>
							<p class="font-mono text-xs text-zinc-500">
								{item.digest.replace('sha256:', '').slice(0, 12)}
							</p>
						</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap">
							<div class="flex items-center gap-2">
								{#if item.status === 'Pending'}
									<span class="h-2 w-2 rounded-full bg-yellow-400"></span>
									<span class="text-zinc-300">Pending</span>
								{:else if item.status === 'Started'}
									<span class="h-2 w-2 rounded-full bg-blue-400"></span>
									<span class="text-zinc-300">Started</span>
								{:else if item.status === 'Success'}
									<span class="h-2 w-2 rounded-full bg-emerald-400"></span>
									<span class="font-medium text-emerald-400">Success</span>
								{:else if item.status === 'Failed'}
									<span class="h-2 w-2 rounded-full bg-red-400"></span>
									<span class="font-medium text-red-400">Failed</span>
								{:else if item.status === 'RollbackFinished'}
									<span class="h-2 w-2 rounded-full bg-indigo-400"></span>
									<span class="font-medium text-indigo-400">Rolled Back</span>
								{:else if item.status === 'NoUpdate'}
									<span class="h-2 w-2 rounded-full bg-zinc-500"></span>
									<span class="text-zinc-500">No Update</span>
								{:else}
									<span class="text-zinc-400">{item.status}</span>
								{/if}
							</div>
						</td>
						<td class="px-6 py-4 text-sm whitespace-nowrap text-zinc-500">
							{item.created_at}
						</td>
					</tr>
				{/each}
			</tbody>
		</table>
	</div>
{/if}
