<script lang="ts">
  import { onMount } from "svelte";
  import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import type { LicenseStatus } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let license = $state<LicenseStatus | null>(null);
  let requestCode = $state("");
  let upgradeKey = $state("");
  let notice = $state("");
  let error = $state("");
  let busy = $state(false);
  let loading = $state(true);

  async function load() {
    loading = true;
    try {
      const [status, code] = await Promise.all([api.license.status(), api.license.requestCode()]);
      license = status;
      requestCode = code;
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

  async function copyRequestCode() {
    notice = "";
    try {
      await navigator.clipboard.writeText(requestCode);
      notice = "Request code copied";
    } catch {
      notice = "Select the code above and copy it manually.";
    }
  }

  async function saveRequestCode() {
    notice = "";
    error = "";
    try {
      const path = await saveDialog({
        defaultPath: "ledgerly-request-code.txt",
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
      if (!path) return;
      await api.license.saveRequestFile(path);
      notice = `Request code saved to ${path}`;
    } catch (e) {
      error = String(e);
    }
  }

  async function activateKey() {
    notice = "";
    error = "";
    if (!upgradeKey.trim()) return;

    busy = true;
    try {
      license = await api.license.importKey(upgradeKey.trim());
      upgradeKey = "";
      notice = "Licence activated.";
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function loadLicenceFile() {
    notice = "";
    error = "";
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{ name: "Ledgerly licence", extensions: ["lic", "json"] }],
      });
      if (typeof selected !== "string") return;

      busy = true;
      license = await api.license.importFile(selected);
      notice = "Licence activated.";
    } catch (e) {
      error = String(e);
    } finally {
      busy = false;
    }
  }

  async function startTrial() {
    notice = "";
    error = "";
    try {
      license = await api.license.startTrial();
      notice = `Pro trial started: ${license.daysRemaining} days.`;
    } catch (e) {
      error = String(e);
    }
  }

  async function deactivate() {
    notice = "";
    error = "";
    try {
      license = await api.license.deactivate();
      notice = "Licence removed from this machine.";
    } catch (e) {
      error = String(e);
    }
  }

  const badge = $derived(
    !license ? "draft" : license.licensed ? "paid" : "draft"
  );
  const badgeLabel = $derived(
    !license ? "Free" : license.source === "trial" ? "Pro trial" : license.licensed ? "Pro" : "Free"
  );
</script>

<PageHeader title="Licence" subtitle="Activation is entirely offline. This app sends nothing anywhere.">
  <span class={`badge ${badge}`}>{badgeLabel}</span>
</PageHeader>

<AsyncState {loading} {error} rows={5} columns={2}>
  {#if license}
    <div class="card">
      <h2>Your plan</h2>
      {#if license.licensed}
        <p>
          {#if license.source === "trial"}
            Pro trial, {license.daysRemaining} day(s) left, until {license.expiresAt}.
          {:else if license.plan === "perpetual"}
            Pro licence, perpetual.
          {:else}
            Pro licence ({license.plan}), {license.daysRemaining} day(s) left, until {license.expiresAt}.
          {/if}
        </p>
        <p class="muted">Unlocked: {license.features.join(", ") || "nothing yet"}</p>
      {:else}
        <p class="muted">
          Ledgerly is free and fully usable. Pro adds reports, extra PDF templates, custom branding,
          scheduled backups, batch export and multi-currency.
        </p>
      {/if}

      {#if license.problem}
        <p class="error">{license.problem}</p>
      {/if}

      {#if !license.licensed && license.trialAvailable}
        <div class="row wrap spaced">
          <button class="btn secondary" onclick={startTrial} disabled={busy}>
            Try Pro for 14 days
          </button>
        </div>
      {/if}
    </div>

    <div class="card">
      <h2>This installation</h2>
      <p class="muted">
        Your licence is bound to this machine. The machine ID is a salted hash, so no raw hardware
        identifier is stored or shown.
      </p>
      <div class="grid-2">
        <div class="field">
          <label for="licenceSerial">Installation serial</label>
          <input id="licenceSerial" value={license.serial} readonly />
        </div>
        <div class="field">
          <label for="licenceMachine">Machine ID</label>
          <input id="licenceMachine" value={license.machineFingerprint} readonly />
        </div>
      </div>
    </div>

    <div class="card">
      <h2>Upgrade to Pro</h2>
      <p class="muted">
        Send the request code to the vendor, then paste the key you get back.
      </p>
      <div class="field">
        <label for="requestCode">Request code</label>
        <textarea id="requestCode" rows="4" readonly value={requestCode}></textarea>
      </div>
      <div class="row wrap">
        <button class="btn secondary" onclick={copyRequestCode}>Copy request code</button>
        <button class="btn secondary" onclick={saveRequestCode}>Save to file</button>
      </div>

      <div class="field spaced">
        <label for="upgradeKey">Upgrade key</label>
        <textarea
          id="upgradeKey"
          rows="3"
          bind:value={upgradeKey}
          placeholder="Paste the key, or use Load .lic file"
        ></textarea>
      </div>
      <div class="row wrap">
        <button class="btn" onclick={activateKey} disabled={busy || !upgradeKey.trim()}>
          Activate
        </button>
        <button class="btn secondary" onclick={loadLicenceFile} disabled={busy}>
          Load .lic file
        </button>
        {#if license.source === "license"}
          <button class="btn danger" onclick={deactivate}>Deactivate</button>
        {/if}
      </div>

      {#if notice}<p class="ok">{notice}</p>{/if}
      {#if error}<p class="error">{error}</p>{/if}
    </div>
  {/if}
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-4);
  }
</style>
