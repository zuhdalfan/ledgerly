<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import { emptyToNull, fromMinor, toMinor } from "$lib/format";
  import type { CatalogItem } from "$lib/types";

  let items = $state<CatalogItem[]>([]);
  let currency = $state("USD");
  let query = $state("");
  let includeArchived = $state(false);
  let error = $state("");
  let adding = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  let name = $state("");
  let price = $state("");
  let unit = $state("");
  let description = $state("");

  async function refresh() {
    try {
      items = await api.catalog.list(query, includeArchived);
      error = "";
    } catch (e) {
      error = String(e);
    }
  }

  function onSearch() {
    clearTimeout(timer);
    timer = setTimeout(() => void refresh(), 200);
  }

  onMount(async () => {
    try {
      currency = (await api.business.get()).defaultCurrency;
    } catch {
      // fall back to the default currency label
    }
    await refresh();
  });

  async function add() {
    if (!name.trim()) return;
    adding = true;
    try {
      await api.catalog.create({
        name: name.trim(),
        description: emptyToNull(description),
        unit: emptyToNull(unit),
        unitPriceMinor: toMinor(price),
        currency: null,
        sku: null,
      });
      name = "";
      price = "";
      unit = "";
      description = "";
      await refresh();
    } catch (e) {
      error = String(e);
    } finally {
      adding = false;
    }
  }

  async function toggleArchive(item: CatalogItem) {
    try {
      await api.catalog.setArchived(item.id, item.archivedAt === null);
      await refresh();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<div class="page-head">
  <h1>Catalog</h1>
</div>

<div class="card">
  <h2>Add item</h2>
  <div class="grid-2">
    <div class="field">
      <label for="itemName">Name *</label>
      <input id="itemName" bind:value={name} placeholder="e.g. Design hour" />
    </div>
    <div class="field">
      <label for="itemPrice">Unit price ({currency})</label>
      <input id="itemPrice" bind:value={price} placeholder="0.00" />
    </div>
    <div class="field">
      <label for="itemUnit">Unit</label>
      <input id="itemUnit" bind:value={unit} placeholder="hour / day / item" />
    </div>
    <div class="field">
      <label for="itemDescription">Description</label>
      <input id="itemDescription" bind:value={description} />
    </div>
  </div>
  <button class="btn" onclick={add} disabled={adding || !name.trim()}>Add item</button>
</div>

<div class="toolbar" style="margin-top: 1.25rem;">
  <input placeholder="Search by name or SKU…" bind:value={query} oninput={onSearch} />
  <label class="row muted" style="white-space: nowrap;">
    <input
      type="checkbox"
      bind:checked={includeArchived}
      onchange={() => refresh()}
      style="width: auto;"
    />
    Show archived
  </label>
</div>

{#if error}<p class="error">{error}</p>{/if}

<div class="card">
  {#if items.length === 0}
    <p class="muted">No catalog items yet.</p>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Name</th>
          <th>Description</th>
          <th>Unit</th>
          <th>Price</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        {#each items as item (item.id)}
          <tr>
            <td>
              {item.name}
              {#if item.archivedAt}<span class="muted"> (archived)</span>{/if}
            </td>
            <td>{item.description ?? "—"}</td>
            <td>{item.unit ?? "—"}</td>
            <td>{fromMinor(item.unitPriceMinor, item.currency ?? currency)}</td>
            <td>
              <button class="btn secondary" onclick={() => toggleArchive(item)}>
                {item.archivedAt ? "Restore" : "Archive"}
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>
