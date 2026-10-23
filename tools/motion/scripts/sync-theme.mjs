// Regenerates src/theme.ts from the app's design tokens.
//
//   node scripts/sync-theme.mjs
//
// The film must not invent a palette. This reads the :root block of
// ../../src/app.css and emits the same custom properties as a typed object, so a
// token rename in the product breaks the motion project at type-check time
// rather than drifting silently.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const sourcePath = resolve(here, "../../../src/app.css");
const outputPath = resolve(here, "../src/theme.ts");

const css = readFileSync(sourcePath, "utf8");

const start = css.indexOf(":root {");
if (start === -1) {
  console.error("sync-theme: could not find `:root {` in src/app.css");
  process.exit(1);
}
const end = css.indexOf("\n}", start);
if (end === -1) {
  console.error("sync-theme: could not find the end of the :root block");
  process.exit(1);
}
const block = css.slice(start, end);

const toCamel = (name) =>
  name
    .split("-")
    .map((part, index) => (index === 0 ? part : part.charAt(0).toUpperCase() + part.slice(1)))
    .join("");

const entries = [];
const pattern = /--([a-z0-9-]+)\s*:\s*([^;]+);/g;
for (const match of block.matchAll(pattern)) {
  const [, name, raw] = match;
  const value = raw.trim().replace(/\s+/g, " ");
  entries.push([toCamel(name), value]);
}

if (entries.length === 0) {
  console.error("sync-theme: parsed zero custom properties — the CSS shape changed");
  process.exit(1);
}

const body = entries
  .map(([key, value]) => `  ${key}: ${JSON.stringify(value)},`)
  .join("\n");

const output = `// GENERATED FILE — do not edit by hand.
//
// Source: src/app.css :root
// Regenerate: node scripts/sync-theme.mjs
//
// Values are the app's own OKLCH strings. Remotion renders in Chromium, so they
// are used verbatim; there is no second palette to keep in sync.

export const tokens = {
${body}
} as const;

export type Token = keyof typeof tokens;
`;

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, output, "utf8");
console.log(`sync-theme: wrote ${entries.length} tokens to src/theme.ts`);
