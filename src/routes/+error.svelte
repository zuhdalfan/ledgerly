<script lang="ts">
  import { page } from "$app/state";
  import Mascot from "$lib/components/Mascot.svelte";
  import { t } from "$lib/i18n.svelte";

  const isMissing = $derived(page.status === 404);
</script>

<div class="error-page">
  <Mascot expression={isMissing ? "thinking" : "worried"} size={88} />
  <h1>{isMissing ? t("error.notHere") : t("error.wentWrong")}</h1>
  <p class="muted">
    {isMissing
      ? t("error.notHereBody")
      : (page.error?.message ?? t("error.wentWrongBody"))}
  </p>
  {#if page.status !== 404}
    <p class="muted status">{t("error.status", { status: page.status })}</p>
  {/if}
  <div class="row wrap">
    <a class="btn" href="/">{t("error.backToDashboard")}</a>
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
