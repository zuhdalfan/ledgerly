<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { AppInfo, Business } from "$lib/types";

  let info = $state<AppInfo | null>(null);
  let business = $state<Business | null>(null);
  let clientCount = $state(0);
  let catalogCount = $state(0);
  let error = $state("");

  onMount(async () => {
    try {
      const [appInfo, biz, clients, catalog] = await Promise.all([
        api.appInfo(),
        api.business.get(),
        api.clients.list(),
        api.catalog.list(),
      ]);
      info = appInfo;
      business = biz;
      clientCount = clients.length;
      catalogCount = catalog.length;
    } catch (e) {
      error = String(e);
    }
  });
</script>

<div class="page-head">
  <h1>Dashboard</h1>
</div>

{#if error}
  <p class="error">{error}</p>
{/if}

<div class="grid-2">
  <div class="card">
    <h2>Clients</h2>
    <p class="stat">{clientCount}</p>
    <a class="link" href="/clients">Manage clients</a>
  </div>

  <div class="card">
    <h2>Catalog items</h2>
    <p class="stat">{catalogCount}</p>
    <a class="link" href="/catalog">Manage catalog</a>
  </div>

  <div class="card">
    <h2>Business</h2>
    <p>{business?.displayName || "Not set"}</p>
    <p class="muted">Default currency: {business?.defaultCurrency ?? "—"}</p>
    <a class="link" href="/settings">Business settings</a>
  </div>
</div>

<div class="card">
  <h2>Local data</h2>
  <p class="muted">
    Everything is stored on this machine. No account, no cloud, no network.
  </p>
  <p class="muted">Data directory</p>
  <code>{info?.dataDir ?? "…"}</code>
  <p class="muted">Database</p>
  <code>{info?.databasePath ?? "…"}</code>
</div>
