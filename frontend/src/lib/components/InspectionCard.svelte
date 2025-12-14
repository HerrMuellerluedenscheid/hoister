<script lang="ts">

    import * as Card from "$lib/components/ui/card/index.js";
    import {onDestroy, onMount} from "svelte";

    type InspectionType = {
        Id: string;
        State: {
            Status: string;
            Running: boolean;
            Paused: boolean;
            Error: string;
            FinishedAt: string;  // ISO 8601 date string
            StartedAt: string;   // ISO 8601 date string
        };
        Config: {
            Image: string;
            Hostname: string,
            User: string,
            Labels: Record<string, string>;
        };
    };

    let { inspection }: { inspection: InspectionType } = $props();

    let hoisterEnabled = inspection.Config.Labels?.["hoister.enable"] === "true";
    let hoisterBackupVolumes = inspection.Config.Labels?.["hoister.backup-volumes"] === "true";

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

<a href="/containers/{inspection.Id}" class="block">

<Card.Root class="shadow-sm hover:shadow-md transition-shadow min-h-50">
    <Card.Header>
        <Card.Title class="flex justify-between items-center">{inspection.Config.Image}
            <span class="px-3 py-1 rounded-full text-xs font-medium bg-gray-100 text-gray-700 capitalize
                     {inspection.State.Status === 'running' ? 'bg-green-100 text-green-800' : ''}
                     {inspection.State.Status === 'exited' ? 'bg-red-100 text-red-800' : ''}
                     {inspection.State.Status === 'paused' ? 'bg-yellow-100 text-yellow-800' : ''}">
            {inspection.State.Status}
        </span>
        </Card.Title>
        <Card.Description>
            <p class="text-sm text-gray-500">
                Uptime: {uptime}
            </p>
        </Card.Description>
    </Card.Header>
    <Card.Content>
        <h3 class="text-sm font-mono font-medium text-gray-900">
            Container ID: {inspection.Id.slice(0, 12)}
        </h3>
    </Card.Content>
    <Card.Footer>
        <div class="gap-2 flex flex-wrap">
            {#if (hoisterEnabled) }
            <span class="text-xs text-green-700 px-3 py-1 rounded-full border border-green-500 inline-flex items-center gap-2">
                Hoister enabled
                <svg class="w-4 h-4" viewBox="0 0 32 32" fill="currentColor">
                    <path d="M29,23.9c0-0.1,0-0.1,0-0.2c0-0.1-0.1-0.1-0.1-0.2c0-0.1-0.1-0.1-0.1-0.2c0-0.1-0.1-0.1-0.2-0.1c0,0-0.1-0.1-0.1-0.1
            l-10.6-5.3c0.5-0.7,0.5-1.6,0.2-2.4l-1-2.6V11h1c0.3,0,0.6-0.2,0.8-0.4l2-3C20.9,7.4,21,7.2,21,7V3c0-0.6-0.4-1-1-1h-8
            c-0.6,0-1,0.4-1,1v4c0,0.2,0.1,0.4,0.2,0.6l2,3c0.2,0.3,0.5,0.4,0.8,0.4h1v2c0,0.1,0,0.3,0.1,0.4l1.1,2.8c0.1,0.3,0,0.5-0.1,0.6
            c-0.1,0.1-0.2,0.3-0.5,0.3c-0.3,0-0.6-0.3-0.6-0.6V16c0-0.6-0.4-1-1-1s-1,0.4-1,1v0.4c0,0.6,0.2,1.2,0.6,1.7l-10.1,5
            c0,0-0.1,0.1-0.1,0.1c-0.1,0-0.1,0.1-0.2,0.1c0,0-0.1,0.1-0.1,0.2c0,0.1-0.1,0.1-0.1,0.2c0,0.1,0,0.1,0,0.2c0,0,0,0.1,0,0.1v5
            c0,0.6,0.4,1,1,1h24c0.6,0,1-0.4,1-1v-5C29,24,29,23.9,29,23.9z M16,19.1l7.8,3.9H8.2L16,19.1z"/>
                </svg>
            </span>
            {/if}
            {#if (hoisterBackupVolumes) }
                <span class="text-xs text-green-700 px-3 py-1 rounded-full border border-green-500 inline-flex gap-2">
                    Backup Volumes
                <svg class="w-4 h-4" viewBox="0 0 32 32" fill="currentColor">
                    <path d="M4 26.016q0 1.632 1.6 3.008t4.384 2.176 6.016 0.8 6.016-0.8 4.384-2.176 1.6-3.008v-3.392q0 1.632-1.632 2.88t-4.32 1.856-6.048 0.64-6.048-0.64-4.32-1.856-1.632-2.88v3.392zM4 20q0 1.632 1.6 3.008t4.384 2.208 6.016 0.8 6.016-0.8 4.384-2.208 1.6-3.008v-3.36q0 1.6-1.632 2.848t-4.32 1.888-6.048 0.64-6.048-0.64-4.32-1.888-1.632-2.848v3.36zM4 14.016q0 1.632 1.6 3.008t4.384 2.176 6.016 0.8 6.016-0.8 4.384-2.176 1.6-3.008v-3.392q0 1.632-1.632 2.88t-4.32 1.856-6.048 0.64-6.048-0.64-4.32-1.856-1.632-2.88v3.392zM4 8q0 1.632 1.6 3.008t4.384 2.208 6.016 0.8 6.016-0.8 4.384-2.208 1.6-3.008v-1.984q0-1.632-1.6-3.008t-4.384-2.176-6.016-0.832-6.016 0.832-4.384 2.176-1.6 3.008v1.984z"></path>
                </svg>
                </span>
            {/if}
        </div>
    </Card.Footer>
</Card.Root>

</a>
