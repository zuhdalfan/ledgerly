# Ledgerly motion — `tools/motion`

The Remotion build of the 35s brand film described in
[`docs/marketing/motion-brief.md`](../../docs/marketing/motion-brief.md).

**This directory is development tooling.** It is deliberately not a dependency of
the app: its `package.json` is separate, nothing here is imported by `src/`, and
Remotion's headless-Chrome download never touches the shipped build. Do not hoist
these packages into the root manifest — `check:offline` and `check:licenses` guard
the product, and they should stay unrelated to this folder.

## Run

```sh
cd tools/motion
npm install
npm run studio        # syncs tokens + assets, then opens Remotion Studio
```

`npm run sync` is also run automatically by every render script. It does two things:

- `scripts/sync-theme.mjs` regenerates `src/theme.ts` from `src/app.css`, so the
  film cannot drift from the product palette.
- `scripts/sync-assets.mjs` copies the Inter TTFs and the logo into `public/`
  (gitignored), because `staticFile()` only serves from there.

## Audio

The master carries a music bed and nine voiceover lines. Both are optional at render
time — a missing file makes that layer silent instead of breaking the build — and
`npm run sync` regenerates `src/audio-manifest.ts` from whatever is on disk.

| Drop in | What happens |
|---|---|
| `public/audio/music.mp3` (or `.m4a`, `.wav`, `.ogg`) | plays under the whole edit, ducked automatically under each VO line |
| `public/audio/vo/<id>.wav` | replaces the scratch read for that line |
| `public/audio/vo-scratch/*` | generated placeholder, gitignored, never ships |

Line ids and the script live in `src/vo.ts`; the brief's §5.2 is the same table in
prose. `npm run vo:scratch` (Windows only) synthesizes a scratch read so the edit can
be timed before a human exists.

Music must be cleared for commercial use — record the licence next to the file.

## Render

```sh
npm run render            # out/ledgerly-master.mp4        1920×1080, 35.0s, with audio
npm run render:silent     # out/ledgerly-master-silent.mp4 same edit, no audio
npm run render:vertical   # out/ledgerly-vertical.mp4      1080×1920, 35.0s
npm run render:square     # out/ledgerly-square.mp4        1080×1080, 35.0s
npm run render:cutdown    # out/ledgerly-17s.mp4           1920×1080, 17.0s
```

## Where things live

| File | Role |
|---|---|
| `src/timeline.ts` | The single timing source. Every composition derives from it. |
| `src/vo.ts` | The voiceover script, line by line, placed against beats. |
| `src/audio.ts` | Audio lookups; everything resolves to `null` when a file is absent. |
| `src/theme.ts` | **Generated.** Token values from `src/app.css`. Never hand-edit. |
| `src/fonts.ts` | Local Inter loading — no CDN, same rule as the app. |
| `src/components/` | The app's chrome and the film's devices (LED, tray, notification). |
| `src/scenes/` | One file per beat. Scenes are identical across all four formats. |

## Known gap

`Vertical` (1080×1920) reuses the landscape stage, scaled and centred. A 16:9
window in a 9:16 frame leaves large margins, so the portrait composition still
needs its own pass — the app's narrow layout (`<820px`: top bar + drawer) is the
right reference. The landscape master is unaffected.

## Rules the film has to keep

- **Tray mode is opt-in.** Beat 5 says "Opt in to keep it running" and never
  implies the app ships resident.
- **The notification carries a count and nothing else.** Ship the copy verbatim;
  a Rust test guards it.
- **No invented palette.** Colours come from `theme.ts`; motion easing matches
  `--ease-out`.
- **No remote assets.** Fonts and the mark are copied from the repo at sync time.
- **Fictional data only.** `DEMO` in `src/lib/format.ts` is the one place to change it.
