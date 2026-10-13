<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import { formatDate, fromMinor, statusLabel } from "$lib/format";
  import type { Estimate } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import SearchField from "$lib/components/SearchField.svelte";
  import { capped, RENDER_LIMIT } from "$lib/list";
  import { t } from "$lib/i18n.svelte";

  const statuses = ["draft", "sent", "accepted", "declined", "expired", "converted"] as const;

  let estimates = $state<Estimate[]>([]);
  let currency = $state("");
  let loading = $state(true);
  let error = $state("");
  let search = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  const query = $derived(page.url.searchParams.get("q") ?? "");
  const status = $derived(page.url.searchParams.get("status") ?? "");
  const visible = $derived(capped(estimates));

  async function load() {
    loading = true;
    try {
      const [business, rows] = await Promise.all([
        api.business.get(),
        api.estimates.list(query, status),
      ]);
      currency = business.defaultCurrency;
      estimates = rows;
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

  let applied = "";
  $effect(() => {
    const key = `${query}|${status}`;
    if (key === applied) return;
    applied = key;
    search = query;
    void load();
  });

  const filtered = $derived(Boolean(query) || Boolean(status));
</script>

<PageHeader title={t("estimates.title")}>
  <a class="btn" href="/estimates/new">{t("estimates.new")}</a>
</PageHeader>

<div class="toolbar">
  <SearchField
    bind:value={search}
    label={t("estimates.searchLabel")}
    example={t("estimates.searchExample")}
    oninput={onSearch}
  />
  <label class="filter">
    <span class="sr-only">{t("estimates.filterSrOnly")}</span>
    <select value={status} onchange={(e) => setParam("status", e.currentTarget.value)}>
      <option value="">{t("common.allStatuses")}</option>
      {#each statuses as value (value)}
        <option {value}>{statusLabel(value)}</option>
      {/each}
    </select>
  </label>
  {#if filtered}
    <button class="btn secondary" onclick={() => setParam("status", "")}>{t("common.clearFilter")}</button>
  {/if}
</div>

<AsyncState {loading} {error} rows={6} columns={6}>
  <div class="card">
    {#if estimates.length === 0}
      {#if filtered}
        <EmptyState
          title={t("estimates.nothingMatches")}
          body={t("estimates.nothingMatchesBody")}
        />
      {:else}
        <EmptyState title={t("estimates.none")} body={t("estimates.noneBody")}>
          <a class="btn" href="/estimates/new">{t("action.createFirstEstimate")}</a>
        </EmptyState>
      {/if}
    {:else}
      <table>
        <thead>
          <tr>
            <th>{t("common.number")}</th>
            <th>{t("common.client")}</th>
            <th>{t("common.issued")}</th>
            <th>{t("common.validUntil")}</th>
            <th class="numeric">{t("common.total")}</th>
            <th>{t("common.status")}</th>
          </tr>
        </thead>
        <tbody>
          {#each visible.shown as estimate (estimate.id)}
            <tr>
              <td><a class="link" href={`/estimates/${estimate.id}`}>{estimate.number}</a></td>
              <td>{estimate.clientName ?? t("common.emDash")}</td>
              <td>{formatDate(estimate.issueDate)}</td>
              <td>{formatDate(estimate.validUntil)}</td>
              <td class="numeric">{fromMinor(estimate.totalMinor, estimate.currency)}</td>
              <td>
                <span class={`badge ${estimate.displayStatus}`}>
                  {statusLabel(estimate.displayStatus)}
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if visible.hidden > 0}
        <p class="muted truncation">
          {t("list.showingFirstFilter", { limit: RENDER_LIMIT, total: estimates.length })}
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
</style>
