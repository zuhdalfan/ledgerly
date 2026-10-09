<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import { emptyToNull, nullToEmpty } from "$lib/format";
  import type { ClientInput } from "$lib/types";

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

  const id = $derived(page.params.id ?? "");
  const isNew = $derived(id === "new");

  onMount(async () => {
    if (isNew) return;
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
    } catch (e) {
      loadError = String(e);
    }
  });

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
        await goto(`/clients/${created.id}`);
      } else {
        await api.clients.update(id, toInput());
        saved = true;
      }
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

<div class="page-head">
  <h1>{isNew ? "New client" : "Edit client"}</h1>
  <a class="link" href="/clients">Back to clients</a>
</div>

{#if loadError}
  <p class="error">{loadError}</p>
{/if}

<div class="card">
  <h2>Details</h2>

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

  <h2>Billing</h2>
  <div class="grid-2">
    <div class="field">
      <label for="taxId">Tax / VAT ID</label>
      <input id="taxId" bind:value={form.taxId} />
    </div>
    <div class="field">
      <label for="currency">Currency</label>
      <input id="currency" bind:value={form.currency} placeholder="e.g. USD" />
    </div>
  </div>

  <div class="field">
    <label for="notes">Notes</label>
    <textarea id="notes" bind:value={form.notes}></textarea>
  </div>

  <div class="row wrap">
    <button class="btn" onclick={save} disabled={saving}>
      {saving ? "Saving…" : "Save"}
    </button>
    {#if !isNew}
      <button class="btn secondary" onclick={toggleArchive}>
        {archived ? "Restore" : "Archive"}
      </button>
    {/if}
    {#if saved}<span class="ok">Saved</span>{/if}
    {#if saveError}<span class="error">{saveError}</span>{/if}
  </div>
</div>
