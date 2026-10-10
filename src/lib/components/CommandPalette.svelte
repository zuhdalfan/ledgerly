<script lang="ts">
  import { goto } from "$app/navigation";
  import { api } from "$lib/api";

  let { open = $bindable(false) }: { open?: boolean } = $props();

  type Command = { id: string; label: string; hint: string; href: string };

  let query = $state("");
  let options = $state<Command[]>([]);
  let loading = $state(false);
  let input = $state<HTMLInputElement | null>(null);

  const base: Command[] = [
    { id: "new-invoice", label: "New invoice", hint: "Create", href: "/invoices/new" },
    { id: "new-estimate", label: "New estimate", hint: "Create", href: "/estimates/new" },
    { id: "new-client", label: "New client", hint: "Create", href: "/clients/new" },
    { id: "dashboard", label: "Dashboard", hint: "Go to", href: "/" },
    { id: "invoices", label: "Invoices", hint: "Go to", href: "/invoices" },
    { id: "overdue", label: "Overdue invoices", hint: "Go to", href: "/invoices?status=overdue" },
    { id: "estimates", label: "Estimates", hint: "Go to", href: "/estimates" },
    { id: "reports", label: "Reports", hint: "Go to", href: "/reports" },
    { id: "clients", label: "Clients", hint: "Go to", href: "/clients" },
    { id: "catalog", label: "Catalog", hint: "Go to", href: "/catalog" },
    { id: "settings", label: "Settings: Business", hint: "Settings", href: "/settings" },
    {
      id: "settings-invoicing",
      label: "Settings: Invoicing",
      hint: "Settings",
      href: "/settings/invoicing",
    },
    {
      id: "settings-security",
      label: "Settings: Security",
      hint: "Settings",
      href: "/settings/security",
    },
    { id: "settings-data", label: "Settings: Data", hint: "Settings", href: "/settings/data" },
    {
      id: "settings-licence",
      label: "Settings: Licence",
      hint: "Settings",
      href: "/settings/licence",
    },
    { id: "settings-about", label: "Settings: About", hint: "Settings", href: "/settings/about" },
  ];

  const filtered = $derived.by(() => {
    const needle = query.trim().toLowerCase();
    if (!needle) return options.slice(0, 24);
    return options
      .filter((option) => `${option.label} ${option.hint}`.toLowerCase().includes(needle))
      .slice(0, 24);
  });

  // Records are only needed once, and only when the palette is actually used.
  let loadedRecords = false;

  async function loadRecords() {
    if (loadedRecords) return;
    loadedRecords = true;
    loading = true;
    try {
      const [clients, invoices] = await Promise.all([api.clients.list(), api.invoices.list()]);
      options = [
        ...base,
        ...clients.map((client) => ({
          id: `client-${client.id}`,
          label: client.displayName,
          hint: "Client",
          href: `/clients/${client.id}`,
        })),
        ...invoices.slice(0, 100).map((invoice) => ({
          id: `invoice-${invoice.id}`,
          label: invoice.clientName ?? "No client",
          hint: `Invoice ${invoice.number}`,
          href: `/invoices/${invoice.id}`,
        })),
      ];
    } catch {
      // Navigation still works without the record commands.
      options = base;
    } finally {
      loading = false;
    }
  }

  $effect(() => {
    if (open) {
      query = "";
      options = base;
      void loadRecords();
      // Let the element mount before focusing it.
      queueMicrotask(() => input?.focus());
    }
  });

  async function choose(command: Command | undefined) {
    if (!command) return;
    open = false;
    await goto(command.href);
  }

  function onKeydown(event: KeyboardEvent) {
    if (event.key === "Escape") {
      event.preventDefault();
      open = false;
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      void choose(filtered[0]);
      return;
    }

    if (event.key === "ArrowDown" && filtered.length > 0) {
      event.preventDefault();
      const first = document.querySelector<HTMLButtonElement>("[data-palette-option]");
      first?.focus();
    }
  }

  function onOptionKeydown(event: KeyboardEvent, command: Command) {
    if (event.key === "Escape") {
      event.preventDefault();
      open = false;
      return;
    }

    if (event.key === "ArrowDown" || event.key === "ArrowUp") {
      event.preventDefault();
      const buttons = Array.from(
        document.querySelectorAll<HTMLButtonElement>("[data-palette-option]")
      );
      const at = buttons.indexOf(event.currentTarget as HTMLButtonElement);
      const next = event.key === "ArrowDown" ? at + 1 : at - 1;
      buttons[next]?.focus();
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      void choose(command);
    }
  }
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
  <div class="scrim" role="presentation" onclick={() => (open = false)}>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
    <div
      class="palette"
      role="dialog"
      aria-modal="true"
      aria-label="Command palette"
      tabindex="-1"
      onclick={(event) => event.stopPropagation()}
    >
      <input
        bind:this={input}
        bind:value={query}
        onkeydown={onKeydown}
        aria-label="Search commands, clients and invoices"
        placeholder="Search, or jump to…"
        spellcheck="false"
      />

      {#if filtered.length === 0}
        <p class="none muted">
          {loading ? "Searching…" : "Nothing matches that."}
        </p>
      {:else}
        <ul>
          {#each filtered as command, position (command.id)}
            <li>
              <button
                data-palette-option
                class:first={position === 0}
                onclick={() => choose(command)}
                onkeydown={(event) => onOptionKeydown(event, command)}
              >
                <span class="label">{command.label}</span>
                <span class="hint muted">{command.hint}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 40;
    display: flex;
    justify-content: center;
    align-items: flex-start;
    padding: 12vh var(--space-4) var(--space-4);
    background: oklch(0.24 0.014 70 / 0.35);
  }

  .palette {
    width: 100%;
    max-width: 34rem;
    overflow: hidden;
    background: var(--paper-0);
    border: 1px solid var(--paper-4);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-2);
  }

  .palette input {
    border: 0;
    border-bottom: 1px solid var(--paper-3);
    border-radius: 0;
    padding: var(--space-4);
    font-size: var(--text-lg);
  }

  .palette input:focus-visible {
    outline: none;
    border-bottom-color: var(--accent-text);
  }

  ul {
    margin: 0;
    padding: var(--space-1);
    list-style: none;
    max-height: 22rem;
    overflow-y: auto;
  }

  li button {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    text-align: left;
    color: var(--ink-1);
    background: transparent;
    border: 0;
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  li button:hover {
    background: var(--paper-2);
  }

  li button:focus-visible {
    outline: 2px solid var(--accent-text);
    outline-offset: -2px;
  }

  .first {
    background: var(--paper-2);
  }

  .hint {
    font-size: var(--text-sm);
  }

  .none {
    padding: var(--space-4);
  }
</style>
