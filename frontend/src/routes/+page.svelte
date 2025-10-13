<script lang="ts">
    import '../app.css';
    import Deployments from "./Deployments.svelte";

    let { data } = $props();
    import { source } from 'sveltekit-sse'
    const value = source('/custom-event').select('message')

</script>
<div>
    {$value}

    <div class="flex items-center space-x-2 m-4">
        <h1 class="text-2xl font-bold text-gray-800">🏗 Hoister</h1>
    </div>
</div>
<Deployments {data}></Deployments>
