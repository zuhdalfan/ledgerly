<script lang="ts">
  import { fromMinor } from "$lib/format";
  import { emptyItem, type ItemForm } from "$lib/lineItems";
  import type { LineTotals, TaxRate } from "$lib/types";

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

  function addItem() {
    items = [...items, emptyItem()];
  }

  function removeItem(index: number) {
    const next = items.filter((_, i) => i !== index);
    items = next.length ? next : [emptyItem()];
  }
</script>

<table>
  <thead>
    <tr>
      <th style="min-width: 160px;">Description</th>
      <th style="width: 70px;">Qty</th>
      <th style="width: 70px;">Unit</th>
      <th style="width: 100px;">Price</th>
      <th style="width: 100px;">Disc.</th>
      <th style="width: 90px;">Value</th>
      <th style="width: 120px;">Tax</th>
      <th style="width: 60px;">Incl.</th>
      <th style="width: 100px;">Total</th>
      <th></th>
    </tr>
  </thead>
  <tbody>
    {#each items as item, index (index)}
      <tr>
        <td><input bind:value={item.description} placeholder="Description" /></td>
        <td><input bind:value={item.quantity} inputmode="decimal" /></td>
        <td><input bind:value={item.unit} placeholder="hour" /></td>
        <td><input bind:value={item.price} inputmode="decimal" placeholder="0.00" /></td>
        <td>
          <select bind:value={item.discountType}>
            <option value="">—</option>
            <option value="percent">%</option>
            <option value="fixed">Fixed</option>
          </select>
        </td>
        <td>
          {#if item.discountType}
            <input bind:value={item.discountValue} inputmode="decimal" placeholder="0" />
          {/if}
        </td>
        <td>
          <select bind:value={item.taxRateId}>
            <option value="">—</option>
            {#each taxRates as rate (rate.id)}
              <option value={rate.id}>{rate.name}</option>
            {/each}
          </select>
        </td>
        <td style="text-align: center;">
          <input type="checkbox" bind:checked={item.taxInclusive} style="width: auto;" />
        </td>
        <td class="numeric">{fromMinor(lines[index]?.totalMinor ?? 0, currency)}</td>
        <td>
          <button class="btn secondary" onclick={() => removeItem(index)} aria-label="Remove line">
            ×
          </button>
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<div class="row" style="margin-top: 0.75rem;">
  <button class="btn secondary" onclick={addItem}>Add line</button>
</div>
