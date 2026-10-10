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

  const statuses = ["draft", "sent", "accepted", "declined", "expired", "converted"] as const;

  let estimates = $state<Estimate[]>([]);
  let currency = $state("");
  let loading = $state(true);
  let error = $state("");
  let search = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  const query = $derived(page.url.searchParams.get("q") ?? "");
  const status = $derived(page.url.searchParams.get("status") ?? "");

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

<PageHeader title="Estimates">
  <a class="btn" href="/estimates/new">New estimate</a>
</PageHeader>

<div class="toolbar">
  <SearchField
    bind:value={search}
    label="Search estimates by number or client"
    example="Number or client"
    oninput={onSearch}
  />
  <label class="filter">
    <span class="sr-only">Filter estimates by status</span>
    <select value={status} onchange={(e) => setParam("status", e.currentTarget.value)}>
      <option value="">All statuses</option>
      {#each statuses as value (value)}
        <option {value}>{statusLabel(value)}</option>
      {/each}
    </select>
  </label>
  {#if filtered}
    <button class="btn secondary" onclick={() => setParam("status", "")}>Clear filter</button>
  {/if}
</div>

<AsyncState {loading} {error} rows={6} columns={6}>
  <div class="card">
    {#if estimates.length === 0}
      {#if filtered}
        <EmptyState
          title="Nothing matches that"
          body="No estimate has that number or client with this status."
        />
      {:else}
        <EmptyState
          title="No estimates yet"
          body="An estimate is a quote. Send it, and convert it into an invoice once the work is agreed."
        >
          <a class="btn" href="/estimates/new">Create your first estimate</a>
        </EmptyState>
      {/if}
    {:else}
      <table>
        <thead>
          <tr>
            <th>Number</th>
            <th>Client</th>
            <th>Issued</th>
            <th>Valid until</th>
            <th class="numeric">Total</th>
            <th>Status</th>
          </tr>
        </thead>
        <tbody>
          {#each estimates as estimate (estimate.id)}
            <tr>
              <td><a class="link" href={`/estimates/${estimate.id}`}>{estimate.number}</a></td>
              <td>{estimate.clientName ?? "—"}</td>
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
</style>
