<script lang="ts">
  import InspectionCard from '$lib/components/InspectionCard.svelte';
  import type { Inspection } from '$lib/api/inspect';
  import type { PendingUpdate } from '$lib/api/pendingUpdates';
  import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';
  import { invalidateAll } from '$app/navigation';
  import { onDestroy, onMount } from 'svelte';

  let { data }: { data: Inspection & { pendingUpdates: PendingUpdate[] } } = $props();

  let refreshInterval: ReturnType<typeof setInterval>;

  onMount(() => {
    refreshInterval = setInterval(() => {
      invalidateAll();
    }, 10_000);
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  function groupByHostname(
    inspections: ContainerStateResponse[]
  ): Map<string, ContainerStateResponse[]> {
    const grouped = new Map<string, ContainerStateResponse[]>();
    for (const inspection of inspections) {
      const host = inspection.hostname;
      if (!grouped.has(host)) {
        grouped.set(host, []);
      }
      grouped.get(host)!.push(inspection);
    }
    return grouped;
  }

  const grouped = $derived(groupByHostname(data.inspections));

  async function applyUpdate(update: PendingUpdate) {
    await fetch(
      `/api/pending-updates/${encodeURIComponent(update.hostname)}/${encodeURIComponent(update.project_name)}/${encodeURIComponent(update.service_name)}/apply`,
      { method: 'POST' }
    );
    invalidateAll();
  }
</script>

<div class="mx-auto max-w-7xl p-6">
  {#if data.error}
    <div class="mb-4 rounded border border-red-200 bg-red-50 px-4 py-3 text-red-700">
      <p class="font-bold">Error</p>
      <p>{data.error}</p>
    </div>
  {/if}

  {#if data.pendingUpdates && data.pendingUpdates.length > 0}
    <div class="mb-8">
      <h2 class="mb-4 text-lg font-semibold text-amber-700">Pending Updates</h2>
      <div class="overflow-hidden rounded-lg border border-amber-200 bg-amber-50">
        <table class="min-w-full divide-y divide-amber-200">
          <thead>
            <tr class="bg-amber-100">
              <th class="px-4 py-2 text-left text-xs font-semibold text-amber-800">Host</th>
              <th class="px-4 py-2 text-left text-xs font-semibold text-amber-800">Service</th>
              <th class="px-4 py-2 text-left text-xs font-semibold text-amber-800">Image</th>
              <th class="px-4 py-2 text-left text-xs font-semibold text-amber-800">New Digest</th>
              <th class="px-4 py-2 text-left text-xs font-semibold text-amber-800">Detected</th>
              <th class="px-4 py-2"></th>
            </tr>
          </thead>
          <tbody class="divide-y divide-amber-100">
            {#each data.pendingUpdates as update}
              <tr>
                <td class="px-4 py-2 text-sm text-gray-700">{update.hostname}</td>
                <td class="px-4 py-2 text-sm font-medium text-gray-900">{update.service_name}</td>
                <td class="px-4 py-2 font-mono text-xs text-gray-600">{update.image_name}</td>
                <td class="px-4 py-2 font-mono text-xs text-gray-500">{update.new_digest.slice(0, 20)}…</td>
                <td class="px-4 py-2 text-xs text-gray-500">{new Date(update.detected_at).toLocaleString()}</td>
                <td class="px-4 py-2">
                  <button
                    onclick={() => applyUpdate(update)}
                    class="rounded bg-amber-500 px-3 py-1 text-xs font-semibold text-white hover:bg-amber-600 active:bg-amber-700"
                  >
                    Apply
                  </button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
    </div>
  {/if}

  {#if data.inspections.length === 0}
    <div class="test-blue-600 mb-4 rounded border border-blue-200 bg-blue-50 px-4 py-3">
      <p class="font-bold">Error</p>
      <p>Waiting for running containers...</p>
    </div>
  {/if}

  {#each [...grouped] as [hostname, inspections]}
    <div class="mb-8">
      <h2 class="mb-4 text-lg font-semibold text-gray-700">{hostname}</h2>
      <div class="grid grid-cols-[repeat(auto-fit,minmax(350px,1fr))] gap-4">
        {#each inspections as inspection_data}
          <InspectionCard {inspection_data} />
        {/each}
      </div>
    </div>
  {/each}
</div>
