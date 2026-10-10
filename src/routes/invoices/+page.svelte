<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import { formatDate, fromMinor, statusLabel } from "$lib/format";
  import type { Invoice } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import SearchField from "$lib/components/SearchField.svelte";
  import { capped, RENDER_LIMIT } from "$lib/list";

  const statuses = ["draft", "sent", "unpaid", "overdue", "paid", "void"] as const;

  let invoices = $state<Invoice[]>([]);
  let currency = $state("");
  let loading = $state(true);
  let error = $state("");
  let search = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  // The filter lives in the URL, so "show overdue" is a real link and the back
  // button behaves.
  const query = $derived(page.url.searchParams.get("q") ?? "");
  const status = $derived(page.url.searchParams.get("status") ?? "");
  const age = $derived(page.url.searchParams.get("age") ?? "");
  const issued = $derived(page.url.searchParams.get("issued") ?? "");

  const AGE_LABELS: Record<string, string> = {
    not_due: "not yet due",
    "1-30": "1–30 days late",
    "31-60": "31–60 days late",
    "61-90": "61–90 days late",
    "90plus": "more than 90 days late",
  };
  const ageLabel = $derived(AGE_LABELS[age] ?? "");

  async function load() {
    loading = true;
    try {
      const [business, rows] = await Promise.all([
        api.business.get(),
        // No client filter here: this page is the whole book.
        api.invoices.list(query, status, undefined, age, issued),
      ]);
      currency = business.defaultCurrency;
      invoices = rows;
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function setParam(key: string, value: string) {
    const next = new URL(page.url);
    if (value) next.searchParams.set(key, value);
    else next.searchParams.delete(key);
    await goto(next, { replaceState: true, keepFocus: true, noScroll: true });
  }

  function onSearch() {
    clearTimeout(timer);
    timer = setTimeout(() => void setParam("q", search), 200);
  }

  // Reload when the URL changes, including via the back button.
  let applied = "";
  $effect(() => {
    const key = `${query}|${status}|${age}|${issued}`;
    if (key === applied) return;
    applied = key;
    search = query;
    void load();
  });

  const filtered = $derived(Boolean(query) || Boolean(status) || Boolean(age) || Boolean(issued));
  const visible = $derived(capped(invoices));

  async function clearFilters() {
    const next = new URL(page.url);
    for (const key of ["q", "status", "age", "issued"]) next.searchParams.delete(key);
    await goto(next, { replaceState: true, keepFocus: true, noScroll: true });
  }
</script>

<PageHeader title="Invoices">
  <a class="btn" href="/invoices/new">New invoice</a>
</PageHeader>

<div class="toolbar">
  <SearchField
    bind:value={search}
    label="Search invoices by number or client"
    example="Number or client"
    oninput={onSearch}
  />
  <label class="filter">
    <span class="sr-only">Filter invoices by status</span>
    <select value={status} onchange={(e) => setParam("status", e.currentTarget.value)}>
      <option value="">All statuses</option>
      {#each statuses as value (value)}
        <option {value}>{statusLabel(value)}</option>
      {/each}
    </select>
  </label>
  {#if filtered}
    <button class="btn secondary" onclick={clearFilters}>Clear filter</button>
  {/if}
</div>

{#if ageLabel || issued}
  <p class="muted describing">
    {#if issued}Invoices issued in {issued}.{/if}
    {#if ageLabel} {ageLabel[0].toUpperCase()}{ageLabel.slice(1)}.{/if}
  </p>
{/if}

<AsyncState {loading} {error} rows={6} columns={7}>
  <div class="card">
    {#if invoices.length === 0}
      {#if filtered}
        <EmptyState
          title="Nothing matches that"
          body="No invoice has that number or client with this status."
        />
      {:else}
        <EmptyState
          title="No invoices yet"
          body="An invoice records what you did, what it cost, and when it is due."
        >
          <a class="btn" href="/invoices/new">Create your first invoice</a>
        </EmptyState>
      {/if}
    {:else}
      <table>
        <thead>
          <tr>
            <th>Number</th>
            <th>Client</th>
            <th>Issued</th>
            <th>Due</th>
            <th class="numeric">Total</th>
            <th class="numeric">Balance</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {#each visible.shown as invoice (invoice.id)}
            <tr>
              <td><a class="link" href={`/invoices/${invoice.id}`}>{invoice.number}</a></td>
              <td>{invoice.clientName ?? "—"}</td>
              <td>{formatDate(invoice.issueDate)}</td>
              <td>{formatDate(invoice.dueDate)}</td>
              <td class="numeric">{fromMinor(invoice.totalMinor, invoice.currency)}</td>
              <td class="numeric">{fromMinor(invoice.balanceMinor, invoice.currency)}</td>
              <td>
                <span class={`badge ${invoice.displayStatus}`}>
                  {statusLabel(invoice.displayStatus)}
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if visible.hidden > 0}
        <p class="muted truncation">
          Showing the first {RENDER_LIMIT} of {invoices.length}. Search or filter to narrow it down.
        </p>
      {/if}
    {/if}
  </div>
</AsyncState>

<style>
  .filter {
    flex: 0 0 auto;
  }

  .filter select {
    width: auto;
  }

  .truncation {
    margin-top: var(--space-3);
  }

  .describing {
    margin-bottom: var(--space-3);
  }
</style>
