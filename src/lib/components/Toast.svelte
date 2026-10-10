<script lang="ts">
  // Recoverable actions get an undo rather than a confirmation. The toast says
  // what happened, in the past tense, and offers the way back.
  let {
    open = $bindable(false),
    message = "",
    actionLabel = "Undo",
    onaction,
  }: {
    open?: boolean;
    message?: string;
    actionLabel?: string;
    onaction?: () => void;
  } = $props();
</script>

{#if open}
  <div class="toast rise" role="status" aria-live="polite">
    <span class="message">{message}</span>
    {#if onaction}
      <button
        class="undo"
        onclick={() => {
          open = false;
          onaction();
        }}
      >
        {actionLabel}
      </button>
    {/if}
    <button class="dismiss" onclick={() => (open = false)} aria-label="Dismiss">×</button>
  </div>
{/if}

<style>
  .toast {
    position: fixed;
    left: 50%;
    bottom: var(--space-5);
    z-index: 45;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    max-width: min(32rem, calc(100vw - 2rem));
    padding: var(--space-3) var(--space-4);
    transform: translateX(-50%);
    background: var(--ink-1);
    color: var(--ink-inverse);
    border-radius: var(--radius-full);
    box-shadow: var(--shadow-2);
  }

  .message {
    font-size: var(--text-md);
  }

  .undo {
    font: inherit;
    font-size: var(--text-md);
    font-weight: 700;
    color: var(--accent);
    background: transparent;
    border: 0;
    cursor: pointer;
    text-decoration: underline;
    text-underline-offset: 2px;
  }

  .dismiss {
    font: inherit;
    font-size: var(--text-lg);
    line-height: 1;
    color: var(--ink-inverse);
    background: transparent;
    border: 0;
    cursor: pointer;
  }
</style>
