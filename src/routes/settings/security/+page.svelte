<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import type { SecurityStatus } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let security = $state<SecurityStatus | null>(null);
  let lockPassphrase = $state("");
  let lockConfirm = $state("");
  let lockCurrent = $state("");
  let error = $state("");
  let message = $state("");
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      security = await api.security.status();
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

  async function enableLock() {
    error = "";
    message = "";
    if (lockPassphrase !== lockConfirm) {
      error = "The two passphrases do not match.";
      return;
    }

    try {
      security = await api.security.enable(
        lockPassphrase,
        security?.settings.autoLockMinutes ?? 15
      );
      lockPassphrase = "";
      lockConfirm = "";
      message = "App lock enabled. It will be asked for on next start.";
    } catch (e) {
      error = String(e);
    }
  }

  async function disableLock() {
    error = "";
    message = "";
    try {
      security = await api.security.disable(lockCurrent);
      lockCurrent = "";
      lockPassphrase = "";
      lockConfirm = "";
      message = "App lock turned off.";
    } catch (e) {
      error = String(e);
    }
  }

  async function saveAutoLock(minutes: number) {
    error = "";
    try {
      security = await api.security.setAutoLock(minutes);
    } catch (e) {
      error = String(e);
    }
  }

  async function lockNow() {
    error = "";
    try {
      await api.security.lock();
      // The shell owns the lock state, so reload to show the lock screen.
      window.location.reload();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<PageHeader title="Security" subtitle="The app lock is always free, never a Pro feature." />

<AsyncState {loading} {error} rows={4} columns={1}>
  <div class="card">
    {#if security?.settings.enabled}
      <h2>App lock is on</h2>
      <p class="muted">
        Ledgerly asks for your passphrase at start, and again after
        {security.settings.autoLockMinutes} minutes of inactivity.
      </p>

      <div class="field">
        <label for="autoLock">Lock after (minutes of inactivity)</label>
        <input
          id="autoLock"
          type="number"
          min="1"
          max="240"
          value={security.settings.autoLockMinutes}
          onchange={(e) => saveAutoLock(Number(e.currentTarget.value))}
        />
      </div>

      <div class="row wrap">
        <button class="btn secondary" onclick={lockNow}>Lock now</button>
      </div>

      <h2 class="spaced">Turn the lock off</h2>
      <div class="field">
        <label for="lockCurrent">Current passphrase</label>
        <input id="lockCurrent" type="password" bind:value={lockCurrent} />
      </div>
      <div class="row wrap">
        <button class="btn danger" onclick={disableLock} disabled={!lockCurrent}>Turn off</button>
      </div>
    {:else}
      <h2>App lock</h2>
      <p class="muted">
        Optional: require a passphrase to open Ledgerly, and re-lock automatically when you step
        away. The passphrase is never stored, only a slow Argon2id verifier, so it cannot be read
        back.
      </p>

      <div class="grid-2">
        <div class="field">
          <label for="lockPassphrase">Passphrase</label>
          <input id="lockPassphrase" type="password" bind:value={lockPassphrase} />
        </div>
        <div class="field">
          <label for="lockConfirm">Confirm passphrase</label>
          <input id="lockConfirm" type="password" bind:value={lockConfirm} />
        </div>
      </div>
      <p class="muted">
        At least {security?.minPassphraseLength ?? 8} characters. There is no recovery, so keep a
        backup if you might forget it.
      </p>
      <div class="row wrap">
        <button class="btn" onclick={enableLock} disabled={!lockPassphrase || !lockConfirm}>
          Enable app lock
        </button>
      </div>
    {/if}

    {#if message}<p class="ok">{message}</p>{/if}
    {#if error}<p class="error">{error}</p>{/if}
  </div>
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-5);
  }
</style>
