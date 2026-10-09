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

/** Today as YYYY-MM-DD in the local timezone (not UTC). */
export function localDate(): string {
  const now = new Date();
  const local = new Date(now.getTime() - now.getTimezoneOffset() * 60_000);
  return local.toISOString().slice(0, 10);
}

/** Human label for a derived invoice status. */
export function statusLabel(status: string): string {
  switch (status) {
    case "partially_paid":
      return "Partially paid";
    case "overdue":
      return "Overdue";
    case "paid":
      return "Paid";
    case "sent":
      return "Sent";
    case "void":
      return "Void";
    default:
      return "Draft";
  }
}

export function formatDate(value: string | null): string {
  if (!value) return "—";
  const date = new Date(value.replace(" ", "T") + "Z");
  return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString();
}
