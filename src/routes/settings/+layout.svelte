<script lang="ts">
  import { page } from "$app/state";
  import { t } from "$lib/i18n.svelte";

  let { children } = $props();

  const sections = [
    { href: "/settings", key: "settings.nav.business" },
    { href: "/settings/invoicing", key: "settings.nav.invoicing" },
    { href: "/settings/security", key: "settings.nav.security" },
    { href: "/settings/ai", key: "settings.nav.ai" },
    { href: "/settings/data", key: "settings.nav.data" },
    { href: "/settings/support", key: "settings.nav.support" },
    { href: "/settings/appearance", key: "settings.nav.appearance" },
    { href: "/settings/about", key: "settings.nav.about" },
  ] as const;

  function isActive(href: string): boolean {
    return href === "/settings" ? page.url.pathname === "/settings" : page.url.pathname === href;
  }
</script>

<nav class="sections" aria-label={t("settings.sectionsLabel")}>
  {#each sections as section (section.href)}
    <a href={section.href} class:active={isActive(section.href)} aria-current={isActive(section.href) ? "page" : undefined}>
      {t(section.key)}
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
