import { formatDate, fromMinor } from "./format";
import { t, plural } from "./i18n.svelte";

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
export function reminder(invoice: ReminderInput, signature = ""): Reminder {
  const amount = fromMinor(invoice.balanceMinor, invoice.currency);
  const late = invoice.daysPastDue ?? 0;
  const overdue = invoice.displayStatus === "overdue" && late > 0;
  const days = plural(late, "chase.daysOne", "chase.daysOther", { count: late });

  const due = invoice.dueDate ? formatDate(invoice.dueDate) : "";
  const subject = overdue
    ? t("chase.subjectOverdue", { number: invoice.number, days })
    : t("chase.subjectFor", { number: invoice.number, amount });

  const status = overdue
    ? t("chase.bodyOverdue", { number: invoice.number, amount, due, days })
    : due
      ? t("chase.bodyDue", { number: invoice.number, amount, due })
      : t("chase.bodyOutstanding", { number: invoice.number, amount });

  const instructions = invoice.paymentInstructions?.trim();
  const name = signature.trim();

  const lines = [
    invoice.clientName ? t("chase.helloNamed", { name: invoice.clientName }) : t("chase.hello"),
    "",
    status,
    ...(instructions ? ["", t("chase.paymentInstructions"), instructions] : []),
    "",
    t("chase.ifPaid"),
    "",
    name ? t("chase.thanksNamed", { name }) : t("chase.thanks"),
  ];

  return { subject, body: lines.join("\n") };
}
