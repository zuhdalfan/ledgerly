<script lang="ts">
  import { page } from "$app/state";

  let { children } = $props();

  const sections = [
    { href: "/settings", label: "Business" },
    { href: "/settings/invoicing", label: "Invoicing" },
    { href: "/settings/security", label: "Security" },
    { href: "/settings/data", label: "Data" },
    { href: "/settings/licence", label: "Licence" },
    { href: "/settings/appearance", label: "Appearance" },
    { href: "/settings/about", label: "About" },
  ];

  function isActive(href: string): boolean {
    return href === "/settings" ? page.url.pathname === "/settings" : page.url.pathname === href;
  }
</script>

<nav class="sections" aria-label="Settings sections">
  {#each sections as section (section.href)}
    <a href={section.href} class:active={isActive(section.href)} aria-current={isActive(section.href) ? "page" : undefined}>
      {section.label}
    </a>
  {/each}
</nav>

{@render children()}

<style>
  .sections {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-1);
    margin-bottom: var(--space-5);
    padding-bottom: var(--space-3);
    border-bottom: 1px solid var(--paper-3);
  }

  .sections a {
    padding: var(--space-2) var(--space-3);
    border-radius: var(--radius-full);
    color: var(--ink-2);
    font-size: var(--text-md);
    font-weight: 700;
    text-decoration: none;
    transition: background var(--dur-fast) var(--ease-out), color var(--dur-fast) var(--ease-out);
  }

  .sections a:hover {
    background: var(--paper-2);
    color: var(--ink-1);
  }

  .sections a.active {
    background: var(--ink-1);
    color: var(--ink-inverse);
  }
</style>
