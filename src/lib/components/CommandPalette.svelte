<script lang="ts">
  import { goto } from "$app/navigation";
  import { api } from "$lib/api";
  import { t } from "$lib/i18n.svelte";

  let { open = $bindable(false) }: { open?: boolean } = $props();

  type Command = { id: string; label: string; hint: string; href: string };

  let query = $state("");
  let options = $state<Command[]>([]);
  let loading = $state(false);
  let input = $state<HTMLInputElement | null>(null);

  // Built when the palette opens (and after a language change), so the labels
  // reflect the current locale.
  function baseCommands(): Command[] {
    return [
      { id: "new-invoice", label: t("palette.newInvoice"), hint: t("palette.create"), href: "/invoices/new" },
      { id: "new-estimate", label: t("palette.newEstimate"), hint: t("palette.create"), href: "/estimates/new" },
      { id: "new-client", label: t("palette.newClient"), hint: t("palette.create"), href: "/clients/new" },
      { id: "dashboard", label: t("nav.dashboard"), hint: t("palette.goTo"), href: "/" },
      { id: "invoices", label: t("nav.invoices"), hint: t("palette.goTo"), href: "/invoices" },
      {
        id: "overdue",
        label: t("palette.overdueInvoices"),
        hint: t("palette.goTo"),
        href: "/invoices?status=overdue",
      },
      { id: "estimates", label: t("nav.estimates"), hint: t("palette.goTo"), href: "/estimates" },
      { id: "reports", label: t("nav.reports"), hint: t("palette.goTo"), href: "/reports" },
      { id: "clients", label: t("nav.clients"), hint: t("palette.goTo"), href: "/clients" },
      { id: "catalog", label: t("nav.catalog"), hint: t("palette.goTo"), href: "/catalog" },
      {
        id: "settings",
        label: t("palette.settingsBusiness"),
        hint: t("palette.settingsHint"),
        href: "/settings",
      },
      {
        id: "settings-invoicing",
        label: t("palette.settingsInvoicing"),
        hint: t("palette.settingsHint"),
        href: "/settings/invoicing",
      },
      {
        id: "settings-security",
        label: t("palette.settingsSecurity"),
        hint: t("palette.settingsHint"),
        href: "/settings/security",
      },
      {
        id: "settings-data",
        label: t("palette.settingsData"),
        hint: t("palette.settingsHint"),
        href: "/settings/data",
      },
      {
        id: "settings-support",
        label: t("palette.settingsSupport"),
        hint: t("palette.settingsHint"),
        href: "/settings/support",
      },
      {
        id: "settings-about",
        label: t("palette.settingsAbout"),
        hint: t("palette.settingsHint"),
        href: "/settings/about",
      },
    ];
  }

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
        ...baseCommands(),
        ...clients.map((client) => ({
          id: `client-${client.id}`,
          label: client.displayName,
          hint: t("palette.clientHint"),
          href: `/clients/${client.id}`,
        })),
        ...invoices.slice(0, 100).map((invoice) => ({
          id: `invoice-${invoice.id}`,
          label: invoice.clientName ?? t("palette.noClient"),
          hint: t("palette.invoiceHint", { number: invoice.number }),
          href: `/invoices/${invoice.id}`,
        })),
      ];
    } catch {
      // Navigation still works without the record commands.
      options = baseCommands();
    } finally {
      loading = false;
    }
  }

  // Focus belongs to the input while the palette is open, then goes back to
  // whatever had it before.
  let previousFocus: HTMLElement | null = null;

  $effect(() => {
    if (open) {
      previousFocus = document.activeElement as HTMLElement | null;
      query = "";
      options = baseCommands();
      void loadRecords();
      // Let the element mount before focusing it.
      queueMicrotask(() => input?.focus());
    } else if (previousFocus) {
      previousFocus.focus();
      previousFocus = null;
    }
  });

  /// Keeps Tab inside the dialog, wrapping at both ends.
  function onTrapKeydown(event: KeyboardEvent) {
    if (event.key !== "Tab") return;

    const focusable = Array.from(
      (event.currentTarget as HTMLElement).querySelectorAll<HTMLElement>(
        'input, button, [href], select, textarea, [tabindex]:not([tabindex="-1"])'
      )
    ).filter((element) => !element.hasAttribute("disabled"));

    if (focusable.length === 0) return;

    const first = focusable[0];
    const last = focusable[focusable.length - 1];

    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

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
      aria-label={t("palette.title")}
      tabindex="-1"
      onclick={(event) => event.stopPropagation()}
      onkeydown={onTrapKeydown}
    >
      <input
        bind:this={input}
        bind:value={query}
        onkeydown={onKeydown}
        aria-label={t("palette.inputLabel")}
        placeholder={t("palette.placeholder")}
        spellcheck="false"
      />

      {#if filtered.length === 0}
        <p class="none muted">
          {loading ? t("palette.searching") : t("palette.nothing")}
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
