import { getLocale } from "./i18n.svelte";

/** Parse a user-entered amount into integer minor units (e.g. cents). */
export function toMinor(value: string): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? Math.round(parsed * 100) : 0;
}

// Number formatting is locale-dependent and hot (every row of every table), so
// the formatters are cached per locale+currency rather than rebuilt per call.
const moneyFormatters = new Map<string, Intl.NumberFormat>();

function formatterFor(locale: string, currency: string): Intl.NumberFormat {
  const key = `${locale}|${currency}`;
  let formatter = moneyFormatters.get(key);
  if (!formatter) {
    formatter = new Intl.NumberFormat(locale, {
      style: "currency",
      currency,
      currencyDisplay: "code",
    });
    moneyFormatters.set(key, formatter);
  }
  return formatter;
}

/** Render integer minor units as a currency string. The ISO code is always
 * shown, so mixed-currency lists read unambiguously ("USD 3,000.00", never a
 * bare "$" that could mean several dollars). */
export function fromMinor(minor: number, currency = "USD"): string {
  return formatterFor(getLocale(), currency).format(minor / 100);
}

export function nullToEmpty(value: string | null): string {
  return value ?? "";
}

export function emptyToNull(value: string): string | null {
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}

/** Plain decimal for spreadsheet cells — no symbol or grouping. */
export function minorToCsv(minor: number): string {
  return minorToInput(minor);
}

/** Render minor units as a plain decimal string for an input field. */
export function minorToInput(minor: number): string {
  return (minor / 100).toFixed(2);
}

/** Today as YYYY-MM-DD in the local timezone (not UTC). */
export function localDate(): string {
  const now = new Date();
  const local = new Date(now.getTime() - now.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 10);
}

export function formatDate(value: string | null): string {
  if (!value) return "—";
  const date = new Date(value.replace(" ", "T") + "Z");
  return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString(getLocale());
}

// A status label is only meaningful with a language, so it lives in the i18n
// module and is re-exported here for the existing `$lib/format` imports.
export { statusLabel } from "./i18n.svelte";
