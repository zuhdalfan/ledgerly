<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { save as saveDialog } from "@tauri-apps/plugin-dialog";
  import { api } from "$lib/api";
  import { emptyToNull, formatDate, fromMinor, nullToEmpty, statusLabel } from "$lib/format";
  import type { ClientInput, Estimate, Invoice } from "$lib/types";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { useDirtyGuard } from "$lib/guard";
  import { LOCALES, plural, t } from "$lib/i18n.svelte";

  type Form = Record<keyof ClientInput, string>;

  const emptyForm = (): Form => ({
    displayName: "",
    company: "",
    contactPerson: "",
    email: "",
    phone: "",
    addressLine1: "",
    addressLine2: "",
    city: "",
    region: "",
    postalCode: "",
    country: "",
    taxId: "",
    currency: "",
    notes: "",
    locale: "",
  });

  let form = $state<Form>(emptyForm());
  let archived = $state(false);
  let loadError = $state("");
  let saveError = $state("");
  let saved = $state(false);
  let saving = $state(false);

  let invoices = $state<Invoice[]>([]);
  let estimates = $state<Estimate[]>([]);
  let documentsLoading = $state(false);

  const id = $derived(page.params.id ?? "");
  const isNew = $derived(id === "new");

  // Dirty tracking compares the form against the last state we loaded or saved,
  // so an untouched form never nags.
  let baseline = $state("");
  const dirty = $derived(baseline !== "" && JSON.stringify(toInput()) !== baseline);
  useDirtyGuard(() => dirty, t("client.unsaved"));

  // Invoices can be in different currencies, so a single total would be a lie.
  const outstanding = $derived.by(() => {
    const totals = new Map<string, number>();
    for (const invoice of invoices) {
      if (invoice.displayStatus === "void" || invoice.balanceMinor <= 0) continue;
      totals.set(invoice.currency, (totals.get(invoice.currency) ?? 0) + invoice.balanceMinor);
    }
    return [...totals.entries()].map(([code, amountMinor]) => ({ code, amountMinor }));
  });

  const overdueCount = $derived(
    invoices.filter((invoice) => invoice.displayStatus === "overdue").length
  );

  onMount(async () => {
    if (isNew) {
      baseline = JSON.stringify(toInput());
      return;
    }
    try {
      const client = await api.clients.get(id);
      if (!client) {
        loadError = t("client.notFound");
        return;
      }
      form = {
        displayName: client.displayName,
        company: nullToEmpty(client.company),
        contactPerson: nullToEmpty(client.contactPerson),
        email: nullToEmpty(client.email),
        phone: nullToEmpty(client.phone),
        addressLine1: nullToEmpty(client.addressLine1),
        addressLine2: nullToEmpty(client.addressLine2),
        city: nullToEmpty(client.city),
        region: nullToEmpty(client.region),
        postalCode: nullToEmpty(client.postalCode),
        country: nullToEmpty(client.country),
        taxId: nullToEmpty(client.taxId),
        currency: nullToEmpty(client.currency),
        notes: nullToEmpty(client.notes),
        locale: nullToEmpty(client.locale),
      };
      archived = client.archivedAt !== null;
      await loadDocuments();
      baseline = JSON.stringify(toInput());
    } catch (e) {
      loadError = String(e);
    }
  });

  /// Filtered in SQL rather than in the browser, so a client with years of
  /// history does not pull the whole ledger across the IPC boundary.
  async function loadDocuments() {
    documentsLoading = true;
    try {
      const [theirInvoices, theirEstimates] = await Promise.all([
        api.invoices.list(undefined, undefined, id),
        api.estimates.list(undefined, undefined, id),
      ]);
      invoices = theirInvoices;
      estimates = theirEstimates;
    } catch (e) {
      loadError = String(e);
    } finally {
      documentsLoading = false;
    }
  }

  let statementBusy = $state(false);
  let statementMessage = $state("");

  /// A statement is a document the user sends themselves: the app builds it and
  /// opens it, and stops there.
  async function exportStatement() {
    statementMessage = "";
    statementBusy = true;
    try {
      const name = (form.displayName || "client").replace(/[^\w]+/g, "-").replace(/^-|-$/g, "");
      const path = await saveDialog({
        defaultPath: `statement-${name || "client"}.pdf`,
        filters: [{ name: "PDF", extensions: ["pdf"] }],
      });
      if (!path) return;

      await api.clients.exportStatement(id, path);
      await api.openInViewer(path);
      statementMessage = t("client.statementSaved", { path });
    } catch (e) {
      statementMessage = String(e);
    } finally {
      statementBusy = false;
    }
  }

  function toInput(): ClientInput {
    return {
      displayName: form.displayName.trim(),
      company: emptyToNull(form.company),
      contactPerson: emptyToNull(form.contactPerson),
      email: emptyToNull(form.email),
      phone: emptyToNull(form.phone),
      addressLine1: emptyToNull(form.addressLine1),
      addressLine2: emptyToNull(form.addressLine2),
      city: emptyToNull(form.city),
      region: emptyToNull(form.region),
      postalCode: emptyToNull(form.postalCode),
      country: emptyToNull(form.country),
      taxId: emptyToNull(form.taxId),
      currency: emptyToNull(form.currency),
      notes: emptyToNull(form.notes),
      locale: emptyToNull(form.locale),
    };
  }

  async function save() {
    saveError = "";
    saved = false;
    if (!form.displayName.trim()) {
      saveError = t("client.nameRequired");
      return;
    }

    saving = true;
    try {
      if (isNew) {
        const created = await api.clients.create(toInput());
        // Replace rather than push, so Back does not land on a blank form.
        await goto(`/clients/${created.id}`, { replaceState: true });
        // The route component is reused for the new id, so hydrate rather than
        // waiting for a remount that never comes.
        archived = created.archivedAt !== null;
        await loadDocuments();
        saved = true;
      } else {
        await api.clients.update(id, toInput());
        saved = true;
      }

      baseline = JSON.stringify(toInput());
    } catch (e) {
      saveError = String(e);
    } finally {
      saving = false;
    }
  }

  async function toggleArchive() {
    saveError = "";
    try {
      await api.clients.setArchived(id, !archived);
      archived = !archived;
    } catch (e) {
      saveError = String(e);
    }
  }
