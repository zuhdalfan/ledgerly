<script lang="ts">
  import { onMount } from "svelte";
  import { ask, message, open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { localDate } from "$lib/format";
  import { getLocale, t } from "$lib/i18n.svelte";
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
    return epoch ? new Date(epoch * 1000).toLocaleString(getLocale()) : "—";
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
        filters: [{ name: t("settings.data.backupFilter"), extensions: ["ledgerly"] }],
      });
      if (!path) return;

      busy = true;
      notice = t("settings.data.noticeSaved", { path: await api.backup.now(path) });
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

    const confirmed = await ask(t("settings.data.confirmRestore"), {
      title: t("settings.data.confirmRestoreTitle"),
      kind: "warning",
    });
    if (!confirmed) return;

    busy = true;
    try {
      const safety = await api.backup.restore(path);
      await message(t("settings.data.restored", { path: safety }), {
        title: t("settings.data.restoredTitle"),
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
      filters: [{ name: t("settings.data.backupFilter"), extensions: ["ledgerly"] }],
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
      notice = t("settings.data.noticeSettingsSaved");
      await load();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<PageHeader
  title={t("settings.nav.data")}
  subtitle={t("settings.data.subtitle")}
/>

<AsyncState {loading} {error} rows={5} columns={1}>
  <div class="card">
    <h2>{t("settings.data.whereTitle")}</h2>
    <code>{info?.dataDir ?? "—"}</code>
    <div class="row wrap spaced">
      <button class="btn secondary" onclick={() => info && api.openInViewer(info.dataDir)}>
        {t("action.openFolder")}
      </button>
      <button class="btn" onclick={backupNow} disabled={busy}>{t("action.backUpNow")}</button>
      <button class="btn secondary" onclick={chooseBackupFile} disabled={busy}>
        {t("action.restoreFromFile")}
      </button>
    </div>
  </div>

  <div class="card">
    <h2>{t("settings.data.automatic")}</h2>
    <p class="muted">{t("settings.data.automaticBody")}</p>
    <div class="grid-2">
      <div class="field">
        <span class="field-label">{t("settings.data.schedule")}</span>
        <label class="row muted">
          <input id="autoBackup" type="checkbox" bind:checked={backupSettings.enabled} />
          {t("settings.data.scheduleToggle")}
        </label>
      </div>
      <div class="field">
        <label for="keep">{t("settings.data.keepNewest")}</label>
        <input id="keep" type="number" min="1" max="365" bind:value={backupSettings.keep} />
      </div>
    </div>
    <div class="field">
      <label for="backupFolder">{t("settings.data.folder")}</label>
      <input id="backupFolder" bind:value={backupSettings.folder} />
    </div>

    <div class="row wrap">
      <button class="btn" onclick={saveBackupSettings}>{t("action.saveBackupSettings")}</button>
      {#if backupSettings.lastRun}
        <span class="muted">{t("settings.data.lastRun", { when: backupSettings.lastRun })}</span>
      {/if}
      {#if notice}<span class="ok">{notice}</span>{/if}
    </div>
  </div>

  <div class="card">
    <h2>{t("settings.data.recent")}</h2>
    {#if backups.length === 0}
      <EmptyState
        title={t("settings.data.empty")}
        body={t("settings.data.emptyBody", {
          folder: backupSettings.folder || t("settings.data.theBackupFolder"),
        })}
      >
        <button class="btn" onclick={backupNow} disabled={busy}>{t("action.backUpNow")}</button>
      </EmptyState>
    {:else}
      <table>
        <thead>
          <tr>
            <th>{t("settings.data.file")}</th>
            <th class="numeric">{t("settings.data.size")}</th>
            <th>{t("settings.data.created")}</th>
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
                  {t("action.restore")}
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
