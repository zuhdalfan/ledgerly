<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import LineItems from "$lib/components/LineItems.svelte";
  import {
    emptyToNull,
    fromMinor,
    localDate,
    minorToInput,
    statusLabel,
    toMinor,
  } from "$lib/format";
  import { emptyItem, toItemForm, toItemInputs, type ItemForm } from "$lib/lineItems";
  import type { Client, Estimate, EstimateInput, InvoiceTotals, TaxRate } from "$lib/types";

  let clients = $state<Client[]>([]);
  let taxRates = $state<TaxRate[]>([]);

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
  let saveError = $state("");
  let saved = $state(false);
  let saving = $state(false);
  let converting = $state(false);

  const id = $derived(page.params.id ?? "");
  const isNew = $derived(id === "new");
  const summary = $derived(totals ?? savedTotals);
  const converted = $derived(displayStatus === "converted");

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
            : discountValue,
      notes: emptyToNull(notes),
      terms: emptyToNull(terms),
      paymentInstructions: emptyToNull(paymentInstructions),
      items: toItemInputs(items),
    };
  }

  onMount(async () => {
    try {
      const [clientList, rates, business] = await Promise.all([
        api.clients.list(),
        api.taxRates.list(),
        api.business.get(),
      ]);
      clients = clientList;
      taxRates = rates;
      currency = business.defaultCurrency;

      if (isNew) {
        issueDate = localDate();
        notes = business.defaultNotes ?? "";
        terms = business.defaultTerms ?? "";
        paymentInstructions = business.defaultPaymentInstructions ?? "";
        return;
      }

      const estimate = await api.estimates.get(id);
      if (!estimate) {
        loadError = "Estimate not found.";
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
    } catch (e) {
      loadError = String(e);
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
        .then((value) => (totals = value))
        .catch(() => {});
    }, 150);
  });

  async function save() {
    saveError = "";
    saved = false;
    saving = true;
    try {
      if (isNew) {
        const created = await api.estimates.create(toInput());
        await goto(`/estimates/${created.id}`);
      } else {
        const updated = await api.estimates.update(id, toInput());
        savedTotals = {
          subtotalMinor: updated.subtotalMinor,
          discountMinor: updated.discountMinor,
          taxMinor: updated.taxMinor,
          totalMinor: updated.totalMinor,
        };
        applyEstimate(updated);
        saved = true;
      }
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
</script>

<div class="page-head">
  <div class="row">
    <h1>{isNew ? "New estimate" : number || "Estimate"}</h1>
    {#if !isNew}
      <span class={`badge ${displayStatus}`}>{statusLabel(displayStatus)}</span>
    {/if}
  </div>
  <a class="link" href="/estimates">Back to estimates</a>
</div>

{#if loadError}<p class="error">{loadError}</p>{/if}

{#if converted && convertedInvoiceId}
  <div class="card">
    <p>
      This estimate was converted to an invoice.
      <a class="link" href={`/invoices/${convertedInvoiceId}`}>Open the invoice</a>
    </p>
  </div>
{:else if daysUntilExpiry !== null && daysUntilExpiry < 0}
  <div class="card">
    <p class="error">This estimate expired on {validUntil}.</p>
  </div>
{:else if daysUntilExpiry !== null && daysUntilExpiry <= 7}
  <div class="card">
    <p class="muted">Expires in {daysUntilExpiry} day(s) — on {validUntil}.</p>
  </div>
{/if}

<div class="card">
  <h2>Details</h2>
  <div class="grid-2">
    <div class="field">
      <label for="client">Client</label>
      <select id="client" bind:value={clientId}>
        <option value="">— none —</option>
        {#each clients as client (client.id)}
          <option value={client.id}>{client.displayName}</option>
        {/each}
      </select>
    </div>
    <div class="field">
      <label for="po">PO / reference</label>
      <input id="po" bind:value={poReference} />
    </div>
    <div class="field">
      <label for="issue">Issue date</label>
      <input id="issue" type="date" bind:value={issueDate} />
    </div>
    <div class="field">
      <label for="valid">Valid until</label>
      <input id="valid" type="date" bind:value={validUntil} />
    </div>
    <div class="field">
      <label for="currency">Currency</label>
      <input id="currency" bind:value={currency} />
    </div>
    <div class="field">
      <label for="status">Status</label>
      {#if isNew || converted}
        <input id="status" value={statusLabel(displayStatus)} disabled />
      {:else}
        <select id="status" value={status} onchange={(e) => changeStatus(e.currentTarget.value)}>
          <option value="draft">Draft</option>
          <option value="sent">Sent</option>
          <option value="accepted">Accepted</option>
          <option value="declined">Declined</option>
        </select>
      {/if}
    </div>
  </div>
</div>

<div class="card">
  <h2>Line items</h2>
  <LineItems bind:items {taxRates} {currency} lines={totals?.lines ?? []} />
</div>

<div class="card">
  <h2>Totals</h2>
  <div class="grid-2">
    <div>
      <div class="field">
        <label for="discType">Estimate discount</label>
        <div class="row">
          <select id="discType" bind:value={discountType} style="width: auto;">
            <option value="">None</option>
            <option value="percent">Percent</option>
            <option value="fixed">Fixed</option>
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
          <tr><td>Subtotal</td><td>{fromMinor(summary.subtotalMinor, currency)}</td></tr>
          <tr><td>Discount</td><td>−{fromMinor(summary.discountMinor, currency)}</td></tr>
          <tr><td>Tax</td><td>{fromMinor(summary.taxMinor, currency)}</td></tr>
          <tr>
            <td><strong>Total</strong></td>
            <td><strong>{fromMinor(summary.totalMinor, currency)}</strong></td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
</div>

<div class="card">
  <h2>Notes &amp; terms</h2>
  <div class="field">
    <label for="notes">Notes</label>
    <textarea id="notes" bind:value={notes}></textarea>
  </div>
  <div class="field">
    <label for="terms">Terms</label>
    <textarea id="terms" bind:value={terms}></textarea>
  </div>
  <div class="field">
    <label for="payments">Payment instructions</label>
    <textarea id="payments" bind:value={paymentInstructions}></textarea>
  </div>
</div>

<div class="row wrap" style="margin-top: 1rem;">
  <button class="btn" onclick={save} disabled={saving || converted}>
    {saving ? "Saving…" : "Save"}
  </button>
  {#if !isNew}
    <button class="btn secondary" onclick={duplicate}>Duplicate</button>
    {#if !converted}
      <button class="btn" onclick={convert} disabled={converting}>
        {converting ? "Converting…" : "Convert to invoice"}
      </button>
    {/if}
  {/if}
  {#if saved}<span class="ok">Saved</span>{/if}
  {#if saveError}<span class="error">{saveError}</span>{/if}
</div>
