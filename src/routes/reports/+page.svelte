<script lang="ts">
  import { onMount } from "svelte";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { fromMinor, localDate, minorToCsv } from "$lib/format";
  import { t } from "$lib/i18n.svelte";
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

  // The aged buckets arrive labelled in English from the core; the UI knows
  // which bucket a row is by its position, and prints the current language.
  const AGE_KEYS = [
    "reports.age.notYetDue",
    "reports.age.days1to30",
    "reports.age.days31to60",
    "reports.age.days61to90",
    "reports.age.days90plus",
  ] as const;

  function ageLabel(index: number, fallback: string): string {
    const key = AGE_KEYS[index];
    return key ? t(key) : fallback;
  }

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
      message = t("reports.saved", { name, path });
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
      message = t("reports.savedAll", { path });
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="page-head">
  <h1>{t("reports.title")}</h1>
  <div class="row">
    <input type="date" bind:value={from} style="width: auto;" />
    <span class="muted">{t("common.to")}</span>
    <input type="date" bind:value={to} style="width: auto;" />
    <select bind:value={granularity} style="width: auto;">
      <option value="day">{t("reports.granularityDay")}</option>
      <option value="week">{t("reports.granularityWeek")}</option>
      <option value="month">{t("reports.granularityMonth")}</option>
      <option value="year">{t("reports.granularityYear")}</option>
    </select>
    <button class="btn" onclick={load} disabled={loading}>
      {loading ? t("action.loading") : t("action.run")}
    </button>
  </div>
</div>

{#if error}<p class="error">{error}</p>{/if}
{#if message}<p class="ok">{message}</p>{/if}

<div class="card">
  <div class="page-head">
    <h2>{t("reports.revenue")}</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "revenue",
          [
            t("common.period"),
            t("common.invoices"),
            `${t("common.invoiced")} (${currency})`,
            `${t("common.tax")} (${currency})`,
            `${t("common.received")} (${currency})`,
          ],
          revenue.map((row) => [
            row.period,
            String(row.invoiceCount),
            minorToCsv(row.invoicedMinor),
            minorToCsv(row.taxMinor),
            minorToCsv(row.paidMinor),
          ])
        )}
    >
      {t("action.exportCsv")}
    </button>
  </div>

  {#if revenue.length === 0}
    <p class="muted">{t("reports.noInvoices")}</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>{t("common.period")}</th>
          <th class="numeric">{t("common.invoices")}</th>
          <th class="numeric">{t("common.invoiced")}</th>
          <th class="numeric">{t("common.tax")}</th>
          <th class="numeric">{t("common.received")}</th>
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
          <td><strong>{t("common.total")}</strong></td>
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
    <h2>{t("reports.revenueByClient")}</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "revenue-by-client",
          [
            t("common.client"),
            t("common.invoices"),
            `${t("common.invoiced")} (${currency})`,
            `${t("common.paid")} (${currency})`,
            `${t("common.outstanding")} (${currency})`,
          ],
          byClient.map((row) => [
            row.clientName,
            String(row.invoiceCount),
            minorToCsv(row.invoicedMinor),
            minorToCsv(row.paidMinor),
            minorToCsv(row.outstandingMinor),
          ])
        )}
    >
      {t("action.exportCsv")}
    </button>
  </div>

  {#if byClient.length === 0}
    <p class="muted">{t("reports.noInvoices")}</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>{t("common.client")}</th>
          <th class="numeric">{t("common.invoices")}</th>
          <th class="numeric">{t("common.invoiced")}</th>
          <th class="numeric">{t("common.paid")}</th>
          <th class="numeric">{t("common.outstanding")}</th>
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
          <td><strong>{t("common.total")}</strong></td>
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
    <h2>{t("reports.agedReceivables")}</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "aged-receivables",
          [t("common.age"), t("common.invoices"), `${t("common.balance")} (${currency})`],
          aged.map((row, index) => [
            ageLabel(index, row.label),
            String(row.invoiceCount),
            minorToCsv(row.balanceMinor),
          ])
        )}
    >
      {t("action.exportCsv")}
    </button>
  </div>

  <table>
    <thead>
      <tr>
        <th>{t("common.age")}</th>
        <th class="numeric">{t("common.invoices")}</th>
        <th class="numeric">{t("common.balance")}</th>
      </tr>
    </thead>
    <tbody>
      {#each aged as row, index (row.label)}
        <tr>
          <td>{ageLabel(index, row.label)}</td>
          <td class="numeric">{row.invoiceCount}</td>
          <td class="numeric">{fromMinor(row.balanceMinor, currency)}</td>
        </tr>
      {/each}
      <tr>
        <td><strong>{t("reports.totalOutstanding")}</strong></td>
        <td class="numeric"></td>
        <td class="numeric"><strong>{fromMinor(agedTotal, currency)}</strong></td>
      </tr>
    </tbody>
  </table>
</div>

<div class="card">
  <div class="page-head">
    <h2>{t("reports.taxCollected")}</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "tax-collected",
          [
            t("common.rate"),
            "%",
            `${t("common.taxable")} (${currency})`,
            `${t("common.tax")} (${currency})`,
          ],
          tax.map((row) => [
            row.name,
            (row.rateBp / 100).toFixed(2),
            minorToCsv(row.taxableMinor),
            minorToCsv(row.taxMinor),
          ])
        )}
    >
      {t("action.exportCsv")}
    </button>
  </div>

  {#if tax.length === 0}
    <p class="muted">{t("reports.noTax")}</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>{t("common.rate")}</th>
          <th class="numeric">%</th>
          <th class="numeric">{t("common.taxable")}</th>
          <th class="numeric">{t("common.tax")}</th>
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
          <td><strong>{t("common.total")}</strong></td>
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
    <h2>{t("reports.paymentsByMethod")}</h2>
    <button
      class="btn secondary"
      onclick={() =>
        saveCsv(
          "payments-by-method",
          [t("common.method"), t("common.payments"), `${t("common.amount")} (${currency})`],
          byMethod.map((row) => [row.method, String(row.paymentCount), minorToCsv(row.amountMinor)])
        )}
    >
      {t("action.exportCsv")}
    </button>
  </div>

  {#if byMethod.length === 0}
    <p class="muted">{t("reports.noPayments")}</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>{t("common.method")}</th>
          <th class="numeric">{t("common.payments")}</th>
          <th class="numeric">{t("common.amount")}</th>
        </tr>
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
          <td><strong>{t("common.total")}</strong></td>
          <td class="numeric"></td>
          <td class="numeric"><strong>{fromMinor(methodTotal, currency)}</strong></td>
        </tr>
      </tbody>
    </table>
  {/if}
</div>

<div class="card">
  <h2>{t("reports.dataExport")}</h2>
  <p class="muted">{t("reports.dataExportBody")}</p>
  <button class="btn secondary" onclick={saveJson}>{t("action.exportAllData")}</button>
</div>
