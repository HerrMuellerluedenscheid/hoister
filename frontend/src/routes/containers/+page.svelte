<script lang="ts">
    import InspectionCard from "$lib/components/InspectionCard.svelte";

    let { data }: { data: PageData } = $props();

</script>
<div class="max-w-4xl mx-auto p-6">
    <h1 class="text-2xl font-bold mb-6 text-gray-900">Inspections</h1>

    <div class="columns-2 gap-4 space-y-4">
        {#each data.inspections as inspection}
            <InspectionCard {inspection} />
        {/each}
    </div>
</div>
