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
  let passphrase = $state("");
  let unlockError = $state("");
  let unlocking = $state(false);
  let idleTimer: ReturnType<typeof setTimeout> | undefined;

  async function loadSecurity() {
    try {
      security = await api.security.status();
    } catch (e) {
      unlockError = String(e);
    }
  }

  async function unlock() {
    unlockError = "";
    unlocking = true;
    try {
      security = await api.security.unlock(passphrase);
      passphrase = "";
      restartIdleTimer();
    } catch (e) {
      unlockError = String(e);
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

  // Re-arms the idle timer on any activity; the lock itself is enforced in Rust.
  function restartIdleTimer() {
    clearTimeout(idleTimer);
    const minutes = security?.settings.autoLockMinutes ?? 0;
    if (!security?.settings.enabled || minutes <= 0) return;
    idleTimer = setTimeout(() => void lockNow(), minutes * 60_000);
  }

  function onActivity() {
    if (!security?.locked) restartIdleTimer();
  }

  onMount(async () => {
    await loadSecurity();
    restartIdleTimer();

    for (const event of ["mousemove", "keydown", "click", "scroll"]) {
      window.addEventListener(event, onActivity);
    }
  });
</script>

{#if security?.locked}
  <div class="lock-screen">
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
        {#if unlockError}<span class="error">{unlockError}</span>{/if}
      </div>
    </div>
  </div>
{:else}
  <div class="app">
    <aside>
      <div class="brand">Ledgerly</div>
      <nav>
        {#each nav as item (item.href)}
          <a href={item.href} class:active={isActive(item.href)}>{item.label}</a>
        {/each}
      </nav>
    </aside>

    <main>
      {@render children()}
    </main>
  </div>
{/if}
