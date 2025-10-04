<script lang="ts">
    import '../app.css';
    import favicon from '$lib/assets/favicon.svg';

    let { data } = $props();
    let items = data.data.data;

</script>

<div class="p-4">
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
                    Created At
                </th>
            </tr>
            </thead>
            <tbody class="divide-y divide-gray-200">
            {#each items as item}
                <tr class="hover:bg-gray-50 transition-colors">
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                        {item.digest}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-600">
                        {item.status}
                    </td>
                    <td class="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                        {item.created_at}
                    </td>
                </tr>
            {/each}
            </tbody>
        </table>
    </div>
</div>
