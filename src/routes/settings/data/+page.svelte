<script lang="ts">
  import { onMount } from "svelte";
  import { ask, message, open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { localDate } from "$lib/format";
  import type { AppInfo, BackupInfo, BackupSettings } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let info = $state<AppInfo | null>(null);
  let backupSettings = $state<BackupSettings>({
    enabled: false,
    folder: "",
    keep: 7,
    lastRun: null,
  });
  let backups = $state<BackupInfo[]>([]);
  let notice = $state("");
  let error = $state("");
  let busy = $state(false);
  let loading = $state(true);

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatWhen(epoch: number | null): string {
    return epoch ? new Date(epoch * 1000).toLocaleString() : "—";
  }

  async function load() {
    loading = true;
    try {
      const [appInfo, settings, rows] = await Promise.all([
        api.appInfo(),
        api.backup.getSettings(),
        api.backup.list(),
      ]);
      info = appInfo;
      backupSettings = settings;
      backups = rows;
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void load();
  });

  async function backupNow() {
    notice = "";
    error = "";
    try {
      const path = await saveDialog({
        defaultPath: `ledgerly-${localDate()}.ledgerly`,
        filters: [{ name: "Ledgerly backup", extensions: ["ledgerly"] }],
      });
      if (!path) return;

      busy = true;
      notice = `Backup saved to ${await api.backup.now(path)}`;
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function restore(path: string) {
    notice = "";
    error = "";

    const confirmed = await ask(
      "Replace all current data with this backup?\n\nA safety copy of your current data is saved first.",
      { title: "Restore backup", kind: "warning" }
    );
    if (!confirmed) return;

    busy = true;
    try {
      const safety = await api.backup.restore(path);
      await message(`Backup restored.\n\nYour previous data was saved to:\n${safety}`, {
        title: "Restore complete",
      });
      window.location.reload();
    } catch (e) {
      error = String(e);
      busy = false;
    }
  }

  async function chooseBackupFile() {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: "Ledgerly backup", extensions: ["ledgerly"] }],
    });
    if (typeof selected === "string") await restore(selected);
  }

  async function saveBackupSettings() {
    notice = "";
    error = "";
    try {
      backupSettings = await api.backup.saveSettings({
        enabled: backupSettings.enabled,
        folder: backupSettings.folder,
        keep: backupSettings.keep,
      });
      notice = "Backup settings saved";
      await load();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<PageHeader
  title="Data"
  subtitle="One folder, on this machine. Back it up somewhere else and you can never lose it."
/>

<AsyncState {loading} {error} rows={5} columns={1}>
  <div class="card">
    <h2>Where your data lives</h2>
    <code>{info?.dataDir ?? "—"}</code>
    <div class="row wrap spaced">
      <button class="btn secondary" onclick={() => info && api.openInViewer(info.dataDir)}>
        Open folder
      </button>
      <button class="btn" onclick={backupNow} disabled={busy}>Back up now</button>
      <button class="btn secondary" onclick={chooseBackupFile} disabled={busy}>
        Restore from file
      </button>
    </div>
  </div>

  <div class="card">
    <h2>Automatic backups</h2>
    <p class="muted">
      When enabled, a backup is written at most once a day, and older ones are rotated away.
    </p>
    <div class="grid-2">
      <div class="field">
        <span class="field-label">Schedule</span>
        <label class="row muted">
          <input id="autoBackup" type="checkbox" bind:checked={backupSettings.enabled} />
          Back up once a day on start
        </label>
      </div>
      <div class="field">
        <label for="keep">Keep the newest</label>
        <input id="keep" type="number" min="1" max="365" bind:value={backupSettings.keep} />
      </div>
    </div>
    <div class="field">
      <label for="backupFolder">Backup folder</label>
      <input id="backupFolder" bind:value={backupSettings.folder} />
    </div>

    <div class="row wrap">
      <button class="btn" onclick={saveBackupSettings}>Save backup settings</button>
      {#if backupSettings.lastRun}
        <span class="muted">Last automatic run {backupSettings.lastRun}</span>
      {/if}
      {#if notice}<span class="ok">{notice}</span>{/if}
    </div>
  </div>

  <div class="card">
    <h2>Recent backups</h2>
    {#if backups.length === 0}
      <EmptyState
        title="No backups yet"
        body={`Nothing found in ${backupSettings.folder || "the backup folder"}.`}
      >
        <button class="btn" onclick={backupNow} disabled={busy}>Back up now</button>
      </EmptyState>
    {:else}
      <table>
        <thead>
          <tr>
            <th>File</th>
            <th class="numeric">Size</th>
            <th>Created</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each backups as backup (backup.path)}
            <tr>
              <td>{backup.name}</td>
              <td class="numeric">{formatSize(backup.sizeBytes)}</td>
              <td>{formatWhen(backup.modifiedAt)}</td>
              <td>
                <button class="btn secondary" onclick={() => restore(backup.path)} disabled={busy}>
                  Restore
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-3);
  }

  .field-label {
    font-size: var(--text-sm);
    font-weight: 700;
    color: var(--ink-2);
  }
</style>
