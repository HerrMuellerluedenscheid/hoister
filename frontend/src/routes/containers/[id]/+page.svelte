<script lang="ts">
    import type { PageData } from './$types';

    let { data }: { data: PageData } = $props();
</script>

<div class="max-w-4xl mx-auto p-6">
    <pre class="text-xs">{JSON.stringify(data.inspections, null, 2)}</pre>
</div>
