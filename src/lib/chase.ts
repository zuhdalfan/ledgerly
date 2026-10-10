import { formatDate, fromMinor } from "./format";

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
  const days = `${late} ${late === 1 ? "day" : "days"}`;

  const due = invoice.dueDate ? formatDate(invoice.dueDate) : "";
  const subject = overdue
    ? `Invoice ${invoice.number} is ${days} overdue`
    : `Invoice ${invoice.number} for ${amount}`;

  const status = overdue
    ? `This is a reminder that invoice ${invoice.number} for ${amount} was due on ${due} and is now ${days} overdue.`
    : due
      ? `A reminder that invoice ${invoice.number} for ${amount} is due on ${due}.`
      : `A reminder that invoice ${invoice.number} for ${amount} is still outstanding.`;

  const instructions = invoice.paymentInstructions?.trim();
  const name = signature.trim();

  const lines = [
    invoice.clientName ? `Hello ${invoice.clientName},` : "Hello,",
    "",
    status,
    ...(instructions ? ["", "Payment instructions:", instructions] : []),
    "",
    "If this has already been paid, thank you, and please ignore this note.",
    "",
    name ? `Thanks,\n${name}` : "Thanks,",
  ];

  return { subject, body: lines.join("\n") };
}
