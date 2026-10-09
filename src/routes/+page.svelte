<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { fromMinor } from "$lib/format";
  import type { AppInfo, Business, Invoice } from "$lib/types";

  let info = $state<AppInfo | null>(null);
  let business = $state<Business | null>(null);
  let clientCount = $state(0);
  let catalogCount = $state(0);
  let invoices = $state<Invoice[]>([]);
  let error = $state("");

  const currency = $derived(business?.defaultCurrency ?? "USD");

  // Unpaid, non-void invoices. Balance and status are computed by the Rust core.
  const open = $derived(
    invoices.filter((invoice) => invoice.displayStatus !== "void" && invoice.balanceMinor > 0)
  );
  const outstanding = $derived(open.reduce((sum, invoice) => sum + invoice.balanceMinor, 0));
  const overdue = $derived(
    invoices
      .filter((invoice) => invoice.displayStatus === "overdue")
      .reduce((sum, invoice) => sum + invoice.balanceMinor, 0)
  );

  const aging = $derived.by(() => {
    const buckets = [
      { label: "Not yet due", amount: 0 },
      { label: "1–30 days", amount: 0 },
      { label: "31–60 days", amount: 0 },
      { label: "61–90 days", amount: 0 },
      { label: "90+ days", amount: 0 },
    ];

    for (const invoice of open) {
      const days = invoice.daysPastDue ?? 0;
      const index = days <= 0 ? 0 : days <= 30 ? 1 : days <= 60 ? 2 : days <= 90 ? 3 : 4;
      buckets[index].amount += invoice.balanceMinor;
    }

    return buckets;
  });

  onMount(async () => {
    try {
      const [appInfo, biz, clients, catalog, allInvoices] = await Promise.all([
        api.appInfo(),
        api.business.get(),
        api.clients.list(),
        api.catalog.list(),
        api.invoices.list(),
      ]);
      info = appInfo;
      business = biz;
      clientCount = clients.length;
      catalogCount = catalog.length;
      invoices = allInvoices;
    } catch (e) {
      error = String(e);
    }
  });
</script>

<div class="page-head">
  <h1>Dashboard</h1>
  <a class="btn" href="/invoices/new">New invoice</a>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

<div class="grid-2">
  <div class="card">
    <h2>Invoices</h2>
    <p class="stat">{invoices.filter((invoice) => invoice.displayStatus !== "void").length}</p>
    <a class="link" href="/invoices">View invoices</a>
  </div>

  <div class="card">
    <h2>Outstanding</h2>
    <p class="stat">{fromMinor(outstanding, currency)}</p>
    <p class="muted">Across {open.length} unpaid invoice(s)</p>
  </div>

  <div class="card">
    <h2>Overdue</h2>
    <p class="stat">{fromMinor(overdue, currency)}</p>
    <a class="link" href="/invoices">Show overdue</a>
  </div>

  <div class="card">
    <h2>Clients</h2>
    <p class="stat">{clientCount}</p>
    <a class="link" href="/clients">Manage clients</a>
  </div>
</div>

<div class="card">
  <h2>Aged receivables</h2>
  <table>
    <thead>
      <tr><th>Age</th><th class="numeric">Balance</th></tr>
    </thead>
    <tbody>
      {#each aging as bucket (bucket.label)}
        <tr>
          <td>{bucket.label}</td>
          <td class="numeric">{fromMinor(bucket.amount, currency)}</td>
        </tr>
      {/each}
      <tr>
        <td><strong>Total outstanding</strong></td>
        <td class="numeric"><strong>{fromMinor(outstanding, currency)}</strong></td>
      </tr>
    </tbody>
  </table>
</div>

<div class="grid-2">
  <div class="card">
    <h2>Business</h2>
    <p>{business?.displayName || "Not set"}</p>
    <p class="muted">Default currency: {business?.defaultCurrency ?? "—"}</p>
    <p class="muted">Catalog items: {catalogCount}</p>
    <a class="link" href="/settings">Business settings</a>
  </div>

  <div class="card">
    <h2>Local data</h2>
    <p class="muted">Everything is stored on this machine. No account, no cloud, no network.</p>
    <p class="muted">Data directory</p>
    <code>{info?.dataDir ?? "…"}</code>
  </div>
</div>
