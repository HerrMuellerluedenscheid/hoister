<script lang="ts">
  import type { ContainerPageData } from './+page.server';
  import Deployments from '$lib/components/Deployments.svelte';
  import { invalidateAll } from '$app/navigation';
  import { onDestroy, onMount } from 'svelte';

  let { data }: { data: ContainerPageData } = $props();
  const container = $derived(data.inspections.container_inspections);
  const deployments = $derived(data.deployments.slice(0, 8));
  const hostname = $derived(data.inspections.hostname);
  const service_name = $derived(data.inspections.service_name);
  const project_name = $derived(data.inspections.project_name);
  const last_updated = $derived(data.inspections.last_updated);

  let stale = $state(false);
  let refreshInterval: ReturnType<typeof setInterval>;

  function checkStale() {
    stale = new Date().getTime() - new Date(last_updated).getTime() > 60_000;
  }

  onMount(() => {
    checkStale();
    refreshInterval = setInterval(() => {
      checkStale();
      invalidateAll();
    }, 10_000);
  });

  onDestroy(() => {
    clearInterval(refreshInterval);
  });

  function formatDate(dateString: string) {
    return new Date(dateString).toLocaleString();
  }

  function getStatusColor(status) {
    const colors = {
      running: 'bg-green-100 text-green-800',
      exited: 'bg-gray-100 text-gray-800',
      paused: 'bg-yellow-100 text-yellow-800',
      restarting: 'bg-blue-100 text-blue-800',
      dead: 'bg-red-100 text-red-800'
    };
    return colors[status.toLowerCase()] || 'bg-gray-100 text-gray-800';
  }
</script>

