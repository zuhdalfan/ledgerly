<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { ask, save as saveDialog } from "@tauri-apps/plugin-dialog";
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
  import Toast from "$lib/components/Toast.svelte";
  import Skeleton from "$lib/components/Skeleton.svelte";
  import { useDirtyGuard } from "$lib/guard";
  import { reminder } from "$lib/chase";
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

  // Creating a client must not mean abandoning a half-typed invoice.
  let newClientOpen = $state(false);
  let newClientName = $state("");
  let newClientEmail = $state("");
  let newClientError = $state("");
  let newClientBusy = $state(false);

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
  let loading = $state(true);
  let previewError = $state("");
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
  let businessName = $state("");
  let lastChasedAt = $state<string | null>(null);
  let chaseCount = $state(0);
  let daysPastDue = $state<number | null>(null);
  let reminderMessage = $state("");

  let payDate = $state("");
  let payAmount = $state("");
  let payMethod = $state("");
  let payReference = $state("");
  let payNote = $state("");
  let paymentError = $state("");
  let paymentBusy = $state(false);

  // `toMinor` yields 0 for anything it cannot parse, so a typo would otherwise
  // record a zero payment without complaint.
  const payAmountMinor = $derived(toMinor(payAmount));

  let celebrating = $state(false);
  let celebrationAmount = $state("");

  let undoOpen = $state(false);
  let undoMessage = $state("");
  let undoAction = $state<(() => void) | undefined>(undefined);

  function offerUndo(message: string, action: () => void) {
    undoMessage = message;
    undoAction = action;
    undoOpen = true;
  }

  const id = $derived(page.params.id ?? "");
  const isNew = $derived(id === "new");
  const summary = $derived(totals ?? savedTotals);
  const clientName = $derived(
    clients.find((client) => client.id === clientId)?.displayName ?? ""
  );

  // Dirty tracking compares what the form would save against the last state we
  // loaded or saved, so an untouched form never nags.
  let baseline = $state("");
  const dirty = $derived(baseline !== "" && JSON.stringify(toInput()) !== baseline);
  useDirtyGuard(() => dirty, "This invoice has unsaved changes. Leave and lose them?");

  /// Status and balance are derived on the Rust side, so take them from the
  /// invoice the backend returns rather than recomputing them here.
  function applyDerived(invoice: Invoice) {
    status = invoice.status;
    displayStatus = invoice.displayStatus;
    paidMinor = invoice.paidMinor;
    balanceMinor = invoice.balanceMinor;
    lastChasedAt = invoice.lastChasedAt;
    chaseCount = invoice.chaseCount;
    daysPastDue = invoice.daysPastDue;
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
      const [clientList, rates, business, savedTemplate] = await Promise.all([
        api.clients.list(),
        api.taxRates.list(),
        api.business.get(),
        api.settings.get("pdf.template").catch(() => null),
      ]);
      clients = clientList;
      taxRates = rates;
      currency = business.defaultCurrency;
      businessName = business.displayName;
      template = savedTemplate ?? "classic";

      if (isNew) {
        issueDate = localDate();
        payDate = localDate();
        notes = business.defaultNotes ?? "";
        terms = business.defaultTerms ?? "";
        paymentInstructions = business.defaultPaymentInstructions ?? "";
        // A client workspace can start an invoice for that client directly.
        const preset = page.url.searchParams.get("client");
        if (preset && clientList.some((client) => client.id === preset)) clientId = preset;
        baseline = JSON.stringify(toInput());
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
      baseline = JSON.stringify(toInput());
    } catch (e) {
      loadError = String(e);
    } finally {
      loading = false;
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

  function applyTotals(invoice: Invoice) {
    savedTotals = {
      subtotalMinor: invoice.subtotalMinor,
      discountMinor: invoice.discountMinor,
      taxMinor: invoice.taxMinor,
      totalMinor: invoice.totalMinor,
    };
  }

  async function save() {
    saveError = "";
    saved = false;
    saving = true;
    try {
      if (isNew) {
        const created = await api.invoices.create(toInput());
        // Replace rather than push, so Back does not land on a blank form.
        await goto(`/invoices/${created.id}`, { replaceState: true });
        // The route component is reused for the new id, so hydrate from what
        // was just created instead of waiting for a remount that never comes.
        number = created.number;
        applyTotals(created);
        applyDerived(created);
        payments = await api.payments.list(created.id);
        saved = true;
      } else {
        const updated = await api.invoices.update(id, toInput());
        applyTotals(updated);
        applyDerived(updated);
        saved = true;
      }

      baseline = JSON.stringify(toInput());
    } catch (e) {
      saveError = String(e);
    } finally {
      saving = false;
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
      const copy = await api.invoices.duplicate(id);
      await goto(`/invoices/${copy.id}`);
    } catch (e) {
      saveError = String(e);
    }
  }

  async function toggleVoid() {
    // Voiding a financial document gets a confirmation, unlike a reversible
    // toggle: it changes what the client was asked to pay.
    if (status !== "void") {
      const confirmed = await ask(
        "Void this invoice? It will be marked cancelled. You can restore it afterwards.",
        { title: "Void invoice", kind: "warning" }
      );
      if (!confirmed) return;
    }

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
    if (payAmountMinor <= 0) {
      paymentError = "Enter an amount like 250.00.";
      return;
    }

    paymentBusy = true;
    try {
      const amountMinor = payAmountMinor;
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
    const removed = payments.find((payment) => payment.id === paymentId);
    try {
      await api.payments.remove(paymentId);
      await reloadPayments();

      // A payment is real financial history, so the delete is recoverable.
      if (removed) {
        offerUndo("Payment removed.", () => void restorePayment(removed));
      }
    } catch (e) {
      paymentError = String(e);
    }
  }

  async function restorePayment(payment: Payment) {
    paymentError = "";
    try {
      await api.payments.create(id, {
        paidOn: payment.paidOn,
        amountMinor: payment.amountMinor,
        method: payment.method,
        reference: payment.reference,
        note: payment.note,
      });
      await reloadPayments();
    } catch (e) {
      paymentError = String(e);
    }
  }

  const reminderText = $derived(
    reminder(
      {
        number,
        clientName,
        currency,
        balanceMinor,
        dueDate,
        daysPastDue,
        displayStatus,
        paymentInstructions,
      },
      businessName
    )
  );

  async function copyReminder() {
    reminderMessage = "";
    try {
      await navigator.clipboard.writeText(`${reminderText.subject}\n\n${reminderText.body}`);
      reminderMessage = "Copied";
    } catch {
      reminderMessage = "Select the text above and copy it manually.";
    }
  }

  async function setChased(chased: boolean) {
    reminderMessage = "";
    try {
      applyDerived(await api.invoices.setChased(id, chased));
      reminderMessage = chased ? "Marked as chased" : "Chased mark cleared";
    } catch (e) {
      saveError = String(e);
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

{#if loadError}<p class="error" role="alert">{loadError}</p>{/if}

{#if loading}
  <div class="card">
    <h2>Details</h2>
    <Skeleton rows={6} columns={2} />
  </div>
{:else}

<div class="card">
  <h2>Details</h2>
  <div class="grid-2">
    <div class="field">
      <label for="client">Client</label>
      <div class="row">
        <select id="client" bind:value={clientId}>
          <option value="">None</option>
          {#each clients as client (client.id)}
            <option value={client.id}>{client.displayName}</option>
          {/each}
        </select>
        <button class="btn secondary" onclick={() => (newClientOpen = !newClientOpen)}>
          New client
        </button>
      </div>
    </div>

    {#if newClientOpen}
      <div class="inline-new">
        <div class="grid-2">
          <div class="field">
            <label for="newClientName">Client name</label>
            <input id="newClientName" bind:value={newClientName} />
          </div>
          <div class="field">
            <label for="newClientEmail">Email</label>
            <input id="newClientEmail" type="email" bind:value={newClientEmail} />
          </div>
        </div>
        <div class="row wrap">
          <button class="btn" onclick={createClient} disabled={newClientBusy || !newClientName.trim()}>
            Add client
          </button>
          <button class="btn secondary" onclick={() => (newClientOpen = false)}>Cancel</button>
          {#if newClientError}<span class="error">{newClientError}</span>{/if}
        </div>
      </div>
    {/if}
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
  {#if previewError}
    <p class="error" role="alert">
      The live preview could not be calculated, so these figures may be out of date. Saving
      recomputes them.
    </p>
  {/if}
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
                <button class="btn danger" onclick={() => removePayment(payment.id)}>
                  Delete
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}

    <h2 class="spaced">Record a payment</h2>
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
      {#if payAmount.trim() && payAmountMinor <= 0}
        <span class="error">Enter an amount like 250.00.</span>
      {/if}
      {#if paymentError}<span class="error">{paymentError}</span>{/if}
    </div>
  </div>
{/if}

{#if !isNew && balanceMinor > 0}
  <div class="card">
    <h2>Chasing this one</h2>
    <p class="muted">
      {#if lastChasedAt}
        Last chased {formatDate(lastChasedAt.slice(0, 10))}{chaseCount > 1
          ? ` (${chaseCount} times)`
          : ""}.
      {:else}
        Not chased yet.
      {/if}
      Ledgerly sends nothing itself. Copy this into your own email.
    </p>

    <div class="field">
      <label for="reminderSubject">Subject</label>
      <input id="reminderSubject" readonly value={reminderText.subject} />
    </div>
    <div class="field">
      <label for="reminderBody">Message</label>
      <textarea id="reminderBody" rows="10" readonly value={reminderText.body}></textarea>
    </div>

    <div class="row wrap">
      <button class="btn" onclick={copyReminder}>Copy reminder</button>
      <button class="btn secondary" onclick={() => setChased(!lastChasedAt)}>
        {lastChasedAt ? "Clear the chased mark" : "Mark as chased"}
      </button>
      {#if reminderMessage}<span class="ok">{reminderMessage}</span>{/if}
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

<div class="row wrap actions">
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
      <label class="template">
        <span class="sr-only">PDF template</span>
        <select bind:value={template} onchange={saveTemplate}>
          <option value="classic">Classic</option>
          <option value="modern">Modern</option>
          <option value="minimal">Minimal</option>
        </select>
      </label>
      <button class="btn" onclick={exportPdf} disabled={exporting}>
        {exporting ? "Exporting…" : "Export PDF"}
      </button>
      {#if exportMessage}<span class="muted">{exportMessage}</span>{/if}
    </div>
  </div>
{/if}

{/if}

<Celebration bind:open={celebrating} amount={celebrationAmount} />

<Toast bind:open={undoOpen} message={undoMessage} onaction={undoAction} />

<style>
  .template select {
    width: auto;
  }

  .spaced {
    margin-top: var(--space-5);
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
