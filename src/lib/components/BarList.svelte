<script lang="ts">
  import { fromMinor } from "$lib/format";
  import { t, plural } from "$lib/i18n.svelte";

  // A bar list rather than an abstract chart: the label and the number are the
  // accessible content, the bar is the reading aid. A row may carry a link, in
  // which case the link is a real anchor with the action in its name.
  let {
    rows,
    currency,
    label,
    columnLabel = "",
  }: {
    rows: {
      label: string;
      amountMinor: number;
      count?: number;
      href?: string;
      tone?: "ink" | "danger";
    }[];
    currency: string;
    label: string;
    columnLabel?: string;
  } = $props();

  const peak = $derived(Math.max(1, ...rows.map((row) => row.amountMinor)));
</script>

<table aria-label={label}>
  <thead>
    <tr>
      <th>{t("common.age")}</th>
      <th class="numeric">{t("common.balance")}</th>
      <th class="share-head"><span class="sr-only">{columnLabel || t("barlist.share")}</span></th>
    </tr>
  </thead>
  <tbody>
    {#each rows as row (row.label)}
      <tr>
        <td>
          {#if row.href}
            <a
              class="link"
              href={row.href}
              aria-label={plural(row.count ?? 0, "barlist.rowAriaOne", "barlist.rowAriaOther", {
                label: row.label,
                amount: fromMinor(row.amountMinor, currency),
              })}
            >
              {row.label}
            </a>
          {:else}
            {row.label}
          {/if}
          {#if row.count !== undefined}
            <span class="muted"> · {row.count}</span>
          {/if}
        </td>
        <td class="numeric">{fromMinor(row.amountMinor, currency)}</td>
        <td class="share">
          {#if row.amountMinor > 0}
            <span
              class={`bar ${row.tone ?? "ink"}`}
              style={`--w: ${Math.round((row.amountMinor / peak) * 100)}%`}
            ></span>
          {/if}
        </td>
      </tr>
    {/each}
  </tbody>
</table>

<style>
  .share-head,
  .share {
    width: 40%;
  }

  .share {
    vertical-align: middle;
  }

  .bar {
    display: block;
    height: 0.5rem;
    width: var(--w, 0%);
    min-width: 2px;
    border-radius: var(--radius-full);
    background: var(--ink-1);
  }

  .bar.danger {
    background: var(--danger);
  }
</style>