<div class="min-h-screen bg-gray-50 py-8">
  <div class="mx-auto max-w-6xl px-4">
    <!-- Header -->
    <div class="mb-8">
      <h1 class="mb-2 text-3xl font-bold text-gray-900">{project_name} | {service_name}</h1>
      <p class="text-sm text-gray-500">Host: {hostname}</p>
      <p class="font-mono text-sm text-gray-500">{container.Id}</p>
      <p class="text-sm text-gray-400">Last updated: {formatDate(last_updated)}</p>
    </div>

    {#if stale}
      <div class="mb-6 rounded-lg border border-amber-300 bg-amber-50 px-4 py-3 text-amber-800">
        <p class="font-semibold">Stale data</p>
        <p class="text-sm">This container has not reported in over a minute. The information below may be outdated.</p>
      </div>
    {/if}

    <!-- Status Card -->
    <div class="mb-6 rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">Status</h2>
      <div class="grid grid-cols-2 gap-4 md:grid-cols-4">
        <div>
          <span class="text-sm text-gray-600">State</span>
          <p class="mt-1">
            <span
              class="inline-flex items-center rounded-full px-3 py-1 text-sm font-medium {getStatusColor(
                container.State.Status
              )}"
            >
              {container.State.Status}
            </span>
          </p>
        </div>
        <div>
          <span class="text-sm text-gray-600">Exit Code</span>
          <p class="mt-1 font-mono text-gray-900">{container.State.ExitCode}</p>
        </div>
        <div>
          <span class="text-sm text-gray-600">PID</span>
          <p class="mt-1 font-mono text-gray-900">{container.State.Pid}</p>
        </div>
        <div>
          <span class="text-sm text-gray-600">OOM Killed</span>
          <p class="mt-1 text-gray-900">{container.State.OOMKilled ? 'Yes' : 'No'}</p>
        </div>
      </div>
      <div class="mt-4 grid grid-cols-1 gap-4 border-t pt-4 md:grid-cols-2">
        <div>
          <span class="text-sm text-gray-600">Created</span>
          <p class="mt-1 text-sm text-gray-900">{formatDate(container.Created)}</p>
        </div>
        <div>
          <span class="text-sm text-gray-600">Started</span>
          <p class="mt-1 text-sm text-gray-900">{formatDate(container.State.StartedAt)}</p>
        </div>
        <div>
          <span class="text-sm text-gray-600">Finished</span>
          <p class="mt-1 text-sm text-gray-900">{formatDate(container.State.FinishedAt)}</p>
        </div>
      </div>
    </div>

    <!-- Deployments -->
    <div class="mb-6 rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">Most recent Deployments</h2>
      <Deployments data={deployments} />
    </div>

    <!-- Configuration Card -->
    <div class="mb-6 rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">Configuration</h2>
      <div class="space-y-4">
        <div>
          <span class="text-sm font-medium text-gray-600">Image</span>
          <p class="mt-1 font-mono text-sm text-gray-900">{container.Config.Image}</p>
        </div>
        <div>
          <span class="text-sm font-medium text-gray-600">Hostname</span>
          <p class="mt-1 font-mono text-sm text-gray-900">{container.Config.Hostname}</p>
        </div>
        <div>
          <span class="text-sm font-medium text-gray-600">Working Directory</span>
          <p class="mt-1 font-mono text-sm text-gray-900">{container.Config.WorkingDir}</p>
        </div>
        <div>
          <span class="text-sm font-medium text-gray-600">Command</span>
          <p class="mt-1 rounded bg-gray-50 p-3 font-mono text-sm text-gray-900">
            {container.Config.Cmd ? container.Config.Cmd.join(' ') : 'N/A'}
          </p>
        </div>
      </div>
    </div>

    <!-- Environment Variables -->
    <div class="mb-6 rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">Environment Variables</h2>
      <div class="space-y-2">
        {#each container.Config.Env as env}
          {#if env.includes('=')}
            {@const [key, ...valueParts] = env.split('=')}
            {@const value = valueParts.join('=')}
            <div class="flex items-start border-b py-2 last:border-b-0">
              <span class="w-64 flex-shrink-0 font-mono text-sm text-gray-600">{key}</span>
              <span class="font-mono text-sm break-all text-gray-900">{value}</span>
            </div>
          {/if}
        {/each}
      </div>
    </div>

    <!-- Network Settings -->
    <div class="mb-6 rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">Network Settings</h2>
      <div class="space-y-4">
        {#each Object.entries(container.NetworkSettings.Networks) as [networkName, network]}
          <div class="rounded-lg border p-4">
            <h3 class="mb-3 font-medium text-gray-900">{networkName}</h3>
            <div class="grid grid-cols-1 gap-4 text-sm md:grid-cols-3">
              <div>
                <span class="text-gray-600">IP Address</span>
                <p class="font-mono text-gray-900">{network.IPAddress || 'N/A'}</p>
              </div>
              <div>
                <span class="text-gray-600">Gateway</span>
                <p class="font-mono text-gray-900">{network.Gateway || 'N/A'}</p>
              </div>
              <div>
                <span class="text-gray-600">MAC Address</span>
                <p class="font-mono text-gray-900">{network.MacAddress || 'N/A'}</p>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Mounts -->
    <div class="mb-6 rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">Mounts</h2>
      <div class="space-y-3">
        {#each container.Mounts as mount}
          <div class="rounded-lg border p-4">
            <div class="mb-2 flex items-center justify-between">
              <span
                class="inline-flex items-center rounded bg-blue-100 px-2.5 py-0.5 text-xs font-medium text-blue-800"
              >
                {mount.Type}
              </span>
              <span class="text-xs text-gray-600">
                {mount.RW ? 'Read/Write' : 'Read-Only'}
              </span>
            </div>
            <div class="space-y-1 text-sm">
              <div class="flex">
                <span class="w-24 text-gray-600">Source:</span>
                <span class="font-mono break-all text-gray-900">{mount.Source}</span>
              </div>
              <div class="flex">
                <span class="w-24 text-gray-600">Destination:</span>
                <span class="font-mono text-gray-900">{mount.Destination}</span>
              </div>
            </div>
          </div>
        {/each}
      </div>
    </div>

    <!-- Labels -->
    <div class="mb-6 rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">Labels</h2>
      <div class="space-y-2">
        {#each Object.entries(container.Config.Labels) as [key, value]}
          <div class="flex items-start border-b py-2 last:border-b-0">
            <span class="w-80 flex-shrink-0 font-mono text-sm text-gray-600">{key}</span>
            <span class="font-mono text-sm break-all text-gray-900">{value}</span>
          </div>
        {/each}
      </div>
    </div>

    <!-- Host Config -->
    <div class="rounded-lg bg-white p-6 shadow">
      <h2 class="mb-4 text-xl font-semibold text-gray-900">Host Configuration</h2>
      <div class="grid grid-cols-1 gap-4 md:grid-cols-3">
        <div>
          <span class="text-sm text-gray-600">Memory Limit</span>
          <p class="mt-1 font-mono text-gray-900">
            {container.HostConfig.Memory === 0
              ? 'Unlimited'
              : `${container.HostConfig.Memory} bytes`}
          </p>
        </div>
        <div>
          <span class="text-sm text-gray-600">CPU Shares</span>
          <p class="mt-1 font-mono text-gray-900">{container.HostConfig.CpuShares || 'Default'}</p>
        </div>
        <div>
          <span class="text-sm text-gray-600">Restart Policy</span>
          <p class="mt-1 text-gray-900">{container.HostConfig.RestartPolicy.Name}</p>
        </div>
      </div>
    </div>
  </div>
</div>
