<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import type { Client } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import SearchField from "$lib/components/SearchField.svelte";
  import { capped, RENDER_LIMIT } from "$lib/list";
  import { t } from "$lib/i18n.svelte";

  let clients = $state<Client[]>([]);
  let loading = $state(true);
  let error = $state("");
  let search = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  const query = $derived(page.url.searchParams.get("q") ?? "");
  const includeArchived = $derived(page.url.searchParams.get("archived") === "1");
  const visible = $derived(capped(clients));

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

<PageHeader title={t("clients.title")}>
  <a class="btn" href="/clients/new">{t("clients.new")}</a>
</PageHeader>

<div class="toolbar">
  <SearchField
    bind:value={search}
    label={t("clients.searchLabel")}
    example={t("clients.searchExample")}
    oninput={onSearch}
  />
  <label class="row muted archived">
    <input
      type="checkbox"
      checked={includeArchived}
      onchange={(e) => setParam("archived", e.currentTarget.checked ? "1" : "")}
    />
    {t("common.showArchived")}
  </label>
</div>

<AsyncState {loading} {error} rows={6} columns={4}>
  <div class="card">
    {#if clients.length === 0}
      <EmptyState title={t("clients.empty")} body={t("clients.emptyBody")}>
        <a class="btn" href="/clients/new">{t("action.addYourFirstClient")}</a>
      </EmptyState>
    {:else}
      <table>
        <thead>
          <tr>
            <th>{t("common.name")}</th>
            <th>{t("common.company")}</th>
            <th>{t("common.email")}</th>
            <th>{t("common.currency")}</th>
          </tr>
        </thead>
        <tbody>
          {#each visible.shown as client (client.id)}
            <tr>
              <td>
                <a class="link" href={`/clients/${client.id}`}>{client.displayName}</a>
                {#if client.archivedAt}<span class="muted"> {t("common.archived")}</span>{/if}
              </td>
              <td>{client.company ?? t("common.emDash")}</td>
              <td>{client.email ?? t("common.emDash")}</td>
              <td>{client.currency ?? t("common.emDash")}</td>
            </tr>
          {/each}
        </tbody>
      </table>
      {#if visible.hidden > 0}
        <p class="muted truncation">
          {t("list.showingFirstSearch", { limit: RENDER_LIMIT, total: clients.length })}
        </p>
      {/if}
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

  .truncation {
    margin-top: var(--space-3);
  }
</style>
