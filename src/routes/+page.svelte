<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  type AppInfo = {
    name: string;
    version: string;
    dataDir: string;
    databasePath: string;
  };

  let info = $state<AppInfo | null>(null);
  let businessName = $state("");
  let status = $state("");

  async function load() {
    info = await invoke<AppInfo>("app_info");
    businessName =
      (await invoke<string | null>("settings_get", { key: "business.name" })) ?? "";
  }

  async function save() {
    await invoke("settings_set", { key: "business.name", value: businessName });
    status = "Saved";
    setTimeout(() => (status = ""), 1500);
  }

  $effect(() => {
    void load();
  });
</script>

<main>
  <header>
    <h1>{info?.name ?? "Ledgerly"}</h1>
    <span class="version">v{info?.version ?? "…"}</span>
  </header>

  <section>
    <h2>Business profile</h2>
    <label for="business-name">Business name</label>
    <div class="row">
      <input id="business-name" bind:value={businessName} placeholder="Your business" />
      <button onclick={save}>Save</button>
      {#if status}<span class="status">{status}</span>{/if}
    </div>
  </section>

  <footer>
    <p>Data directory</p>
    <code>{info?.dataDir ?? "…"}</code>
    <p>Database</p>
    <code>{info?.databasePath ?? "…"}</code>
  </footer>
</main>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 15px;
    line-height: 1.5;
    color: #1a1a1a;
    background-color: #f6f6f7;
    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  main {
    max-width: 720px;
    margin: 0 auto;
    padding: 2.5rem 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }

  header {
    display: flex;
    align-items: baseline;
    gap: 0.6rem;
  }

  h1 {
    margin: 0;
    font-size: 1.5rem;
  }

  h2 {
    margin: 0 0 0.75rem;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: #6b6b70;
  }

  .version {
    color: #8a8a90;
    font-size: 0.85rem;
  }

  section {
    background: #ffffff;
    border: 1px solid #e4e4e7;
    border-radius: 10px;
    padding: 1.25rem;
  }

  label {
    display: block;
    margin-bottom: 0.35rem;
    font-size: 0.85rem;
    color: #4b4b52;
  }

  .row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  input {
    flex: 1;
    padding: 0.55rem 0.7rem;
    border: 1px solid #d4d4d8;
    border-radius: 7px;
    font: inherit;
    color: inherit;
    background: #fff;
  }

  input:focus-visible,
  button:focus-visible {
    outline: 2px solid #4f46e5;
    outline-offset: 1px;
  }

  button {
    padding: 0.55rem 1rem;
    border: 1px solid transparent;
    border-radius: 7px;
    background: #4f46e5;
    color: #fff;
    font: inherit;
    font-weight: 500;
    cursor: pointer;
  }

  button:hover {
    background: #4338ca;
  }

  .status {
    color: #15803d;
    font-size: 0.85rem;
  }

  footer {
    color: #6b6b70;
    font-size: 0.85rem;
  }

  footer p {
    margin: 0.75rem 0 0.2rem;
  }

  code {
    display: block;
    padding: 0.5rem 0.65rem;
    background: #f0f0f2;
    border-radius: 6px;
    font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
    font-size: 0.8rem;
    word-break: break-all;
    color: #3f3f46;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #e5e5e7;
      background-color: #1c1c1f;
    }

    section {
      background: #26262a;
      border-color: #35353a;
    }

    h2,
    footer {
      color: #9a9aa2;
    }

    input {
      background: #1c1c1f;
      border-color: #3f3f46;
      color: inherit;
    }

    code {
      background: #1c1c1f;
      color: #d4d4d8;
    }
  }
</style>
