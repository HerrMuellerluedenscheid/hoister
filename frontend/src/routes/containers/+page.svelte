<script lang="ts">
    import InspectionCard from "$lib/components/InspectionCard.svelte";

    let { data }: { data: PageData } = $props();

</script>
<div class="max-w-7xl mx-auto p-6">
    <div class="grid grid-cols-[repeat(auto-fit,minmax(350px,1fr))] gap-4">
        {#each data.inspections as inspection}
            <InspectionCard {inspection} />
        {/each}
    </div>
</div>
