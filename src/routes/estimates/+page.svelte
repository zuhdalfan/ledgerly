<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { formatDate, fromMinor, statusLabel } from "$lib/format";
  import type { Estimate } from "$lib/types";

  let estimates = $state<Estimate[]>([]);
  let currency = $state("USD");
  let query = $state("");
  let status = $state("");
  let error = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function refresh() {
    try {
      estimates = await api.estimates.list(query, status);
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  function onSearch() {
    clearTimeout(timer);
    timer = setTimeout(() => void refresh(), 200);
  }

  onMount(async () => {
    try {
      currency = (await api.business.get()).defaultCurrency;
    } catch {
      // fall back to the default currency label
    }
    await refresh();
  });
</script>

<div class="page-head">
  <h1>Estimates</h1>
  <a class="btn" href="/estimates/new">New estimate</a>
</div>

<div class="toolbar">
  <input placeholder="Search number or client…" bind:value={query} oninput={onSearch} />
  <select bind:value={status} onchange={() => refresh()} style="width: auto;">
    <option value="">All statuses</option>
    <option value="draft">Draft</option>
    <option value="sent">Sent</option>
    <option value="accepted">Accepted</option>
    <option value="declined">Declined</option>
    <option value="expired">Expired</option>
    <option value="converted">Converted</option>
  </select>
</div>

{#if error}<p class="error">{error}</p>{/if}

<div class="card">
  {#if estimates.length === 0}
    <p class="muted">
      No estimates yet. <a class="link" href="/estimates/new">Create your first estimate</a>.
    </p>
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
