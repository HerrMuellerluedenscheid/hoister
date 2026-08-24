<script lang="ts">
  import { ansiStyleToCss, parseAnsi } from '$lib/ansi';
  import RedactedText from './RedactedText.svelte';

  // Renders a container log tail the way a terminal would: colour and emphasis
  // from ANSI escape sequences are applied instead of leaking into the output
  // as literal `[2m`, and the agent's redaction marker still becomes a badge.
  // Meant to be placed inside a `<pre>` so line breaks survive.
  let { text }: { text: string } = $props();

  let parts = $derived(
    parseAnsi(text).map((segment) => ({ text: segment.text, css: ansiStyleToCss(segment.style) }))
  );
</script>

{#each parts as part, i (i)}{#if part.css}<span style={part.css}><RedactedText text={part.text} /></span
  >{:else}<RedactedText text={part.text} />{/if}{/each}
