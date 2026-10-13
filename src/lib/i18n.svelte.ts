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

let locale = $state<Locale>(DEFAULT_LOCALE);

type Params = Record<string, string | number>;

function isLocale(value: string | null | undefined): value is Locale {
  return value === "en" || value === "id";
}

export function getLocale(): Locale {
  return locale;
}

/** Apply a language: updates the rune and the document for assistive tech. */
export function setLocale(next: Locale): void {
  locale = next;
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

/** Translate a key. Falls back to English, then to the key itself — never throws. */
export function t(key: MessageKey, params?: Params): string {
  const message = CATALOGS[locale][key] ?? en[key] ?? key;
  return fill(message, params);
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
  return t(count === 1 ? oneKey : otherKey, { count, ...params });
}

/** Translate a derived invoice/estimate status. */
export function statusLabel(status: string): string {
  const key = `status.${status === "partially_paid" ? "partiallyPaid" : status}` as MessageKey;
  return key in en ? t(key) : t("status.draft");
}
