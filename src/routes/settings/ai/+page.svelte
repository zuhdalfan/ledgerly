<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
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
      message = "Saved";
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
      saveError = "Could not reach the clipboard. Select and copy by hand.";
    }
  }
</script>

<PageHeader
  title="AI access"
  subtitle="Let a desktop AI assistant read and change your data. The connection is local and never touches the network."
/>

<AsyncState {loading} {error} rows={4} columns={2}>
  {#if info}
    {@const serverPath = info.serverPath}
    <div class="card">
      <h2>Access</h2>
      <p class="muted">
        Ledgerly ships a small companion program, <code>ledgerly-mcp</code>, that speaks the Model
        Context Protocol over standard input and output. Your AI assistant launches it; nothing is
        exposed on a port, and Ledgerly still makes no network requests.
      </p>

      <label class="toggle">
        <input
          type="checkbox"
          checked={info.enabled}
          disabled={busy}
          onchange={(e) => setSetting("mcp.enabled", e.currentTarget.checked)}
        />
        <span>
          <strong>Enable AI access</strong>
          <span class="muted">Allow an assistant to talk to Ledgerly at all.</span>
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
          <strong>Allow changes</strong>
          <span class="muted">Let the assistant create and edit records. Off means read-only.</span>
        </span>
      </label>

      {#if message}<p class="ok">{message}</p>{/if}
      {#if saveError}<p class="error">{saveError}</p>{/if}
    </div>

    <div class="card">
      <h2>Connect an assistant</h2>
      <p class="muted">Point your MCP client at this program:</p>
      <div class="copy">
        <code class="path">{serverPath}</code>
        <button class="btn secondary" onclick={() => copy(serverPath, "Path copied")}>
          Copy
        </button>
      </div>

      <h3>Claude Desktop</h3>
      <p class="muted">
        Open <strong>Settings → Developer → Edit Config</strong>, add this under <code>mcpServers</code>,
        then restart Claude Desktop.
      </p>
      <div class="copy">
        <pre>{snippet}</pre>
        <button class="btn secondary" onclick={() => copy(snippet, "Config copied")}>Copy</button>
      </div>

      <p class="muted">
        Any other client that supports local (stdio) MCP servers works the same way: give it the path
        above as the command.
      </p>
    </div>

    <div class="card">
      <h2>What the assistant can do</h2>
      {#if info.lockEnabled}
        <p class="muted">
          Your app lock is on, so the config above must include your passphrase as
          <code>LEDGERLY_MCP_PASSPHRASE</code>. Without it, every request is refused.
        </p>
      {/if}
      <p class="muted">
        <strong>Read</strong> — clients, catalog, tax rates, invoices, estimates, payments, the
        business profile, settings and all reports.
      </p>
      <p class="muted">
        <strong>Change</strong> — {info.allowWrite ? "enabled" : "disabled"}. Creating and editing
        clients, catalog items, tax rates, invoices, estimates and payments, and taking a backup.
      </p>
      <p class="muted">
        <strong>Never</strong> — restoring a backup, voiding an invoice, deleting a payment, changing
        the app lock, or exporting the database. Destructive and identity actions stay manual.
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
