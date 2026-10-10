<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { emptyToNull, nullToEmpty } from "$lib/format";
  import type { BusinessInput } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

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
  let loading = $state(true);
  let error = $state("");
  let saved = $state(false);
  let saving = $state(false);

  async function load() {
    loading = true;
    try {
      const business = await api.business.get();
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
</script>

<PageHeader
  title="Business"
  subtitle="Your details, and the wording that fills into every new invoice."
/>

<AsyncState {loading} {error} rows={5} columns={2}>
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

    <h2 class="spaced">Address</h2>
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

    <h2 class="spaced">Invoice defaults</h2>
    <div class="field">
      <label for="defaultTerms">Default terms</label>
      <input id="defaultTerms" bind:value={form.defaultTerms} placeholder="Net 30" />
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
        {saving ? "Saving…" : "Save business details"}
      </button>
      {#if saved}<span class="ok">Saved</span>{/if}
      {#if error}<span class="error">{error}</span>{/if}
    </div>
  </div>
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-5);
  }
</style>
