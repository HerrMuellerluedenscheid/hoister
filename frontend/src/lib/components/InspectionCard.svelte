<script lang="ts">
  import * as Card from '$lib/components/ui/card/index.js';
  import { onDestroy, onMount } from 'svelte';
  import type { ContainerStateResponse } from '../../bindings/ContainerStateResponse';

  const { inspection_data }: { inspection_data: ContainerStateResponse } = $props();

  const inspection = inspection_data.container_inspections;
  let hoisterEnabled = inspection.Config.Labels?.['hoister.enable'] === 'true';
  let hoisterBackupVolumes = inspection.Config.Labels?.['hoister.backup-volumes'] === 'true';

  let uptime = $state(getUptime(inspection.State.StartedAt));
  let interval: number;

  onMount(() => {
    interval = setInterval(() => {
      uptime = getUptime(inspection.State.StartedAt);
    }, 1000);
  });

  onDestroy(() => {
    clearInterval(interval);
  });

  function getUptime(startedAt: string): string {
    const start = new Date(startedAt);
    const now = new Date();
    const diffMs = now.getTime() - start.getTime();

    const seconds = Math.floor(diffMs / 1000);
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const days = Math.floor(hours / 24);

    if (days > 0) {
      return `${days}d ${hours % 24}h`;
    } else if (hours > 0) {
      return `${hours}h ${minutes % 60}m`;
    } else if (minutes > 0) {
      return `${minutes}m ${seconds % 60}s`;
    } else {
      return `${seconds}s`;
    }
  }
</script>

<a
  href="/containers/{inspection_data.hostname}/{inspection_data.project_name}/{inspection_data.service_name}"
  class="block"
>
  <Card.Root class="min-h-50 shadow-sm transition-shadow hover:shadow-md">
    <Card.Header>
      <Card.Title class="flex items-center justify-between"
        >{inspection_data.service_name}
        <span
          class="rounded-full bg-gray-100 px-3 py-1 text-xs font-medium text-gray-700 capitalize
                     {inspection.State.Status === 'running' ? 'bg-green-100 text-green-800' : ''}
                     {inspection.State.Status === 'exited' ? 'bg-red-100 text-red-800' : ''}
                     {inspection.State.Status === 'paused' ? 'bg-yellow-100 text-yellow-800' : ''}"
        >
          {inspection.State.Status}
        </span>
      </Card.Title>
      <Card.Description>
        <p class="text-xs text-gray-600">
          Uptime: {uptime}
        </p>
      </Card.Description>
    </Card.Header>
    <Card.Content>
      <h3 class="text-sm font-medium text-gray-600">
        Host: {inspection_data.hostname}
      </h3>
      <h3 class="text-sm font-medium text-gray-600">
        Image: {inspection.Config.Image}
      </h3>
      <h3 class="font-mono text-sm font-medium text-gray-900">
        Container ID: {inspection.Id.slice(0, 12)}
      </h3>
    </Card.Content>
    <Card.Footer>
      <div class="flex flex-wrap gap-2">
        {#if hoisterEnabled}
          <span
            class="inline-flex items-center gap-2 rounded-full border border-green-500 px-3 py-1 text-xs text-green-700"
          >
            Hoister enabled
            <svg class="h-4 w-4" viewBox="0 0 32 32" fill="currentColor">
              <path
                d="M29,23.9c0-0.1,0-0.1,0-0.2c0-0.1-0.1-0.1-0.1-0.2c0-0.1-0.1-0.1-0.1-0.2c0-0.1-0.1-0.1-0.2-0.1c0,0-0.1-0.1-0.1-0.1
            l-10.6-5.3c0.5-0.7,0.5-1.6,0.2-2.4l-1-2.6V11h1c0.3,0,0.6-0.2,0.8-0.4l2-3C20.9,7.4,21,7.2,21,7V3c0-0.6-0.4-1-1-1h-8
            c-0.6,0-1,0.4-1,1v4c0,0.2,0.1,0.4,0.2,0.6l2,3c0.2,0.3,0.5,0.4,0.8,0.4h1v2c0,0.1,0,0.3,0.1,0.4l1.1,2.8c0.1,0.3,0,0.5-0.1,0.6
            c-0.1,0.1-0.2,0.3-0.5,0.3c-0.3,0-0.6-0.3-0.6-0.6V16c0-0.6-0.4-1-1-1s-1,0.4-1,1v0.4c0,0.6,0.2,1.2,0.6,1.7l-10.1,5
            c0,0-0.1,0.1-0.1,0.1c-0.1,0-0.1,0.1-0.2,0.1c0,0-0.1,0.1-0.1,0.2c0,0.1-0.1,0.1-0.1,0.2c0,0.1,0,0.1,0,0.2c0,0,0,0.1,0,0.1v5
            c0,0.6,0.4,1,1,1h24c0.6,0,1-0.4,1-1v-5C29,24,29,23.9,29,23.9z M16,19.1l7.8,3.9H8.2L16,19.1z"
              />
            </svg>
          </span>
        {/if}
        {#if hoisterBackupVolumes}
          <span
            class="inline-flex gap-2 rounded-full border border-green-500 px-3 py-1 text-xs text-green-700"
          >
            Backup Volumes
            <svg class="h-4 w-4" viewBox="0 0 32 32" fill="currentColor">
              <path
                d="M4 26.016q0 1.632 1.6 3.008t4.384 2.176 6.016 0.8 6.016-0.8 4.384-2.176 1.6-3.008v-3.392q0 1.632-1.632 2.88t-4.32 1.856-6.048 0.64-6.048-0.64-4.32-1.856-1.632-2.88v3.392zM4 20q0 1.632 1.6 3.008t4.384 2.208 6.016 0.8 6.016-0.8 4.384-2.208 1.6-3.008v-3.36q0 1.6-1.632 2.848t-4.32 1.888-6.048 0.64-6.048-0.64-4.32-1.888-1.632-2.848v3.36zM4 14.016q0 1.632 1.6 3.008t4.384 2.176 6.016 0.8 6.016-0.8 4.384-2.176 1.6-3.008v-3.392q0 1.632-1.632 2.88t-4.32 1.856-6.048 0.64-6.048-0.64-4.32-1.856-1.632-2.88v3.392zM4 8q0 1.632 1.6 3.008t4.384 2.208 6.016 0.8 6.016-0.8 4.384-2.208 1.6-3.008v-1.984q0-1.632-1.6-3.008t-4.384-2.176-6.016-0.832-6.016 0.832-4.384 2.176-1.6 3.008v1.984z"
              ></path>
            </svg>
          </span>
        {/if}
      </div>
    </Card.Footer>
  </Card.Root>
</a>
