# Marketing cuts

Short, captioned cuts of the real app — one message each, ~8–12s, with a brand
chip and an end card. These are **not** the README walkthrough (which is
complete and uncaptioned, in `docs/preview/`); these are for a landing hero,
feature cards, social and ads.

| Cut | Message | landscape | portrait |
|---|---|---|---|
| `hero` | Get paid, without the busywork | `hero-landscape.mp4` | `hero-portrait.mp4` |
| `create` | An invoice in seconds | `create-landscape.mp4` | `create-portrait.mp4` |
| `offline` | No account. No cloud. No network. | `offline-landscape.mp4` | `offline-portrait.mp4` |
| `estimate` | Quote it, then invoice it | `estimate-landscape.mp4` | `estimate-portrait.mp4` |
| `chase` | Chase late payments in one click | `chase-landscape.mp4` | `chase-portrait.mp4` |
| `mcp` | Your AI assistant can read your books | `mcp-landscape.mp4` | `mcp-portrait.mp4` |

- **landscape** 1280×720 — landing pages, X/LinkedIn, YouTube.
- **portrait** 720×1280 — Reels, TikTok, Shorts (the app's drawer layout).
- **MP4 is the primary format**: platforms autoplay it and it is small (~200–700
  KB per cut). A **gif** fallback for image-only embeds is opt-in
  (`encode_marketing.py --gif`) — a gif of moving UI runs to megabytes.

`manifest.json` carries the machine-readable index (subject, format, sizes, trim
offsets).

## Regenerate

```sh
python tools/preview-record/marketing.py --format landscape
python tools/preview-record/marketing.py --format portrait
python tools/preview-record/encode_marketing.py          # add --gif for gifs
```

`marketing.py --list` lists the cuts; `--only hero` records one; `--scale 0.75`
tightens the pacing. Requires Python 3 + Playwright and `ffmpeg` (see
`tools/preview-record/README.md`).

## How a cut is built

The overlay lives in `tools/preview-record/marketing.js` and is injected only for
these recordings: a brand chip, a caption card and an end card, sized with
`vmin` so one script reads correctly at every aspect ratio. Navigation goes
through the command palette, which works whether or not the sidebar is visible
(it is hidden in portrait), so a single script produces all three frames.

## Known polish items

- The brand chip overlaps the app's own top-bar wordmark in portrait.
- Captions are English only (the overlay text is per-script, easy to localize).
- Cut length is driven by the script's beats; the hero is the longest at ~12s.
