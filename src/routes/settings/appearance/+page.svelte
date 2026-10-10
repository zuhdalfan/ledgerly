<script lang="ts">
  import { onMount } from "svelte";
  import { api } from "$lib/api";
  import AsyncState from "$lib/components/AsyncState.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import Mascot from "$lib/components/Mascot.svelte";

  const levels = [
    { value: "none", label: "No motion", hint: "Everything appears instantly." },
    { value: "reduced", label: "Reduced", hint: "Short fades, nothing moves across the screen." },
    { value: "standard", label: "Standard", hint: "The default. Quick, and never in the way." },
    { value: "enhanced", label: "Enhanced", hint: "Longer, springier, more expressive." },
  ];

  let level = $state("standard");
  let loading = $state(true);
  let error = $state("");
  let notice = $state("");

  async function load() {
    loading = true;
    try {
      level = (await api.settings.get("ui.motion")) ?? "standard";
      error = "";
    } catch (e) {
      error = String(e);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void load();
  });

  async function choose(value: string) {
    notice = "";
    error = "";
    level = value;
    document.documentElement.dataset.motion = value;
    try {
      await api.settings.set("ui.motion", value);
      notice = `Motion set to ${value}.`;
    } catch (e) {
      error = String(e);
    }
  }
</script>

<PageHeader
  title="Appearance"
  subtitle="How much the interface moves. Your operating system setting always wins if it asks for less."
/>

<AsyncState {loading} {error} rows={4} columns={1}>
  <div class="card">
    <h2>Motion</h2>

    <fieldset class="levels">
      <legend class="sr-only">Motion level</legend>
      {#each levels as option (option.value)}
        <label class="level" class:chosen={level === option.value}>
          <input
            type="radio"
            name="motion"
            value={option.value}
            checked={level === option.value}
            onchange={() => choose(option.value)}
          />
          <span class="text">
            <span class="name">{option.label}</span>
            <span class="muted">{option.hint}</span>
          </span>
        </label>
      {/each}
    </fieldset>

    {#if notice}<p class="ok">{notice}</p>{/if}
    {#if error}<p class="error">{error}</p>{/if}
  </div>

  <div class="card preview">
    <Mascot expression={level === "enhanced" ? "celebrating" : "pleased"} size={56} />
    <p class="muted">
      Tally shows up when a list is empty, when an invoice gets paid, and when something goes
      wrong. Never on a document you send to a client.
    </p>
  </div>
</AsyncState>

<style>
  .levels {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    margin: 0;
    padding: 0;
    border: 0;
  }

  .level {
    display: flex;
    gap: var(--space-3);
    align-items: flex-start;
    padding: var(--space-3);
    border: 1px solid var(--paper-3);
    border-radius: var(--radius-md);
    cursor: pointer;
    transition: border-color var(--dur-fast) var(--ease-out), background var(--dur-fast) var(--ease-out);
  }

  .level:hover {
    background: var(--paper-2);
  }

  .level.chosen {
    border-color: var(--accent-text);
    background: var(--accent-soft);
  }

  .text {
    display: flex;
    flex-direction: column;
  }

  .name {
    font-weight: 700;
  }

  .preview {
    display: flex;
    gap: var(--space-4);
    align-items: center;
  }
</style>
