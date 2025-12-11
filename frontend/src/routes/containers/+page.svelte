<script lang="ts">

    let { data } = $props();

</script>
{data.inspections}

{#each data.inspections as item}
    <div class="item">
        <h2>{item.Id}</h2>
        <p>{item.State.Status}</p>
    </div>
{/each}
