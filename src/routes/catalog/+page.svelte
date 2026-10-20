<script lang="ts">
  import { goto } from "$app/navigation";
  import { page } from "$app/state";
  import { api } from "$lib/api";
  import { emptyToNull, fromMinor, minorToInput, toMinor } from "$lib/format";
  import { t } from "$lib/i18n.svelte";
  import type { CatalogItem } from "$lib/types";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import EmptyState from "$lib/components/EmptyState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import SearchField from "$lib/components/SearchField.svelte";
  import { capped, RENDER_LIMIT } from "$lib/list";

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
  const visible = $derived(capped(items));

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

<PageHeader title={t("catalog.title")} subtitle={t("catalog.subtitle")} />

<div class="card">
  <h2>{t("catalog.addTitle")}</h2>
  <div class="grid-2">
    <div class="field">
      <label for="itemName">{t("catalog.nameLabel")}</label>
      <input id="itemName" bind:value={name} placeholder={t("catalog.namePlaceholder")} />
    </div>
    <div class="field">
      <label for="itemPrice">{t("catalog.unitPrice", { currency: currency || t("common.emDash") })}</label>
      <input id="itemPrice" bind:value={price} placeholder="0.00" inputmode="decimal" />
    </div>
    <div class="field">
      <label for="itemUnit">{t("common.unit")}</label>
      <input id="itemUnit" bind:value={unit} placeholder={t("catalog.unitPlaceholder")} />
    </div>
    <div class="field">
      <label for="itemDescription">{t("common.description")}</label>
      <input id="itemDescription" bind:value={description} />
    </div>
  </div>
  <button class="btn" onclick={add} disabled={adding || !name.trim()}>{t("action.addItem")}</button>
</div>

<div class="toolbar spaced">
  <SearchField
    bind:value={search}
    label={t("catalog.searchLabel")}
    example={t("catalog.searchExample")}
    oninput={onSearch}
  />
  <label class="row muted archived">
    <input
      type="checkbox"
      checked={includeArchived}
      onchange={(e) => setParam("archived", e.currentTarget.checked ? "1" : "")}
    />
    {t("common.showArchived")}
  </label>
</div>

<AsyncState {loading} {error} rows={5} columns={4}>
  <div class="card">
    {#if items.length === 0}
      <EmptyState title={t("catalog.empty")} body={t("catalog.emptyBody")} />
    {:else}
      <table>
        <thead>
          <tr>
            <th>{t("common.name")}</th>
            <th>{t("common.description")}</th>
            <th>{t("common.unit")}</th>
            <th class="numeric">{t("common.price")}</th>
            <th></th>
          </tr>
        </thead>
        <tbody>
          {#each visible.shown as item (item.id)}
            {#if editing?.id === item.id}
              <tr>
                <td><input bind:value={draft.name} aria-label={t("common.name")} /></td>
                <td><input bind:value={draft.description} aria-label={t("common.description")} /></td>
                <td><input bind:value={draft.unit} aria-label={t("common.unit")} /></td>
                <td>
                  <input
                    bind:value={draft.price}
                    inputmode="decimal"
                    aria-label={t("catalog.priceAria")}
                  />
                </td>
                <td>
                  <div class="row-actions">
                    <button class="btn" onclick={saveEdit} disabled={!draft.name.trim()}>
                      {t("action.save")}
                    </button>
                    <button class="btn secondary" onclick={() => (editing = null)}>
                      {t("action.cancel")}
                    </button>
                  </div>
                </td>
              </tr>
            {:else}
              <tr>
                <td>
                  {item.name}
                  {#if item.archivedAt}<span class="muted"> {t("common.archived")}</span>{/if}
                </td>
                <td>{item.description ?? t("common.emDash")}</td>
                <td>{item.unit ?? t("common.emDash")}</td>
                <td class="numeric">{fromMinor(item.unitPriceMinor, item.currency ?? currency)}</td>
                <td>
                  <div class="row-actions">
                    <button class="btn secondary" onclick={() => startEdit(item)}>
                      {t("action.edit")}
                    </button>
                    <button class="btn secondary" onclick={() => toggleArchive(item)}>
                      {item.archivedAt ? t("action.restore") : t("action.archive")}
                    </button>
                  </div>
                </td>
              </tr>
            {/if}
          {/each}
        </tbody>
      </table>
      {#if visible.hidden > 0}
        <p class="muted truncation">
          {t("list.showingFirstSearch", { limit: RENDER_LIMIT, total: items.length })}
        </p>
      {/if}
    {/if}
  </div>
</AsyncState>

<style>
  .spaced {
    margin-top: var(--space-4);
  }

  .truncation {
    margin-top: var(--space-3);
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
