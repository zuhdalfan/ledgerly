<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import { emptyToNull, fromMinor, minorToInput, toMinor } from "$lib/format";
  import type { CatalogItem } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import SearchField from "$lib/components/SearchField.svelte";

  let items = $state<CatalogItem[]>([]);
  let currency = $state("");
  let loading = $state(true);
  let error = $state("");
  let adding = $state(false);
  let search = $state("");
  let timer: ReturnType<typeof setTimeout> | undefined;

  let name = $state("");
  let price = $state("");
  let unit = $state("");
  let description = $state("");

  const query = $derived(page.url.searchParams.get("q") ?? "");
  const includeArchived = $derived(page.url.searchParams.get("archived") === "1");

  async function load() {
    loading = true;
    try {
      const [business, rows] = await Promise.all([
        api.business.get(),
        api.catalog.list(query, includeArchived),
      ]);
      currency = business.defaultCurrency;
      items = rows;
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  async function setParam(key: string, value: string) {
    const next = new URL(page.url);
    if (value) next.searchParams.set(key, value);
    else next.searchParams.delete(key);
    await goto(next, { replaceState: true, keepFocus: true, noScroll: true });
  }

  function onSearch() {
    clearTimeout(timer);
    timer = setTimeout(() => void setParam("q", search), 200);
  }

  let applied = "";
  $effect(() => {
    const key = `${query}|${includeArchived}`;
    if (key === applied) return;
    applied = key;
    search = query;
    void load();
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
      await load();
    } catch (e) {
      error = String(e);
    } finally {
      adding = false;
    }
  }

  async function toggleArchive(item: CatalogItem) {
    try {
      await api.catalog.setArchived(item.id, item.archivedAt === null);
      await load();
    } catch (e) {
      error = String(e);
    }
  }

  // ---- Editing ----

  let editing = $state<CatalogItem | null>(null);
  let draft = $state({ name: "", description: "", unit: "", price: "" });

  function startEdit(item: CatalogItem) {
    editing = item;
    draft = {
      name: item.name,
      description: item.description ?? "",
      unit: item.unit ?? "",
      price: minorToInput(item.unitPriceMinor),
    };
  }

  async function saveEdit() {
    if (!editing || !draft.name.trim()) return;
    try {
      await api.catalog.update(editing.id, {
        name: draft.name.trim(),
        description: emptyToNull(draft.description),
        unit: emptyToNull(draft.unit),
        unitPriceMinor: toMinor(draft.price),
        // Neither is shown in this table, so carry them through untouched
        // rather than blanking them on save.
        currency: editing.currency,
        sku: editing.sku,
      });
      editing = null;
      await load();
    } catch (e) {
      error = String(e);
    }
  }
</script>

<PageHeader
  title="Catalog"
  subtitle="What you sell, priced once, so a line item is a pick rather than a typing job."
/>

<div class="card">
  <h2>Add item</h2>
  <div class="grid-2">
    <div class="field">
      <label for="itemName">Name *</label>
      <input id="itemName" bind:value={name} placeholder="Design hour" />
    </div>
    <div class="field">
      <label for="itemPrice">Unit price ({currency || "—"})</label>
      <input id="itemPrice" bind:value={price} placeholder="0.00" inputmode="decimal" />
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

<div class="toolbar spaced">
  <SearchField
    bind:value={search}
    label="Search catalog by name or SKU"
    example="Name or SKU"
    oninput={onSearch}
  />
  <label class="row muted archived">
    <input
      type="checkbox"
      checked={includeArchived}
      onchange={(e) => setParam("archived", e.currentTarget.checked ? "1" : "")}
    />
    Show archived
  </label>
</div>

<AsyncState {loading} {error} rows={5} columns={4}>
  <div class="card">
    {#if items.length === 0}
      <EmptyState
        title="Nothing in the catalog yet"
        body="Add the things you bill for. Then an invoice line is a pick, not a typing job."
      />
    {:else}
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Description</th>
            <th>Unit</th>
            <th class="numeric">Price</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each items as item (item.id)}
            {#if editing?.id === item.id}
              <tr>
                <td><input bind:value={draft.name} aria-label="Name" /></td>
                <td><input bind:value={draft.description} aria-label="Description" /></td>
                <td><input bind:value={draft.unit} aria-label="Unit" /></td>
                <td>
                  <input
                    bind:value={draft.price}
                    inputmode="decimal"
                    aria-label="Unit price"
                  />
                </td>
                <td>
                  <div class="row-actions">
                    <button class="btn" onclick={saveEdit} disabled={!draft.name.trim()}>
                      Save
                    </button>
                    <button class="btn secondary" onclick={() => (editing = null)}>Cancel</button>
                  </div>
                </td>
              </tr>
            {:else}
              <tr>
                <td>
                  {item.name}
                  {#if item.archivedAt}<span class="muted"> (archived)</span>{/if}
                </td>
                <td>{item.description ?? "—"}</td>
                <td>{item.unit ?? "—"}</td>
                <td class="numeric">{fromMinor(item.unitPriceMinor, item.currency ?? currency)}</td>
                <td>
                  <div class="row-actions">
                    <button class="btn secondary" onclick={() => startEdit(item)}>Edit</button>
                    <button class="btn secondary" onclick={() => toggleArchive(item)}>
                      {item.archivedAt ? "Restore" : "Archive"}
                    </button>
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
    {/if}
  </div>
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-4);
  }

  .archived {
    white-space: nowrap;
  }

  .archived input {
    width: auto;
  }

  .row-actions {
    display: flex;
    gap: var(--space-2);
    justify-content: flex-end;
  }
</style>
