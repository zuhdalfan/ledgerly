<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import LineItems from "$lib/components/LineItems.svelte";
  import Skeleton from "$lib/components/Skeleton.svelte";
  import { useDirtyGuard } from "$lib/guard";
  import {
    emptyToNull,
    fromMinor,
    localDate,
    minorToInput,
    normalizeDecimal,
    statusLabel,
    toMinor,
  } from "$lib/format";
  import { emptyItem, toItemForm, toItemInputs, type ItemForm } from "$lib/lineItems";
  import { plural, t } from "$lib/i18n.svelte";
  import type { Client, Estimate, EstimateInput, InvoiceTotals, TaxRate } from "$lib/types";

  let clients = $state<Client[]>([]);
  let taxRates = $state<TaxRate[]>([]);

  // Creating a client must not mean abandoning a half-typed estimate.
  let newClientOpen = $state(false);
  let newClientName = $state("");
  let newClientEmail = $state("");
  let newClientError = $state("");
  let newClientBusy = $state(false);

  let clientId = $state("");
  let issueDate = $state("");
  let validUntil = $state("");
  let currency = $state("USD");
  let poReference = $state("");
  let discountType = $state("");
  let discountValue = $state("");
  let notes = $state("");
  let terms = $state("");
  let paymentInstructions = $state("");
  let items = $state<ItemForm[]>([emptyItem()]);

  let number = $state("");
  let status = $state("draft");
  let displayStatus = $state("draft");
  let convertedInvoiceId = $state<string | null>(null);
  let daysUntilExpiry = $state<number | null>(null);
  let savedTotals = $state({
    subtotalMinor: 0,
    discountMinor: 0,
    taxMinor: 0,
    totalMinor: 0,
  });
  let totals = $state<InvoiceTotals | null>(null);

  let loadError = $state("");
  let loading = $state(true);
  let previewError = $state("");
  let saveError = $state("");
  let saved = $state(false);
  let saving = $state(false);
  let converting = $state(false);

  let template = $state("classic");
  let exporting = $state(false);
  let exportMessage = $state("");

  const id = $derived(page.params.id ?? "");
  const isNew = $derived(id === "new");
  const summary = $derived(totals ?? savedTotals);
  const converted = $derived(displayStatus === "converted");
  const clientName = $derived(
    clients.find((client) => client.id === clientId)?.displayName ?? ""
  );

  // Dirty tracking compares what the form would save against the last state we
  // loaded or saved, so an untouched form never nags.
  let baseline = $state("");
  const dirty = $derived(baseline !== "" && JSON.stringify(toInput()) !== baseline);
  useDirtyGuard(() => dirty, t("estimate.unsaved"));

  /// Status and expiry are derived on the Rust side.
  function applyEstimate(estimate: Estimate) {
    status = estimate.status;
    displayStatus = estimate.displayStatus;
    convertedInvoiceId = estimate.convertedInvoiceId;
    daysUntilExpiry = estimate.daysUntilExpiry;
  }

  function toInput(): EstimateInput {
    return {
      clientId: clientId || null,
      issueDate,
      validUntil: emptyToNull(validUntil),
      currency,
      poReference: emptyToNull(poReference),
      discountType: discountType || null,
      discountValue:
        discountType === ""
          ? null
          : discountType === "fixed"
            ? String(toMinor(discountValue))
            : normalizeDecimal(discountValue),
      notes: emptyToNull(notes),
      terms: emptyToNull(terms),
      paymentInstructions: emptyToNull(paymentInstructions),
      items: toItemInputs(items),
    };
  }

  onMount(async () => {
    try {
      const [clientList, rates, business, savedTemplate] = await Promise.all([
        api.clients.list(),
        api.taxRates.list(),
        api.business.get(),
        api.settings.get("pdf.template").catch(() => null),
      ]);
      clients = clientList;
      taxRates = rates;
      currency = business.defaultCurrency;
      template = savedTemplate ?? "classic";

      if (isNew) {
        issueDate = localDate();
        notes = business.defaultNotes ?? "";
        terms = business.defaultTerms ?? "";
        paymentInstructions = business.defaultPaymentInstructions ?? "";
        // A client workspace can start an estimate for that client directly.
        const preset = page.url.searchParams.get("client");
        if (preset && clientList.some((client) => client.id === preset)) clientId = preset;
        baseline = JSON.stringify(toInput());
        return;
      }

      const estimate = await api.estimates.get(id);
      if (!estimate) {
        loadError = t("estimate.notFound");
        return;
      }

      number = estimate.number;
      clientId = estimate.clientId ?? "";
      issueDate = estimate.issueDate;
      validUntil = estimate.validUntil ?? "";
      currency = estimate.currency;
      poReference = estimate.poReference ?? "";
      discountType = estimate.discountType ?? "";
      discountValue =
        estimate.discountType === "fixed" && estimate.discountValue
          ? minorToInput(Number(estimate.discountValue))
          : (estimate.discountValue ?? "");
      notes = estimate.notes ?? "";
      terms = estimate.terms ?? "";
      paymentInstructions = estimate.paymentInstructions ?? "";
      items = estimate.items.length ? estimate.items.map(toItemForm) : [emptyItem()];
      savedTotals = {
        subtotalMinor: estimate.subtotalMinor,
        discountMinor: estimate.discountMinor,
        taxMinor: estimate.taxMinor,
        totalMinor: estimate.totalMinor,
      };
      applyEstimate(estimate);
      baseline = JSON.stringify(toInput());
    } catch (e) {
      loadError = String(e);
    } finally {
      loading = false;
    }
  });

  // Live totals come from the Rust core so the preview and the saved estimate agree.
  let previewTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const snapshot = JSON.stringify([discountType, discountValue, items]);
    void snapshot;
    clearTimeout(previewTimer);
    previewTimer = setTimeout(() => {
      api.estimates
        .totals(toInput())
        .then((value) => {
          totals = value;
          previewError = "";
        })
        .catch((error) => {
          // Never fail silently: stale figures with no warning is worse than
          // an obvious problem.
          previewError = String(error);
        });
    }, 150);
  });

  function applyTotals(estimate: Estimate) {
    savedTotals = {
      subtotalMinor: estimate.subtotalMinor,
      discountMinor: estimate.discountMinor,
      taxMinor: estimate.taxMinor,
      totalMinor: estimate.totalMinor,
    };
  }

  async function save() {
    saveError = "";
    saved = false;
    saving = true;
    try {
      if (isNew) {
        const created = await api.estimates.create(toInput());
        // Replace rather than push, so Back does not land on a blank form.
        await goto(`/estimates/${created.id}`, { replaceState: true });
        // The route component is reused for the new id, so hydrate from what
        // was just created instead of waiting for a remount that never comes.
        number = created.number;
        applyTotals(created);
        applyEstimate(created);
        saved = true;
      } else {
        const updated = await api.estimates.update(id, toInput());
        applyTotals(updated);
        applyEstimate(updated);
        saved = true;
      }

      baseline = JSON.stringify(toInput());
    } catch (e) {
      saveError = String(e);
    } finally {
      saving = false;
    }
  }

  async function changeStatus(next: string) {
    saveError = "";
    try {
      applyEstimate(await api.estimates.setStatus(id, next));
    } catch (e) {
      saveError = String(e);
    }
  }

  async function createClient() {
    if (!newClientName.trim()) return;
    newClientBusy = true;
    newClientError = "";
    try {
      const created = await api.clients.create({
        displayName: newClientName.trim(),
        company: null,
        contactPerson: null,
        email: emptyToNull(newClientEmail),
        phone: null,
        addressLine1: null,
        addressLine2: null,
        city: null,
        region: null,
        postalCode: null,
        country: null,
        taxId: null,
        currency: null,
        notes: null,
      });
      clients = [...clients, created];
      clientId = created.id;
      newClientOpen = false;
      newClientName = "";
      newClientEmail = "";
    } catch (e) {
      newClientError = String(e);
    } finally {
      newClientBusy = false;
    }
  }

  async function duplicate() {
    try {
      const copy = await api.estimates.duplicate(id);
      await goto(`/estimates/${copy.id}`);
    } catch (e) {
      saveError = String(e);
    }
  }

  async function convert() {
    saveError = "";
    converting = true;
    try {
      const invoice = await api.estimates.convert(id);
      await goto(`/invoices/${invoice.id}`);
    } catch (e) {
      saveError = String(e);
      converting = false;
    }
  }

  /// The keyboard bar the PRD asks for: save and export without the mouse.
  /// Both are prevented from reaching the webview's own handlers.
  function onShortcut(event: KeyboardEvent) {
    if (!(event.ctrlKey || event.metaKey)) return;
    const key = event.key.toLowerCase();

    if (key === "s") {
      event.preventDefault();
      void save();
    } else if (key === "p" && !isNew) {
      event.preventDefault();
      void exportPdf();
    }
  }

  onMount(() => {
    window.addEventListener("keydown", onShortcut);
    return () => window.removeEventListener("keydown", onShortcut);
  });

  async function saveTemplate() {
    try {
      await api.settings.set("pdf.template", template);
    } catch {
      // Remembering the template is a nicety; it must never block an export.
    }
  }

  async function exportPdf() {
    exportMessage = "";
    exporting = true;
    try {
      const suggested = `${number || "estimate"}-${clientName || "client"}.pdf`.replace(
        /[^\w.\-]+/g,
        "-"
      );
      const path = await saveDialog({
        defaultPath: suggested,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (!path) return;

      await api.estimates.exportPdf(id, path, template);
      await api.openInViewer(path);
      exportMessage = t("editor.savedTo", { path });
    } catch (e) {
      exportMessage = String(e);
    } finally {
      exporting = false;
    }
  }
</script>

<div class="page-head">
  <div class="row">
    <h1>{isNew ? t("action.newEstimate") : number || t("estimate.fallbackTitle")}</h1>
    {#if !isNew}
      <span class={`badge ${displayStatus}`}>{statusLabel(displayStatus)}</span>
    {/if}
  </div>
  <a class="link" href="/estimates">{t("action.backToEstimates")}</a>
</div>

{#if loadError}<p class="error" role="alert">{loadError}</p>{/if}

{#if loading}
  <div class="card">
    <h2>{t("editor.details")}</h2>
    <Skeleton rows={6} columns={2} />
  </div>
{:else}

{#if converted && convertedInvoiceId}
  <div class="card">
    <p>
      {t("estimate.converted")}
      <a class="link" href={`/invoices/${convertedInvoiceId}`}>{t("action.openInvoice")}</a>
    </p>
  </div>
{:else if daysUntilExpiry !== null && daysUntilExpiry < 0}
  <div class="card">
    <p class="error">{t("estimate.expired", { date: validUntil })}</p>
  </div>
{:else if daysUntilExpiry !== null && daysUntilExpiry <= 7}
  <div class="card">
    <p class="muted">
      {plural(daysUntilExpiry, "estimate.expiresInOne", "estimate.expiresInOther", {
        date: validUntil,
      })}
    </p>
  </div>
{/if}

<div class="card">
  <h2>{t("editor.details")}</h2>
  <div class="grid-2">
    <div class="field">
      <label for="client">{t("common.client")}</label>
      <div class="row">
        <select id="client" bind:value={clientId}>
          <option value="">{t("common.none")}</option>
          {#each clients as client (client.id)}
            <option value={client.id}>{client.displayName}</option>
          {/each}
        </select>
        <button class="btn secondary" onclick={() => (newClientOpen = !newClientOpen)}>
          {t("action.newClient")}
        </button>
      </div>
    </div>

    {#if newClientOpen}
      <div class="inline-new">
        <div class="grid-2">
          <div class="field">
            <label for="newClientName">{t("editor.clientName")}</label>
            <input id="newClientName" bind:value={newClientName} />
          </div>
          <div class="field">
            <label for="newClientEmail">{t("common.email")}</label>
            <input id="newClientEmail" type="email" bind:value={newClientEmail} />
          </div>
        </div>
        <div class="row wrap">
          <button class="btn" onclick={createClient} disabled={newClientBusy || !newClientName.trim()}>
            {t("action.addClient")}
          </button>
          <button class="btn secondary" onclick={() => (newClientOpen = false)}>
            {t("action.cancel")}
          </button>
          {#if newClientError}<span class="error">{newClientError}</span>{/if}
        </div>
      </div>
    {/if}
    <div class="field">
      <label for="po">{t("common.poReference")}</label>
      <input id="po" bind:value={poReference} />
    </div>
    <div class="field">
      <label for="issue">{t("editor.issueDate")}</label>
      <input id="issue" type="date" bind:value={issueDate} />
    </div>
    <div class="field">
      <label for="valid">{t("common.validUntil")}</label>
      <input id="valid" type="date" bind:value={validUntil} />
    </div>
    <div class="field">
      <label for="currency">{t("common.currency")}</label>
      <input id="currency" bind:value={currency} />
    </div>
    <div class="field">
      <label for="status">{t("common.status")}</label>
      {#if isNew || converted}
        <input id="status" value={statusLabel(displayStatus)} disabled />
      {:else}
        <select id="status" value={status} onchange={(e) => changeStatus(e.currentTarget.value)}>
          <option value="draft">{t("status.draft")}</option>
          <option value="sent">{t("status.sent")}</option>
          <option value="accepted">{t("status.accepted")}</option>
          <option value="declined">{t("status.declined")}</option>
        </select>
      {/if}
    </div>
  </div>
</div>

<div class="card">
  <h2>{t("editor.lineItems")}</h2>
  <LineItems bind:items {taxRates} {currency} lines={totals?.lines ?? []} />
</div>

<div class="card">
  <h2>{t("editor.totals")}</h2>
  {#if previewError}
    <p class="error" role="alert">{t("editor.previewStale")}</p>
  {/if}
  <div class="grid-2">
    <div>
      <div class="field">
        <label for="discType">{t("estimate.discount")}</label>
        <div class="row">
          <select id="discType" bind:value={discountType} style="width: auto;">
            <option value="">{t("common.none")}</option>
            <option value="percent">{t("common.percent")}</option>
            <option value="fixed">{t("common.fixed")}</option>
          </select>
          {#if discountType}
            <input bind:value={discountValue} inputmode="decimal" placeholder="0" />
          {/if}
        </div>
      </div>
    </div>
    <div>
      <table>
        <tbody>
          <tr>
            <td>{t("common.subtotal")}</td>
            <td>{fromMinor(summary.subtotalMinor, currency)}</td>
          </tr>
          <tr>
            <td>{t("common.discount")}</td>
            <td>−{fromMinor(summary.discountMinor, currency)}</td>
          </tr>
          <tr><td>{t("common.tax")}</td><td>{fromMinor(summary.taxMinor, currency)}</td></tr>
          <tr>
            <td><strong>{t("common.total")}</strong></td>
            <td><strong>{fromMinor(summary.totalMinor, currency)}</strong></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</div>

<div class="card">
  <h2>{t("editor.notesTerms")}</h2>
  <div class="field">
    <label for="notes">{t("common.notes")}</label>
    <textarea id="notes" bind:value={notes}></textarea>
  </div>
  <div class="field">
    <label for="terms">{t("common.terms")}</label>
    <textarea id="terms" bind:value={terms}></textarea>
  </div>
  <div class="field">
    <label for="payments">{t("settings.business.paymentInstructions")}</label>
    <textarea id="payments" bind:value={paymentInstructions}></textarea>
  </div>
</div>

<div class="row wrap actions">
  <button class="btn" onclick={save} disabled={saving || converted}>
    {saving ? t("action.saving") : t("action.save")}
  </button>
  {#if !isNew}
    <button class="btn secondary" onclick={duplicate}>{t("action.duplicate")}</button>
    {#if !converted}
      <button class="btn" onclick={convert} disabled={converting}>
        {converting ? t("action.converting") : t("action.convertToInvoice")}
      </button>
    {/if}
  {/if}
  {#if saved}<span class="ok">{t("action.saved")}</span>{/if}
  {#if saveError}<span class="error">{saveError}</span>{/if}
</div>

{#if !isNew}
  <div class="card">
    <h2>PDF</h2>
    <div class="row wrap">
      <label class="template">
        <span class="sr-only">{t("editor.pdfTemplate")}</span>
        <select bind:value={template} onchange={saveTemplate}>
          <option value="classic">{t("editor.templateClassic")}</option>
          <option value="modern">{t("editor.templateModern")}</option>
          <option value="minimal">{t("editor.templateMinimal")}</option>
        </select>
      </label>
      <button class="btn" onclick={exportPdf} disabled={exporting}>
        {exporting ? t("action.exporting") : t("action.exportPdf")}
      </button>
      {#if exportMessage}<span class="muted">{exportMessage}</span>{/if}
    </div>
  </div>
{/if}

{/if}

<style>
  .template select {
    width: auto;
  }

  .actions {
    margin-top: var(--space-4);
  }

  .inline-new {
    margin-bottom: var(--space-3);
    padding: var(--space-3);
    background: var(--paper-2);
    border-radius: var(--radius-md);
  }
</style>
