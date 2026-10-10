<script lang="ts">
  import { api } from "$lib/api";
  import { formatDate, fromMinor } from "$lib/format";
  import type { AppInfo, Business, Invoice } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let info = $state<AppInfo | null>(null);
  let business = $state<Business | null>(null);
  let clientCount = $state(0);
  let invoices = $state<Invoice[]>([]);
  let loading = $state(true);
  let error = $state("");

  const currency = $derived(business?.defaultCurrency ?? "");

  // Unpaid, non-void invoices. Balance and status are computed by the Rust core.
  const open = $derived(
    invoices.filter((invoice) => invoice.displayStatus !== "void" && invoice.balanceMinor > 0)
  );
  const outstanding = $derived(open.reduce((sum, invoice) => sum + invoice.balanceMinor, 0));
  const overdueInvoices = $derived(
    invoices
      .filter((invoice) => invoice.displayStatus === "overdue")
      .sort((a, b) => (b.daysPastDue ?? 0) - (a.daysPastDue ?? 0))
  );
  const overdue = $derived(overdueInvoices.reduce((sum, invoice) => sum + invoice.balanceMinor, 0));

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

  // First-run path. Each step disappears once it is done, so the dashboard
  // teaches the product without a separate tour.
  const steps = $derived([
    {
      label: "Fill in your business profile",
      hint: "Your name, address and currency. They print on every invoice.",
      href: "/settings",
      done: Boolean(business?.displayName?.trim()),
    },
    {
      label: "Add a client",
      hint: "Who you bill.",
      href: "/clients/new",
      done: clientCount > 0,
    },
    {
      label: "Create your first invoice",
      hint: "Then export it and send it.",
      href: "/invoices/new",
      done: invoices.length > 0,
    },
  ]);
  const remaining = $derived(steps.filter((step) => !step.done).length);

  async function load() {
    loading = true;
    try {
      const [appInfo, biz, clients, allInvoices] = await Promise.all([
        api.appInfo(),
        api.business.get(),
        api.clients.list(),
        api.invoices.list(),
      ]);
      info = appInfo;
      business = biz;
      clientCount = clients.length;
      invoices = allInvoices;
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    void load();
  });
</script>

<PageHeader title="Dashboard" subtitle={business?.displayName?.trim() || undefined}>
  <a class="btn" href="/invoices/new">New invoice</a>
</PageHeader>

<AsyncState {loading} {error} rows={4} columns={3}>
  {#if remaining > 0}
    <div class="card setup">
      <h2>Finish setting up</h2>
      <ul>
        {#each steps as step (step.label)}
          <li class:done={step.done}>
            <span class="mark" aria-hidden="true">{step.done ? "✓" : ""}</span>
            <span class="text">
              {#if step.done}
                <s>{step.label}</s>
              {:else}
                <a class="link" href={step.href}>{step.label}</a>
                <span class="muted hint">{step.hint}</span>
              {/if}
            </span>
          </li>
        {/each}
      </ul>
    </div>
  {/if}

  <div class="card hero">
    <div class="hero-main">
      <h2>Outstanding</h2>
      <p class="figure">{fromMinor(outstanding, currency)}</p>
      <p class="muted">
        across {open.length}
        {open.length === 1 ? "unpaid invoice" : "unpaid invoices"}
      </p>
    </div>
    <dl class="hero-side">
      <div>
        <dt>Overdue</dt>
        <dd class={overdue > 0 ? "alarm" : ""}>{fromMinor(overdue, currency)}</dd>
      </div>
      <div>
        <dt>Invoices</dt>
        <dd>{invoices.filter((invoice) => invoice.displayStatus !== "void").length}</dd>
      </div>
      <div>
        <dt>Clients</dt>
        <dd>{clientCount}</dd>
      </div>
    </dl>
  </div>

  {#if overdueInvoices.length > 0}
    <div class="card">
      <div class="card-head">
        <h2>Needs chasing</h2>
        <a class="link" href="/invoices?status=overdue">View all overdue</a>
      </div>
      <table>
        <thead>
          <tr>
            <th>Number</th>
            <th>Client</th>
            <th>Due</th>
            <th class="numeric">Late by</th>
            <th class="numeric">Balance</th>
          </tr>
        </thead>
        <tbody>
          {#each overdueInvoices.slice(0, 5) as invoice (invoice.id)}
            <tr>
              <td><a class="link" href={`/invoices/${invoice.id}`}>{invoice.number}</a></td>
              <td>{invoice.clientName ?? "—"}</td>
              <td>{formatDate(invoice.dueDate)}</td>
              <td class="numeric">{invoice.daysPastDue ?? 0} days</td>
              <td class="numeric">{fromMinor(invoice.balanceMinor, invoice.currency)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}

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

  <div class="card">
    <h2>Your data</h2>
    <p class="muted">
      Everything is stored on this machine. No account, no cloud, no network.
    </p>
    <p class="muted">Data directory</p>
    <code>{info?.dataDir ?? "—"}</code>
  </div>
</AsyncState>

<style>
  .setup ul {
    margin: 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .setup li {
    display: flex;
    gap: var(--space-3);
    align-items: baseline;
  }

  .setup .mark {
    flex: 0 0 1.25rem;
    color: var(--ok);
    font-weight: 700;
  }

  .setup li.done .mark {
    color: var(--ok);
  }

  .setup .text {
    display: flex;
    flex-direction: column;
  }

  .setup .hint {
    font-size: var(--text-sm);
  }

  .hero {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(0, 1fr);
    gap: var(--space-5);
    align-items: center;
  }

  .figure {
    margin: 0;
    font-size: var(--text-3xl);
    font-weight: 700;
    letter-spacing: -0.03em;
    font-variant-numeric: tabular-nums;
    text-wrap: balance;
  }

  .hero-side {
    display: flex;
    gap: var(--space-5);
    margin: 0;
  }

  .hero-side dt {
    font-size: var(--text-sm);
    font-weight: 700;
    color: var(--ink-2);
  }

  .hero-side dd {
    margin: var(--space-1) 0 0;
    font-size: var(--text-lg);
    font-variant-numeric: tabular-nums;
  }

  .hero-side .alarm {
    color: var(--danger);
  }

  .card-head {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: var(--space-4);
  }

  @media (max-width: 720px) {
    .hero {
      grid-template-columns: 1fr;
    }
  }
</style>
