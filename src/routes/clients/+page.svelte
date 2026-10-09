<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { Client } from "$lib/types";

  let clients = $state<Client[]>([]);
  let query = $state("");
  let includeArchived = $state(false);
  let error = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  async function refresh() {
    try {
      clients = await api.clients.list(query, includeArchived);
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  function onSearch() {
    clearTimeout(timer);
    timer = setTimeout(() => void refresh(), 200);
  }

  onMount(() => {
    void refresh();
  });
</script>

<div class="page-head">
  <h1>Clients</h1>
  <a class="btn" href="/clients/new">New client</a>
</div>

<div class="toolbar">
  <input placeholder="Search name, company or email…" bind:value={query} oninput={onSearch} />
  <label class="row muted" style="white-space: nowrap;">
    <input
      type="checkbox"
      bind:checked={includeArchived}
      onchange={() => refresh()}
      style="width: auto;"
    />
    Show archived
  </label>
</div>

{#if error}<p class="error">{error}</p>{/if}

<div class="card">
  {#if clients.length === 0}
    <p class="muted">
      No clients yet. <a class="link" href="/clients/new">Add your first client</a>.
    </p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Company</th>
          <th>Email</th>
          <th>Currency</th>
        </tr>
      </thead>
      <tbody>
        {#each clients as client (client.id)}
          <tr>
            <td>
              <a class="link" href={`/clients/${client.id}`}>{client.displayName}</a>
              {#if client.archivedAt}<span class="muted"> (archived)</span>{/if}
            </td>
            <td>{client.company ?? "—"}</td>
            <td>{client.email ?? "—"}</td>
            <td>{client.currency ?? "—"}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
