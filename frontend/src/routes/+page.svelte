<script lang="ts">
  import '../app.css';

  let { data } = $props();
  import { source } from 'sveltekit-sse';
  const value = source('/custom-event').select('message');
</script>

<div>
  {$value}
</div>
