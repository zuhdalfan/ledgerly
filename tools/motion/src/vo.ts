import type { BeatId } from "./timeline";

/**
 * The voiceover, one clip per line so a re-take or a retime touches one file.
 *
 * `local` is frames after the beat starts, which is what lets the same line
 * follow its beat into the 16s cutdown without a second script. `duration` is
 * the frames reserved before the next line — a clip longer than that is cut.
 *
 * Budget: about 2.2 words a second, which is a relaxed read, not an announcer.
 * The script mirrors docs/marketing/motion-brief.md §5.2. Keep them in step.
 */
export type VoLine = {
  id: string;
  beat: BeatId;
  local: number;
  duration: number;
  text: string;
};

export const VO_LINES: VoLine[] = [
  {
    id: "01-promise",
    beat: "promise",
    local: 8,
    duration: 84,
    text: "Invoicing that never touches the internet.",
  },
  {
    id: "02-invoice",
    beat: "invoice",
    local: 6,
    duration: 146,
    text: "An invoice in under a minute. Line items, tax, totals.",
  },
  {
    id: "03-paid",
    beat: "paid",
    local: 8,
    duration: 108,
    text: "Record a payment. The status updates itself.",
  },
  {
    id: "04-chase",
    beat: "chase",
    local: 8,
    duration: 100,
    text: "Chase what's late without another app.",
  },
  {
    id: "05a-close",
    beat: "tray",
    local: 4,
    duration: 44,
    text: "Close the window.",
  },
  {
    // Deliberately after the silent stretch: beat 5b is the film's only silence.
    id: "05c-remind",
    beat: "tray",
    local: 94,
    duration: 96,
    text: "Daily, it flags what's overdue.",
  },
  {
    id: "06-quote",
    beat: "quote",
    local: 10,
    duration: 126,
    text: "Quotes become invoices. Invoices become clean PDFs.",
  },
  {
    id: "07-mcp",
    beat: "mcp",
    local: 10,
    duration: 138,
    text: "Your AI can read it all, without a network.",
  },
  {
    id: "08-close",
    beat: "close",
    local: 10,
    duration: 106,
    text: "No account. No cloud. No network.",
  },
];
