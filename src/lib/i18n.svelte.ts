/**
 * Tiny, offline i18n. No dependency, no network — two catalogs and a rune.
 *
 * `locale` is module-level `$state`, so any template that calls `t()` re-renders
 * when the language changes: reading the state is what subscribes the caller.
 * (A value captured once into a local `$state` would not update — always call
 * `t()` during render.)
 */
import { en, type MessageKey } from "./locales/en";
import { id } from "./locales/id";

export type Locale = "en" | "id";

/** Endonyms, deliberately not translated: a language is listed in its own words. */
export const LOCALES: { value: Locale; label: string }[] = [
  { value: "en", label: "English" },
  { value: "id", label: "Bahasa Indonesia" },
];

const CATALOGS: Record<Locale, Record<string, string>> = { en, id };

export const DEFAULT_LOCALE: Locale = "en";

// Mirrors the last applied `ui.locale` so the first paint can be right. The
// database stays authoritative: this is only a pre-paint cache, and it is what
// lets the lock gate speak the chosen language, since settings are unreadable
// until the app is unlocked.
const STORAGE_KEY = "ledgerly.ui.locale";

function storedLocale(): Locale {
  try {
    if (typeof localStorage === "undefined") return DEFAULT_LOCALE;
    const value = localStorage.getItem(STORAGE_KEY);
    return isLocale(value) ? value : DEFAULT_LOCALE;
  } catch {
    // Private mode or a disabled store: fall back to the default.
    return DEFAULT_LOCALE;
  }
}

type Params = Record<string, string | number>;

function isLocale(value: string | null | undefined): value is Locale {
  return value === "en" || value === "id";
}

const initial = storedLocale();
let locale = $state<Locale>(initial);

// `app.html` ships with lang="en"; correct it before the first paint. Reading
// `initial` rather than the rune keeps this a one-time correction, which is
// what we want — `setLocale` owns every later change.
if (typeof document !== "undefined") {
  document.documentElement.lang = initial;
  document.documentElement.dataset.locale = initial;
}

export function getLocale(): Locale {
  return locale;
}

/** Apply a language: updates the rune, the cache and the document. */
export function setLocale(next: Locale): void {
  locale = next;
  try {
    localStorage.setItem(STORAGE_KEY, next);
  } catch {
    // A failed cache write must never block the switch itself.
  }
  if (typeof document !== "undefined") {
    document.documentElement.lang = next;
    document.documentElement.dataset.locale = next;
  }
}

/** Coerce a stored setting value to a known locale, defaulting to English. */
export function localeFromSetting(value: string | null | undefined): Locale {
  return isLocale(value) ? value : DEFAULT_LOCALE;
}

function fill(message: string, params?: Params): string {
  if (!params) return message;
  return message.replace(/\{(\w+)\}/g, (whole, name: string) =>
    name in params ? String(params[name]) : whole
  );
}

/** Translate a key for an explicit locale, regardless of the UI language.
 * Client-facing text (the reminder) uses the document language instead. */
export function tFor(locale: Locale, key: MessageKey, params?: Params): string {
  const message = CATALOGS[locale][key] ?? en[key] ?? key;
  return fill(message, params);
}

/** Translate a key in the current UI language. */
export function t(key: MessageKey, params?: Params): string {
  return tFor(locale, key, params);
}

/**
 * Pick a singular or plural message in an explicit locale. Indonesian has no
 * plural inflection, so `id` holds the same text for both keys.
 */
export function pluralFor(
  locale: Locale,
  count: number,
  oneKey: MessageKey,
  otherKey: MessageKey,
  params?: Params
): string {
  return tFor(locale, count === 1 ? oneKey : otherKey, { count, ...params });
}

/**
 * Pick a singular or plural message. Indonesian has no plural inflection, so
 * `id` simply holds the same text for both keys — the two-key shape is what
 * keeps English grammatical.
 */
export function plural(
  count: number,
  oneKey: MessageKey,
  otherKey: MessageKey,
  params?: Params
): string {
  return pluralFor(locale, count, oneKey, otherKey, params);
}

/** Translate a derived invoice/estimate status. */
export function statusLabel(status: string): string {
  const key = `status.${status === "partially_paid" ? "partiallyPaid" : status}` as MessageKey;
  return key in en ? t(key) : t("status.draft");
}
