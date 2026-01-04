<script lang="ts">
  let { data } = $props();
  import Deployments from '$lib/components/Deployments.svelte';
</script>

<Deployments data={data.deployments} />
