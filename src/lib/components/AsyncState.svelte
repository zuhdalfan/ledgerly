<script lang="ts">
  import type { Snippet } from "svelte";
  import Skeleton from "./Skeleton.svelte";

  // One place decides what "still working", "broken" and "nothing here" look
  // like, so eight screens cannot drift apart.
  let {
    loading = false,
    error = "",
    rows = 4,
    columns = 1,
    children,
  }: {
    loading?: boolean;
    error?: string;
    rows?: number;
    columns?: number;
    children: Snippet;
  } = $props();
</script>

{#if error}
  <p class="error" role="alert">{error}</p>
{:else if loading}
  <Skeleton {rows} {columns} />
{:else}
  {@render children()}
{/if}
