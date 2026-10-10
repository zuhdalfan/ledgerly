<script lang="ts">
  let { rows = 4, columns = 1 }: { rows?: number; columns?: number } = $props();
</script>

<div class="skeleton" aria-hidden="true">
  {#each Array(rows) as _, row (row)}
    <div class="skeleton-row">
      {#each Array(columns) as _, column (column)}
        <span class="skeleton-bar"></span>
      {/each}
    </div>
  {/each}
</div>

<style>
  .skeleton {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .skeleton-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(80px, 1fr));
    gap: var(--space-3);
  }

  .skeleton-bar {
    height: 12px;
    border-radius: var(--radius-sm);
    background: var(--paper-2);
  }

  /* First bar of each row stands in for the label column. */
  .skeleton-bar:first-child {
    background: var(--paper-3);
  }

  @media (prefers-reduced-motion: no-preference) {
    .skeleton-bar {
      animation: pulse 1.4s var(--ease-out) infinite;
    }

    @keyframes pulse {
      0%,
      100% {
        opacity: 1;
      }
      50% {
        opacity: 0.55;
      }
    }
  }
</style>
