<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    title,
    subtitle = "",
    children,
  }: { title: string; subtitle?: string; children?: Snippet } = $props();
</script>

<div class="page-head">
  <div>
    <h1>{title}</h1>
    {#if subtitle}<p class="muted sub">{subtitle}</p>{/if}
  </div>
  {#if children}
    <div class="row wrap">{@render children()}</div>
  {/if}
</div>

<style>
  .sub {
    margin: var(--space-1) 0 0;
  }
</style>
