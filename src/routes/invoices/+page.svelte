<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { formatDate, fromMinor } from "$lib/format";
  import type { Invoice } from "$lib/types";

  let invoices = $state<Invoice[]>([]);
  let currency = $state("USD");
  let query = $state("");
  let status = $state("");
  let error = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function refresh() {
    try {
      invoices = await api.invoices.list(query, status);
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
  <h1>Invoices</h1>
  <a class="btn" href="/invoices/new">New invoice</a>
</div>

<div class="toolbar">
  <input placeholder="Search number or client…" bind:value={query} oninput={onSearch} />
  <select bind:value={status} onchange={() => refresh()} style="width: auto;">
    <option value="">All statuses</option>
    <option value="draft">Draft</option>
    <option value="sent">Sent</option>
    <option value="void">Void</option>
  </select>
</div>

{#if error}<p class="error">{error}</p>{/if}

<div class="card">
  {#if invoices.length === 0}
    <p class="muted">
      No invoices yet. <a class="link" href="/invoices/new">Create your first invoice</a>.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Number</th>
          <th>Client</th>
          <th>Issued</th>
          <th>Due</th>
          <th>Total</th>
          <th>Status</th>
        </tr>
      </thead>
      <tbody>
        {#each invoices as invoice (invoice.id)}
          <tr>
            <td><a class="link" href={`/invoices/${invoice.id}`}>{invoice.number}</a></td>
            <td>{invoice.clientName ?? "—"}</td>
            <td>{formatDate(invoice.issueDate)}</td>
            <td>{formatDate(invoice.dueDate)}</td>
            <td>{fromMinor(invoice.totalMinor, invoice.currency)}</td>
            <td>{invoice.status}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
