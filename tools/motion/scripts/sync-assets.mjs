// Copies the fonts and brand mark the app already ships into Remotion's public
// directory, and writes the audio manifest from whatever is actually on disk.
//
//   node scripts/sync-assets.mjs
//
// Static files must live under public/ for staticFile() to serve them. The audio
// manifest is generated rather than hand-written so a missing track can never
// break a render — the film simply plays without it.

import { copyFileSync, mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const repo = resolve(here, "../../../");
const publicDir = resolve(here, "../public");

mkdirSync(resolve(publicDir, "fonts"), { recursive: true });

const copies = [
  ["static/fonts/Inter-Regular.ttf", "public/fonts/Inter-Regular.ttf"],
  ["static/fonts/Inter-Bold.ttf", "public/fonts/Inter-Bold.ttf"],
  ["static/fonts/OFL.txt", "public/fonts/OFL.txt"],
  ["static/logo.svg", "public/logo.svg"],
];

for (const [from, to] of copies) {
  copyFileSync(resolve(repo, from), resolve(here, "..", to));
}

// A missing font would otherwise surface as a silent fallback in the render.
const regular = readFileSync(resolve(publicDir, "fonts/Inter-Regular.ttf"));
if (regular.byteLength < 10_000) {
  console.error("sync-assets: Inter-Regular.ttf looks wrong — check static/fonts/");
  process.exit(1);
}

// --- Audio -----------------------------------------------------------------
// Music is a single bed. VO lines are per-clip; a real read in public/audio/vo
// wins over the generated scratch read in public/audio/vo-scratch.
const isFile = (path) => {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
};

const musicFile = ["music.mp3", "music.m4a", "music.wav", "music.ogg"].find((name) =>
  isFile(resolve(publicDir, "audio", name))
);
const music = musicFile ? `audio/${musicFile}` : null;

// The ids live in src/vo.ts; parsing them keeps this script from owning a second
// copy of the script.
const voSource = readFileSync(resolve(here, "../src/vo.ts"), "utf8");
const voIds = [...voSource.matchAll(/^\s*id: "([^"]+)",$/gm)].map((match) => match[1]);

const voiceover = {};
for (const id of voIds) {
  const real = `audio/vo/${id}.wav`;
  const scratch = `audio/vo-scratch/${id}.wav`;
  voiceover[id] = isFile(resolve(publicDir, real))
    ? real
    : isFile(resolve(publicDir, scratch))
      ? scratch
      : null;
}

const lines = Object.entries(voiceover)
  .map(([id, path]) => `  ${JSON.stringify(id)}: ${path ? JSON.stringify(path) : "null"},`)
  .join("\n");

const manifest = `// GENERATED FILE — do not edit by hand.
//
// Regenerate: node scripts/sync-assets.mjs
//
// Lists the audio that is actually present in public/. A null means the film
// renders that part silently, so a missing track never breaks a render.

export const AUDIO_MANIFEST = {
  music: ${music ? JSON.stringify(music) : "null"},
  voiceover: {
${lines}
  },
} as const;
`;

writeFileSync(resolve(here, "../src/audio-manifest.ts"), manifest, "utf8");

const found = Object.values(voiceover).filter(Boolean).length;
console.log(
  `sync-assets: copied ${copies.length} files; audio — music ${music ? "yes" : "none"}, ${found}/${voIds.length} VO lines`
);
