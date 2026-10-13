import { beforeNavigate, goto } from "$app/navigation";
import { onMount } from "svelte";
import { ask } from "@tauri-apps/plugin-dialog";
import { t } from "./i18n.svelte";

/// Warns before a filled-in form is thrown away.
///
/// Call during component initialisation, with a getter so the guard sees live
/// state. Covers in-app navigation and the webview's own unload; it does not
/// cover the window's close button, which Tauri handles outside the page.
export function useDirtyGuard(isDirty: () => boolean, message: string) {
  let bypass = false;

  beforeNavigate(async (navigation) => {
    if (bypass || !isDirty()) return;

    navigation.cancel();
    const discard = await ask(message, { title: t("guard.discardTitle"), kind: "warning" });
    if (!discard || !navigation.to) return;

    bypass = true;
    await goto(navigation.to.url);
    bypass = false;
  });

  onMount(() => {
    const handler = (event: BeforeUnloadEvent) => {
      if (!isDirty()) return;
      event.preventDefault();
      // Older webviews still need a returnValue before they will prompt.
      event.returnValue = "";
    };

    window.addEventListener("beforeunload", handler);
    return () => window.removeEventListener("beforeunload", handler);
  });
}
