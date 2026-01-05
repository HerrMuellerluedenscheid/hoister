<script lang="ts">
  import InspectionCard from '$lib/components/InspectionCard.svelte';
  import type { Inspection } from '$lib/api/inspect';

  let { data }: { data: Inspection } = $props();
</script>

<div class="mx-auto max-w-7xl p-6">

  {#if data.error}
    <div class="mb-4 rounded border border-red-200 bg-red-50 px-4 py-3 text-red-700">
      <p class="font-bold">Error</p>
      <p>{data.error}</p>
    </div>
  {/if}

  {#if data.inspections.length === 0}
    <div class="mb-4 rounded border border-blue-200 bg-blue-50 px-4 py-3 test-blue-600">
      <p class="font-bold">Error</p>
      <p>Waiting for running containers...</p>
    </div>
  {/if}

  <div class="grid grid-cols-[repeat(auto-fit,minmax(350px,1fr))] gap-4">
    {#each data.inspections as inspection_data}
      <InspectionCard {inspection_data} />
    {/each}
  </div>
</div>
