# Ledgerly promo

A 47.7 s launch promo for Ledgerly (1920×1080, 30 fps), built with
[video-shotcraft](https://vincentwei1021.github.io/video-shotcraft/) in
**autonomous free creation** mode. Every shot composites a **real capture of the
running app**, not a mock-up.

- **`out/promo.mp4`** — the film, with the music bed.
- **`out/promo-nobgm.mp4`** — the same timeline and the same picture, SFX only
  (for adding your own music).
- **`DESIGN-SPEC.md`** — the full record: product brief, requirements-to-execution
  decisions, visual direction and tokens, feature-to-shot mapping, storyboard with
  the frame timeline, the audio plan, data handling, and every deliberate
  deviation from the craft rules (including what was planned and then not built).

## Re-render

```sh
npm install
npx remotion render src/index.ts LedgerlyPromo out/promo.mp4
npx remotion render src/index.ts LedgerlyPromo out/promo-nobgm.mp4 --props=props-nobgm.json
```

`src/Main.tsx` holds the timeline (`SHOTS`) plus the narration table and the SFX
cue table. It is the single source of truth: the storyboard, the captions, the
flash cuts, the sound cues and the workbench manifest are all derived from it, so
changing a shot's duration moves everything else with it.

## Preview and tweak

```sh
npx remotion studio src/index.ts          # Remotion Studio
node <video-shotcraft>/workbench/scripts/open.mjs .   # the visual workbench
```

The workbench (see `src/workbench.ts`) decomposes the film into shots, captions,
transitions and sound cues so you can move, retime, recut and re-export it in a
browser without editing code. Title-card and narration copy are editable there;
the live scenes expose timing and layer transforms.

## Known gaps

Recorded honestly in `DESIGN-SPEC.md` §8. In short: four recipe cards were
reproduced only in part (their object-close-up or 3D rigs don't fit this
product's flat, light UI), and two planned cards — `chart-live-moves` and
`document-typewriter-reveal` for the reports shot — were not implemented at all;
that shot is a camera walk over the real reports page instead.

## Assets

- `public/textures/live/` — 2× page captures, element cutouts and the coordinate
  table, produced by `capture/capture.py` from the running app.
- `public/fonts/` — Ledgerly's own Inter files, copied from `static/fonts/`.
- `public/audio/` — BGM and SFX. Free for commercial use; provenance and licences
  are in the video-shotcraft repo at `assets/audio/ATTRIBUTION.md`.

## Regenerating the captures

```sh
python capture/capture.py        # needs Python 3 + Playwright, and Ledgerly's deps installed
```

It starts Ledgerly's dev server, stubs the Tauri IPC with fictional sample data
(`capture/stub.js`), and writes the textures plus `src/live-layout.json`. The demo
business identity is fictional (Northwind Studio, example.com addresses), so
nothing sensitive reaches the film.
