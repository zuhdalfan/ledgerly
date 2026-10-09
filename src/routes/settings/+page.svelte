<script lang="ts">
  import { onMount } from "svelte";
  import { ask, message, open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { emptyToNull, localDate, nullToEmpty } from "$lib/format";
  import type {
    AppInfo,
    BackupInfo,
    BackupSettings,
    BusinessInput,
    LicenseStatus,
    TaxRate,
  } from "$lib/types";

  type Form = Record<keyof BusinessInput, string>;

  const emptyForm = (): Form => ({
    displayName: "",
    legalName: "",
    email: "",
    phone: "",
    website: "",
    addressLine1: "",
    addressLine2: "",
    city: "",
    region: "",
    postalCode: "",
    country: "",
    taxId: "",
    defaultCurrency: "USD",
    defaultTerms: "",
    defaultNotes: "",
    defaultPaymentInstructions: "",
  });

  let form = $state<Form>(emptyForm());
  let info = $state<AppInfo | null>(null);
  let error = $state("");
  let saved = $state(false);
  let saving = $state(false);

  // Tax rates
  let taxRates = $state<TaxRate[]>([]);
  let taxName = $state("");
  let taxPercent = $state("");
  let taxError = $state("");

  // Numbering
  let prefix = $state("INV-");
  let nextValue = $state(1);
  let padding = $state(4);
  let resetYearly = $state(false);
  let numberError = $state("");
  let numberSaved = $state(false);

  const numberPreview = $derived(
    `${prefix}${resetYearly ? `${new Date().getFullYear()}-` : ""}${String(nextValue).padStart(
      Math.min(Math.max(padding, 1), 12),
      "0"
    )}`
  );

  async function loadTaxRates() {
    taxRates = await api.taxRates.list(true);
  }

  onMount(async () => {
    try {
      const [business, appInfo, numbering] = await Promise.all([
        api.business.get(),
        api.appInfo(),
        api.numbering.get("invoice"),
      ]);
      info = appInfo;
      form = {
        displayName: business.displayName,
        legalName: nullToEmpty(business.legalName),
        email: nullToEmpty(business.email),
        phone: nullToEmpty(business.phone),
        website: nullToEmpty(business.website),
        addressLine1: nullToEmpty(business.addressLine1),
        addressLine2: nullToEmpty(business.addressLine2),
        city: nullToEmpty(business.city),
        region: nullToEmpty(business.region),
        postalCode: nullToEmpty(business.postalCode),
        country: nullToEmpty(business.country),
        taxId: nullToEmpty(business.taxId),
        defaultCurrency: business.defaultCurrency,
        defaultTerms: nullToEmpty(business.defaultTerms),
        defaultNotes: nullToEmpty(business.defaultNotes),
        defaultPaymentInstructions: nullToEmpty(business.defaultPaymentInstructions),
      };

      prefix = numbering.prefix;
      nextValue = numbering.nextValue;
      padding = numbering.padding;
      resetYearly = numbering.resetYearly;

      await loadTaxRates();
    } catch (e) {
      error = String(e);
    }
  });

  async function save() {
    saving = true;
    saved = false;
    error = "";
    try {
      await api.business.update({
        displayName: form.displayName.trim(),
        legalName: emptyToNull(form.legalName),
        email: emptyToNull(form.email),
        phone: emptyToNull(form.phone),
        website: emptyToNull(form.website),
        addressLine1: emptyToNull(form.addressLine1),
        addressLine2: emptyToNull(form.addressLine2),
        city: emptyToNull(form.city),
        region: emptyToNull(form.region),
        postalCode: emptyToNull(form.postalCode),
        country: emptyToNull(form.country),
        taxId: emptyToNull(form.taxId),
        defaultCurrency: form.defaultCurrency.trim() || "USD",
        defaultTerms: emptyToNull(form.defaultTerms),
        defaultNotes: emptyToNull(form.defaultNotes),
        defaultPaymentInstructions: emptyToNull(form.defaultPaymentInstructions),
      });
      saved = true;
    } catch (e) {
      error = String(e);
    } finally {
      saving = false;
    }
  }

  async function addTaxRate() {
    if (!taxName.trim()) return;
    taxError = "";
    try {
      const percent = Number.parseFloat(taxPercent);
      await api.taxRates.create({
        name: taxName.trim(),
        rateBp: Number.isFinite(percent) ? Math.round(percent * 100) : 0,
      });
      taxName = "";
      taxPercent = "";
      await loadTaxRates();
    } catch (e) {
      taxError = String(e);
    }
  }

  async function toggleTaxRate(rate: TaxRate) {
    try {
      await api.taxRates.setArchived(rate.id, rate.archivedAt === null);
      await loadTaxRates();
    } catch (e) {
      taxError = String(e);
    }
  }

  async function saveNumbering() {
    numberError = "";
    numberSaved = false;
    try {
      const updated = await api.numbering.update("invoice", {
        prefix,
        nextValue: Number(nextValue) || 1,
        padding: Number(padding) || 4,
        resetYearly,
      });
      prefix = updated.prefix;
      nextValue = updated.nextValue;
      padding = updated.padding;
      resetYearly = updated.resetYearly;
      numberSaved = true;
    } catch (e) {
      numberError = String(e);
    }
  }

  // ---- Data & backups ----

  let backupSettings = $state<BackupSettings>({
    enabled: false,
    folder: "",
    keep: 7,
    lastRun: null,
  });
  let backups = $state<BackupInfo[]>([]);
  let backupMessage = $state("");
  let backupError = $state("");
  let backupBusy = $state(false);

  function formatSize(bytes: number): string {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function formatWhen(epoch: number | null): string {
    return epoch ? new Date(epoch * 1000).toLocaleString() : "—";
  }

  async function loadBackups() {
    try {
      backupSettings = await api.backup.getSettings();
      backups = await api.backup.list();
    } catch (e) {
      backupError = String(e);
    }
  }

  async function backupNow() {
    backupMessage = "";
    backupError = "";
    try {
      const path = await saveDialog({
        defaultPath: `ledgerly-${localDate()}.ledgerly`,
        filters: [{ name: "Ledgerly backup", extensions: ["ledgerly"] }],
      });
      if (!path) return;

      backupBusy = true;
      backupMessage = `Backup saved to ${await api.backup.now(path)}`;
      await loadBackups();
    } catch (e) {
      backupError = String(e);
    } finally {
      backupBusy = false;
    }
  }

  async function restore(path: string) {
    backupMessage = "";
    backupError = "";

    const confirmed = await ask(
      "Replace all current data with this backup?\n\nA safety copy of your current data is saved first.",
      { title: "Restore backup", kind: "warning" }
    );
    if (!confirmed) return;

    backupBusy = true;
    try {
      const safety = await api.backup.restore(path);
      await message(
        `Backup restored.\n\nYour previous data was saved to:\n${safety}`,
        { title: "Restore complete" }
      );
      window.location.reload();
    } catch (e) {
      backupError = String(e);
      backupBusy = false;
    }
  }

  async function chooseBackupFile() {
    const selected = await openDialog({
      multiple: false,
      filters: [{ name: "Ledgerly backup", extensions: ["ledgerly"] }],
    });
    if (typeof selected === "string") await restore(selected);
  }

  async function saveBackupSettings() {
    backupMessage = "";
    backupError = "";
    try {
      backupSettings = await api.backup.saveSettings({
        enabled: backupSettings.enabled,
        folder: backupSettings.folder,
        keep: backupSettings.keep,
      });
      backupMessage = "Backup settings saved";
      await loadBackups();
    } catch (e) {
      backupError = String(e);
    }
  }

  onMount(() => {
    void loadBackups();
  });

  // ---- Licence ----

  let license = $state<LicenseStatus | null>(null);
  let requestCode = $state("");
  let upgradeKey = $state("");
  let licenseMessage = $state("");
  let licenseError = $state("");
  let licenseBusy = $state(false);

  async function loadLicense() {
    try {
      license = await api.license.status();
      requestCode = await api.license.requestCode();
    } catch (e) {
      licenseError = String(e);
    }
  }

  async function copyRequestCode() {
    licenseMessage = "";
    try {
      await navigator.clipboard.writeText(requestCode);
      licenseMessage = "Request code copied";
    } catch {
      licenseMessage = "Select the code above and copy it manually.";
    }
  }

  async function saveRequestCode() {
    licenseMessage = "";
    licenseError = "";
    try {
      const path = await saveDialog({
        defaultPath: "ledgerly-request-code.txt",
        filters: [{ name: "Text", extensions: ["txt"] }],
      });
      if (!path) return;
      await api.license.saveRequestFile(path);
      licenseMessage = `Request code saved to ${path}`;
    } catch (e) {
      licenseError = String(e);
    }
  }

  async function activateKey() {
    licenseMessage = "";
    licenseError = "";
    if (!upgradeKey.trim()) return;

    licenseBusy = true;
    try {
      license = await api.license.importKey(upgradeKey.trim());
      upgradeKey = "";
      licenseMessage = "Licence activated — thank you!";
    } catch (e) {
      licenseError = String(e);
    } finally {
      licenseBusy = false;
    }
  }

  async function loadLicenceFile() {
    licenseMessage = "";
    licenseError = "";
    try {
      const selected = await openDialog({
        multiple: false,
        filters: [{ name: "Ledgerly licence", extensions: ["lic", "json"] }],
      });
      if (typeof selected !== "string") return;

      licenseBusy = true;
      license = await api.license.importFile(selected);
      licenseMessage = "Licence activated — thank you!";
    } catch (e) {
      licenseError = String(e);
    } finally {
      licenseBusy = false;
    }
  }

  async function startTrial() {
    licenseMessage = "";
    licenseError = "";
    try {
      license = await api.license.startTrial();
      licenseMessage = `Pro trial started — ${license.daysRemaining} days.`;
    } catch (e) {
      licenseError = String(e);
    }
  }

  async function deactivate() {
    licenseMessage = "";
    licenseError = "";
    try {
      license = await api.license.deactivate();
      licenseMessage = "Licence removed from this machine.";
    } catch (e) {
      licenseError = String(e);
    }
  }

  onMount(() => {
    void loadLicense();
  });
</script>

<div class="page-head">
  <h1>Settings</h1>
</div>

<div class="card">
  <h2>Business profile</h2>

  <div class="field">
    <label for="displayName">Business name</label>
    <input id="displayName" bind:value={form.displayName} placeholder="Your business" />
  </div>

  <div class="grid-2">
    <div class="field">
      <label for="legalName">Legal name</label>
      <input id="legalName" bind:value={form.legalName} />
    </div>
    <div class="field">
      <label for="taxId">Tax / VAT ID</label>
      <input id="taxId" bind:value={form.taxId} />
    </div>
    <div class="field">
      <label for="email">Email</label>
      <input id="email" type="email" bind:value={form.email} />
    </div>
    <div class="field">
      <label for="phone">Phone</label>
      <input id="phone" bind:value={form.phone} />
    </div>
    <div class="field">
      <label for="website">Website</label>
      <input id="website" bind:value={form.website} />
    </div>
    <div class="field">
      <label for="defaultCurrency">Default currency</label>
      <input id="defaultCurrency" bind:value={form.defaultCurrency} />
    </div>
  </div>

  <h2>Address</h2>
  <div class="grid-2">
    <div class="field">
      <label for="addressLine1">Address line 1</label>
      <input id="addressLine1" bind:value={form.addressLine1} />
    </div>
    <div class="field">
      <label for="addressLine2">Address line 2</label>
      <input id="addressLine2" bind:value={form.addressLine2} />
    </div>
    <div class="field">
      <label for="city">City</label>
      <input id="city" bind:value={form.city} />
    </div>
    <div class="field">
      <label for="region">Region / state</label>
      <input id="region" bind:value={form.region} />
    </div>
    <div class="field">
      <label for="postalCode">Postal code</label>
      <input id="postalCode" bind:value={form.postalCode} />
    </div>
    <div class="field">
      <label for="country">Country</label>
      <input id="country" bind:value={form.country} />
    </div>
  </div>

  <h2>Invoice defaults</h2>
  <div class="field">
    <label for="defaultTerms">Default terms</label>
    <input id="defaultTerms" bind:value={form.defaultTerms} placeholder="e.g. Net 30" />
  </div>
  <div class="field">
    <label for="defaultNotes">Default notes</label>
    <textarea id="defaultNotes" bind:value={form.defaultNotes}></textarea>
  </div>
  <div class="field">
    <label for="defaultPaymentInstructions">Payment instructions</label>
    <textarea id="defaultPaymentInstructions" bind:value={form.defaultPaymentInstructions}
    ></textarea>
  </div>

  <div class="row wrap">
    <button class="btn" onclick={save} disabled={saving}>
      {saving ? "Saving…" : "Save"}
    </button>
    {#if saved}<span class="ok">Saved</span>{/if}
    {#if error}<span class="error">{error}</span>{/if}
  </div>
</div>

<div class="card">
  <h2>Tax rates</h2>

  <div class="toolbar">
    <input bind:value={taxName} placeholder="Name (e.g. VAT)" />
    <input
      bind:value={taxPercent}
      inputmode="decimal"
      placeholder="Rate %"
      style="width: 110px;"
    />
    <button class="btn" onclick={addTaxRate} disabled={!taxName.trim()}>Add</button>
  </div>

  {#if taxRates.length === 0}
    <p class="muted">No tax rates yet.</p>
  {:else}
    <table>
      <thead>
        <tr><th>Name</th><th>Rate</th><th></th></tr>
      </thead>
      <tbody>
        {#each taxRates as rate (rate.id)}
          <tr>
            <td>
              {rate.name}
              {#if rate.archivedAt}<span class="muted"> (archived)</span>{/if}
            </td>
            <td>{(rate.rateBp / 100).toFixed(2)}%</td>
            <td>
              <button class="btn secondary" onclick={() => toggleTaxRate(rate)}>
                {rate.archivedAt ? "Restore" : "Archive"}
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}

  {#if taxError}<p class="error">{taxError}</p>{/if}
</div>

<div class="card">
  <h2>Invoice numbering</h2>

  <div class="grid-2">
    <div class="field">
      <label for="prefix">Prefix</label>
      <input id="prefix" bind:value={prefix} placeholder="INV-" />
    </div>
    <div class="field">
      <label for="nextValue">Next number</label>
      <input id="nextValue" type="number" min="1" bind:value={nextValue} />
    </div>
    <div class="field">
      <label for="padding">Zero padding</label>
      <input id="padding" type="number" min="1" max="12" bind:value={padding} />
    </div>
    <div class="field">
      <label for="reset">Reset yearly</label>
      <label class="row muted">
        <input id="reset" type="checkbox" bind:checked={resetYearly} style="width: auto;" />
        Include the year and restart at 1 each January
      </label>
    </div>
  </div>

  <p class="muted">Next invoice will be <code style="display: inline;">{numberPreview}</code></p>

  <div class="row wrap">
    <button class="btn" onclick={saveNumbering}>Save numbering</button>
    {#if numberSaved}<span class="ok">Saved</span>{/if}
    {#if numberError}<span class="error">{numberError}</span>{/if}
  </div>
</div>

<div class="card">
  <h2>Data &amp; backups</h2>

  <p class="muted">Where your data lives — one folder, on this machine.</p>
  <code>{info?.dataDir ?? "…"}</code>
  <div class="row wrap" style="margin-top: 0.75rem;">
    <button class="btn secondary" onclick={() => info && api.openInViewer(info.dataDir)}>
      Open folder
    </button>
    <button class="btn" onclick={backupNow} disabled={backupBusy}>Back up now…</button>
    <button class="btn secondary" onclick={chooseBackupFile} disabled={backupBusy}>
      Restore from file…
    </button>
  </div>

  <h2 style="margin-top: 1.5rem;">Automatic backups</h2>
  <p class="muted">
    When enabled, a backup is written at most once a day, and older ones are rotated away.
  </p>
  <div class="grid-2">
    <div class="field">
      <label for="autoBackup">Schedule</label>
      <label class="row muted">
        <input
          id="autoBackup"
          type="checkbox"
          bind:checked={backupSettings.enabled}
          style="width: auto;"
        />
        Back up once a day on start
      </label>
    </div>
    <div class="field">
      <label for="keep">Keep the newest</label>
      <input id="keep" type="number" min="1" max="365" bind:value={backupSettings.keep} />
    </div>
  </div>
  <div class="field">
    <label for="backupFolder">Backup folder</label>
    <input id="backupFolder" bind:value={backupSettings.folder} />
  </div>

  <div class="row wrap">
    <button class="btn" onclick={saveBackupSettings}>Save backup settings</button>
    {#if backupSettings.lastRun}
      <span class="muted">Last automatic run {backupSettings.lastRun}</span>
    {/if}
    {#if backupMessage}<span class="ok">{backupMessage}</span>{/if}
  </div>
  {#if backupError}<p class="error">{backupError}</p>{/if}

  <h2 style="margin-top: 1.5rem;">Recent backups</h2>
  {#if backups.length === 0}
    <p class="muted">No backups found in {backupSettings.folder || "the backup folder"}.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>File</th>
          <th class="numeric">Size</th>
          <th>Created</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each backups as backup (backup.path)}
          <tr>
            <td>{backup.name}</td>
            <td class="numeric">{formatSize(backup.sizeBytes)}</td>
            <td>{formatWhen(backup.modifiedAt)}</td>
            <td>
              <button
                class="btn secondary"
                onclick={() => restore(backup.path)}
                disabled={backupBusy}
              >
                Restore
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<div class="card">
  <div class="page-head">
    <h2>Licence</h2>
    {#if license}
      <span class={`badge ${license.licensed ? "paid" : "draft"}`}>
        {#if license.source === "trial"}Pro trial{:else if license.licensed}Pro{:else}Free{/if}
      </span>
    {/if}
  </div>

  {#if !license}
    <p class="muted">Reading licence…</p>
  {:else}
    {#if license.licensed}
      <p>
        {#if license.source === "trial"}
          Pro trial — {license.daysRemaining} day(s) left, until {license.expiresAt}.
        {:else if license.plan === "perpetual"}
          Pro licence, perpetual.
        {:else}
          Pro licence ({license.plan}) — {license.daysRemaining} day(s) left, until {license.expiresAt}.
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

    <h2 style="margin-top: 1.25rem;">This installation</h2>
    <div class="grid-2">
      <div class="field">
        <label for="licenceSerial">Installation serial</label>
        <input id="licenceSerial" value={license.serial} disabled />
      </div>
      <div class="field">
        <label for="licenceMachine">Machine ID</label>
        <input id="licenceMachine" value={license.machineFingerprint} disabled />
      </div>
    </div>

    {#if !license.licensed && license.trialAvailable}
      <div class="row wrap">
        <button class="btn secondary" onclick={startTrial} disabled={licenseBusy}>
          Try Pro for 14 days
        </button>
      </div>
    {/if}

    <h2 style="margin-top: 1.5rem;">Upgrade to Pro</h2>
    <p class="muted">
      Send the request code to the vendor, then paste the key you get back. Activation happens
      entirely offline — this app sends nothing anywhere.
    </p>
    <div class="field">
      <label for="requestCode">Request code</label>
      <textarea id="requestCode" rows="4" readonly value={requestCode}></textarea>
    </div>
    <div class="row wrap">
      <button class="btn secondary" onclick={copyRequestCode}>Copy</button>
      <button class="btn secondary" onclick={saveRequestCode}>Save to file…</button>
    </div>

    <div class="field" style="margin-top: 1rem;">
      <label for="upgradeKey">Upgrade key</label>
      <textarea
        id="upgradeKey"
        rows="3"
        bind:value={upgradeKey}
        placeholder="Paste the key, or load a .lic file"
      ></textarea>
    </div>
    <div class="row wrap">
      <button class="btn" onclick={activateKey} disabled={licenseBusy || !upgradeKey.trim()}>
        Activate
      </button>
      <button class="btn secondary" onclick={loadLicenceFile} disabled={licenseBusy}>
        Load .lic file…
      </button>
      {#if license.source === "license"}
        <button class="btn danger" onclick={deactivate}>Deactivate</button>
      {/if}
    </div>

    {#if licenseMessage}<p class="ok">{licenseMessage}</p>{/if}
    {#if licenseError}<p class="error">{licenseError}</p>{/if}
  {/if}
</div>

<div class="card">
  <h2>About</h2>
  <p class="muted">
    {info?.name ?? "Ledgerly"} v{info?.version ?? "…"} — fully local and offline. No account, no
    cloud, no network.
  </p>
  <p class="muted">Data directory</p>
  <code>{info?.dataDir ?? "…"}</code>
  <p class="muted">Database</p>
  <code>{info?.databasePath ?? "…"}</code>
</div>
