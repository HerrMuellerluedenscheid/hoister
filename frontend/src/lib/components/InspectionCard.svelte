<script lang="ts">

    import * as Card from "$lib/components/ui/card/index.js";

    type Inspection = {
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
        };
    };

    let { inspection }: { inspection: Inspection } = $props();

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

<Card.Root class="shadow-sm hover:shadow-md transition-shadow">
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
                Uptime: {getUptime(inspection.State.StartedAt)}
            </p>
        </Card.Description>
    </Card.Header>
    <Card.Content>
        <h3 class="text-sm font-mono font-medium text-gray-900">
            Container ID: {inspection.Id.slice(0, 12)}
        </h3>
    </Card.Content>
    <Card.Footer>
        <p>Card Footer</p>
    </Card.Footer>
</Card.Root>

