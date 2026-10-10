<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { AppInfo } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let info = $state<AppInfo | null>(null);
  let loading = $state(true);
  let error = $state("");

  onMount(async () => {
    try {
      info = await api.appInfo();
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  });
</script>

<PageHeader title="About" />

<AsyncState {loading} {error} rows={3} columns={1}>
  <div class="card">
    <h2>{info?.name ?? "Ledgerly"} {info?.version ? `v${info.version}` : ""}</h2>
    <p class="muted">
      Fully local and fully offline. No account, no cloud, no telemetry, and no network code at
      all: the build fails if an HTTP client ever appears in the dependency graph.
    </p>
  </div>

  <div class="card">
    <h2>Where things are</h2>
    <p class="muted">Data directory</p>
    <code>{info?.dataDir ?? "—"}</code>
    <p class="muted spaced">Database file</p>
    <code>{info?.databasePath ?? "—"}</code>
  </div>
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-3);
  }
</style>
