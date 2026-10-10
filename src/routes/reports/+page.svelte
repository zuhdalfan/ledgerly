<script lang="ts">
  import { onMount } from "svelte";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { fromMinor, localDate, minorToCsv } from "$lib/format";
  import type { AgedRow, ClientRevenueRow, MethodRow, PeriodRow, TaxRow } from "$lib/types";

  let from = $state(`${new Date().getFullYear()}-01-01`);
  let to = $state(localDate());
  let granularity = $state("month");

  let revenue = $state<PeriodRow[]>([]);
  let byClient = $state<ClientRevenueRow[]>([]);
  let aged = $state<AgedRow[]>([]);
  let tax = $state<TaxRow[]>([]);
  let byMethod = $state<MethodRow[]>([]);
  let currency = $state("USD");

  let error = $state("");
  let message = $state("");
  let loading = $state(false);

  const revenueTotals = $derived({
    count: revenue.reduce((sum, row) => sum + row.invoiceCount, 0),
    invoiced: revenue.reduce((sum, row) => sum + row.invoicedMinor, 0),
    tax: revenue.reduce((sum, row) => sum + row.taxMinor, 0),
    paid: revenue.reduce((sum, row) => sum + row.paidMinor, 0),
  });
  const clientTotals = $derived({
    invoiced: byClient.reduce((sum, row) => sum + row.invoicedMinor, 0),
    paid: byClient.reduce((sum, row) => sum + row.paidMinor, 0),
    outstanding: byClient.reduce((sum, row) => sum + row.outstandingMinor, 0),
  });
  const agedTotal = $derived(aged.reduce((sum, row) => sum + row.balanceMinor, 0));
  const taxTotals = $derived({
    taxable: tax.reduce((sum, row) => sum + row.taxableMinor, 0),
    tax: tax.reduce((sum, row) => sum + row.taxMinor, 0),
  });
  const methodTotal = $derived(byMethod.reduce((sum, row) => sum + row.amountMinor, 0));

  async function load() {
    loading = true;
    error = "";
    try {
      const [rev, client, ageing, taxRows, methods] = await Promise.all([
        api.reports.revenue(from, to, granularity),
        api.reports.revenueByClient(from, to),
        api.reports.agedReceivables(),
        api.reports.tax(from, to),
        api.reports.paymentsByMethod(from, to),
      ]);
      revenue = rev;
      byClient = client;
      aged = ageing;
      tax = taxRows;
      byMethod = methods;
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    try {
      currency = (await api.business.get()).defaultCurrency;
    } catch {
      // fall back to the default currency label
    }
    await load();
  });

  async function saveCsv(name: string, headers: string[], rows: string[][]) {
    message = "";
    error = "";
    try {
      const path = await saveDialog({
        defaultPath: `${name}-${from}-to-${to}.csv`.replace(/[^\w.\-]+/g, "-"),
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!path) return;
      await api.exportCsv(path, headers, rows);
      message = `Saved ${name} to ${path}`;
    } catch (e) {
      error = String(e);
    }
  }

  async function saveJson() {
    message = "";
    error = "";
    try {
      const path = await saveDialog({
        defaultPath: `ledgerly-data-${localDate()}.json`,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) return;
      await api.exportJson(path);
      message = `Saved all data to ${path}`;
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="page-head">
  <h1>Reports</h1>
  <div class="row">
    <input type="date" bind:value={from} style="width: auto;" />
    <span class="muted">to</span>
    <input type="date" bind:value={to} style="width: auto;" />
    <select bind:value={granularity} style="width: auto;">
      <option value="day">Daily</option>
      <option value="week">Weekly</option>
      <option value="month">Monthly</option>
      <option value="year">Yearly</option>
    </select>
    <button class="btn" onclick={load} disabled={loading}>{loading ? "Loading…" : "Run"}</button>
  </div>
</div>

{#if error}<p class="error">{error}</p>{/if}
{#if message}<p class="ok">{message}</p>{/if}

<div class="card">
  <div class="page-head">
    <h2>Revenue</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "revenue",
          ["Period", `Invoices`, `Invoiced (${currency})`, `Tax (${currency})`, `Received (${currency})`],
          revenue.map((row) => [
            row.period,
            String(row.invoiceCount),
            minorToCsv(row.invoicedMinor),
            minorToCsv(row.taxMinor),
            minorToCsv(row.paidMinor),
          ])
        )}
    >
      Export CSV
    </button>
  </div>

  {#if revenue.length === 0}
    <p class="muted">No invoices in this range.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Period</th>
          <th class="numeric">Invoices</th>
          <th class="numeric">Invoiced</th>
          <th class="numeric">Tax</th>
          <th class="numeric">Received</th>
        </tr>
      </thead>
      <tbody>
        {#each revenue as row (row.period)}
          <tr>
            <td>{row.period}</td>
            <td class="numeric">{row.invoiceCount}</td>
            <td class="numeric">{fromMinor(row.invoicedMinor, currency)}</td>
            <td class="numeric">{fromMinor(row.taxMinor, currency)}</td>
            <td class="numeric">{fromMinor(row.paidMinor, currency)}</td>
          </tr>
        {/each}
        <tr>
          <td><strong>Total</strong></td>
          <td class="numeric"><strong>{revenueTotals.count}</strong></td>
          <td class="numeric"><strong>{fromMinor(revenueTotals.invoiced, currency)}</strong></td>
          <td class="numeric"><strong>{fromMinor(revenueTotals.tax, currency)}</strong></td>
          <td class="numeric"><strong>{fromMinor(revenueTotals.paid, currency)}</strong></td>
        </tr>
      </tbody>
    </table>
  {/if}
</div>

<div class="card">
  <div class="page-head">
    <h2>Revenue by client</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "revenue-by-client",
          ["Client", "Invoices", `Invoiced (${currency})`, `Paid (${currency})`, `Outstanding (${currency})`],
          byClient.map((row) => [
            row.clientName,
            String(row.invoiceCount),
            minorToCsv(row.invoicedMinor),
            minorToCsv(row.paidMinor),
            minorToCsv(row.outstandingMinor),
          ])
        )}
    >
      Export CSV
    </button>
  </div>

  {#if byClient.length === 0}
    <p class="muted">No invoices in this range.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Client</th>
          <th class="numeric">Invoices</th>
          <th class="numeric">Invoiced</th>
          <th class="numeric">Paid</th>
          <th class="numeric">Outstanding</th>
        </tr>
      </thead>
      <tbody>
        {#each byClient as row (row.clientId ?? "none")}
          <tr>
            <td>{row.clientName}</td>
            <td class="numeric">{row.invoiceCount}</td>
            <td class="numeric">{fromMinor(row.invoicedMinor, currency)}</td>
            <td class="numeric">{fromMinor(row.paidMinor, currency)}</td>
            <td class="numeric">{fromMinor(row.outstandingMinor, currency)}</td>
          </tr>
        {/each}
        <tr>
          <td><strong>Total</strong></td>
          <td class="numeric"></td>
          <td class="numeric"><strong>{fromMinor(clientTotals.invoiced, currency)}</strong></td>
          <td class="numeric"><strong>{fromMinor(clientTotals.paid, currency)}</strong></td>
          <td class="numeric"><strong>{fromMinor(clientTotals.outstanding, currency)}</strong></td>
        </tr>
      </tbody>
    </table>
  {/if}
</div>

<div class="card">
  <div class="page-head">
    <h2>Aged receivables</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "aged-receivables",
          ["Age", "Invoices", `Balance (${currency})`],
          aged.map((row) => [row.label, String(row.invoiceCount), minorToCsv(row.balanceMinor)])
        )}
    >
      Export CSV
    </button>
  </div>

  <table>
    <thead>
      <tr><th>Age</th><th class="numeric">Invoices</th><th class="numeric">Balance</th></tr>
    </thead>
    <tbody>
      {#each aged as row (row.label)}
        <tr>
          <td>{row.label}</td>
          <td class="numeric">{row.invoiceCount}</td>
          <td class="numeric">{fromMinor(row.balanceMinor, currency)}</td>
        </tr>
      {/each}
      <tr>
        <td><strong>Total outstanding</strong></td>
        <td class="numeric"></td>
        <td class="numeric"><strong>{fromMinor(agedTotal, currency)}</strong></td>
      </tr>
    </tbody>
  </table>
</div>

<div class="card">
  <div class="page-head">
    <h2>Tax collected</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "tax-collected",
          ["Rate", "%", `Taxable (${currency})`, `Tax (${currency})`],
          tax.map((row) => [
            row.name,
            (row.rateBp / 100).toFixed(2),
            minorToCsv(row.taxableMinor),
            minorToCsv(row.taxMinor),
          ])
        )}
    >
      Export CSV
    </button>
  </div>

  {#if tax.length === 0}
    <p class="muted">No tax recorded in this range.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Rate</th>
          <th class="numeric">%</th>
          <th class="numeric">Taxable</th>
          <th class="numeric">Tax</th>
        </tr>
      </thead>
      <tbody>
        {#each tax as row (row.taxRateId ?? "none")}
          <tr>
            <td>{row.name}</td>
            <td class="numeric">{(row.rateBp / 100).toFixed(2)}</td>
            <td class="numeric">{fromMinor(row.taxableMinor, currency)}</td>
            <td class="numeric">{fromMinor(row.taxMinor, currency)}</td>
          </tr>
        {/each}
        <tr>
          <td><strong>Total</strong></td>
          <td class="numeric"></td>
          <td class="numeric"><strong>{fromMinor(taxTotals.taxable, currency)}</strong></td>
          <td class="numeric"><strong>{fromMinor(taxTotals.tax, currency)}</strong></td>
        </tr>
      </tbody>
    </table>
  {/if}
</div>

<div class="card">
  <div class="page-head">
    <h2>Payments received by method</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "payments-by-method",
          ["Method", "Payments", `Amount (${currency})`],
          byMethod.map((row) => [row.method, String(row.paymentCount), minorToCsv(row.amountMinor)])
        )}
    >
      Export CSV
    </button>
  </div>

  {#if byMethod.length === 0}
    <p class="muted">No payments in this range.</p>
  {:else}
    <table>
      <thead>
        <tr><th>Method</th><th class="numeric">Payments</th><th class="numeric">Amount</th></tr>
      </thead>
      <tbody>
        {#each byMethod as row (row.method)}
          <tr>
            <td>{row.method}</td>
            <td class="numeric">{row.paymentCount}</td>
            <td class="numeric">{fromMinor(row.amountMinor, currency)}</td>
          </tr>
        {/each}
        <tr>
          <td><strong>Total</strong></td>
          <td class="numeric"></td>
          <td class="numeric"><strong>{fromMinor(methodTotal, currency)}</strong></td>
        </tr>
      </tbody>
    </table>
  {/if}
</div>

<div class="card">
  <h2>Data export</h2>
  <p class="muted">
    A single JSON file containing every table: clients, invoices, payments, estimates and
    settings. Yours to keep, readable without this app.
  </p>
  <button class="btn secondary" onclick={saveJson}>Export all data (JSON)</button>
</div>
