<script lang="ts">
  import type { Snippet } from "svelte";
  import Mascot, { type Mood } from "./Mascot.svelte";

  // The empty state is the onboarding. Say what belongs here, and what fills it.
  let {
    title,
    body = "",
    mood = "idle",
    children,
  }: {
    title: string;
    body?: string;
    mood?: Mood;
    children?: Snippet;
  } = $props();
</script>

<div class="empty">
  <Mascot expression={mood} size={64} />
  <p class="title">{title}</p>
  {#if body}<p class="muted">{body}</p>{/if}
  {#if children}
    <div class="action">{@render children()}</div>
  {/if}
</div>

<style>
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-6) var(--space-4);
    text-align: center;
  }

  .title {
    margin: 0;
    font-size: var(--text-lg);
    font-weight: 700;
    text-wrap: balance;
  }

  .empty :global(.muted) {
    margin: 0 auto;
    max-width: 46ch;
  }

  .action {
    margin-top: var(--space-2);
  }
</style>
