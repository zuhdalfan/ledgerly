<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import CommandPalette from "$lib/components/CommandPalette.svelte";
  import Mascot from "$lib/components/Mascot.svelte";
  import type { SecurityStatus } from "$lib/types";

  let { children } = $props();

  const nav = [
    { href: "/", label: "Dashboard" },
    { href: "/invoices", label: "Invoices" },
    { href: "/estimates", label: "Estimates" },
    { href: "/reports", label: "Reports" },
    { href: "/clients", label: "Clients" },
    { href: "/catalog", label: "Catalog" },
    { href: "/settings", label: "Settings" },
  ];

  function isActive(href: string): boolean {
    return href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href);
  }

  let security = $state<SecurityStatus | null>(null);
  let gateError = $state("");
  let checking = $state(true);
  let passphrase = $state("");
  let unlocking = $state(false);
  let idleTimer: ReturnType<typeof setTimeout> | undefined;
  let paletteOpen = $state(false);
  let navOpen = $state(false);
  let narrow = $state(false);

  async function loadSecurity() {
    checking = true;
    try {
      security = await api.security.status();
      gateError = "";
      restartIdleTimer();
    } catch (e) {
      // Fail closed. If we cannot confirm whether the lock is on, we do not
      // render the data — the whole point of the setting is that it holds.
      security = null;
      gateError = String(e);
    } finally {
      checking = false;
    }
  }

  async function unlock() {
    gateError = "";
    unlocking = true;
    try {
      security = await api.security.unlock(passphrase);
      passphrase = "";
      restartIdleTimer();
    } catch (e) {
      gateError = String(e);
    } finally {
      unlocking = false;
    }
  }

  async function lockNow() {
    try {
      security = await api.security.lock();
    } catch {
      // Locking is best-effort; the Rust side still refuses gated calls.
    }
  }

  function restartIdleTimer() {
    clearTimeout(idleTimer);
    const minutes = security?.settings.autoLockMinutes ?? 0;
    if (!security?.settings.enabled || minutes <= 0) return;
    idleTimer = setTimeout(() => void lockNow(), minutes * 60_000);
  }

  function onActivity() {
    if (!security?.locked) restartIdleTimer();
  }

  function isTyping(target: EventTarget | null): boolean {
    const element = target as HTMLElement | null;
    if (!element) return false;
    const tag = element.tagName;
    return tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT" || element.isContentEditable;
  }

  async function applyMotion() {
    let level = "standard";
    try {
      level = (await api.settings.get("ui.motion")) ?? "standard";
    } catch {
      // A failed preference read must not hold up the shell.
    }
    document.documentElement.dataset.motion = level;
  }

  // The motion preference lives in the database, so it applies once it can be
  // read. The OS setting can still only lower it from there.
  $effect(() => {
    if (!security || security.locked) return;
    void applyMotion();
  });

  // A tap on a nav link must always close the drawer, including when the link
  // points at the page already open, so this cannot rely on the route changing.
  $effect(() => {
    void page.url.pathname;
    navOpen = false;
  });

  function onKeydown(event: KeyboardEvent) {
    // Nothing in the shell responds while the gate is up.
    if (!security || security.locked) return;

    // Escape closes the drawer. The command palette handles its own Escape.
    if (event.key === "Escape" && navOpen) {
      event.preventDefault();
      navOpen = false;
      return;
    }

    const mod = event.ctrlKey || event.metaKey;

    if (mod && event.key.toLowerCase() === "k") {
      event.preventDefault();
      paletteOpen = !paletteOpen;
      return;
    }

    if (mod && event.key.toLowerCase() === "n") {
      event.preventDefault();
      void goto("/invoices/new");
      return;
    }

    // The PRD's "/" search binding, and never while the user is typing.
    if (event.key === "/" && !isTyping(event.target)) {
      event.preventDefault();
      paletteOpen = true;
    }
  }

  onMount(() => {
    void loadSecurity();

    for (const event of ["mousemove", "keydown", "click", "scroll"]) {
      window.addEventListener(event, onActivity);
    }

    window.addEventListener("keydown", onKeydown);

    // The drawer only exists below the breakpoint. Track it so the closed
    // drawer can be made inert, and so widening the window closes it.
    const query = window.matchMedia("(max-width: 820px)");
    const syncNarrow = () => {
      narrow = query.matches;
      if (!narrow) navOpen = false;
    };
    syncNarrow();
    query.addEventListener("change", syncNarrow);

    return () => query.removeEventListener("change", syncNarrow);
  });
