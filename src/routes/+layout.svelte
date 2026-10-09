<script lang="ts">
  import "../app.css";
  import { page } from "$app/state";

  let { children } = $props();

  const nav = [
    { href: "/", label: "Dashboard" },
    { href: "/invoices", label: "Invoices" },
    { href: "/clients", label: "Clients" },
    { href: "/catalog", label: "Catalog" },
    { href: "/settings", label: "Settings" },
  ];

  function isActive(href: string): boolean {
    return href === "/" ? page.url.pathname === "/" : page.url.pathname.startsWith(href);
  }
</script>

<div class="app">
  <aside>
    <div class="brand">Ledgerly</div>
    <nav>
      {#each nav as item (item.href)}
        <a href={item.href} class:active={isActive(item.href)}>{item.label}</a>
      {/each}
    </nav>
  </aside>

  <main>
    {@render children()}
  </main>
</div>
