<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
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
      localeMessage = "Document language saved";
    } catch (e) {
      error = String(e);
    }
  }
</script>

<PageHeader
  title="Invoicing"
  subtitle="Tax rates, how invoices are numbered, and the language your documents print in."
/>

<AsyncState {loading} {error} rows={5} columns={2}>
  <div class="card">
    <h2>Tax rates</h2>

    <div class="add-row">
      <div class="field">
        <label for="taxName">Name</label>
        <input id="taxName" bind:value={taxName} placeholder="VAT" />
      </div>
      <div class="field rate">
        <label for="taxPercent">Rate %</label>
        <input id="taxPercent" bind:value={taxPercent} inputmode="decimal" placeholder="20" />
      </div>
      <button class="btn" onclick={addTaxRate} disabled={!taxName.trim()}>Add tax rate</button>
    </div>

    {#if taxRates.length === 0}
      <p class="muted">No tax rates yet.</p>
    {:else}
      <table>
        <thead>
          <tr><th>Name</th><th>Rate</th><th></th></tr>
        </thead>
        <tbody>
          {#each taxRates as rate (rate.id)}
            <tr>
              <td>
                {rate.name}
                {#if rate.archivedAt}<span class="muted"> (archived)</span>{/if}
              </td>
              <td>{(rate.rateBp / 100).toFixed(2)}%</td>
              <td>
                <button class="btn secondary" onclick={() => toggleTaxRate(rate)}>
                  {rate.archivedAt ? "Restore" : "Archive"}
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
    <h2>Invoice numbering</h2>

    <div class="grid-2">
      <div class="field">
        <label for="prefix">Prefix</label>
        <input id="prefix" bind:value={prefix} placeholder="INV-" />
      </div>
      <div class="field">
        <label for="nextValue">Next number</label>
        <input id="nextValue" type="number" min="1" bind:value={nextValue} />
      </div>
      <div class="field">
        <label for="padding">Zero padding</label>
        <input id="padding" type="number" min="1" max="12" bind:value={padding} />
      </div>
      <div class="field">
        <span class="field-label">Year in the number</span>
        <label class="row muted">
          <input id="reset" type="checkbox" bind:checked={resetYearly} />
          Include the year, and restart at 1 each January
        </label>
      </div>
    </div>

    <p class="muted">Next invoice will be <code class="inline">{numberPreview}</code></p>

    <div class="row wrap">
      <button class="btn" onclick={saveNumbering}>Save numbering</button>
      {#if numberSaved}<span class="ok">Saved</span>{/if}
      {#if numberError}<span class="error">{numberError}</span>{/if}
    </div>
  </div>

  <div class="card">
    <h2>Document language</h2>
    <p class="muted">
      The language used for the labels printed on invoices and estimates. The app itself is in
      English.
    </p>
    <div class="row wrap">
      <label class="locale">
        <span class="sr-only">Document language</span>
        <select bind:value={documentLocale}>
          <option value="en">English</option>
        </select>
      </label>
      <button class="btn secondary" onclick={saveDocumentLocale}>Save language</button>
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
