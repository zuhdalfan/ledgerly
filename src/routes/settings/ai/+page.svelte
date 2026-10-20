<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { t } from "$lib/i18n.svelte";
  import type { McpInfo } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let info = $state<McpInfo | null>(null);
  let loading = $state(true);
  let error = $state("");
  let message = $state("");
  let saveError = $state("");
  let busy = $state(false);

  // The exact JSON an MCP client needs. Built from the real server path so it
  // can be copied straight into Claude Desktop's config file.
  const snippet = $derived.by(() => {
    if (!info) return "";
    const server: Record<string, unknown> = { command: info.serverPath };
    if (info.lockEnabled) {
      server.env = { LEDGERLY_MCP_PASSPHRASE: "your passphrase" };
    }
    return JSON.stringify({ mcpServers: { ledgerly: server } }, null, 2);
  });

  async function load() {
    loading = true;
    try {
      info = await api.mcp.info();
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

  async function setSetting(key: "mcp.enabled" | "mcp.allowWrite", value: boolean) {
    message = "";
    saveError = "";
    busy = true;
    try {
      await api.settings.set(key, value ? "true" : "false");
      info = await api.mcp.info();
      message = t("settings.ai.saved");
    } catch (e) {
      saveError = String(e);
    } finally {
      busy = false;
    }
  }

  async function copy(text: string, confirmation: string) {
    message = "";
    saveError = "";
    try {
      await navigator.clipboard.writeText(text);
      message = confirmation;
    } catch {
      saveError = t("settings.ai.clipboardError");
    }
  }
</script>

<PageHeader title={t("settings.ai.title")} subtitle={t("settings.ai.subtitle")} />

<AsyncState {loading} {error} rows={4} columns={2}>
  {#if info}
    {@const serverPath = info.serverPath}
    <div class="card">
      <h2>{t("settings.ai.access")}</h2>
      <p class="muted">
        {t("settings.ai.accessBodyStart")}
        <code>ledgerly-mcp</code>
        {t("settings.ai.accessBodyEnd")}
      </p>

      <label class="toggle">
        <input
          type="checkbox"
          checked={info.enabled}
          disabled={busy}
          onchange={(e) => setSetting("mcp.enabled", e.currentTarget.checked)}
        />
        <span>
          <strong>{t("settings.ai.enable")}</strong>
          <span class="muted">{t("settings.ai.enableHint")}</span>
        </span>
      </label>

      <label class="toggle">
        <input
          type="checkbox"
          checked={info.allowWrite}
          disabled={busy || !info.enabled}
          onchange={(e) => setSetting("mcp.allowWrite", e.currentTarget.checked)}
        />
        <span>
          <strong>{t("settings.ai.allowChanges")}</strong>
          <span class="muted">{t("settings.ai.allowChangesHint")}</span>
        </span>
      </label>

      {#if message}<p class="ok">{message}</p>{/if}
      {#if saveError}<p class="error">{saveError}</p>{/if}
    </div>

    <div class="card">
      <h2>{t("settings.ai.connect")}</h2>
      <p class="muted">{t("settings.ai.pointClient")}</p>
      <div class="copy">
        <code class="path">{serverPath}</code>
        <button class="btn secondary" onclick={() => copy(serverPath, t("settings.ai.pathCopied"))}>
          {t("action.copy")}
        </button>
      </div>

      <h3>{t("settings.ai.claudeDesktop")}</h3>
      <p class="muted">
        {t("settings.ai.claudeOpen")}
        <strong>{t("settings.ai.claudeSteps")}</strong>{t("settings.ai.claudeAdd")}
        <code>mcpServers</code>{t("settings.ai.claudeRestart")}
      </p>
      <div class="copy">
        <pre>{snippet}</pre>
        <button class="btn secondary" onclick={() => copy(snippet, t("settings.ai.configCopied"))}>
          {t("action.copy")}
        </button>
      </div>

      <p class="muted">{t("settings.ai.otherClient")}</p>
    </div>

    <div class="card">
      <h2>{t("settings.ai.canDo")}</h2>
      {#if info.lockEnabled}
        <p class="muted">
          {t("settings.ai.lockBodyStart")}
          <code>LEDGERLY_MCP_PASSPHRASE</code>{t("settings.ai.lockBodyEnd")}
        </p>
      {/if}
      <p class="muted">
        <strong>{t("settings.ai.read")}</strong>
        — {t("settings.ai.readBody")}
      </p>
      <p class="muted">
        <strong>{t("settings.ai.change")}</strong>
        — {info.allowWrite ? t("settings.ai.enabled") : t("settings.ai.disabled")}.
        {t("settings.ai.changeBody")}
      </p>
      <p class="muted">
        <strong>{t("settings.ai.never")}</strong>
        — {t("settings.ai.neverBody")}
      </p>
    </div>
  {/if}
</AsyncState>

<style>
  .toggle {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    padding: var(--space-2) 0;
    cursor: pointer;
  }

  .toggle input {
    margin-top: 3px;
  }

  .toggle span {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .toggle .muted {
    font-size: var(--text-sm);
  }

  .copy {
    display: flex;
    gap: var(--space-2);
    align-items: flex-start;
    margin: var(--space-3) 0;
  }

  .copy code,
  .copy pre {
    flex: 1 1 auto;
    min-width: 0;
    margin: 0;
    padding: var(--space-3);
    background: var(--paper-2);
    border: 1px solid var(--paper-3);
    border-radius: var(--radius-md);
    overflow-x: auto;
  }

  .copy pre {
    white-space: pre;
  }

  .path {
    word-break: break-all;
  }

  h3 {
    margin: var(--space-5) 0 var(--space-2);
    font-size: var(--text-md);
  }
</style>
