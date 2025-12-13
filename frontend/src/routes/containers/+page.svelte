<script lang="ts">
    import InspectionCard from "$lib/components/InspectionCard.svelte";

    let { data }: { data: PageData } = $props();

</script>
{#if data.error}
    <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded mb-4">
        <p class="font-bold">Error</p>
        <p>{data.error}</p>
    </div>
{/if}


<div class="max-w-7xl mx-auto p-6">
    <div class="grid grid-cols-[repeat(auto-fit,minmax(350px,1fr))] gap-4">
        {#each data.inspections as inspection}
            <InspectionCard {inspection} />
        {/each}
    </div>
</div>
