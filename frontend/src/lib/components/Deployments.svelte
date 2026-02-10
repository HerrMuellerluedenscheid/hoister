<script lang="ts">
  import {goto} from "$app/navigation";

  let { data } = $props();
</script>

<div class="p-4">
  {#if data.length === 0 && !data.error}
    <div class="rounded border border-blue-200 bg-blue-50 px-4 py-3 text-blue-700">
      <p>No deployments found.</p>
    </div>
  {:else if data.length > 0}
    <div class="overflow-x-auto">
      <table class="min-w-full rounded-lg border border-gray-200 bg-white shadow">
        <thead class="bg-gray-50">
          <tr           >
            <th
              class="border-b px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-700 uppercase"
            >
              Host
            </th>
            <th
              class="border-b px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-700 uppercase"
            >
              Project | Service
            </th>
            <th
              class="border-b px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-700 uppercase"
            >
              Finished with Status
            </th>
            <th
              class="border-b px-6 py-3 text-left text-xs font-medium tracking-wider text-gray-700 uppercase"
            >
              Date
            </th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#each data as item}
            <tr class="transition-colors hover:bg-gray-50 cursor-pointer"
                onclick={() => goto(`/containers/${item.hostname}/${item.project_name}/${item.service_name}`)}
            >
              <td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500">
                {item.hostname}
              </td>
              <td class="px-6 py-4 text-sm whitespace-nowrap text-gray-900">
                <p>{item.project_name} | {item.service_name}</p>
                <p class="font-mono text-xs text-gray-500">Image ID: {item.digest.replace("sha256:", "").slice(0, 12)}</p>
              </td>
              <td class="px-6 py-4 text-sm whitespace-nowrap text-gray-600">
                <div class="flex items-center gap-2">
                  {#if item.status === 'Pending'}
                    <span class="text-lg">⏳</span>
                    <span>Pending</span>
                  {:else if item.status === 'Started'}
                    <span class="text-lg">🚀</span>
                    <span>Started</span>
                  {:else if item.status === 'Success'}
                    <span class="text-lg">✅</span>
                    <span class="font-medium text-green-600">Success</span>
                  {:else if item.status === 'Failed'}
                    <span class="text-lg">❌</span>
                    <span class="font-medium text-red-600">Update Failed</span>
                  {:else if item.status === 'RollbackFinished'}
                    <span class="text-lg">🔁</span>
                    <span class="font-medium text-blue-500">Rolled Back</span>
                  {:else if item.status === 'NoUpdate'}
                    <span class="text-lg">➖</span>
                    <span class="text-gray-500">No Update</span>
                  {:else}
                    <span>{item.status}</span>
                  {/if}
                </div>
              </td>
              <td class="px-6 py-4 text-sm whitespace-nowrap text-gray-500">
                {item.created_at}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>
