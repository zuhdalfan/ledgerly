<script lang="ts">
  import { api } from "$lib/api";
  import { formatDate, fromMinor } from "$lib/format";
  import type { AppInfo, Business, Invoice } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import BarList from "$lib/components/BarList.svelte";
  import ColumnChart from "$lib/components/ColumnChart.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { reminder } from "$lib/chase";
  import { t, plural, getLocale, localeFromSetting, type Locale } from "$lib/i18n.svelte";

  let info = $state<AppInfo | null>(null);
  let business = $state<Business | null>(null);
  let clientCount = $state(0);
  let invoices = $state<Invoice[]>([]);
  let revenue = $state<
    { period: string; label: string; primary: number; secondary: number }[]
  >([]);
  let loading = $state(true);
  let error = $state("");
  // Client-facing text follows the document language, not the UI language.
  let documentLocale = $state<Locale>("en");
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

  // The revenue report only returns months that have activity, so "this month"
  // has to be matched by period. Taking the last entry would quietly mean "the
  // last month with data", which is not the same thing.
  const collectedThisMonth = $derived(
    revenue.find((point) => point.period === periodOf(new Date()))?.primary ?? 0
  );

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
    // `key` is the filter the invoices list understands, so a bucket and the
    // rows behind it cannot drift apart.
    const buckets = [
      { key: "not_due", label: t("dash.bucket.notYetDue"), amountMinor: 0, count: 0 },
      { key: "1-30", label: t("dash.bucket.days1to30"), amountMinor: 0, count: 0 },
      { key: "31-60", label: t("dash.bucket.days31to60"), amountMinor: 0, count: 0 },
      { key: "61-90", label: t("dash.bucket.days61to90"), amountMinor: 0, count: 0 },
      { key: "90plus", label: t("dash.bucket.days90plus"), amountMinor: 0, count: 0 },
    ];

    for (const invoice of open) {
      const days = invoice.daysPastDue ?? 0;
      const index = days <= 0 ? 0 : days <= 30 ? 1 : days <= 60 ? 2 : days <= 90 ? 3 : 4;
      buckets[index].amountMinor += invoice.balanceMinor;
      buckets[index].count += 1;
    }

    return buckets;
  });
  // The oldest bucket is the one that matters most, so it is the only colour.
  const agingRows = $derived(
    aging.map((bucket, index) => ({
      ...bucket,
      href: `/invoices?age=${bucket.key}`,
      tone: index === 4 ? ("danger" as const) : ("ink" as const),
    }))
  );

  // First-run path. Each step disappears once it is done, so the dashboard
  // teaches the product without a separate tour.
  const steps = $derived([
    {
      label: t("dash.setup.business"),
      hint: t("dash.setup.businessHint"),
      href: "/settings",
      done: Boolean(business?.displayName?.trim()),
    },
    {
      label: t("dash.setup.client"),
      hint: t("dash.setup.clientHint"),
      href: "/clients/new",
      done: clientCount > 0,
    },
    {
      label: t("dash.setup.invoice"),
      hint: t("dash.setup.invoiceHint"),
      href: "/invoices/new",
      done: invoices.length > 0,
    },
  ]);
  const remaining = $derived(steps.filter((step) => !step.done).length);

  async function copyReminderFor(invoice: Invoice) {
    chaseMessage = "";
    const { subject, body } = reminder(invoice, business?.displayName ?? "", documentLocale);
    try {
      await navigator.clipboard.writeText(`${subject}\n\n${body}`);
      chaseMessage = t("dash.reminderCopied", { number: invoice.number });
    } catch {
      chaseMessage = t("dash.reminderFailed");
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
    return new Date(Number(year), Number(month) - 1, 1).toLocaleString(getLocale(), {
      month: "short",
    });
  }

  /// Matches the period format the report groups by: "2026-10".
  function periodOf(date: Date): string {
    const month = String(date.getMonth() + 1).padStart(2, "0");
    return `${date.getFullYear()}-${month}`;
  }

  async function load() {
    loading = true;
    try {
      const now = new Date();
      const from = isoLocal(new Date(now.getFullYear(), now.getMonth() - 5, 1));
      const to = isoLocal(now);

      const [appInfo, biz, clients, allInvoices, periods, pdfLocale] = await Promise.all([
        api.appInfo(),
        api.business.get(),
        api.clients.list(),
        api.invoices.list(),
        // A failed chart must not blank the whole dashboard.
        api.reports.revenue(from, to, "month").catch(() => []),
        // Same for the document language: fall back to English.
        api.settings.get("pdf.locale").catch(() => null),
      ]);
      info = appInfo;
      business = biz;
      clientCount = clients.length;
      invoices = allInvoices;
      documentLocale = localeFromSetting(pdfLocale);
      revenue = periods.map((row) => ({
        period: row.period,
        label: monthLabel(row.period),
        primary: row.paidMinor,
        secondary: row.invoicedMinor,
        // A month's link goes to what was *billed* then, which is the series
        // that has invoices behind it.
        href: `/invoices?issued=${row.period}`,
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

<PageHeader title={t("dash.title")} subtitle={business?.displayName?.trim() || undefined}>
  <a class="btn" href="/invoices/new">{t("dash.newInvoice")}</a>
</PageHeader>

<AsyncState {loading} {error} rows={4} columns={3}>
  {#if remaining > 0}
    <div class="card setup rise" style="--i: 0">
      <h2>{t("dash.setup.title")}</h2>
      <ul>
        {#each steps as step (step.href)}
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
      <h2>{t("dash.outstanding")}</h2>
      <p class="figure">{fromMinor(outstanding, currency)}</p>
      <p class="muted">
        {plural(open.length, "dash.acrossUnpaidOne", "dash.acrossUnpaidOther", {
          count: open.length,
        })}
      </p>
    </div>
    <dl class="hero-side">
      <div>
        <dt>{t("dash.overdue")}</dt>
        <dd class={overdue > 0 ? "alarm" : ""}>{fromMinor(overdue, currency)}</dd>
      </div>
      <div>
        <dt>{t("dash.collectedThisMonth")}</dt>
        <dd>{fromMinor(collectedThisMonth, currency)}</dd>
      </div>
      <div>
        <dt>{t("dash.clients")}</dt>
        <dd>{clientCount}</dd>
      </div>
    </dl>
  </div>

  {#if overdueInvoices.length > 0}
    <div class="card rise" style="--i: 2">
      <div class="card-head">
        <h2>{t("dash.needsChasing")}</h2>
        <a class="link" href="/invoices?status=overdue">{t("action.viewAllOverdue")}</a>
      </div>
      <table>
        <thead>
          <tr>
            <th>{t("common.number")}</th>
            <th>{t("common.client")}</th>
            <th>{t("common.due")}</th>
            <th class="numeric">{t("dash.lateBy")}</th>
            <th class="numeric">{t("common.balance")}</th>
            <th>{t("dash.chased")}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each needsChasing.slice(0, 5) as invoice (invoice.id)}
            <tr>
              <td><a class="link" href={`/invoices/${invoice.id}`}>{invoice.number}</a></td>
              <td>{invoice.clientName ?? t("common.emDash")}</td>
              <td>{formatDate(invoice.dueDate)}</td>
              <td class="numeric">
                {plural(invoice.daysPastDue ?? 0, "dash.dayOne", "dash.dayOther", {
                  count: invoice.daysPastDue ?? 0,
                })}
              </td>
              <td class="numeric">{fromMinor(invoice.balanceMinor, invoice.currency)}</td>
              <td>
                {invoice.lastChasedAt
                  ? formatDate(invoice.lastChasedAt.slice(0, 10))
                  : t("dash.never")}
              </td>
              <td>
                <div class="row-actions">
                  <button class="btn secondary" onclick={() => copyReminderFor(invoice)}>
                    {t("dash.copy")}
                  </button>
                  <button class="btn secondary" onclick={() => toggleChased(invoice)}>
                    {invoice.lastChasedAt ? t("action.unmark") : t("action.markChased")}
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
    <h2>{t("dash.agedReceivables")}</h2>
    <BarList
      rows={agingRows}
      {currency}
      label={t("dash.agedLabel")}
      columnLabel={t("dash.agedColumn")}
    />
    <p class="muted hint">{t("dash.agedHint")}</p>
  </div>

  <div class="card rise" style="--i: 4">
    <h2>{t("dash.billedCollected")}</h2>
    <p class="muted">{t("dash.billedCollectedBody")}</p>
    <ColumnChart
      points={revenue}
      {currency}
      label={t("dash.billedCollectedLabel")}
      primaryName={t("dash.collected")}
      secondaryName={t("dash.billed")}
    />
  </div>

  <div class="card rise" style="--i: 5">
    <h2>{t("dash.yourData")}</h2>
    <p class="muted">{t("dash.yourDataBody")}</p>
    <p class="muted">{t("dash.dataDirectory")}</p>
    <code>{info?.dataDir ?? t("common.emDash")}</code>
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

  .hero-main {
    min-width: 0;
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
    flex-wrap: wrap;
    gap: var(--space-5);
    margin: 0;
    min-width: 0;
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

  .hint {
    margin-top: var(--space-3);
    margin-bottom: 0;
    font-size: var(--text-sm);
  }

  @media (max-width: 820px) {
    .hero {
      grid-template-columns: 1fr;
    }
  }
</style>
