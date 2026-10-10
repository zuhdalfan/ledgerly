<script lang="ts">
  import { page } from "$app/state";
  import Mascot from "$lib/components/Mascot.svelte";

  const isMissing = $derived(page.status === 404);
</script>

<div class="error-page">
  <Mascot expression={isMissing ? "thinking" : "worried"} size={88} />
  <h1>{isMissing ? "That page is not here" : "Something went wrong"}</h1>
  <p class="muted">
    {isMissing
      ? "The link may be old, or the record may have been removed."
      : page.error?.message ?? "Ledgerly could not finish that. Your data has not been changed."}
  </p>
  {#if page.status !== 404}
    <p class="muted status">Error {page.status}</p>
  {/if}
  <div class="row wrap">
    <a class="btn" href="/">Back to the dashboard</a>
  </div>
</div>

<style>
  .error-page {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-3);
    max-width: 30rem;
    margin: 0 auto;
    padding: var(--space-6) var(--space-4);
    text-align: center;
  }

  h1 {
    font-size: var(--text-xl);
  }

  .status {
    font-size: var(--text-sm);
  }
</style>
