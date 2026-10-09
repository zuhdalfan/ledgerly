/** Parse a user-entered amount into integer minor units (e.g. cents). */
export function toMinor(value: string): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? Math.round(parsed * 100) : 0;
}

/** Render integer minor units as a currency string. */
export function fromMinor(minor: number, currency = "USD"): string {
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency,
  }).format(minor / 100);
}

export function nullToEmpty(value: string | null): string {
  return value ?? "";
}

export function emptyToNull(value: string): string | null {
  const trimmed = value.trim();
  return trimmed === "" ? null : trimmed;
}

export function formatDate(value: string | null): string {
  if (!value) return "—";
  const date = new Date(value.replace(" ", "T") + "Z");
  return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString();
}
