<script lang="ts">
  import Mascot from "./Mascot.svelte";

  // The one earned celebration in the product: a payment landed. It is a
  // confirmation, not a slot machine, and it respects the motion setting.
  let {
    amount,
    open = $bindable(false),
  }: { amount: string; open?: boolean } = $props();
</script>

{#if open}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
  <div class="scrim" role="presentation" onclick={() => (open = false)}>
    <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_element_interactions -->
    <div
      class="sheet rise"
      role="status"
      aria-live="polite"
      onclick={(event) => event.stopPropagation()}
    >
      <Mascot expression="celebrating" size={80} />
      <p class="amount">{amount}</p>
      <p class="receipt">stamped paid</p>
      <button class="btn secondary" onclick={() => (open = false)}>Close</button>
    </div>
  </div>
{/if}

<style>
  .scrim {
    position: fixed;
    inset: 0;
    z-index: 50;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: var(--space-4);
    background: oklch(0.24 0.014 70 / 0.3);
  }

  .sheet {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-6) var(--space-5);
    background: var(--paper-0);
    border: 1px solid var(--paper-4);
    border-radius: var(--radius-xl);
    box-shadow: var(--shadow-2);
    text-align: center;
  }

  .amount {
    margin: 0;
    font-size: var(--text-2xl);
    font-weight: 700;
    letter-spacing: -0.02em;
    font-variant-numeric: tabular-nums;
  }

  /* The stamp is the payoff: a rotated mark, like a rubber stamp on paper. */
  .receipt {
    margin: 0 0 var(--space-2);
    padding: var(--space-1) var(--space-3);
    font-size: var(--text-sm);
    font-weight: 700;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--ok);
    border: 2px solid currentColor;
    border-radius: var(--radius-sm);
    transform: rotate(-6deg);
  }

  .sheet.rise {
    --i: 0;
  }
</style>
