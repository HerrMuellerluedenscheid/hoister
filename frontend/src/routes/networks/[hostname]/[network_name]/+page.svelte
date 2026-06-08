<script lang="ts">
  import { invalidateAll } from '$app/navigation';
  import { onDestroy, onMount } from 'svelte';
  import type { ContainerStateResponse } from '../../../../bindings/ContainerStateResponse';

  let {
    data
  }: {
    data: {
      hostname: string;
      networkName: string;
      members: ContainerStateResponse[];
      error: string | null;
    };
  } = $props();

  let refreshInterval: ReturnType<typeof setInterval>;

  onMount(() => {
    refreshInterval = setInterval(() => invalidateAll(), 10_000);
  });

  onDestroy(() => clearInterval(refreshInterval));

  const members = $derived(
    [...data.members].sort((a, b) => a.service_name.localeCompare(b.service_name))
  );

  function status(inspection: ContainerStateResponse): string {
    return inspection.container_inspections?.State?.Status ?? 'unknown';
  }

  function ipOnNetwork(inspection: ContainerStateResponse): string {
    const net = inspection.container_inspections?.NetworkSettings?.Networks?.[data.networkName];
    return net?.IPAddress || 'N/A';
  }

  function getStatusColor(s: string): string {
    const colors: Record<string, string> = {
      running: 'bg-green-100 text-green-800',
      exited: 'bg-gray-100 text-gray-800',
      paused: 'bg-yellow-100 text-yellow-800',
      restarting: 'bg-blue-100 text-blue-800',
      dead: 'bg-red-100 text-red-800'
    };
    return colors[s.toLowerCase()] || 'bg-gray-100 text-gray-800';
  }
</script>

<div class="min-h-screen bg-gray-50 py-8">
  <div class="mx-auto max-w-6xl px-4">
    <!-- Header -->
    <div class="mb-8">
      <a href="/containers" class="text-sm text-gray-500 hover:text-gray-700">← Containers</a>
      <h1 class="mt-2 text-3xl font-bold text-gray-900">
        Network: <span class="font-mono break-all">{data.networkName}</span>
      </h1>
      <p class="text-sm text-gray-500">Host: {data.hostname}</p>
    </div>

    {#if data.error}
      <div class="mb-6 rounded border border-red-200 bg-red-50 px-4 py-3 text-red-700">
        <p class="font-bold">Error</p>
        <p>{data.error}</p>
      </div>
    {/if}

    <div class="rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">
        Services on this network
        <span class="text-base font-normal text-gray-500">({members.length})</span>
      </h2>

      {#if members.length === 0}
        <p class="text-sm text-gray-500">No reporting services are connected to this network.</p>
      {:else}
        <div class="grid grid-cols-1 gap-4 sm:grid-cols-2 lg:grid-cols-3">
          {#each members as member (member.project_name + '/' + member.service_name)}
            <a
              href="/containers/{encodeURIComponent(member.hostname)}/{encodeURIComponent(
                member.project_name
              )}/{encodeURIComponent(member.service_name)}"
              class="block rounded-lg border p-4 transition-shadow hover:shadow-md"
            >
              <div class="mb-2 flex items-start justify-between gap-3">
                <h3 class="font-medium break-all text-gray-900">{member.service_name}</h3>
                <span
                  class="rounded-full px-3 py-1 text-xs font-medium capitalize {getStatusColor(
                    status(member)
                  )}"
                >
                  {status(member)}
                </span>
              </div>
              <p class="text-sm text-gray-600">Project: {member.project_name}</p>
              <p class="font-mono text-sm text-gray-600">IP: {ipOnNetwork(member)}</p>
            </a>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
