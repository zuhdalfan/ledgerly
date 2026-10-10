<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import { emptyToNull, formatDate, fromMinor, nullToEmpty, statusLabel } from "$lib/format";
  import type { ClientInput, Estimate, Invoice } from "$lib/types";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import { useDirtyGuard } from "$lib/guard";

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
  useDirtyGuard(() => dirty, "This client has unsaved changes. Leave and lose them?");

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
        loadError = "Client not found.";
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
    };
  }

  async function save() {
    saveError = "";
    saved = false;
    if (!form.displayName.trim()) {
      saveError = "Name is required.";
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

<PageHeader title={isNew ? "New client" : form.displayName || "Client"}>
  {#if !isNew}
    <a class="btn" href={`/invoices/new?client=${id}`}>New invoice</a>
    <a class="btn secondary" href={`/estimates/new?client=${id}`}>New estimate</a>
  {/if}
</PageHeader>

{#if loadError}
  <p class="error">{loadError}</p>
{/if}

{#if !isNew}
  <div class="card rise" style="--i: 0">
    <h2>Balance</h2>
    {#if documentsLoading}
      <p class="muted">Adding up their invoices…</p>
    {:else if outstanding.length === 0}
      <p class="muted">Nothing outstanding. Every invoice is settled.</p>
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
          {overdueCount}
          {overdueCount === 1 ? "invoice is" : "invoices are"} overdue.
          <a class="link" href={`/invoices?status=overdue`}>See them all</a>
        </p>
      {/if}
    {/if}
  </div>

  <div class="card rise" style="--i: 1">
    <h2>Invoices</h2>
    {#if documentsLoading}
      <p class="muted">Reading their invoices…</p>
    {:else if invoices.length === 0}
      <EmptyState
        title="No invoices for {form.displayName || "this client"} yet"
        body="When you bill them, it will show up here with what is still owed."
      >
        <a class="btn" href={`/invoices/new?client=${id}`}>Create an invoice</a>
      </EmptyState>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Number</th>
            <th>Issued</th>
            <th>Due</th>
            <th class="numeric">Balance</th>
            <th>Status</th>
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
    <h2>Estimates</h2>
    {#if documentsLoading}
      <p class="muted">Reading their estimates…</p>
    {:else if estimates.length === 0}
      <EmptyState
        title="No estimates yet"
        body="A quote for this client, ready to convert into an invoice once the work is agreed."
      >
        <a class="btn secondary" href={`/estimates/new?client=${id}`}>Create an estimate</a>
      </EmptyState>
    {:else}
      <table>
        <thead>
          <tr>
            <th>Number</th>
            <th>Issued</th>
            <th>Valid until</th>
            <th class="numeric">Total</th>
            <th>Status</th>
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
  <h2>{isNew ? "Details" : "Their details"}</h2>

  <div class="field">
    <label for="displayName">Name *</label>
    <input id="displayName" bind:value={form.displayName} placeholder="Client or company name" />
  </div>

  <div class="grid-2">
    <div class="field">
      <label for="company">Company</label>
      <input id="company" bind:value={form.company} />
    </div>
    <div class="field">
      <label for="contactPerson">Contact person</label>
      <input id="contactPerson" bind:value={form.contactPerson} />
    </div>
    <div class="field">
      <label for="email">Email</label>
      <input id="email" type="email" bind:value={form.email} />
    </div>
    <div class="field">
      <label for="phone">Phone</label>
      <input id="phone" bind:value={form.phone} />
    </div>
  </div>

  <h2 class="spaced">Address</h2>
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

  <h2 class="spaced">Billing</h2>
  <div class="grid-2">
    <div class="field">
      <label for="taxId">Tax / VAT ID</label>
      <input id="taxId" bind:value={form.taxId} />
    </div>
    <div class="field">
      <label for="currency">Currency</label>
      <input id="currency" bind:value={form.currency} placeholder="USD" />
    </div>
  </div>

  <div class="field">
    <label for="notes">Notes</label>
    <textarea id="notes" bind:value={form.notes}></textarea>
  </div>

  <div class="row wrap">
    <button class="btn" onclick={save} disabled={saving}>
      {saving ? "Saving…" : "Save client"}
    </button>
    {#if !isNew}
      <button class="btn secondary" onclick={toggleArchive}>
        {archived ? "Restore" : "Archive"}
      </button>
      <a class="link" href="/clients">Back to clients</a>
    {/if}
    {#if saved}<span class="ok">Saved</span>{/if}
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
