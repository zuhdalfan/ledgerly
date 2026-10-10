<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import type { Client } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import SearchField from "$lib/components/SearchField.svelte";

  let clients = $state<Client[]>([]);
  let loading = $state(true);
  let error = $state("");
  let search = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  const query = $derived(page.url.searchParams.get("q") ?? "");
  const includeArchived = $derived(page.url.searchParams.get("archived") === "1");

  async function load() {
    loading = true;
    try {
      clients = await api.clients.list(query, includeArchived);
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
    const key = `${query}|${includeArchived}`;
    if (key === applied) return;
    applied = key;
    search = query;
    void load();
  });
</script>

<PageHeader title="Clients">
  <a class="btn" href="/clients/new">New client</a>
</PageHeader>

<div class="toolbar">
  <SearchField
    bind:value={search}
    label="Search clients by name, company or email"
    example="Name, company or email"
    oninput={onSearch}
  />
  <label class="row muted archived">
    <input
      type="checkbox"
      checked={includeArchived}
      onchange={(e) => setParam("archived", e.currentTarget.checked ? "1" : "")}
    />
    Show archived
  </label>
</div>

<AsyncState {loading} {error} rows={6} columns={4}>
  <div class="card">
    {#if clients.length === 0}
      <EmptyState
        title="No clients yet"
        body="A client is who you bill. Add one, and their details fill themselves into every invoice."
      >
        <a class="btn" href="/clients/new">Add your first client</a>
      </EmptyState>
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
</AsyncState>

<style>
  .archived {
    white-space: nowrap;
  }

  .archived input {
    width: auto;
  }
</style>
