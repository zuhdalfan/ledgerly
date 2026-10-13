<script lang="ts">
  import { fromMinor } from "$lib/format";
  import { t } from "$lib/i18n.svelte";

  // Two series over time: what you billed and what actually landed. The gap
  // between them is the whole point, so the columns sit side by side.
  //
  // A column may carry a link. When it does, the whole column is an anchor with
  // the figures and the action in its accessible name, and the bars stay
  // hidden from the tree because the values are already in that name.
  let {
    points,
    currency,
    label,
    primaryName,
    secondaryName = "",
  }: {
    points: { label: string; primary: number; secondary?: number; href?: string }[];
    currency: string;
    label: string;
    primaryName: string;
    secondaryName?: string;
  } = $props();

  const peak = $derived(
    Math.max(1, ...points.flatMap((point) => [point.primary, point.secondary ?? 0]))
  );

  const share = (value: number | undefined) => `${Math.round(((value ?? 0) / peak) * 100)}%`;

  function name(point: { label: string; primary: number; secondary?: number }): string {
    const parts = [point.label, `${primaryName.toLowerCase()} ${fromMinor(point.primary, currency)}`];
    if (secondaryName) {
      parts.push(`${secondaryName.toLowerCase()} ${fromMinor(point.secondary ?? 0, currency)}`);
    }
    parts.push(t("chart.showThoseInvoices"));
    return parts.join(", ");
  }
</script>

<figure class="chart">
  <figcaption class="legend">
    <span class="key"><span class="swatch primary"></span>{primaryName}</span>
    {#if secondaryName}
      <span class="key"><span class="swatch secondary"></span>{secondaryName}</span>
    {/if}
  </figcaption>

  <div class="plot">
    {#each points as point (point.label)}
      <div class="column">
        {#if point.href}
          <a class="hit" href={point.href} aria-label={name(point)}>
            {@render column(point)}
          </a>
        {:else}
          {@render column(point)}
        {/if}
      </div>
    {/each}
  </div>

  <table class="sr-only">
    <caption>{label}</caption>
    <thead>
      <tr>
        <th>{t("common.period")}</th>
        <th>{primaryName}</th>
        {#if secondaryName}<th>{secondaryName}</th>{/if}
      </tr>
    </thead>
    <tbody>
      {#each points as point (point.label)}
        <tr>
          <td>{point.label}</td>
          <td>{fromMinor(point.primary, currency)}</td>
          {#if secondaryName}<td>{fromMinor(point.secondary ?? 0, currency)}</td>{/if}
        </tr>
      {/each}
    </tbody>
  </table>
</figure>

{#snippet column(point: { label: string; primary: number; secondary?: number })}
  <span class="track" aria-hidden="true">
    {#if secondaryName}
      <span class="bar secondary" style={`--h: ${share(point.secondary)}`}></span>
    {/if}
    <span class="bar primary" style={`--h: ${share(point.primary)}`}></span>
  </span>
  <span class="tick">{point.label}</span>
{/snippet}

<style>
  .chart {
    margin: 0;
  }

  .legend {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-4);
    margin-bottom: var(--space-3);
    font-size: var(--text-sm);
    color: var(--ink-2);
  }

  .key {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
  }

  .swatch {
    width: 0.75rem;
    height: 0.75rem;
    border-radius: var(--radius-sm);
  }

  .swatch.primary {
    background: var(--ink-1);
  }

  .swatch.secondary {
    background: var(--paper-4);
  }

  .plot {
    display: flex;
    align-items: flex-end;
    gap: var(--space-2);
    /* A baseline, so a month with no activity still reads as a month rather
     * than as a gap in the chart. */
    border-bottom: 1px solid var(--paper-3);
  }

  .column {
    flex: 1 1 0;
    display: flex;
    flex-direction: column;
  }

  .hit {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    color: inherit;
    text-decoration: none;
    border-radius: var(--radius-sm);
  }

  .hit:hover .tick,
  .hit:focus-visible .tick {
    color: var(--ink-1);
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .hit:focus-visible {
    outline: 2px solid var(--accent-text);
    outline-offset: 2px;
  }

  .track {
    display: flex;
    align-items: flex-end;
    justify-content: center;
    gap: 2px;
    width: 100%;
    height: 8rem;
  }

  .bar {
    width: min(1.25rem, 50%);
    height: var(--h, 0%);
    min-height: 0;
    border-radius: var(--radius-sm) var(--radius-sm) 0 0;
  }

  .bar.primary {
    background: var(--ink-1);
  }

  .bar.secondary {
    background: var(--paper-4);
  }

  .tick {
    align-self: center;
    font-size: var(--text-sm);
    color: var(--ink-2);
  }
</style>
