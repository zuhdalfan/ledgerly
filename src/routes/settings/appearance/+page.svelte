<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import Mascot from "$lib/components/Mascot.svelte";
  import { t, setLocale, getLocale, localeFromSetting, LOCALES, type Locale } from "$lib/i18n.svelte";

  const levels = [
    { value: "none", key: "appearance.none", hint: "appearance.noneHint" },
    { value: "reduced", key: "appearance.reduced", hint: "appearance.reducedHint" },
    { value: "standard", key: "appearance.standard", hint: "appearance.standardHint" },
    { value: "enhanced", key: "appearance.enhanced", hint: "appearance.enhancedHint" },
  ] as const;

  let level = $state("standard");
  let language = $state<Locale>(getLocale());
  let loading = $state(true);
  let error = $state("");
  let notice = $state("");
  let languageNotice = $state("");

  async function load() {
    loading = true;
    try {
      level = (await api.settings.get("ui.motion")) ?? "standard";
      language = localeFromSetting(await api.settings.get("ui.locale"));
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
      languageNotice = t("appearance.languageSet", { language: value });
    } catch (e) {
      error = String(e);
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
