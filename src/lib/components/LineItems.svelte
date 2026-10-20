<script lang="ts">
  import { api } from "$lib/api";
  import { fromMinor, minorToInput } from "$lib/format";
  import { emptyItem, type ItemForm } from "$lib/lineItems";
  import { t } from "$lib/i18n.svelte";
  import type { CatalogItem, LineTotals, TaxRate } from "$lib/types";

  let {
    items = $bindable(),
    taxRates,
    currency,
    lines,
  }: {
    items: ItemForm[];
    taxRates: TaxRate[];
    currency: string;
    lines: LineTotals[];
  } = $props();

  // Row inputs sit under column headers, which does not name them for a screen
  // reader, so every control carries the column and its row.
  const name = (column: string, index: number) =>
    t("lineItems.aria", { column, line: index + 1 });

  function addItem() {
    items = [...items, emptyItem()];
  }

  function removeItem(index: number) {
    const next = items.filter((_, i) => i !== index);
    items = next.length ? next : [emptyItem()];
  }

  // ---- Catalog picker ----

  let pickerOpen = $state(false);
  let catalog = $state<CatalogItem[]>([]);
  let catalogQuery = $state("");
  let catalogLoading = $state(false);
  let catalogRead = false;

  async function togglePicker() {
    pickerOpen = !pickerOpen;
    if (!pickerOpen || catalogRead) return;

    catalogRead = true;
    catalogLoading = true;
    try {
      catalog = await api.catalog.list("", false);
    } catch {
      // The picker is a convenience; typing a line by hand still works.
    } finally {
      catalogLoading = false;
    }
  }

  const matches = $derived.by(() => {
    const needle = catalogQuery.trim().toLowerCase();
    if (!needle) return catalog.slice(0, 30);
    return catalog
      .filter((item) =>
        `${item.name} ${item.description ?? ""} ${item.sku ?? ""}`.toLowerCase().includes(needle)
      )
      .slice(0, 30);
  });

  function pick(item: CatalogItem) {
    items = [
      ...items,
      {
        ...emptyItem(),
        description: item.name,
        unit: item.unit ?? "",
        price: minorToInput(item.unitPriceMinor),
      },
    ];
    catalogQuery = "";
    pickerOpen = false;
  }
</script>

