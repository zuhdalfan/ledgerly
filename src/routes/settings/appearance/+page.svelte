<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import Mascot from "$lib/components/Mascot.svelte";
  import { t, setLocale, getLocale, localeFromSetting, LOCALES, type Locale } from "$lib/i18n.svelte";
  import type { TrayStatus } from "$lib/types";

  const levels = [
    { value: "none", key: "appearance.none", hint: "appearance.noneHint" },
    { value: "reduced", key: "appearance.reduced", hint: "appearance.reducedHint" },
    { value: "standard", key: "appearance.standard", hint: "appearance.standardHint" },
    { value: "enhanced", key: "appearance.enhanced", hint: "appearance.enhancedHint" },
  ] as const;

  let level = $state("standard");
  let language = $state<Locale>(getLocale());
  let tray = $state<TrayStatus | null>(null);
  let loading = $state(true);
  let error = $state("");
  let notice = $state("");
  let languageNotice = $state("");
  let windowNotice = $state("");
  let testNotice = $state("");
  let windowBusy = $state(false);

  async function load() {
    loading = true;
    try {
      level = (await api.settings.get("ui.motion")) ?? "standard";
      language = localeFromSetting(await api.settings.get("ui.locale"));
      tray = await api.tray.status();
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

  async function choose(value: string) {
    notice = "";
    error = "";
    level = value;
    document.documentElement.dataset.motion = value;
    const option = levels.find((entry) => entry.value === value);
    try {
      await api.settings.set("ui.motion", value);
      notice = t("appearance.motionSet", { level: option ? t(option.key) : value });
    } catch (e) {
      error = String(e);
    }
  }

  async function chooseLanguage(value: Locale) {
    languageNotice = "";
    error = "";
    language = value;
    // Apply immediately so the switch is felt, then persist.
    setLocale(value);
    try {
      await api.settings.set("ui.locale", value);
      const endonym = LOCALES.find((option) => option.value === value)?.label ?? value;
      languageNotice = t("appearance.languageSet", { language: endonym });
    } catch (e) {
      error = String(e);
    }
  }

  async function setTray(enabled: boolean) {
    windowNotice = "";
    error = "";
    windowBusy = true;
    try {
      tray = await api.tray.setEnabled(enabled, tray?.lockOnHide ?? true);
      windowNotice = t(enabled ? "appearance.trayOn" : "appearance.trayOff");
    } catch (e) {
      error = String(e);
      // The backend refused (no tray host) and left the setting off.
      tray = await api.tray.status().catch(() => tray);
    } finally {
      windowBusy = false;
    }
  }

  async function setLockOnHide(value: boolean) {
    windowNotice = "";
    error = "";
    windowBusy = true;
    try {
      tray = await api.tray.setEnabled(tray?.enabled ?? false, value);
    } catch (e) {
      error = String(e);
    } finally {
      windowBusy = false;
    }
  }

  async function setReminders(value: boolean) {
    windowNotice = "";
    error = "";
    windowBusy = true;
    try {
      tray = await api.reminders.setEnabled(value);
    } catch (e) {
      error = String(e);
    } finally {
      windowBusy = false;
    }
  }

  async function sendTest() {
    testNotice = "";
    error = "";
    windowBusy = true;
    try {
      await api.reminders.test();
      testNotice = t("appearance.testSent");
    } catch (e) {
      error = String(e);
    } finally {
      windowBusy = false;
    }
  }
</script>

<PageHeader title={t("appearance.title")} subtitle={t("appearance.subtitle")} />

<AsyncState {loading} {error} rows={4} columns={1}>
  <div class="card">
    <h2>{t("appearance.motion")}</h2>

    <fieldset class="levels">
      <legend class="sr-only">{t("appearance.motionLevel")}</legend>
      {#each levels as option (option.value)}
        <label class="level" class:chosen={level === option.value}>
          <input
            type="radio"
            name="motion"
            value={option.value}
            checked={level === option.value}
            onchange={() => choose(option.value)}
          />
          <span class="text">
            <span class="name">{t(option.key)}</span>
            <span class="muted">{t(option.hint)}</span>
          </span>
        </label>
      {/each}
    </fieldset>

    {#if notice}<p class="ok">{notice}</p>{/if}
  </div>

  <div class="card">
    <h2>{t("appearance.window")}</h2>

    {#if tray}
      {#if !tray.supported}
        <p class="muted">{t("appearance.trayUnsupported")}</p>
      {:else}
        <label class="toggle">
          <input
            type="checkbox"
            checked={tray.enabled}
            disabled={windowBusy}
            onchange={(e) => setTray(e.currentTarget.checked)}
          />
          <span>
            <strong>{t("appearance.tray")}</strong>
            <span class="muted">{t("appearance.trayHint")}</span>
          </span>
        </label>

        {#if tray.enabled && tray.lockEnabled}
          <label class="toggle">
            <input
              type="checkbox"
              checked={tray.lockOnHide}
              disabled={windowBusy}
              onchange={(e) => setLockOnHide(e.currentTarget.checked)}
            />
            <span>
              <strong>{t("appearance.lockOnHide")}</strong>
              <span class="muted">{t("appearance.lockOnHideHint")}</span>
            </span>
          </label>
        {/if}

        {#if tray.enabled}
          <label class="toggle">
            <input
              type="checkbox"
              checked={tray.remindersEnabled}
              disabled={windowBusy}
              onchange={(e) => setReminders(e.currentTarget.checked)}
            />
            <span>
              <strong>{t("appearance.reminders")}</strong>
              <span class="muted">{t("appearance.remindersHint")}</span>
            </span>
          </label>

          {#if tray.notificationPermission === "denied"}
            <p class="muted">{t("appearance.remindersBlocked")}</p>
          {/if}
        {/if}
      {/if}

      <div class="test">
        <button class="btn secondary" type="button" onclick={sendTest} disabled={windowBusy}>
          {t("appearance.testNotification")}
        </button>
        <p class="muted">{t("appearance.testNotificationHint")}</p>
        {#if testNotice}<p class="ok">{testNotice}</p>{/if}
      </div>
    {/if}

    {#if windowNotice}<p class="ok">{windowNotice}</p>{/if}
  </div>

  <div class="card">
    <h2>{t("appearance.language")}</h2>
    <p class="muted">{t("appearance.languageBody")}</p>

    <fieldset class="levels">
      <legend class="sr-only">{t("appearance.language")}</legend>
      {#each LOCALES as option (option.value)}
        <label class="level" class:chosen={language === option.value}>
          <input
            type="radio"
            name="language"
            value={option.value}
            checked={language === option.value}
            onchange={() => chooseLanguage(option.value)}
          />
          <span class="text">
            <span class="name">{option.label}</span>
          </span>
        </label>
      {/each}
    </fieldset>

    {#if languageNotice}<p class="ok">{languageNotice}</p>{/if}
  </div>

  {#if error}<p class="error">{error}</p>{/if}

  <div class="card preview">
    <Mascot expression={level === "enhanced" ? "celebrating" : "pleased"} size={56} />
    <p class="muted">{t("appearance.mascot")}</p>
  </div>
</AsyncState>

<style>
  .levels {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    border: 0;
  }

  .level {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    padding: var(--space-3);
    border: 1px solid var(--paper-3);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease-out), background var(--dur-fast) var(--ease-out);
  }

  .level:hover {
    background: var(--paper-2);
  }

  .level.chosen {
    border-color: var(--accent-text);
    background: var(--accent-soft);
  }

  .toggle {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    padding: var(--space-2) 0;
    cursor: pointer;
  }

  .toggle input {
    margin-top: 3px;
  }

  .toggle span {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .toggle .muted {
    font-size: var(--text-sm);
  }

  .test {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    align-items: flex-start;
    margin-top: var(--space-3);
  }

  .test p {
    margin: 0;
    font-size: var(--text-sm);
  }

  .text {
    display: flex;
    flex-direction: column;
  }

  .name {
    font-weight: 700;
  }

  .preview {
    display: flex;
    gap: var(--space-4);
    align-items: center;
  }
</style>
