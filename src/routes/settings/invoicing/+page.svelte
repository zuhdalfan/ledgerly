<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { t } from "$lib/i18n.svelte";
  import type { TaxRate } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let taxRates = $state<TaxRate[]>([]);
  let taxName = $state("");
  let taxPercent = $state("");
  let taxError = $state("");

  let prefix = $state("INV-");
  let nextValue = $state(1);
  let padding = $state(4);
  let resetYearly = $state(false);
  let numberError = $state("");
  let numberSaved = $state(false);

  let documentLocale = $state("en");
  let localeMessage = $state("");

  let loading = $state(true);
  let error = $state("");

  const numberPreview = $derived(
    `${prefix}${resetYearly ? `${new Date().getFullYear()}-` : ""}${String(nextValue).padStart(
      Math.min(Math.max(padding, 1), 12),
      "0"
    )}`
  );

  async function load() {
    loading = true;
    try {
      const [rates, numbering, locale] = await Promise.all([
        api.taxRates.list(true),
        api.numbering.get("invoice"),
        api.settings.get("pdf.locale"),
      ]);
      taxRates = rates;
      prefix = numbering.prefix;
      nextValue = numbering.nextValue;
      padding = numbering.padding;
      resetYearly = numbering.resetYearly;
      documentLocale = locale ?? "en";
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

  async function loadTaxRates() {
    taxRates = await api.taxRates.list(true);
  }

  async function addTaxRate() {
    if (!taxName.trim()) return;
    taxError = "";
    try {
      const percent = Number.parseFloat(taxPercent);
      await api.taxRates.create({
        name: taxName.trim(),
        rateBp: Number.isFinite(percent) ? Math.round(percent * 100) : 0,
      });
      taxName = "";
      taxPercent = "";
      await loadTaxRates();
    } catch (e) {
      taxError = String(e);
    }
  }

  async function toggleTaxRate(rate: TaxRate) {
    try {
      await api.taxRates.setArchived(rate.id, rate.archivedAt === null);
      await loadTaxRates();
    } catch (e) {
      taxError = String(e);
    }
  }

  async function saveNumbering() {
    numberError = "";
    numberSaved = false;
    try {
      const updated = await api.numbering.update("invoice", {
        prefix,
        nextValue: Number(nextValue) || 1,
        padding: Number(padding) || 4,
        resetYearly,
      });
      prefix = updated.prefix;
      nextValue = updated.nextValue;
      padding = updated.padding;
      resetYearly = updated.resetYearly;
      numberSaved = true;
    } catch (e) {
      numberError = String(e);
    }
  }

  async function saveDocumentLocale() {
    localeMessage = "";
    error = "";
    try {
      await api.settings.set("pdf.locale", documentLocale);
      localeMessage = t("settings.invoicing.languageSaved");
    } catch (e) {
      error = String(e);
    }
  }
</script>

<PageHeader
  title={t("settings.nav.invoicing")}
  subtitle={t("settings.invoicing.subtitle")}
/>

<AsyncState {loading} {error} rows={5} columns={2}>
  <div class="card">
    <h2>{t("settings.invoicing.taxRates")}</h2>

    <div class="add-row">
      <div class="field">
        <label for="taxName">{t("common.name")}</label>
        <input
          id="taxName"
          bind:value={taxName}
          placeholder={t("settings.invoicing.taxNamePlaceholder")}
        />
      </div>
      <div class="field rate">
        <label for="taxPercent">{t("settings.invoicing.ratePercent")}</label>
        <input id="taxPercent" bind:value={taxPercent} inputmode="decimal" placeholder="20" />
      </div>
      <button class="btn" onclick={addTaxRate} disabled={!taxName.trim()}>
        {t("action.addTaxRate")}
      </button>
    </div>

    {#if taxRates.length === 0}
      <p class="muted">{t("settings.invoicing.noTaxRates")}</p>
    {:else}
      <table>
        <thead>
          <tr><th>{t("common.name")}</th><th>{t("common.rate")}</th><th></th></tr>
        </thead>
        <tbody>
          {#each taxRates as rate (rate.id)}
            <tr>
              <td>
                {rate.name}
                {#if rate.archivedAt}<span class="muted"> {t("common.archived")}</span>{/if}
              </td>
              <td>{(rate.rateBp / 100).toFixed(2)}%</td>
              <td>
                <button class="btn secondary" onclick={() => toggleTaxRate(rate)}>
                  {rate.archivedAt ? t("action.restore") : t("action.archive")}
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    {#if taxError}<p class="error">{taxError}</p>{/if}
  </div>

  <div class="card">
    <h2>{t("settings.invoicing.numbering")}</h2>

    <div class="grid-2">
      <div class="field">
        <label for="prefix">{t("settings.invoicing.prefix")}</label>
        <input id="prefix" bind:value={prefix} placeholder="INV-" />
      </div>
      <div class="field">
        <label for="nextValue">{t("settings.invoicing.nextNumber")}</label>
        <input id="nextValue" type="number" min="1" bind:value={nextValue} />
      </div>
      <div class="field">
        <label for="padding">{t("settings.invoicing.padding")}</label>
        <input id="padding" type="number" min="1" max="12" bind:value={padding} />
      </div>
      <div class="field">
        <span class="field-label">{t("settings.invoicing.yearInNumber")}</span>
        <label class="row muted">
          <input id="reset" type="checkbox" bind:checked={resetYearly} />
          {t("settings.invoicing.yearToggle")}
        </label>
      </div>
    </div>

    <p class="muted">
      {t("settings.invoicing.nextPreview")} <code class="inline">{numberPreview}</code>
    </p>

    <div class="row wrap">
      <button class="btn" onclick={saveNumbering}>{t("action.saveNumbering")}</button>
      {#if numberSaved}<span class="ok">{t("action.saved")}</span>{/if}
      {#if numberError}<span class="error">{numberError}</span>{/if}
    </div>
  </div>

  <div class="card">
    <h2>{t("settings.invoicing.documentLanguage")}</h2>
    <p class="muted">{t("settings.invoicing.documentLanguageBody")}</p>
    <div class="row wrap">
      <label class="locale">
        <span class="sr-only">{t("settings.invoicing.documentLanguage")}</span>
        <select bind:value={documentLocale}>
          <option value="en">English</option>
          <option value="id">Bahasa Indonesia</option>
        </select>
      </label>
      <button class="btn secondary" onclick={saveDocumentLocale}>
        {t("action.saveLanguage")}
      </button>
      {#if localeMessage}<span class="ok">{localeMessage}</span>{/if}
    </div>
  </div>
</AsyncState>

<style>
  .add-row {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-3);
    align-items: flex-end;
    margin-bottom: var(--space-4);
  }

  .add-row .field {
    margin-bottom: 0;
  }

  .add-row .rate {
    flex: 0 0 7rem;
  }

  .field-label {
    font-size: var(--text-sm);
    font-weight: 700;
    color: var(--ink-2);
  }

  .locale select {
    width: auto;
  }

  .inline {
    display: inline;
  }
</style>
