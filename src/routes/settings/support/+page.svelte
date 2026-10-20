<script lang="ts">
  import { SUPPORT_URL } from "$lib/support";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import Mascot from "$lib/components/Mascot.svelte";
  import { t } from "$lib/i18n.svelte";

  const year = new Date().getFullYear();
</script>

<PageHeader title={t("settings.nav.support")} subtitle={t("settings.support.subtitle")} />

<div class="card">
  <h2>{t("settings.support.freeTitle")}</h2>
  <p class="muted">{t("settings.support.freeBody")}</p>
  <p class="muted">{t("settings.support.freeBody2")}</p>
</div>

<div class="card support">
  <Mascot expression="waving" size={72} />
  <div>
    <h2>{t("settings.support.keepGoing")}</h2>
    <p class="muted">{t("settings.support.keepGoingBody")}</p>
    {#if SUPPORT_URL}
      <a class="btn" href={SUPPORT_URL} target="_blank" rel="noreferrer">
        {t("action.supportUs")}
      </a>
    {:else}
      <button class="btn" disabled>{t("action.supportUs")}</button>
      <p class="muted note">
        {t("settings.support.notWiredStart")}
        <code>src/lib/support.ts</code>{t("settings.support.notWiredEnd")}
      </p>
    {/if}
  </div>
</div>

<div class="card">
  <h2>{t("settings.support.licence")}</h2>
  <p class="muted">{t("settings.support.licenceBody", { year })}</p>
  <p class="muted">
    {t("settings.support.licenceBody2Start")}
    <code>LICENSE</code>
    {t("settings.support.licenceBody2End")}
  </p>
</div>

<style>
  .support {
    display: flex;
    align-items: center;
    gap: var(--space-5);
  }

  .note {
    margin-top: var(--space-2);
    font-size: var(--text-sm);
  }

  @media (max-width: 640px) {
    .support {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
