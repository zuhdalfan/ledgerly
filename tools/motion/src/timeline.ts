// The single timing source. Every composition derives its sequence from here,
// so the 35s master and the 16s cut cannot drift apart.

import type { FC } from "react";
import { Close } from "./scenes/Close";
import { Chase } from "./scenes/Chase";
import { Invoice } from "./scenes/Invoice";
import { Mcp } from "./scenes/Mcp";
import { Paid } from "./scenes/Paid";
import { Promise } from "./scenes/Promise";
import { QuoteToPdf } from "./scenes/QuoteToPdf";
import { TrayReminder } from "./scenes/TrayReminder";

export const FPS = 30;
export const WIDTH = 1920;
export const HEIGHT = 1080;

export type BeatId =
  | "promise"
  | "invoice"
  | "paid"
  | "chase"
  | "tray"
  | "quote"
  | "mcp"
  | "close";

export type SceneProps = { headline: string };

export type Beat = {
  id: BeatId;
  headline: string;
  from: number;
  duration: number;
};

type BeatSpec = Omit<Beat, "from">;

const sequence = (specs: BeatSpec[]): Beat[] => {
  let at = 0;
  return specs.map((spec) => {
    const beat: Beat = { ...spec, from: at };
    at += spec.duration;
    return beat;
  });
};

export const SCENES: Record<BeatId, FC<SceneProps>> = {
  promise: Promise,
  invoice: Invoice,
  paid: Paid,
  chase: Chase,
  tray: TrayReminder,
  quote: QuoteToPdf,
  mcp: Mcp,
  close: Close,
};

/** 35.0s / 1050 frames at 30fps. */
export const MASTER_BEATS = sequence([
  { id: "promise", headline: "Invoicing that never phones home.", duration: 90 },
  { id: "invoice", headline: "An invoice in 42 seconds.", duration: 150 },
  { id: "paid", headline: "Paid, or partially paid. It just knows.", duration: 114 },
  { id: "chase", headline: "Chase it without leaving the app.", duration: 108 },
  { id: "tray", headline: "Or close it. It still remembers.", duration: 192 },
  { id: "quote", headline: "Quote it. Invoice it. Export it.", duration: 132 },
  { id: "mcp", headline: "An AI that reads your books — over a wire that isn't there.", duration: 144 },
  { id: "close", headline: "No account. No cloud. No network.", duration: 120 },
]);

/**
 * 17.0s / 510 frames. Keeps only one of the two overdue beats: dropping the
 * in-app chase keeps the cut feeling like invoicing, not debt collection.
 * The end card is longer than the master's proportionally so the final VO line
 * ("No account. No cloud. No network.") lands whole instead of being cut.
 */
export const CUTDOWN_BEATS = sequence([
  { id: "promise", headline: "Invoicing that never phones home.", duration: 90 },
  { id: "invoice", headline: "An invoice in 42 seconds.", duration: 118 },
  { id: "tray", headline: "Or close it. It still remembers.", duration: 192 },
  { id: "close", headline: "No account. No cloud. No network.", duration: 110 },
]);

export const MASTER_TOTAL = MASTER_BEATS.reduce((total, beat) => total + beat.duration, 0);
export const CUTDOWN_TOTAL = CUTDOWN_BEATS.reduce((total, beat) => total + beat.duration, 0);
