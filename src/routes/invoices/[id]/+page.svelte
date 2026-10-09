<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { emptyToNull, fromMinor, toMinor } from "$lib/format";
  import type { Client, InvoiceInput, InvoiceTotals, TaxRate } from "$lib/types";

  type ItemForm = {
    description: string;
    quantity: string;
    unit: string;
    price: string;
    discountType: string;
    discountValue: string;
    taxRateId: string;
    taxInclusive: boolean;
  };

  const emptyItem = (): ItemForm => ({
    description: "",
    quantity: "1",
    unit: "",
    price: "",
    discountType: "",
    discountValue: "",
    taxRateId: "",
    taxInclusive: false,
  });

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

  const id = $derived(page.params.id ?? "");
  const isNew = $derived(id === "new");
  const summary = $derived(totals ?? savedTotals);
  const clientName = $derived(
    clients.find((client) => client.id === clientId)?.displayName ?? ""
  );

  function moneyString(minor: number): string {
    return (minor / 100).toFixed(2);
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
      items: items.map((item) => ({
        description: item.description,
        quantity: item.quantity.trim() || "1",
        unit: emptyToNull(item.unit),
        unitPriceMinor: toMinor(item.price),
        discountType: item.discountType || null,
        discountValue:
          item.discountType === ""
            ? null
            : item.discountType === "fixed"
              ? String(toMinor(item.discountValue))
              : item.discountValue,
        taxRateId: item.taxRateId || null,
        taxInclusive: item.taxInclusive,
      })),
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
        issueDate = new Date().toISOString().slice(0, 10);
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
          ? moneyString(Number(invoice.discountValue))
          : (invoice.discountValue ?? "");
      notes = invoice.notes ?? "";
      terms = invoice.terms ?? "";
      paymentInstructions = invoice.paymentInstructions ?? "";
      items = invoice.items.length
        ? invoice.items.map((item) => ({
            description: item.description,
            quantity: item.quantity,
            unit: item.unit ?? "",
            price: moneyString(item.unitPriceMinor),
            discountType: item.discountType ?? "",
            discountValue:
              item.discountType === "fixed" && item.discountValue
                ? moneyString(Number(item.discountValue))
                : (item.discountValue ?? ""),
            taxRateId: item.taxRateId ?? "",
            taxInclusive: item.taxInclusive,
          }))
        : [emptyItem()];
      savedTotals = {
        subtotalMinor: invoice.subtotalMinor,
        discountMinor: invoice.discountMinor,
        taxMinor: invoice.taxMinor,
        totalMinor: invoice.totalMinor,
      };
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

  function addItem() {
    items = [...items, emptyItem()];
  }

  function removeItem(index: number) {
    const next = items.filter((_, i) => i !== index);
    items = next.length ? next : [emptyItem()];
  }

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
      const updated = await api.invoices.setVoid(id, status !== "void");
      status = updated.status;
    } catch (e) {
      saveError = String(e);
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
  <h1>{isNew ? "New invoice" : number || "Invoice"}</h1>
  <a class="link" href="/invoices">Back to invoices</a>
</div>

{#if loadError}<p class="error">{loadError}</p>{/if}

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
      <label for="due">Due date</label>
      <input id="due" type="date" bind:value={dueDate} />
    </div>
    <div class="field">
      <label for="currency">Currency</label>
      <input id="currency" bind:value={currency} />
    </div>
    <div class="field">
      <label for="status">Status</label>
      <input id="status" value={status} disabled />
    </div>
  </div>
</div>

<div class="card">
  <h2>Line items</h2>

  <table>
    <thead>
      <tr>
        <th style="min-width: 160px;">Description</th>
        <th style="width: 70px;">Qty</th>
        <th style="width: 70px;">Unit</th>
        <th style="width: 100px;">Price</th>
        <th style="width: 100px;">Disc.</th>
        <th style="width: 90px;">Value</th>
        <th style="width: 120px;">Tax</th>
        <th style="width: 60px;">Incl.</th>
        <th style="width: 100px;">Total</th>
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each items as item, index (index)}
        <tr>
          <td><input bind:value={item.description} placeholder="Description" /></td>
          <td><input bind:value={item.quantity} inputmode="decimal" /></td>
          <td><input bind:value={item.unit} placeholder="hour" /></td>
          <td><input bind:value={item.price} inputmode="decimal" placeholder="0.00" /></td>
          <td>
            <select bind:value={item.discountType}>
              <option value="">—</option>
              <option value="percent">%</option>
              <option value="fixed">Fixed</option>
            </select>
          </td>
          <td>
            {#if item.discountType}
              <input bind:value={item.discountValue} inputmode="decimal" placeholder="0" />
            {/if}
          </td>
          <td>
            <select bind:value={item.taxRateId}>
              <option value="">—</option>
              {#each taxRates as rate (rate.id)}
                <option value={rate.id}>{rate.name}</option>
              {/each}
            </select>
          </td>
          <td style="text-align: center;">
            <input type="checkbox" bind:checked={item.taxInclusive} style="width: auto;" />
          </td>
          <td>{fromMinor(totals?.lines[index]?.totalMinor ?? 0, currency)}</td>
          <td>
            <button
              class="btn secondary"
              onclick={() => removeItem(index)}
              aria-label="Remove line"
            >
              ×
            </button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>

  <div class="row" style="margin-top: 0.75rem;">
    <button class="btn secondary" onclick={addItem}>Add line</button>
  </div>
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
  <button class="btn" onclick={save} disabled={saving}>
    {saving ? "Saving…" : "Save"}
  </button>
  {#if !isNew}
    <button class="btn secondary" onclick={duplicate}>Duplicate</button>
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