</script>

<PageHeader
  title={isNew ? t("action.newClient") : form.displayName || t("client.fallbackTitle")}
>
  {#if !isNew}
    <a class="btn" href={`/invoices/new?client=${id}`}>{t("action.newInvoice")}</a>
    <a class="btn secondary" href={`/estimates/new?client=${id}`}>{t("action.newEstimate")}</a>
    <button
      class="btn secondary"
      onclick={exportStatement}
      disabled={statementBusy || outstanding.length === 0}
    >
      {statementBusy ? t("action.building") : t("action.statement")}
    </button>
  {/if}
</PageHeader>

{#if statementMessage}<p class="ok">{statementMessage}</p>{/if}

{#if loadError}
  <p class="error">{loadError}</p>
{/if}

{#if !isNew}
  <div class="card rise" style="--i: 0">
    <h2>{t("common.balance")}</h2>
    {#if documentsLoading}
      <p class="muted">{t("client.addingUp")}</p>
    {:else if outstanding.length === 0}
      <p class="muted">{t("client.nothingOutstanding")}</p>
    {:else}
      <div class="figures">
        {#each outstanding as total (total.code)}
          <div>
            <p class="figure">{fromMinor(total.amountMinor, total.code)}</p>
            <p class="muted">{total.code}</p>
          </div>
        {/each}
      </div>
      {#if overdueCount > 0}
        <p class="error">
          {plural(overdueCount, "client.overdueOne", "client.overdueOther")}
          <a class="link" href={`/invoices?status=overdue`}>{t("action.seeThemAll")}</a>
        </p>
      {/if}
    {/if}
  </div>

  <div class="card rise" style="--i: 1">
    <h2>{t("common.invoices")}</h2>
    {#if documentsLoading}
      <p class="muted">{t("client.readingInvoices")}</p>
    {:else if invoices.length === 0}
      <EmptyState
        title={t("client.noInvoicesTitle", {
          client: form.displayName || t("client.fallbackTitle"),
        })}
        body={t("client.noInvoicesBody")}
      >
        <a class="btn" href={`/invoices/new?client=${id}`}>{t("action.createInvoice")}</a>
      </EmptyState>
    {:else}
      <table>
        <thead>
          <tr>
            <th>{t("common.number")}</th>
            <th>{t("common.issued")}</th>
            <th>{t("common.due")}</th>
            <th class="numeric">{t("common.balance")}</th>
            <th>{t("common.status")}</th>
          </tr>
        </thead>
        <tbody>
          {#each invoices as invoice (invoice.id)}
            <tr>
              <td><a class="link" href={`/invoices/${invoice.id}`}>{invoice.number}</a></td>
              <td>{formatDate(invoice.issueDate)}</td>
              <td>{formatDate(invoice.dueDate)}</td>
              <td class="numeric">{fromMinor(invoice.balanceMinor, invoice.currency)}</td>
              <td>
                <span class={`badge ${invoice.displayStatus}`}>
                  {statusLabel(invoice.displayStatus)}
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>

  <div class="card rise" style="--i: 2">
    <h2>{t("nav.estimates")}</h2>
    {#if documentsLoading}
      <p class="muted">{t("client.readingEstimates")}</p>
    {:else if estimates.length === 0}
      <EmptyState title={t("estimates.none")} body={t("client.noEstimatesBody")}>
        <a class="btn secondary" href={`/estimates/new?client=${id}`}>
          {t("action.createEstimate")}
        </a>
      </EmptyState>
    {:else}
      <table>
        <thead>
          <tr>
            <th>{t("common.number")}</th>
            <th>{t("common.issued")}</th>
            <th>{t("common.validUntil")}</th>
            <th class="numeric">{t("common.total")}</th>
            <th>{t("common.status")}</th>
          </tr>
        </thead>
        <tbody>
          {#each estimates as estimate (estimate.id)}
            <tr>
              <td><a class="link" href={`/estimates/${estimate.id}`}>{estimate.number}</a></td>
              <td>{formatDate(estimate.issueDate)}</td>
              <td>{formatDate(estimate.validUntil)}</td>
              <td class="numeric">{fromMinor(estimate.totalMinor, estimate.currency)}</td>
              <td>
                <span class={`badge ${estimate.displayStatus}`}>
                  {statusLabel(estimate.displayStatus)}
                </span>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
{/if}

<div class="card">
  <h2>{isNew ? t("editor.details") : t("client.theirDetails")}</h2>

  <div class="field">
    <label for="displayName">{t("catalog.nameLabel")}</label>
    <input
      id="displayName"
      bind:value={form.displayName}
      placeholder={t("client.namePlaceholder")}
    />
  </div>

  <div class="grid-2">
    <div class="field">
      <label for="company">{t("common.company")}</label>
      <input id="company" bind:value={form.company} />
    </div>
    <div class="field">
      <label for="contactPerson">{t("common.contactPerson")}</label>
      <input id="contactPerson" bind:value={form.contactPerson} />
    </div>
    <div class="field">
      <label for="email">{t("common.email")}</label>
      <input id="email" type="email" bind:value={form.email} />
    </div>
    <div class="field">
      <label for="phone">{t("common.phone")}</label>
      <input id="phone" bind:value={form.phone} />
    </div>
  </div>

  <h2 class="spaced">{t("settings.business.address")}</h2>
  <div class="grid-2">
    <div class="field">
      <label for="addressLine1">{t("common.addressLine1")}</label>
      <input id="addressLine1" bind:value={form.addressLine1} />
    </div>
    <div class="field">
      <label for="addressLine2">{t("common.addressLine2")}</label>
      <input id="addressLine2" bind:value={form.addressLine2} />
    </div>
    <div class="field">
      <label for="city">{t("common.city")}</label>
      <input id="city" bind:value={form.city} />
    </div>
    <div class="field">
      <label for="region">{t("common.region")}</label>
      <input id="region" bind:value={form.region} />
    </div>
    <div class="field">
      <label for="postalCode">{t("common.postalCode")}</label>
      <input id="postalCode" bind:value={form.postalCode} />
    </div>
    <div class="field">
      <label for="country">{t("common.country")}</label>
      <input id="country" bind:value={form.country} />
    </div>
  </div>

  <h2 class="spaced">{t("client.billing")}</h2>
  <div class="grid-2">
    <div class="field">
      <label for="taxId">{t("common.taxId")}</label>
      <input id="taxId" bind:value={form.taxId} />
    </div>
    <div class="field">
      <label for="currency">{t("common.currency")}</label>
      <input id="currency" bind:value={form.currency} placeholder="USD" />
    </div>
    <div class="field">
      <label for="locale">{t("common.documentLanguage")}</label>
      <select id="locale" bind:value={form.locale}>
        <option value="">{t("common.inheritSettings")}</option>
        {#each LOCALES as option (option.value)}
          <option value={option.value}>{option.label}</option>
        {/each}
      </select>
    </div>
  </div>

  <div class="field">
    <label for="notes">{t("common.notes")}</label>
    <textarea id="notes" bind:value={form.notes}></textarea>
  </div>

  <div class="row wrap">
    <button class="btn" onclick={save} disabled={saving}>
      {saving ? t("action.saving") : t("action.saveClient")}
    </button>
    {#if !isNew}
      <button class="btn secondary" onclick={toggleArchive}>
        {archived ? t("action.restore") : t("action.archive")}
      </button>
      <a class="link" href="/clients">{t("action.backToClients")}</a>
    {/if}
    {#if saved}<span class="ok">{t("action.saved")}</span>{/if}
    {#if saveError}<span class="error">{saveError}</span>{/if}
  </div>
</div>

<style>
  .spaced {
    margin-top: var(--space-5);
  }

  .figures {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-5);
  }

  .figure {
    margin: 0;
    font-size: var(--text-2xl);
    font-weight: 700;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
  }
</style>
