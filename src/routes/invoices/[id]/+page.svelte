<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import {
    emptyToNull,
    formatDate,
    fromMinor,
    localDate,
    minorToInput,
    statusLabel,
    toMinor,
  } from "$lib/format";
  import LineItems from "$lib/components/LineItems.svelte";
  import Celebration from "$lib/components/Celebration.svelte";
  import { emptyItem, toItemForm, toItemInputs, type ItemForm } from "$lib/lineItems";
  import type {
    Client,
    Invoice,
    InvoiceInput,
    InvoiceTotals,
    Payment,
    TaxRate,
  } from "$lib/types";

  let clients = $state<Client[]>([]);
  let taxRates = $state<TaxRate[]>([]);

  let clientId = $state("");
  let issueDate = $state("");
  let dueDate = $state("");
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

  let template = $state("classic");
  let exporting = $state(false);
  let exportMessage = $state("");

  let displayStatus = $state("draft");
  let paidMinor = $state(0);
  let balanceMinor = $state(0);
  let payments = $state<Payment[]>([]);

  let payDate = $state("");
  let payAmount = $state("");
  let payMethod = $state("");
  let payReference = $state("");
  let payNote = $state("");
  let paymentError = $state("");
  let paymentBusy = $state(false);

  let celebrating = $state(false);
  let celebrationAmount = $state("");

  const id = $derived(page.params.id ?? "");
  const isNew = $derived(id === "new");
  const summary = $derived(totals ?? savedTotals);
  const clientName = $derived(
    clients.find((client) => client.id === clientId)?.displayName ?? ""
  );

  /// Status and balance are derived on the Rust side, so take them from the
  /// invoice the backend returns rather than recomputing them here.
  function applyDerived(invoice: Invoice) {
    status = invoice.status;
    displayStatus = invoice.displayStatus;
    paidMinor = invoice.paidMinor;
    balanceMinor = invoice.balanceMinor;
  }

  async function reloadPayments() {
    const refreshed = await api.invoices.get(id);
    if (refreshed) applyDerived(refreshed);
    payments = await api.payments.list(id);
  }

  function toInput(): InvoiceInput {
    return {
      clientId: clientId || null,
      issueDate,
      dueDate: emptyToNull(dueDate),
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
        payDate = localDate();
        notes = business.defaultNotes ?? "";
        terms = business.defaultTerms ?? "";
        paymentInstructions = business.defaultPaymentInstructions ?? "";
        return;
      }

      const invoice = await api.invoices.get(id);
      if (!invoice) {
        loadError = "Invoice not found.";
        return;
      }

      number = invoice.number;
      status = invoice.status;
      clientId = invoice.clientId ?? "";
      issueDate = invoice.issueDate;
      dueDate = invoice.dueDate ?? "";
      currency = invoice.currency;
      poReference = invoice.poReference ?? "";
      discountType = invoice.discountType ?? "";
      discountValue =
        invoice.discountType === "fixed" && invoice.discountValue
          ? minorToInput(Number(invoice.discountValue))
          : (invoice.discountValue ?? "");
      notes = invoice.notes ?? "";
      terms = invoice.terms ?? "";
      paymentInstructions = invoice.paymentInstructions ?? "";
      items = invoice.items.length ? invoice.items.map(toItemForm) : [emptyItem()];
      savedTotals = {
        subtotalMinor: invoice.subtotalMinor,
        discountMinor: invoice.discountMinor,
        taxMinor: invoice.taxMinor,
        totalMinor: invoice.totalMinor,
      };
      applyDerived(invoice);
      payments = await api.payments.list(id);
    } catch (e) {
      loadError = String(e);
    }
  });

  // Live totals are computed by the Rust core so the preview and the saved
  // invoice can never disagree.
  let previewTimer: ReturnType<typeof setTimeout> | undefined;
  $effect(() => {
    const snapshot = JSON.stringify([discountType, discountValue, items]);
    void snapshot;
    clearTimeout(previewTimer);
    previewTimer = setTimeout(() => {
      api
        .invoices
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
        const created = await api.invoices.create(toInput());
        await goto(`/invoices/${created.id}`);
      } else {
        const updated = await api.invoices.update(id, toInput());
        savedTotals = {
          subtotalMinor: updated.subtotalMinor,
          discountMinor: updated.discountMinor,
          taxMinor: updated.taxMinor,
          totalMinor: updated.totalMinor,
        };
        applyDerived(updated);
        saved = true;
      }
    } catch (e) {
      saveError = String(e);
    } finally {
      saving = false;
    }
  }

  async function duplicate() {
    try {
      const copy = await api.invoices.duplicate(id);
      await goto(`/invoices/${copy.id}`);
    } catch (e) {
      saveError = String(e);
    }
  }

  async function toggleVoid() {
    try {
      applyDerived(await api.invoices.setVoid(id, status !== "void"));
    } catch (e) {
      saveError = String(e);
    }
  }

  async function toggleSent() {
    saveError = "";
    try {
      applyDerived(await api.invoices.setSent(id, status !== "sent"));
    } catch (e) {
      saveError = String(e);
    }
  }

  async function addPayment() {
    paymentError = "";
    paymentBusy = true;
    try {
      const amountMinor = toMinor(payAmount);
      await api.payments.create(id, {
        paidOn: payDate || localDate(),
        amountMinor,
        method: emptyToNull(payMethod),
        reference: emptyToNull(payReference),
        note: emptyToNull(payNote),
      });
      payAmount = "";
      payMethod = "";
      payReference = "";
      payNote = "";
      await reloadPayments();

      // The one earned celebration: this payment settled the invoice.
      if (balanceMinor <= 0) {
        celebrationAmount = fromMinor(amountMinor, currency);
        celebrating = true;
      }
    } catch (e) {
      paymentError = String(e);
    } finally {
      paymentBusy = false;
    }
  }

  async function removePayment(paymentId: string) {
    paymentError = "";
    try {
      await api.payments.remove(paymentId);
      await reloadPayments();
    } catch (e) {
      paymentError = String(e);
    }
  }

  async function exportPdf() {
    exportMessage = "";
    exporting = true;
    try {
      const suggested = `${number || "invoice"}-${clientName || "client"}.pdf`.replace(
        /[^\w.\-]+/g,
        "-"
      );
      const path = await saveDialog({
        defaultPath: suggested,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (!path) return;

      await api.invoices.exportPdf(id, path, template);
      await api.openInViewer(path);
      exportMessage = `Saved to ${path}`;
    } catch (e) {
      exportMessage = String(e);
    } finally {
      exporting = false;
    }
  }
</script>

<div class="page-head">
  <div class="row">
    <h1>{isNew ? "New invoice" : number || "Invoice"}</h1>
    {#if !isNew}
      <span class={`badge ${displayStatus}`}>{statusLabel(displayStatus)}</span>
    {/if}
  </div>
  <a class="link" href="/invoices">Back to invoices</a>
</div>

{#if loadError}<p class="error">{loadError}</p>{/if}

<div class="card">
  <h2>Details</h2>
  <div class="grid-2">
    <div class="field">
      <label for="client">Client</label>
      <select id="client" bind:value={clientId}>
        <option value="">None</option>
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
      <label for="due">Due date</label>
      <input id="due" type="date" bind:value={dueDate} />
    </div>
    <div class="field">
      <label for="currency">Currency</label>
      <input id="currency" bind:value={currency} />
    </div>
    <div class="field">
      <label for="status">Status</label>
      <input id="status" value={statusLabel(displayStatus)} disabled />
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
        <label for="discType">Invoice discount</label>
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
          {#if !isNew}
            <tr><td>Paid</td><td>{fromMinor(paidMinor, currency)}</td></tr>
            <tr>
              <td><strong>Balance</strong></td>
              <td><strong>{fromMinor(balanceMinor, currency)}</strong></td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

{#if !isNew}
  <div class="card">
    <h2>Payments</h2>

    {#if payments.length === 0}
      <p class="muted">No payments recorded yet.</p>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Date</th>
            <th>Method</th>
            <th>Reference</th>
            <th class="numeric">Amount</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each payments as payment (payment.id)}
            <tr>
              <td>{formatDate(payment.paidOn)}</td>
              <td>{payment.method ?? "—"}</td>
              <td>{payment.reference ?? "—"}</td>
              <td class="numeric">{fromMinor(payment.amountMinor, currency)}</td>
              <td>
                <button class="btn secondary" onclick={() => removePayment(payment.id)}>
                  Delete
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <h2 style="margin-top: 1.25rem;">Record a payment</h2>
    <div class="grid-2">
      <div class="field">
        <label for="payDate">Date</label>
        <input id="payDate" type="date" bind:value={payDate} />
      </div>
      <div class="field">
        <label for="payAmount">Amount ({currency})</label>
        <input id="payAmount" bind:value={payAmount} inputmode="decimal" placeholder="0.00" />
      </div>
      <div class="field">
        <label for="payMethod">Method</label>
        <input id="payMethod" bind:value={payMethod} placeholder="bank / cash / card" />
      </div>
      <div class="field">
        <label for="payReference">Reference</label>
        <input id="payReference" bind:value={payReference} />
      </div>
    </div>
    <div class="field">
      <label for="payNote">Note</label>
      <input id="payNote" bind:value={payNote} />
    </div>

    <div class="row wrap">
      <button class="btn" onclick={addPayment} disabled={paymentBusy || !payAmount.trim()}>
        {paymentBusy ? "Recording…" : "Record payment"}
      </button>
      {#if paymentError}<span class="error">{paymentError}</span>{/if}
    </div>
  </div>
{/if}

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
  <button class="btn" onclick={save} disabled={saving}>
    {saving ? "Saving…" : "Save"}
  </button>
  {#if !isNew}
    <button class="btn secondary" onclick={duplicate}>Duplicate</button>
    {#if status !== "void"}
      <button class="btn secondary" onclick={toggleSent}>
        {status === "sent" ? "Move to draft" : "Mark as sent"}
      </button>
    {/if}
    <button class="btn danger" onclick={toggleVoid}>
      {status === "void" ? "Restore" : "Void"}
    </button>
  {/if}
  {#if saved}<span class="ok">Saved</span>{/if}
  {#if saveError}<span class="error">{saveError}</span>{/if}
</div>

{#if !isNew}
  <div class="card">
    <h2>PDF</h2>
    <div class="row wrap">
      <select bind:value={template} style="width: auto;">
        <option value="classic">Classic</option>
        <option value="modern">Modern</option>
        <option value="minimal">Minimal</option>
      </select>
      <button class="btn" onclick={exportPdf} disabled={exporting}>
        {exporting ? "Exporting…" : "Export PDF"}
      </button>
      {#if exportMessage}<span class="muted">{exportMessage}</span>{/if}
    </div>
  </div>
{/if}

<Celebration bind:open={celebrating} amount={celebrationAmount} />
