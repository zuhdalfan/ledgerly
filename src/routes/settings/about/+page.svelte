<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { t } from "$lib/i18n.svelte";
  import type { AppInfo } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import Logo from "$lib/components/Logo.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let info = $state<AppInfo | null>(null);
  let loading = $state(true);
  let error = $state("");

  onMount(async () => {
    try {
      info = await api.appInfo();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });
</script>

<PageHeader title={t("settings.nav.about")} />

<AsyncState {loading} {error} rows={3} columns={1}>
  <div class="card">
    <div class="about-brand">
      <Logo size={44} />
      <div>
        <h2>{info?.name ?? "Ledgerly"} {info?.version ? `v${info.version}` : ""}</h2>
        <p class="muted">{t("settings.about.tagline")}</p>
      </div>
    </div>
    <p class="muted">{t("settings.about.body")}</p>
    <p class="muted spaced">
      {t("settings.about.source")}
      <a class="link" href="https://github.com/zuhdalfan/ledgerly" target="_blank" rel="noreferrer">
        github.com/zuhdalfan/ledgerly
      </a>
    </p>
  </div>

  <div class="card">
    <h2>{t("settings.about.where")}</h2>
    <p class="muted">{t("settings.about.dataDir")}</p>
    <code>{info?.dataDir ?? "—"}</code>
    <p class="muted spaced">{t("settings.about.databaseFile")}</p>
    <code>{info?.databasePath ?? "—"}</code>
  </div>
</AsyncState>

<style>
  .about-brand {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    margin-bottom: var(--space-4);
  }

  .about-brand h2 {
    margin-bottom: var(--space-1);
  }

  .spaced {
    margin-top: var(--space-3);
  }
</style>
