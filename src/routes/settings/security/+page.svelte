<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { t } from "$lib/i18n.svelte";
  import type { SecurityStatus } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let security = $state<SecurityStatus | null>(null);
  let lockPassphrase = $state("");
  let lockConfirm = $state("");
  let lockCurrent = $state("");
  let error = $state("");
  let message = $state("");
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      security = await api.security.status();
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void load();
  });

  async function enableLock() {
    error = "";
    message = "";
    if (lockPassphrase !== lockConfirm) {
      error = t("settings.security.mismatch");
      return;
    }

    try {
      security = await api.security.enable(
        lockPassphrase,
        security?.settings.autoLockMinutes ?? 15
      );
      lockPassphrase = "";
      lockConfirm = "";
      message = t("settings.security.enabledMessage");
    } catch (e) {
      error = String(e);
    }
  }

  async function disableLock() {
    error = "";
    message = "";
    try {
      security = await api.security.disable(lockCurrent);
      lockCurrent = "";
      lockPassphrase = "";
      lockConfirm = "";
      message = t("settings.security.disabledMessage");
    } catch (e) {
      error = String(e);
    }
  }

  async function saveAutoLock(minutes: number) {
    error = "";
    try {
      security = await api.security.setAutoLock(minutes);
    } catch (e) {
      error = String(e);
    }
  }

  async function lockNow() {
    error = "";
    try {
      await api.security.lock();
      // The shell owns the lock state, so reload to show the lock screen.
      window.location.reload();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<PageHeader
  title={t("settings.nav.security")}
  subtitle={t("settings.security.subtitle")}
/>

<AsyncState {loading} {error} rows={4} columns={1}>
  <div class="card">
    {#if security?.settings.enabled}
      <h2>{t("settings.security.lockOn")}</h2>
      <p class="muted">
        {t("settings.security.lockOnBody", { minutes: security.settings.autoLockMinutes })}
      </p>

      <div class="field">
        <label for="autoLock">{t("settings.security.lockAfter")}</label>
        <input
          id="autoLock"
          type="number"
          min="1"
          max="240"
          value={security.settings.autoLockMinutes}
          onchange={(e) => saveAutoLock(Number(e.currentTarget.value))}
        />
      </div>

      <div class="row wrap">
        <button class="btn secondary" onclick={lockNow}>{t("action.lockNow")}</button>
      </div>

      <h2 class="spaced">{t("settings.security.turnOffTitle")}</h2>
      <div class="field">
        <label for="lockCurrent">{t("settings.security.currentPassphrase")}</label>
        <input id="lockCurrent" type="password" bind:value={lockCurrent} />
      </div>
      <div class="row wrap">
        <button class="btn danger" onclick={disableLock} disabled={!lockCurrent}>
          {t("action.turnOff")}
        </button>
      </div>
    {:else}
      <h2>{t("settings.security.enableTitle")}</h2>
      <p class="muted">{t("settings.security.enableBody")}</p>

      <div class="grid-2">
        <div class="field">
          <label for="lockPassphrase">{t("settings.security.passphrase")}</label>
          <input id="lockPassphrase" type="password" bind:value={lockPassphrase} />
        </div>
        <div class="field">
          <label for="lockConfirm">{t("settings.security.confirmPassphrase")}</label>
          <input id="lockConfirm" type="password" bind:value={lockConfirm} />
        </div>
      </div>
      <p class="muted">
        {t("settings.security.minLength", { count: security?.minPassphraseLength ?? 8 })}
      </p>
      <div class="row wrap">
        <button class="btn" onclick={enableLock} disabled={!lockPassphrase || !lockConfirm}>
          {t("action.enableAppLock")}
        </button>
      </div>
    {/if}

    {#if message}<p class="ok">{message}</p>{/if}
    {#if error}<p class="error">{error}</p>{/if}
  </div>
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-5);
  }
</style>
