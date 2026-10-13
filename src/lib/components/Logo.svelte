<script lang="ts">
  // The brand mark: a torn paper slip, the same artifact Tally is made from.
  // Monochrome and sized in px so it sits at any inline or hero scale.
  let { size = 24, label = "Ledgerly" }: { size?: number; label?: string } = $props();
</script>

<svg class="logo" width={size} height={size} viewBox="0 0 96 96" role="img" aria-label={label}>
  <path
    class="slip"
    d="M24 22A10 10 0 0 1 34 12H62A10 10 0 0 1 72 22V62L66 68 60 62 54 68 48 62 42 68 36 62 30 68 24 62Z"
  />
  <path class="rule" d="M36 31H60" />
  <path class="rule" d="M36 43H60" />
</svg>

<style>
  .logo {
    display: block;
    flex: 0 0 auto;
  }

  .slip {
    fill: currentColor;
  }

  .rule {
    stroke: var(--paper-0);
    stroke-width: 6;
    stroke-linecap: round;
  }
</style>
