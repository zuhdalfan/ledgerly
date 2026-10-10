<script lang="ts">
  import { api } from "$lib/api";
  import { formatDate, fromMinor } from "$lib/format";
  import type { AppInfo, Business, Invoice } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import BarList from "$lib/components/BarList.svelte";
  import ColumnChart from "$lib/components/ColumnChart.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { reminder } from "$lib/chase";

  let info = $state<AppInfo | null>(null);
  let business = $state<Business | null>(null);
  let clientCount = $state(0);
  let invoices = $state<Invoice[]>([]);
  let revenue = $state<{ label: string; primary: number; secondary: number }[]>([]);
  let loading = $state(true);
  let error = $state("");
  let chaseMessage = $state("");

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

  // Chasing order: never chased first, then the ones left longest, then the
  // most overdue. This is the queue the user actually works through.
  const needsChasing = $derived(
    [...overdueInvoices].sort((a, b) => {
      const chased = (invoice: Invoice) => (invoice.lastChasedAt ? 1 : 0);
      if (chased(a) !== chased(b)) return chased(a) - chased(b);
      if (a.lastChasedAt && b.lastChasedAt && a.lastChasedAt !== b.lastChasedAt) {
        return a.lastChasedAt < b.lastChasedAt ? -1 : 1;
      }
      return (b.daysPastDue ?? 0) - (a.daysPastDue ?? 0);
    })
  );

  const aging = $derived.by(() => {
    const buckets = [
      { label: "Not yet due", amountMinor: 0 },
      { label: "1–30 days", amountMinor: 0 },
      { label: "31–60 days", amountMinor: 0 },
      { label: "61–90 days", amountMinor: 0 },
      { label: "90+ days", amountMinor: 0 },
    ];

    for (const invoice of open) {
      const days = invoice.daysPastDue ?? 0;
      const index = days <= 0 ? 0 : days <= 30 ? 1 : days <= 60 ? 2 : days <= 90 ? 3 : 4;
      buckets[index].amountMinor += invoice.balanceMinor;
    }

    return buckets;
  });
  // The oldest bucket is the one that matters most, so it is the only colour.
  const agingRows = $derived(
    aging.map((bucket, index) => ({
      ...bucket,
      tone: index === 4 ? ("danger" as const) : ("ink" as const),
    }))
  );

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

  async function copyReminderFor(invoice: Invoice) {
    chaseMessage = "";
    const { subject, body } = reminder(invoice, business?.displayName ?? "");
    try {
      await navigator.clipboard.writeText(`${subject}\n\n${body}`);
      chaseMessage = `Reminder for ${invoice.number} copied. Paste it into your email.`;
    } catch {
      chaseMessage = "Could not reach the clipboard. Open the invoice to copy it by hand.";
    }
  }

  async function toggleChased(invoice: Invoice) {
    chaseMessage = "";
    try {
      await api.invoices.setChased(invoice.id, invoice.lastChasedAt === null);
      await load();
    } catch (e) {
      error = String(e);
    }
  }

  function isoLocal(date: Date): string {
    const month = String(date.getMonth() + 1).padStart(2, "0");
    const day = String(date.getDate()).padStart(2, "0");
    return `${date.getFullYear()}-${month}-${day}`;
  }

  function monthLabel(period: string): string {
    const [year, month] = period.split("-");
    return new Date(Number(year), Number(month) - 1, 1).toLocaleString(undefined, {
      month: "short",
    });
  }

  async function load() {
    loading = true;
    try {
      const now = new Date();
      const from = isoLocal(new Date(now.getFullYear(), now.getMonth() - 5, 1));
      const to = isoLocal(now);

      const [appInfo, biz, clients, allInvoices, periods] = await Promise.all([
        api.appInfo(),
        api.business.get(),
        api.clients.list(),
        api.invoices.list(),
        // A failed chart must not blank the whole dashboard.
        api.reports.revenue(from, to, "month").catch(() => []),
      ]);
      info = appInfo;
      business = biz;
      clientCount = clients.length;
      invoices = allInvoices;
      revenue = periods.map((row) => ({
        label: monthLabel(row.period),
        primary: row.paidMinor,
        secondary: row.invoicedMinor,
      }));
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
    <div class="card setup rise" style="--i: 0">
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

  <div class="card hero rise" style="--i: 1">
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
    <div class="card rise" style="--i: 2">
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
            <th>Chased</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each needsChasing.slice(0, 5) as invoice (invoice.id)}
            <tr>
              <td><a class="link" href={`/invoices/${invoice.id}`}>{invoice.number}</a></td>
              <td>{invoice.clientName ?? "—"}</td>
              <td>{formatDate(invoice.dueDate)}</td>
              <td class="numeric">{invoice.daysPastDue ?? 0} days</td>
              <td class="numeric">{fromMinor(invoice.balanceMinor, invoice.currency)}</td>
              <td>
                {invoice.lastChasedAt ? formatDate(invoice.lastChasedAt.slice(0, 10)) : "Never"}
              </td>
              <td>
                <div class="row-actions">
                  <button class="btn secondary" onclick={() => copyReminderFor(invoice)}>
                    Copy
                  </button>
                  <button class="btn secondary" onclick={() => toggleChased(invoice)}>
                    {invoice.lastChasedAt ? "Unmark" : "Mark chased"}
                  </button>
                </div>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if chaseMessage}<p class="ok">{chaseMessage}</p>{/if}
    </div>
  {/if}

  <div class="card rise" style="--i: 3">
    <h2>Aged receivables</h2>
    <BarList
      rows={agingRows}
      {currency}
      label="Outstanding balance by age"
      columnLabel="Share of the largest bucket"
    />
  </div>

  <div class="card rise" style="--i: 4">
    <h2>Billed and collected</h2>
    <p class="muted">The last six months. The gap between the bars is money you are owed.</p>
    <ColumnChart
      points={revenue}
      {currency}
      label="Billed and collected over the last six months"
      primaryName="Collected"
      secondaryName="Billed"
    />
  </div>

  <div class="card rise" style="--i: 5">
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

  .row-actions {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
  }

  @media (max-width: 720px) {
    .hero {
      grid-template-columns: 1fr;
    }
  }
</style>
