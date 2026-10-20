/**
 * Turn a backend error into a sentence the reader can understand.
 *
 * The Rust core speaks English and has no notion of the UI language: teaching
 * it one would duplicate the catalogs and make the MCP sidecar locale-dependent
 * for no reason. Instead the IPC boundary (`api.ts`) maps the small, stable set
 * of validation messages and the error-class prefixes here.
 *
 * Anything unmatched is shown as-is. That is deliberate: a raw detail is more
 * useful than a wrong generic sentence, and it makes an untranslated message
 * obvious the first time it appears.
 */
import { t } from "./i18n.svelte";
import type { MessageKey } from "./locales/en";

/** Messages whose wording is fixed, mapped to the key that says them locally. */
const EXACT: Record<string, MessageKey> = {
  "Ledgerly is locked. Unlock to continue.": "errors.locked",
  "That passphrase is not correct.": "errors.passphrase",
  "The stored lock verifier is unreadable.": "errors.generic",
  "That image is larger than 2 MB. Please choose a smaller file.": "errors.logoSize",
  "That file is not a PNG or JPEG image.": "errors.logoType",
  "Estimate not found.": "errors.notFound",
  "Payment not found.": "errors.notFound",
  "The converted invoice could not be read.": "errors.generic",
  "This estimate has already been converted to an invoice.": "errors.estimateConverted",
  "Payment amount must be greater than zero.": "errors.paymentPositive",
  "Payment is more than the outstanding balance. Split it into smaller payments.":
    "errors.paymentTooLarge",
  "This file is not a Ledgerly backup (no manifest).": "errors.backupInvalid",
  "The backup manifest is unreadable.": "errors.backupInvalid",
  "The backup does not contain a database.": "errors.backupInvalid",
};

type Pattern = {
  test: RegExp;
  key: MessageKey;
  params?: (match: RegExpMatchArray) => Record<string, string>;
};

/** Messages that carry a value (a length, a pixel size, a format version). */
const PATTERNS: Pattern[] = [
  {
    test: /^Use at least (\d+) characters for the passphrase\.$/,
    key: "errors.passphraseLength",
    params: (match) => ({ count: match[1] }),
  },
  {
    test: /^That image is (\d+)×(\d+) pixels; the limit is (\d+) on a side\.$/,
    key: "errors.logoPixels",
    params: (match) => ({ width: match[1], height: match[2], limit: match[3] }),
  },
  {
    test: /^This backup was made by a newer version of Ledgerly/,
    key: "errors.backupNewer",
  },
  {
    test: /^'.+' is not a status you can set\.$/,
    key: "errors.generic",
  },
];

export function errorText(error: unknown): string {
  const raw =
    typeof error === "string"
      ? error
      : error instanceof Error
        ? error.message
        : String(error ?? "");
  const message = raw.trim();

  if (!message) return t("errors.generic");

  const exact = EXACT[message];
  if (exact) return t(exact);

  for (const pattern of PATTERNS) {
    const match = message.match(pattern.test);
    if (match) return t(pattern.key, pattern.params?.(match));
  }

  // The classes below can carry SQL text, paths or platform detail. The reader
  // gets the class of failure; the raw text would only be noise.
  if (message.startsWith("database error:")) {
    if (/Query returned no rows/i.test(message)) return t("errors.notFound");
    if (/UNIQUE constraint/i.test(message)) return t("errors.duplicate");
    if (/FOREIGN KEY constraint/i.test(message)) return t("errors.inUse");
    return t("errors.database");
  }
  if (message.startsWith("io error:")) return t("errors.filesystem");
  if (message.startsWith("pdf error:")) return t("errors.pdf");
  if (message.startsWith("backup error:")) return t("errors.backup");
  if (message.startsWith("tauri error:")) return t("errors.generic");

  return message;
}
