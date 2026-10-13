// Renders the composition and writes docs/screenshots/preview.gif.
//
//   node tools/preview/scripts/build-gif.mjs      (equivalently: npm run gif)
//
// HyperFrames renders the frames; ffmpeg builds the GIF from them. A direct
// `hyperframes render --format gif` is about four times larger, because it does
// not get the palette pass or the scaled output the README wants.
//
// Development tooling only: it is not part of the app build and ships with
// nothing.

import { spawnSync } from "node:child_process";
import { existsSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const project = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const renders = join(project, "renders");
const frames = join(renders, "frames");
const filterScript = join(renders, "gif-filter.txt");
const output = resolve(project, "..", "..", "docs", "screenshots", "preview.gif");

function run(result) {
  if (result.status !== 0) process.exit(result.status ?? 1);
}

const runDirect = (command, args) => run(spawnSync(command, args, { stdio: "inherit", cwd: project }));

// npx and friends resolve through a shell on Windows; pass one string so the
// arguments are not concatenated behind our back.
const runShell = (commandLine) => run(spawnSync(commandLine, { stdio: "inherit", shell: true, cwd: project }));

// 1. The app's captures and fonts, which HyperFrames cannot read from above the
//    project root.
runDirect(process.execPath, [join(project, "scripts", "sync-assets.mjs")]);

// 2. Lossless PNG frames at 24fps (resampled to the GIF's 12 below — the extra
//    source frames keep the cursor's glide smooth).
if (existsSync(frames)) rmSync(frames, { recursive: true, force: true });
runShell("npx --yes hyperframes@0.8.150 render --format png-sequence --fps 24 --output renders/frames");

// 3. The GIF: 12fps, 1000px wide, 96 colours. Everything else is secondary to
//    diff_mode=rectangle (store only the changed region): the composition keeps
//    a locked camera and a static progress indicator, so a scene is
//    pixel-identical between cursor moves and most frames cost almost nothing.
//    No dither — the UI is flat paper and ink, so dithering only adds the noise
//    that makes each moving frame heavier. 96 colours leave headroom for the
//    window's soft shadow.
//
//    The graph goes in a script file: passed inline it survives cmd.exe but gets
//    re-split by the spawn call on Windows, and ffmpeg then rejects it.
const graph = [
  "[0:v]fps=12,scale=1000:-2:flags=lanczos,split[s0][s1]",
  "[s0]palettegen=max_colors=96:stats_mode=diff[p]",
  "[s1][p]paletteuse=dither=none:diff_mode=rectangle[out]",
].join(";");
writeFileSync(filterScript, graph);

runDirect("ffmpeg", [
  "-y",
  "-loglevel", "error",
  "-start_number", "1",
  "-framerate", "24",
  "-i", "renders/frames/frame_%06d.png",
  "-filter_complex_script", "renders/gif-filter.txt",
  "-map", "[out]",
  "-loop", "0",
  output,
]);

console.log(`\nWrote ${output}`);
