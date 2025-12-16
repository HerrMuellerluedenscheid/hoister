<script lang="ts">
    import type { PageData } from './$types';

    let { data }: { data: PageData } = $props();
    const container = data.inspections[0]

    function formatDate(dateString) {
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

    <div class="max-w-6xl mx-auto px-4">
        <!-- Header -->
        <div class="mb-8">
            <h1 class="text-3xl font-bold text-gray-900 mb-2">{container.Name}</h1>
            <p class="text-sm text-gray-500 font-mono">{container.Id}</p>
        </div>

        <!-- Status Card -->
        <div class="bg-white rounded-lg shadow mb-6 p-6">
            <h2 class="text-xl font-semibold text-gray-900 mb-4">Status</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4">
                <div>
                    <span class="text-sm text-gray-600">State</span>
                    <p class="mt-1">
            <span class="inline-flex items-center px-3 py-1 rounded-full text-sm font-medium {getStatusColor(container.State.Status)}">
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
            <div class="mt-4 pt-4 border-t grid grid-cols-1 md:grid-cols-2 gap-4">
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

        <!-- Configuration Card -->
        <div class="bg-white rounded-lg shadow mb-6 p-6">
            <h2 class="text-xl font-semibold text-gray-900 mb-4">Configuration</h2>
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
                    <p class="mt-1 font-mono text-sm text-gray-900 bg-gray-50 p-3 rounded">
                        {container.Config.Cmd.join(' ')}
                    </p>
                </div>
            </div>
        </div>

        <!-- Environment Variables -->
        <div class="bg-white rounded-lg shadow mb-6 p-6">
            <h2 class="text-xl font-semibold text-gray-900 mb-4">Environment Variables</h2>
            <div class="space-y-2">
                {#each container.Config.Env as env}
                    {#if env.includes('=')}
                        {@const [key, ...valueParts] = env.split('=')}
                        {@const value = valueParts.join('=')}
                        <div class="flex items-start py-2 border-b last:border-b-0">
                            <span class="font-mono text-sm text-gray-600 w-64 flex-shrink-0">{key}</span>
                            <span class="font-mono text-sm text-gray-900 break-all">{value}</span>
                        </div>
                    {/if}
                {/each}
            </div>
        </div>

        <!-- Network Settings -->
        <div class="bg-white rounded-lg shadow mb-6 p-6">
            <h2 class="text-xl font-semibold text-gray-900 mb-4">Network Settings</h2>
            <div class="space-y-4">
                {#each Object.entries(container.NetworkSettings.Networks) as [networkName, network]}
                    <div class="border rounded-lg p-4">
                        <h3 class="font-medium text-gray-900 mb-3">{networkName}</h3>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
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
        <div class="bg-white rounded-lg shadow mb-6 p-6">
            <h2 class="text-xl font-semibold text-gray-900 mb-4">Mounts</h2>
            <div class="space-y-3">
                {#each container.Mounts as mount}
                    <div class="border rounded-lg p-4">
                        <div class="flex items-center justify-between mb-2">
              <span class="inline-flex items-center px-2.5 py-0.5 rounded text-xs font-medium bg-blue-100 text-blue-800">
                {mount.Type}
              </span>
                            <span class="text-xs text-gray-600">
                {mount.RW ? 'Read/Write' : 'Read-Only'}
              </span>
                        </div>
                        <div class="space-y-1 text-sm">
                            <div class="flex">
                                <span class="text-gray-600 w-24">Source:</span>
                                <span class="font-mono text-gray-900 break-all">{mount.Source}</span>
                            </div>
                            <div class="flex">
                                <span class="text-gray-600 w-24">Destination:</span>
                                <span class="font-mono text-gray-900">{mount.Destination}</span>
                            </div>
                        </div>
                    </div>
                {/each}
            </div>
        </div>

        <!-- Labels -->
        <div class="bg-white rounded-lg shadow mb-6 p-6">
            <h2 class="text-xl font-semibold text-gray-900 mb-4">Labels</h2>
            <div class="space-y-2">
                {#each Object.entries(container.Config.Labels) as [key, value]}
                    <div class="flex items-start py-2 border-b last:border-b-0">
                        <span class="font-mono text-sm text-gray-600 w-80 flex-shrink-0">{key}</span>
                        <span class="font-mono text-sm text-gray-900 break-all">{value}</span>
                    </div>
                {/each}
            </div>
        </div>

        <!-- Host Config -->
        <div class="bg-white rounded-lg shadow p-6">
            <h2 class="text-xl font-semibold text-gray-900 mb-4">Host Configuration</h2>
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                <div>
                    <span class="text-sm text-gray-600">Memory Limit</span>
                    <p class="mt-1 font-mono text-gray-900">
                        {container.HostConfig.Memory === 0 ? 'Unlimited' : `${container.HostConfig.Memory} bytes`}
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