<div class="wrap">
  <table>
    <thead>
      <tr>
        <th class="wide">{t("common.description")}</th>
        <th class="qty">{t("lineItems.qty")}</th>
        <th class="unit">{t("common.unit")}</th>
        <th class="money">{t("common.price")}</th>
        <th class="money">{t("lineItems.discount")}</th>
        <th class="value">{t("lineItems.value")}</th>
        <th class="tax">{t("common.tax")}</th>
        <th class="incl">{t("lineItems.incl")}</th>
        <th class="total">{t("common.total")}</th>
        <th></th>
      </tr>
    </thead>
    <tbody>
      {#each items as item, index (index)}
        <tr>
          <td>
            <input
              bind:value={item.description}
              placeholder={t("common.description")}
              aria-label={name(t("common.description"), index)}
            />
          </td>
          <td>
            <input
              bind:value={item.quantity}
              inputmode="decimal"
              aria-label={name(t("common.quantity"), index)}
            />
          </td>
          <td>
            <input
              bind:value={item.unit}
              placeholder={t("lineItems.unitPlaceholder")}
              aria-label={name(t("common.unit"), index)}
            />
          </td>
          <td>
            <input
              bind:value={item.price}
              inputmode="decimal"
              placeholder="0.00"
              aria-label={name(t("lineItems.unitPrice"), index)}
            />
          </td>
          <td>
            <select
              bind:value={item.discountType}
              aria-label={name(t("lineItems.discountType"), index)}
            >
              <option value="">{t("lineItems.none")}</option>
              <option value="percent">%</option>
              <option value="fixed">{t("lineItems.fixed")}</option>
            </select>
          </td>
          <td>
            {#if item.discountType}
              <input
                bind:value={item.discountValue}
                inputmode="decimal"
                placeholder="0"
                aria-label={name(t("lineItems.discountAmount"), index)}
              />
            {/if}
          </td>
          <td>
            <select bind:value={item.taxRateId} aria-label={name(t("lineItems.taxRate"), index)}>
              <option value="">{t("lineItems.none")}</option>
              {#each taxRates as rate (rate.id)}
                <option value={rate.id}>{rate.name}</option>
              {/each}
            </select>
          </td>
          <td class="centre">
            <input
              type="checkbox"
              bind:checked={item.taxInclusive}
              aria-label={name(t("lineItems.includesTax"), index)}
            />
          </td>
          <td class="numeric">{fromMinor(lines[index]?.totalMinor ?? 0, currency)}</td>
          <td>
            <button
              class="btn secondary"
              onclick={() => removeItem(index)}
              aria-label={name(t("action.remove"), index)}
            >
              ×
            </button>
          </td>
        </tr>
      {/each}
    </tbody>
  </table>
</div>

<div class="row actions">
  <button class="btn secondary" onclick={addItem}>{t("action.addLine")}</button>
  <button class="btn secondary" onclick={togglePicker} aria-expanded={pickerOpen}>
    {t("action.addFromCatalog")}
  </button>

  {#if pickerOpen}
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="picker"
      onkeydown={(event) => {
        if (event.key === "Escape") pickerOpen = false;
      }}
    >
      <label class="search">
        <span class="sr-only">{t("lineItems.searchCatalog")}</span>
        <!-- svelte-ignore a11y_autofocus -->
        <input
          bind:value={catalogQuery}
          placeholder={t("lineItems.searchCatalog")}
          autofocus
          spellcheck="false"
        />
      </label>

      {#if catalogLoading}
        <p class="muted note">{t("lineItems.reading")}</p>
      {:else if matches.length === 0}
        <p class="muted note">
          {t("lineItems.noMatchesStart")}
          <a class="link" href="/catalog">{t("lineItems.catalogPage")}</a>
          {t("lineItems.noMatchesEnd")}
        </p>
      {:else}
        <ul>
          {#each matches as item (item.id)}
            <li>
              <button class="option" onclick={() => pick(item)}>
                <span>{item.name}</span>
                <span class="muted">{fromMinor(item.unitPriceMinor, item.currency ?? currency)}</span>
              </button>
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  {/if}
</div>

<style>
  /* The grid is wider than a narrow window; scroll it rather than clipping. */
  .wrap {
    overflow-x: auto;
  }

  .wrap table {
    min-width: 860px;
  }

  .wide {
    min-width: 160px;
  }

  .qty,
  .unit {
    width: 70px;
  }

  .money {
    width: 100px;
  }

  .value {
    width: 90px;
  }

  .tax {
    width: 120px;
  }

  .incl {
    width: 60px;
  }

  .total {
    width: 100px;
  }

  .centre {
    text-align: center;
  }

  .actions {
    position: relative;
    margin-top: var(--space-3);
  }

  .picker {
    position: absolute;
    top: 100%;
    left: 0;
    z-index: 20;
    width: min(24rem, 100%);
    max-height: 20rem;
    overflow-y: auto;
    margin-top: var(--space-2);
    padding: var(--space-2);
    background: var(--paper-0);
    border: 1px solid var(--paper-4);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-2);
  }

  .search {
    display: block;
    margin-bottom: var(--space-2);
  }

  .picker ul {
    margin: 0;
    padding: 0;
    list-style: none;
  }

  .option {
    display: flex;
    justify-content: space-between;
    gap: var(--space-3);
    width: 100%;
    padding: var(--space-2) var(--space-3);
    font: inherit;
    text-align: left;
    color: var(--ink-1);
    background: transparent;
    border: 0;
    border-radius: var(--radius-md);
    cursor: pointer;
  }

  .option:hover {
    background: var(--paper-2);
  }

  .note {
    margin: var(--space-2) var(--space-3);
  }
</style>
