/** Parse a user-entered amount into integer minor units (e.g. cents). */
export function toMinor(value: string): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) ? Math.round(parsed * 100) : 0;
}

/** Render integer minor units as a currency string. The ISO code is always
 * shown, so mixed-currency lists read unambiguously ("USD 3,000.00", never a
 * bare "$" that could mean several dollars). */
export function fromMinor(minor: number, currency = "USD"): string {
  return new Intl.NumberFormat(undefined, {
    style: "currency",
    currency,
    currencyDisplay: "code",
  }).format(minor / 100);
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
    case "accepted":
      return "Accepted";
    case "declined":
      return "Declined";
    case "converted":
      return "Converted";
    case "expired":
      return "Expired";
    default:
      return "Draft";
  }
}

export function formatDate(value: string | null): string {
  if (!value) return "—";
  const date = new Date(value.replace(" ", "T") + "Z");
  return Number.isNaN(date.getTime()) ? value : date.toLocaleDateString();
}
