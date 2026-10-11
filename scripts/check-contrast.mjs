// Checks every colour pair the interface actually renders, against WCAG 2.
//
//   node scripts/check-contrast.mjs
//
// The palette is authored in OKLCH, so this converts each token to sRGB
// relative luminance and measures the real ratio, rather than trusting that
// the chosen lightness values "look" far enough apart.
//
// Thresholds: 4.5:1 for normal text, 3:1 for large text and for UI boundaries
// (WCAG 2.1 AA).

import { readFileSync } from "node:fs";

const ROOT = new URL("..", import.meta.url).pathname.replace(/^\/([A-Za-z]:)/, "$1");

/** OKLCH -> OKLab -> linear sRGB -> WCAG relative luminance. */
function luminance(l, c, hDeg) {
  const h = (hDeg * Math.PI) / 180;
  const a = c * Math.cos(h);
  const b = c * Math.sin(h);

  const lp = l + 0.3963377774 * a + 0.2158037573 * b;
  const mp = l - 0.1055613458 * a - 0.0638541728 * b;
  const sp = l - 0.0894841775 * a - 1.291485548 * b;

  const lc = lp ** 3;
  const mc = mp ** 3;
  const sc = sp ** 3;

  const r = 4.0767416621 * lc - 3.3077115913 * mc + 0.2309699292 * sc;
  const g = -1.2684380046 * lc + 2.6097574011 * mc - 0.3413193965 * sc;
  const bl = -0.0041960863 * lc - 0.7034186147 * mc + 1.707614701 * sc;

  const clamp = (value) => Math.min(1, Math.max(0, value));
  return 0.2126 * clamp(r) + 0.7152 * clamp(g) + 0.0722 * clamp(bl);
}

function contrast(foreground, background) {
  const a = luminance(...foreground);
  const b = luminance(...background);
  const [hi, lo] = a > b ? [a, b] : [b, a];
  return (hi + 0.05) / (lo + 0.05);
}

// --- read the tokens straight out of the stylesheet -------------------------

const css = readFileSync(`${ROOT}/src/app.css`, "utf8");
const tokens = new Map();
const pattern = /--([a-z0-9-]+):\s*oklch\(([\d.]+)\s+([\d.]+)\s+([\d.]+)\)/g;
let match;
while ((match = pattern.exec(css)) !== null) {
  tokens.set(match[1], [Number(match[2]), Number(match[3]), Number(match[4])]);
}

if (tokens.size === 0) {
  console.error("no OKLCH tokens found in src/app.css");
  process.exit(1);
}

// --- the pairs the interface really renders ---------------------------------

const TEXT = 4.5;
const LARGE = 3;
const UI = 3;

const pairs = [
  ["ink-1", "paper-0", TEXT, "body text on a card"],
  ["ink-1", "paper-1", TEXT, "body text on the canvas"],
  ["ink-1", "paper-2", TEXT, "body text on an inset"],
  ["ink-2", "paper-0", TEXT, "labels and table headers"],
  ["ink-2", "paper-1", TEXT, "nav links"],
  ["ink-3", "paper-0", TEXT, "muted text"],
  ["ink-3", "paper-1", TEXT, "muted text on the canvas"],
  ["ink-3", "paper-2", TEXT, "muted text on an inset"],
  ["ink-inverse", "ink-1", TEXT, "primary button label"],
  ["ink-inverse", "accent-fill", TEXT, "active nav label"],
  ["accent-text", "paper-0", TEXT, "links"],
  ["accent-text", "paper-1", TEXT, "links on the canvas"],
  ["accent-text", "paper-2", TEXT, "links on an inset"],
  ["ok", "paper-0", TEXT, "paid badge, success text"],
  ["warn", "paper-0", TEXT, "partially paid badge"],
  ["danger", "paper-0", TEXT, "overdue badge, error text"],
  ["info", "paper-0", TEXT, "sent badge"],
  ["neutral", "paper-0", TEXT, "void badge"],
  ["accent-state", "paper-0", TEXT, "converted badge"],
  ["line-control", "paper-0", UI, "input and select border"],
  ["line-control", "paper-2", UI, "input border on an inset"],
  ["ink-3", "paper-0", UI, "control border on hover"],
  ["accent-text", "paper-0", UI, "focus ring"],
  ["accent", "paper-0", UI, "mascot accent rule"],
  ["paper-4", "ink-1", TEXT, "toast action on the dark toast"],
];

// Decorative edges. WCAG 1.4.11 covers the visual information needed to
// identify a control or understand content, so a card edge or a table rule is
// not held to 3:1. Reported so the numbers are visible, not enforced.
const informational = [
  ["paper-3", "paper-0", "card edge on a card"],
  ["paper-3", "paper-1", "table rule on the canvas"],
  ["paper-4", "paper-0", "secondary button edge"],
];

const results = [];
let failures = 0;

for (const [fg, bg, threshold, label] of pairs) {
  const foreground = tokens.get(fg);
  const background = tokens.get(bg);
  if (!foreground || !background) continue;

  const ratio = contrast(foreground, background);
  const pass = ratio >= threshold;
  if (!pass) failures += 1;
  results.push({ label, fg, bg, ratio, threshold, pass });
}

const width = Math.max(...results.map((row) => row.label.length));
for (const row of results) {
  const mark = row.pass ? "pass" : "FAIL";
  console.log(
    `${mark}  ${row.label.padEnd(width)}  ${row.ratio.toFixed(2)}:1  (needs ${row.threshold}:1)  ${row.fg} on ${row.bg}`
  );
}

if (failures > 0) {
  console.error(`\nContrast check FAILED: ${failures} pair(s) below threshold.`);
  process.exit(1);
}

console.log("\nDecorative, not held to a threshold:");
for (const [fg, bg, label] of informational) {
  const foreground = tokens.get(fg);
  const background = tokens.get(bg);
  if (!foreground || !background) continue;
  console.log(`  ${label.padEnd(width)}  ${contrast(foreground, background).toFixed(2)}:1`);
}

console.log(`\nContrast check passed: ${results.length} pairs, all above their threshold.`);
