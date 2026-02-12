<script lang="ts">
  import InspectionCard from '$lib/components/InspectionCard.svelte';
  import type { Inspection } from '$lib/api/inspect';
  import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';

  let { data }: { data: Inspection } = $props();

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
</script>

<div class="mx-auto max-w-7xl p-6">
  {#if data.error}
    <div class="mb-4 rounded border border-red-200 bg-red-50 px-4 py-3 text-red-700">
      <p class="font-bold">Error</p>
      <p>{data.error}</p>
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
