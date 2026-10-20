<script lang="ts">
  import { onMount } from "svelte";
  import { open as openDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { emptyToNull, nullToEmpty } from "$lib/format";
  import { t } from "$lib/i18n.svelte";
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
  let logoUrl = $state<string | null>(null);
  let logoBusy = $state(false);
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
      logoUrl = await api.business.logoDataUrl();
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

  async function chooseLogo() {
    const path = await openDialog({
      multiple: false,
      directory: false,
      filters: [{ name: t("settings.business.imagesFilter"), extensions: ["png", "jpg", "jpeg"] }],
    });
    if (typeof path !== "string") return;

    logoBusy = true;
    error = "";
    try {
      logoUrl = await api.business.setLogo(path);
    } catch (e) {
      error = String(e);
    } finally {
      logoBusy = false;
    }
  }

  async function removeLogo() {
    logoBusy = true;
    error = "";
    try {
      await api.business.clearLogo();
      logoUrl = null;
    } catch (e) {
      error = String(e);
    } finally {
      logoBusy = false;
    }
  }
</script>

<PageHeader
  title={t("settings.nav.business")}
  subtitle={t("settings.business.subtitle")}
/>

<AsyncState {loading} {error} rows={5} columns={2}>
  <div class="card">
    <h2>{t("settings.business.profileTitle")}</h2>

    <div class="field">
      <label for="chooseLogo">{t("settings.business.logo")}</label>
      <div class="logo-row">
        {#if logoUrl}
          <img class="logo-preview" src={logoUrl} alt={t("settings.business.logoAlt")} />
        {:else}
          <div class="logo-preview logo-empty" aria-hidden="true">LOGO</div>
        {/if}
        <div class="row wrap">
          <button id="chooseLogo" class="btn secondary" onclick={chooseLogo} disabled={logoBusy}>
            {logoUrl ? t("settings.business.replaceImage") : t("settings.business.chooseImage")}
          </button>
          {#if logoUrl}
            <button class="btn secondary" onclick={removeLogo} disabled={logoBusy}>
              {t("action.remove")}
            </button>
          {/if}
        </div>
      </div>
      <p class="hint">{t("settings.business.logoHint")}</p>
    </div>

    <div class="field">
      <label for="displayName">{t("settings.business.name")}</label>
      <input
        id="displayName"
        bind:value={form.displayName}
        placeholder={t("settings.business.namePlaceholder")}
      />
    </div>

    <div class="grid-2">
      <div class="field">
        <label for="legalName">{t("settings.business.legalName")}</label>
        <input id="legalName" bind:value={form.legalName} />
      </div>
      <div class="field">
        <label for="taxId">{t("common.taxId")}</label>
        <input id="taxId" bind:value={form.taxId} />
      </div>
      <div class="field">
        <label for="email">{t("common.email")}</label>
        <input id="email" type="email" bind:value={form.email} />
      </div>
      <div class="field">
        <label for="phone">{t("common.phone")}</label>
        <input id="phone" bind:value={form.phone} />
      </div>
      <div class="field">
        <label for="website">{t("common.website")}</label>
        <input id="website" bind:value={form.website} />
      </div>
      <div class="field">
        <label for="defaultCurrency">{t("settings.business.defaultCurrency")}</label>
        <input id="defaultCurrency" bind:value={form.defaultCurrency} />
      </div>
    </div>

    <h2 class="spaced">{t("settings.business.address")}</h2>
    <div class="grid-2">
      <div class="field">
        <label for="addressLine1">{t("common.addressLine1")}</label>
        <input id="addressLine1" bind:value={form.addressLine1} />
      </div>
      <div class="field">
        <label for="addressLine2">{t("common.addressLine2")}</label>
        <input id="addressLine2" bind:value={form.addressLine2} />
      </div>
      <div class="field">
        <label for="city">{t("common.city")}</label>
        <input id="city" bind:value={form.city} />
      </div>
      <div class="field">
        <label for="region">{t("common.region")}</label>
        <input id="region" bind:value={form.region} />
      </div>
      <div class="field">
        <label for="postalCode">{t("common.postalCode")}</label>
        <input id="postalCode" bind:value={form.postalCode} />
      </div>
      <div class="field">
        <label for="country">{t("common.country")}</label>
        <input id="country" bind:value={form.country} />
      </div>
    </div>

    <h2 class="spaced">{t("settings.business.invoiceDefaults")}</h2>
    <div class="field">
      <label for="defaultTerms">{t("settings.business.defaultTerms")}</label>
      <input id="defaultTerms" bind:value={form.defaultTerms} placeholder="Net 30" />
    </div>
    <div class="field">
      <label for="defaultNotes">{t("settings.business.defaultNotes")}</label>
      <textarea id="defaultNotes" bind:value={form.defaultNotes}></textarea>
    </div>
    <div class="field">
      <label for="defaultPaymentInstructions">
        {t("settings.business.paymentInstructions")}
      </label>
      <textarea id="defaultPaymentInstructions" bind:value={form.defaultPaymentInstructions}
      ></textarea>
    </div>

    <div class="row wrap">
      <button class="btn" onclick={save} disabled={saving}>
        {saving ? t("action.saving") : t("action.saveBusiness")}
      </button>
      {#if saved}<span class="ok">{t("action.saved")}</span>{/if}
      {#if error}<span class="error">{error}</span>{/if}
    </div>
  </div>
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-5);
  }

  .logo-row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: var(--space-3);
  }

  .logo-preview {
    max-width: 180px;
    max-height: 72px;
    object-fit: contain;
    border-radius: var(--radius-md);
  }

  .logo-empty {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 140px;
    height: 56px;
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.08em;
    color: var(--ink-3);
    background: var(--paper-2);
    border: 1px dashed var(--line-control);
  }

  .hint {
    margin: 0;
    color: var(--ink-3);
    font-size: var(--text-sm);
  }
</style>
