<script lang="ts">
  import { SUPPORT_URL } from "$lib/support";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import Mascot from "$lib/components/Mascot.svelte";

  const year = new Date().getFullYear();
</script>

<PageHeader
  title="Support"
  subtitle="Ledgerly has no paid tier, no account and no upsell. Every feature is yours."
/>

<div class="card">
  <h2>This app is free</h2>
  <p class="muted">
    Every feature is unlocked: reports, all PDF templates, scheduled backups, batch
    export, multi-currency. There is no Pro, no trial, no licence key, and nothing to
    activate. Nothing about the app phones home.
  </p>
  <p class="muted">
    It is open source under the MIT licence, so you can read it, change it and share it.
  </p>
</div>

<div class="card support">
  <Mascot expression="waving" size={72} />
  <div>
    <h2>Keep it going</h2>
    <p class="muted">
      If Ledgerly saves you time, you can buy me a coffee. It is entirely optional and
      it changes nothing inside the app.
    </p>
    {#if SUPPORT_URL}
      <a class="btn" href={SUPPORT_URL} rel="noreferrer">Buy me a coffee</a>
    {:else}
      <button class="btn" disabled>Buy me a coffee</button>
      <p class="muted note">
        Not wired up yet: no donation page has been chosen, so this button is switched
        off rather than pointing somewhere wrong. See <code>src/lib/support.ts</code>.
      </p>
    {/if}
  </div>
</div>

<div class="card">
  <h2>Licence</h2>
  <p class="muted">
    MIT Licence. Copyright (c) {year} Muhammad Zuhdi. You may use, copy, modify, merge,
    publish, distribute, sublicense and sell copies of this software, provided the
    copyright notice and permission notice are included. It comes with no warranty.
  </p>
  <p class="muted">
    That permits commercial use, including selling the software. The full text is in the
    <code>LICENSE</code> file at the root of the repository.
  </p>
</div>

<style>
  .support {
    display: flex;
    align-items: center;
    gap: var(--space-5);
  }

  .note {
    margin-top: var(--space-2);
    font-size: var(--text-sm);
  }

  @media (max-width: 640px) {
    .support {
      flex-direction: column;
      align-items: flex-start;
    }
  }
</style>
