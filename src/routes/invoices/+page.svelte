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
  import { t } from "$lib/i18n.svelte";
  import type { MessageKey } from "$lib/locales/en";

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

  const AGE_KEYS: Record<string, MessageKey> = {
    not_due: "invoices.age.notDue",
    "1-30": "invoices.age.days1to30",
    "31-60": "invoices.age.days31to60",
    "61-90": "invoices.age.days61to90",
    "90plus": "invoices.age.days90plus",
  };
  const ageLabel = $derived(age && AGE_KEYS[age] ? t(AGE_KEYS[age]) : "");

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

<PageHeader title={t("invoices.title")}>
  <a class="btn" href="/invoices/new">{t("invoices.new")}</a>
</PageHeader>

<div class="toolbar">
  <SearchField
    bind:value={search}
    label={t("invoices.searchLabel")}
    example={t("invoices.searchExample")}
    oninput={onSearch}
  />
  <label class="filter">
    <span class="sr-only">{t("invoices.filterSrOnly")}</span>
    <select value={status} onchange={(e) => setParam("status", e.currentTarget.value)}>
      <option value="">{t("common.allStatuses")}</option>
      {#each statuses as value (value)}
        <option {value}>{statusLabel(value)}</option>
      {/each}
    </select>
  </label>
  {#if filtered}
    <button class="btn secondary" onclick={clearFilters}>{t("common.clearFilter")}</button>
  {/if}
</div>

{#if ageLabel || issued}
  <p class="muted describing">
    {#if issued}{t("invoices.issuedIn", { issued })}{/if}
    {#if ageLabel} {ageLabel[0].toUpperCase()}{ageLabel.slice(1)}.{/if}
  </p>
{/if}

<AsyncState {loading} {error} rows={6} columns={7}>
  <div class="card">
    {#if invoices.length === 0}
      {#if filtered}
        <EmptyState
          title={t("invoices.nothingMatches")}
          body={t("invoices.nothingMatchesBody")}
        />
      {:else}
        <EmptyState title={t("invoices.none")} body={t("invoices.noneBody")}>
          <a class="btn" href="/invoices/new">{t("action.createFirstInvoice")}</a>
        </EmptyState>
      {/if}
    {:else}
      <table>
        <thead>
          <tr>
            <th>{t("common.number")}</th>
            <th>{t("common.client")}</th>
            <th>{t("common.issued")}</th>
            <th>{t("common.due")}</th>
            <th class="numeric">{t("common.total")}</th>
            <th class="numeric">{t("common.balance")}</th>
            <th>{t("common.status")}</th>
          </tr>
        </thead>
        <tbody>
          {#each visible.shown as invoice (invoice.id)}
            <tr>
              <td><a class="link" href={`/invoices/${invoice.id}`}>{invoice.number}</a></td>
              <td>{invoice.clientName ?? t("common.emDash")}</td>
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
          {t("list.showingFirstFilter", { limit: RENDER_LIMIT, total: invoices.length })}
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
