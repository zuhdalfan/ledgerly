// Demo data only. Never the owner's real business, clients or balances.
const SGD = new Intl.NumberFormat("en-SG", {
  style: "currency",
  currency: "SGD",
  currencyDisplay: "code",
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

export const money = (value: number): string => SGD.format(value);

export const DEMO = {
  business: "Northwind Studio",
  client: "Acme Corporation",
  invoice: "INV-2026-0042",
  estimate: "EST-2026-0011",
  currency: "SGD",
} as const;

/** The film's only call to action. Read from the git remote, not invented. */
export const CTA = {
  label: "github.com/zuhdalfan/ledgerly",
  url: "https://github.com/zuhdalfan/ledgerly",
} as const;
