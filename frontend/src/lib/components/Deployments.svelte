<script lang="ts">
    let { data } = $props();
</script>

<div class="p-4">
    {#if data.error}
        <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4">
            <p class="font-bold">Error</p>
            <p>{data.error}</p>
        </div>
    {/if}

    {#if data.deployments.length === 0 && !data.error}
        <div class="bg-blue-50 border border-blue-200 text-blue-700 px-4 py-3 rounded">
            <p>No deployments found.</p>
        </div>
    {:else if data.deployments.length > 0}
        <div class="overflow-x-auto">
            <table class="min-w-full bg-white border border-gray-200 rounded-lg shadow">
                <thead class="bg-gray-50">
                <tr>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider border-b">
                        Digest
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider border-b">
                        Status
                    </th>
                    <th class="px-6 py-3 text-left text-xs font-medium text-gray-700 uppercase tracking-wider border-b">
                        Date
                    </th>
                </tr>
                </thead>
                <tbody class="divide-y divide-gray-200">
                {#each data.deployments as item}
                    <tr class="hover:bg-gray-50 transition-colors">
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                            {item.digest}
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-600">
                            <div class="flex items-center gap-2">
                                {#if item.status === 'Pending'}
                                    <span class="text-lg">⏳</span>
                                    <span>Pending</span>
                                {:else if item.status === 'Started'}
                                    <span class="text-lg">🚀</span>
                                    <span>Started</span>
                                {:else if item.status === 'Success'}
                                    <span class="text-lg">✅</span>
                                    <span class="text-green-600 font-medium">Success</span>
                                {:else if item.status === 'RollbackFinished'}
                                    <span class="text-lg">❌</span>
                                    <span class="text-red-600 font-medium">Rolled back</span>
                                {:else if item.status === 'NoUpdate'}
                                    <span class="text-lg">➖</span>
                                    <span class="text-gray-500">No Update</span>
                                {:else}
                                    <span>{item.status}</span>
                                {/if}
                            </div>
                        </td>
                        <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                            {item.created_at}
                        </td>
                    </tr>
                {/each}
                </tbody>
            </table>
        </div>
    {/if}
</div>
