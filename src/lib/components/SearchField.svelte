<script lang="ts">
  // The accessible name lives in a real label, never in the placeholder. The
  // placeholder may only show an example of what to type.
  let {
    value = $bindable(""),
    label,
    example = "",
    oninput,
  }: {
    value?: string;
    label: string;
    example?: string;
    oninput?: () => void;
  } = $props();
</script>

<label class="search">
  <span class="sr-only">{label}</span>
  <input type="search" bind:value {oninput} placeholder={example} spellcheck="false" />
</label>

<style>
  .search {
    display: block;
    flex: 1 1 16rem;
    min-width: 12rem;
  }
</style>
