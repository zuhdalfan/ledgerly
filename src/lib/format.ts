import { getLocale, type Locale } from "./i18n.svelte";

/**
 * Read a user-entered decimal using either convention.
 *
 * The UI shows amounts in the UI language (`1.234,56` in Indonesian), while
 * people also paste figures from English spreadsheets (`1,234.56`). A lone
 * separator is a decimal point when it reads like one (one or two trailing
 * digits); three trailing digits are grouping. Returns `null` when there are
 * no digits at all.
 */
function parseDecimal(value: string): number | null {
  const locale = getLocale();
  const decimal = locale === "id" ? "," : ".";
  const grouping = decimal === "," ? "." : ",";

  const cleaned = value.replace(/[^\d.,+-]/g, "");
  const negative = cleaned.trimStart().startsWith("-");
  const text = cleaned.replace(/[+-]/g, "");
  if (!/\d/.test(text)) return null;

  const count = (char: string) => text.split(char).length - 1;
  const lastDecimal = text.lastIndexOf(decimal);
  const lastGrouping = text.lastIndexOf(grouping);

  let decimalAt = -1;
  if (lastDecimal >= 0 && lastGrouping >= 0) {
    // Both conventions are present: the final separator is the decimal point.
    decimalAt = Math.max(lastDecimal, lastGrouping);
  } else if (lastDecimal >= 0) {
    // The language's own decimal point is trusted, even with three digits.
    decimalAt = count(decimal) === 1 ? lastDecimal : -1;
  } else if (lastGrouping >= 0) {
    // The other convention: only a decimal point when it reads like one.
    decimalAt = count(grouping) === 1 && text.length - lastGrouping - 1 <= 2 ? lastGrouping : -1;
  }

  const whole = decimalAt < 0 ? text : text.slice(0, decimalAt);
  const fraction = decimalAt < 0 ? "" : text.slice(decimalAt + 1);
  const wholeDigits = whole.replace(/[.,]/g, "");
  const fractionDigits = fraction.replace(/[.,]/g, "");
  if (!wholeDigits && !fractionDigits) return null;

  const amount = Number.parseFloat(`${wholeDigits || "0"}.${fractionDigits || "0"}`);
  if (!Number.isFinite(amount)) return null;
  return negative ? -amount : amount;
}

/** Parse a user-entered amount into integer minor units (e.g. cents). */
export function toMinor(value: string): number {
  const parsed = parseDecimal(value);
  return parsed === null ? 0 : Math.round(parsed * 100);
}

/**
 * Normalise a decimal the Rust side will parse: quantities and percentages are
 * sent as text, and `Decimal::from_str` only understands a dot. Without this,
 * `1,5` silently became `1` (or was dropped entirely).
 */
export function normalizeDecimal(value: string): string {
  const parsed = parseDecimal(value);
  return parsed === null ? value.trim() : String(parsed);
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
export function fromMinor(minor: number, currency = "USD", locale: Locale = getLocale()): string {
  return formatterFor(locale, currency).format(minor / 100);
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

export function formatDate(value: string | null, locale: Locale = getLocale()): string {
  if (!value) return "—";
  const date = new Date(value.replace(" ", "T") + "Z");
  return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString(locale);
}

// A status label is only meaningful with a language, so it lives in the i18n
// module and is re-exported here for the existing `$lib/format` imports.
export { statusLabel } from "./i18n.svelte";