</script>

{#if checking}
  <div class="gate">
    <p class="muted">Checking the app lock…</p>
  </div>
{:else if gateError}
  <div class="gate">
    <div class="card">
      <div class="mark"><Mascot expression="worried" size={56} /></div>
      <h2>Ledgerly could not open</h2>
      <p class="muted">
        The app lock state could not be read, so your data has not been shown. Nothing is lost.
      </p>
      <p class="error">{gateError}</p>
      <div class="row wrap">
        <button class="btn" onclick={loadSecurity}>Try again</button>
      </div>
    </div>
  </div>
{:else if security?.locked}
  <div class="gate">
    <div class="card">
      <div class="mark"><Mascot expression="idle" size={56} /></div>
      <h2>Ledgerly is locked</h2>
      <div class="field">
        <label for="passphrase">Passphrase</label>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          id="passphrase"
          type="password"
          bind:value={passphrase}
          autofocus
          onkeydown={(event) => event.key === "Enter" && unlock()}
        />
      </div>
      <div class="row wrap">
        <button class="btn" onclick={unlock} disabled={unlocking || !passphrase}>
          {unlocking ? "Unlocking…" : "Unlock"}
        </button>
        {#if gateError}<span class="error">{gateError}</span>{/if}
      </div>
    </div>
  </div>
{:else}
  <div class="app" inert={paletteOpen}>
    <a class="skip" href="#main">Skip to content</a>

    <header class="topbar">
      <button
        class="menu"
        type="button"
        aria-label="Open navigation"
        aria-expanded={navOpen}
        aria-controls="sidebar"
        onclick={() => (navOpen = true)}
      >
        <span aria-hidden="true">☰</span>
      </button>
      <span class="brand">Ledgerly</span>
    </header>

    <aside id="sidebar" class:open={navOpen} inert={narrow && !navOpen}>
      <div class="brand">Ledgerly</div>
      <button class="search" onclick={() => (paletteOpen = true)}>
        <span>Search</span>
        <kbd>Ctrl K</kbd>
      </button>
      <nav>
        {#each nav as item (item.href)}
          <a href={item.href} class:active={isActive(item.href)} onclick={() => (navOpen = false)}>
            {item.label}
          </a>
        {/each}
      </nav>
    </aside>

    {#if navOpen}
      <button
        class="scrim"
        type="button"
        tabindex="-1"
        aria-label="Close navigation"
        onclick={() => (navOpen = false)}
      ></button>
    {/if}

    <main id="main" tabindex="-1">
      {@render children()}
    </main>
  </div>

  <CommandPalette bind:open={paletteOpen} />
{/if}

<style>
  .gate {
    display: flex;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: var(--space-5);
  }

  .gate .card {
    width: 100%;
    max-width: 360px;
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-2);
  }

  .mark {
    display: flex;
    justify-content: center;
    margin-bottom: var(--space-3);
  }

  .skip {
    position: absolute;
    left: var(--space-2);
    top: var(--space-2);
    z-index: 10;
    padding: var(--space-2) var(--space-3);
    background: var(--ink-1);
    color: var(--ink-inverse);
    border-radius: var(--radius-md);
    text-decoration: none;
    transform: translateY(-200%);
  }

  .skip:focus-visible {
    transform: translateY(0);
  }

  .search {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    font-size: var(--text-md);
    color: var(--ink-2);
    background: var(--paper-1);
    border: 1px solid var(--paper-3);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease-out), color var(--dur-fast) var(--ease-out);
  }

  .search:hover {
    color: var(--ink-1);
    border-color: var(--ink-3);
  }

  kbd {
    font-family: ui-monospace, "Cascadia Code", Consolas, monospace;
    font-size: var(--text-xs);
    color: var(--ink-3);
  }
</style>
