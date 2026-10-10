<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { page } from "$app/state";
  import { api } from "$lib/api";
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

  onMount(() => {
    void loadSecurity();

    for (const event of ["mousemove", "keydown", "click", "scroll"]) {
      window.addEventListener(event, onActivity);
    }
  });
</script>

{#if checking}
  <div class="gate">
    <p class="muted">Checking the app lock…</p>
  </div>
{:else if gateError}
  <div class="gate">
    <div class="card">
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
  <div class="app">
    <a class="skip" href="#main">Skip to content</a>
    <aside>
      <div class="brand">Ledgerly</div>
      <nav>
        {#each nav as item (item.href)}
          <a href={item.href} class:active={isActive(item.href)}>{item.label}</a>
        {/each}
      </nav>
    </aside>

    <main id="main" tabindex="-1">
      {@render children()}
    </main>
  </div>
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
</style>
