<script lang="ts">
    import '../app.css';
    import favicon from '$lib/assets/favicon.svg';
    import type { Deployment } from "../bindings/Deployment";
    import Deployments from "./Deployments.svelte";

    let { data } = $props();
</script>
<div>
    <div class="flex items-center space-x-2 m-4">
        <h1 class="text-2xl font-bold text-gray-800">🏗 Hoister</h1>
    </div>
</div>
<Deployments {data}></Deployments>
