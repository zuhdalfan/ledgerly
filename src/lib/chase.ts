import { formatDate, fromMinor } from "./format";
import { getLocale, pluralFor, tFor, type Locale } from "./i18n.svelte";

export type Reminder = { subject: string; body: string };

/// Only the fields a reminder needs, so both a full invoice and the editor's
/// loose state can produce one.
export type ReminderInput = {
  number: string;
  clientName: string | null;
  currency: string;
  balanceMinor: number;
  dueDate: string | null;
  daysPastDue: number | null;
  displayStatus: string;
  paymentInstructions: string | null;
};

/// A ready-to-send reminder for an unpaid invoice.
///
/// Ledgerly has no network and no mail client, so its whole contribution to
/// chasing is making the message instant to produce. The user copies this into
/// whatever they already use.
///
/// The message goes to the *client*, so it follows the document language
/// (`pdf.locale`), not the language of the app the freelancer is reading.
export function reminder(
  invoice: ReminderInput,
  signature = "",
  locale: Locale = getLocale()
): Reminder {
  const amount = fromMinor(invoice.balanceMinor, invoice.currency, locale);
  const late = invoice.daysPastDue ?? 0;
  const overdue = invoice.displayStatus === "overdue" && late > 0;
  const days = pluralFor(locale, late, "chase.daysOne", "chase.daysOther", { count: late });

  const due = invoice.dueDate ? formatDate(invoice.dueDate, locale) : "";
  const subject = overdue
    ? tFor(locale, "chase.subjectOverdue", { number: invoice.number, days })
    : tFor(locale, "chase.subjectFor", { number: invoice.number, amount });

  const status = overdue
    ? tFor(locale, "chase.bodyOverdue", { number: invoice.number, amount, due, days })
    : due
      ? tFor(locale, "chase.bodyDue", { number: invoice.number, amount, due })
      : tFor(locale, "chase.bodyOutstanding", { number: invoice.number, amount });

  const instructions = invoice.paymentInstructions?.trim();
  const name = signature.trim();

  const lines = [
    invoice.clientName
      ? tFor(locale, "chase.helloNamed", { name: invoice.clientName })
      : tFor(locale, "chase.hello"),
    "",
    status,
    ...(instructions ? ["", tFor(locale, "chase.paymentInstructions"), instructions] : []),
    "",
    tFor(locale, "chase.ifPaid"),
    "",
    name ? tFor(locale, "chase.thanksNamed", { name }) : tFor(locale, "chase.thanks"),
  ];

  return { subject, body: lines.join("\n") };
}
