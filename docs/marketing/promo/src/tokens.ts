// Ledgerly promo — design tokens.
//
// Every value here is copied from the product's own design foundation
// (src/app.css) so the film's visual language grows out of the app rather than
// being bolted on. Comments record the product's rationale, not ours.
import { Easing } from 'remotion';

// Paper ramp — the canvas. Warm, never pure white, and only just tinted.
export const C = {
  paper0: 'oklch(0.995 0.002 80)',
  paper1: 'oklch(0.98 0.006 80)',
  paper2: 'oklch(0.965 0.008 80)',
  paper3: 'oklch(0.92 0.012 80)',
  paper4: 'oklch(0.86 0.014 80)',
  lineControl: 'oklch(0.62 0.012 70)',

  // Ink ramp — three levels, well separated, so hierarchy survives with the
  // hue stripped out.
  ink1: 'oklch(0.24 0.014 70)',
  ink2: 'oklch(0.42 0.014 70)',
  ink3: 'oklch(0.52 0.012 70)',
  inkInverse: 'oklch(0.99 0.002 80)',

  // Accent — charcoal. Emphasis comes from weight and placement, not hue.
  accent: 'oklch(0.30 0.006 75)',
  accentSoft: 'oklch(0.93 0.006 75)',

  // State vocabulary. The only colour in the film, and only where the app
  // itself uses it (invoice status badges).
  ok: 'oklch(0.50 0.10 155)',
  warn: 'oklch(0.52 0.11 70)',
  danger: 'oklch(0.50 0.17 25)',
  info: 'oklch(0.48 0.10 250)',
  neutral: 'oklch(0.52 0.012 70)',
  accentState: 'oklch(0.48 0.13 300)',

  // The lights-down field for the hero shot: warm black, hue preserved.
  stage: 'oklch(0.14 0.012 70)',
} as const;

export const FONT = 'Inter, ui-sans-serif, system-ui, -apple-system, "Segoe UI", sans-serif';

// Motion personality: low energy, serious tone — "professional trust", nudged
// warm. Product motion is short, functional and decelerates sharply; an
// overshoot would read as a toy rather than a tool.
export const M = {
  durFast: 12,
  durBase: 21,
  durSlow: 48,
  easeTrust: Easing.bezier(0, 0, 0.2, 1),
  easeOut: Easing.bezier(0.22, 1, 0.36, 1),
  // Only for things that physically land (card reseat, wordmark stamp).
  easeLand: Easing.bezier(0.2, 1.25, 0.3, 1),
  hold: 30,
} as const;

// Type scale (output px at 1920x1080). Readable text only ever uses `caption`
// and up (aesthetic rule Q11: captions >=5% of frame height).
export const T = {
  hero: 132,
  h1: 92,
  h2: 64,
  caption: 58,
  label: 30,
} as const;

export const tracking = (size: number) => `${(-0.022 * Math.min(1, size / 92)).toFixed(4)}em`;
