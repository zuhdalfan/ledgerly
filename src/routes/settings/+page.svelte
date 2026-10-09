<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { emptyToNull, nullToEmpty } from "$lib/format";
  import type { AppInfo, BusinessInput, TaxRate } from "$lib/types";

  type Form = Record<keyof BusinessInput, string>;

  const emptyForm = (): Form => ({
    displayName: "",
    legalName: "",
    email: "",
    phone: "",
    website: "",
    addressLine1: "",
    addressLine2: "",
    city: "",
    region: "",
    postalCode: "",
    country: "",
    taxId: "",
    defaultCurrency: "USD",
    defaultTerms: "",
    defaultNotes: "",
    defaultPaymentInstructions: "",
  });

  let form = $state<Form>(emptyForm());
  let info = $state<AppInfo | null>(null);
  let error = $state("");
  let saved = $state(false);
  let saving = $state(false);

  // Tax rates
  let taxRates = $state<TaxRate[]>([]);
  let taxName = $state("");
  let taxPercent = $state("");
  let taxError = $state("");

  // Numbering
  let prefix = $state("INV-");
  let nextValue = $state(1);
  let padding = $state(4);
  let resetYearly = $state(false);
  let numberError = $state("");
  let numberSaved = $state(false);

  const numberPreview = $derived(
    `${prefix}${resetYearly ? `${new Date().getFullYear()}-` : ""}${String(nextValue).padStart(
      Math.min(Math.max(padding, 1), 12),
      "0"
    )}`
  );

  async function loadTaxRates() {
    taxRates = await api.taxRates.list(true);
  }

  onMount(async () => {
    try {
      const [business, appInfo, numbering] = await Promise.all([
        api.business.get(),
        api.appInfo(),
        api.numbering.get("invoice"),
      ]);
      info = appInfo;
      form = {
        displayName: business.displayName,
        legalName: nullToEmpty(business.legalName),
        email: nullToEmpty(business.email),
        phone: nullToEmpty(business.phone),
        website: nullToEmpty(business.website),
        addressLine1: nullToEmpty(business.addressLine1),
        addressLine2: nullToEmpty(business.addressLine2),
        city: nullToEmpty(business.city),
        region: nullToEmpty(business.region),
        postalCode: nullToEmpty(business.postalCode),
        country: nullToEmpty(business.country),
        taxId: nullToEmpty(business.taxId),
        defaultCurrency: business.defaultCurrency,
        defaultTerms: nullToEmpty(business.defaultTerms),
        defaultNotes: nullToEmpty(business.defaultNotes),
        defaultPaymentInstructions: nullToEmpty(business.defaultPaymentInstructions),
      };

      prefix = numbering.prefix;
      nextValue = numbering.nextValue;
      padding = numbering.padding;
      resetYearly = numbering.resetYearly;

      await loadTaxRates();
    } catch (e) {
      error = String(e);
    }
  });

  async function save() {
    saving = true;
    saved = false;
    error = "";
    try {
      await api.business.update({
        displayName: form.displayName.trim(),
        legalName: emptyToNull(form.legalName),
        email: emptyToNull(form.email),
        phone: emptyToNull(form.phone),
        website: emptyToNull(form.website),
        addressLine1: emptyToNull(form.addressLine1),
        addressLine2: emptyToNull(form.addressLine2),
        city: emptyToNull(form.city),
        region: emptyToNull(form.region),
        postalCode: emptyToNull(form.postalCode),
        country: emptyToNull(form.country),
        taxId: emptyToNull(form.taxId),
        defaultCurrency: form.defaultCurrency.trim() || "USD",
        defaultTerms: emptyToNull(form.defaultTerms),
        defaultNotes: emptyToNull(form.defaultNotes),
        defaultPaymentInstructions: emptyToNull(form.defaultPaymentInstructions),
      });
      saved = true;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
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
</script>

<div class="page-head">
  <h1>Settings</h1>
</div>

<div class="card">
  <h2>Business profile</h2>

  <div class="field">
    <label for="displayName">Business name</label>
    <input id="displayName" bind:value={form.displayName} placeholder="Your business" />
  </div>

  <div class="grid-2">
    <div class="field">
      <label for="legalName">Legal name</label>
      <input id="legalName" bind:value={form.legalName} />
    </div>
    <div class="field">
      <label for="taxId">Tax / VAT ID</label>
      <input id="taxId" bind:value={form.taxId} />
    </div>
    <div class="field">
      <label for="email">Email</label>
      <input id="email" type="email" bind:value={form.email} />
    </div>
    <div class="field">
      <label for="phone">Phone</label>
      <input id="phone" bind:value={form.phone} />
    </div>
    <div class="field">
      <label for="website">Website</label>
      <input id="website" bind:value={form.website} />
    </div>
    <div class="field">
      <label for="defaultCurrency">Default currency</label>
      <input id="defaultCurrency" bind:value={form.defaultCurrency} />
    </div>
  </div>

  <h2>Address</h2>
  <div class="grid-2">
    <div class="field">
      <label for="addressLine1">Address line 1</label>
      <input id="addressLine1" bind:value={form.addressLine1} />
    </div>
    <div class="field">
      <label for="addressLine2">Address line 2</label>
      <input id="addressLine2" bind:value={form.addressLine2} />
    </div>
    <div class="field">
      <label for="city">City</label>
      <input id="city" bind:value={form.city} />
    </div>
    <div class="field">
      <label for="region">Region / state</label>
      <input id="region" bind:value={form.region} />
    </div>
    <div class="field">
      <label for="postalCode">Postal code</label>
      <input id="postalCode" bind:value={form.postalCode} />
    </div>
    <div class="field">
      <label for="country">Country</label>
      <input id="country" bind:value={form.country} />
    </div>
  </div>

  <h2>Invoice defaults</h2>
  <div class="field">
    <label for="defaultTerms">Default terms</label>
    <input id="defaultTerms" bind:value={form.defaultTerms} placeholder="e.g. Net 30" />
  </div>
  <div class="field">
    <label for="defaultNotes">Default notes</label>
    <textarea id="defaultNotes" bind:value={form.defaultNotes}></textarea>
  </div>
  <div class="field">
    <label for="defaultPaymentInstructions">Payment instructions</label>
    <textarea id="defaultPaymentInstructions" bind:value={form.defaultPaymentInstructions}
    ></textarea>
  </div>

  <div class="row wrap">
    <button class="btn" onclick={save} disabled={saving}>
      {saving ? "Saving…" : "Save"}
    </button>
    {#if saved}<span class="ok">Saved</span>{/if}
    {#if error}<span class="error">{error}</span>{/if}
  </div>
</div>

<div class="card">
  <h2>Tax rates</h2>

  <div class="toolbar">
    <input bind:value={taxName} placeholder="Name (e.g. VAT)" />
    <input
      bind:value={taxPercent}
      inputmode="decimal"
      placeholder="Rate %"
      style="width: 110px;"
    />
    <button class="btn" onclick={addTaxRate} disabled={!taxName.trim()}>Add</button>
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
      <label for="reset">Reset yearly</label>
      <label class="row muted">
        <input id="reset" type="checkbox" bind:checked={resetYearly} style="width: auto;" />
        Include the year and restart at 1 each January
      </label>
    </div>
  </div>

  <p class="muted">Next invoice will be <code style="display: inline;">{numberPreview}</code></p>

  <div class="row wrap">
    <button class="btn" onclick={saveNumbering}>Save numbering</button>
    {#if numberSaved}<span class="ok">Saved</span>{/if}
    {#if numberError}<span class="error">{numberError}</span>{/if}
  </div>
</div>

<div class="card">
  <h2>About</h2>
  <p class="muted">
    {info?.name ?? "Ledgerly"} v{info?.version ?? "…"} — fully local and offline. No account, no
    cloud, no network.
  </p>
  <p class="muted">Data directory</p>
  <code>{info?.dataDir ?? "…"}</code>
  <p class="muted">Database</p>
  <code>{info?.databasePath ?? "…"}</code>
</div>
