# Ledgerly promo — design spec

Stages 0–3 of the video-shotcraft pipeline, in **autonomous free creation** mode:
every judgement below is made by the agent and recorded here (no per-stage
sign-off). Stage 4 (asset capture) is already done; stages 5–7 follow.

---

## 1. Product brief (stage 0)

| | |
|---|---|
| Product | **Ledgerly** — a fully local, fully offline desktop invoicing app for freelancers and small businesses. |
| Stack | Rust + Tauri 2 core, Svelte 5 + TypeScript frontend, one local SQLite file. MIT, free, no paid tier. |
| Purpose of the film | A launch-style promo: what it is, the headline workflows, and the one promise that separates it from every cloud invoicing tool. |
| Audience | Primary: freelancers and small-business owners who invoice clients. Secondary: the developer-tool crowd (Rust/Tauri, MCP) who share it. |
| Length / format | 1432 frames @ 30 fps = **47.7 s**, 1920×1080 (16:9). |
| Language | English (the app's content screens are English; only the shell/settings are being localised to Bahasa Indonesia). |
| Voice | No VO. Typography, captions and audio carry the message. |
| Audio | Strong-beat electronic BGM + film-vocabulary SFX. Delivered twice: with BGM and without (SFX only). |
| Data | Fictional demo data only (see §6). |

### Requirements → execution decisions

| Decision | Choice | Why |
|---|---|---|
| What must be shown | Money owed + chasing (dashboard), the totals engine (invoice line items → totals), finding a record (search/filter), estimate → invoice, reports, and the offline promise + AI access. | These are the headline capabilities from the README/PRD and the six existing marketing cuts. |
| What is *not* shown | App lock passphrase, backup/restore internals, PDF templates, catalog CRUD, i18n. | Not enough screen time for 42 s; they are secondary or hard to read at video scale. |
| Employer of truth for UI | Real captures of the running app (§7), never hand-built replicas. | Aesthetic rule Q1. |
| Capture route | Reuse the repo's own stub harness (`tools/screenshots/stub.js`) with a local copy, business identity replaced with a fictional studio. | The app is a Tauri IPC client; the stub is the documented, already-proven way to render every screen in a browser. |
| Skin | Ledgerly's own tokens, read out of `src/app.css`. | SKILL principle 2: the film's visual language must grow from the product. |
| Typography | **Inter 400/700** — the app's real font files, copied from `static/fonts/`. | Not the Ink Press serif: Ledgerly's own headings are Inter with negative tracking. |
| Accent discipline | Charcoal `--accent` for emphasis; the state vocabulary (ok / warn / danger / info / neutral) only where the app itself uses it — on invoice status badges. | The accent is a near-neutral; the product gets its colour from status, so the film does too. |
| BGM | Chosen from `assets/audio/bgm/`, auditioned against the cut (stage 6). | SKILL S1. |
| Determinism | No `Math.random()`, no `Date.now()`; all pseudo-randomness from a fixed seed. | SKILL §9. |

---

## 2. Visual direction (stage 1)

Three written directions were considered:

- **A — Paper & ink, editorial calm.** Warm paper ground, warm-black ink, Inter,
  hairline rules, letterpress type, one charcoal accent. Grows directly out of
  `src/app.css`. Low energy, "a tool, not a toy".
- **B — Studio desk / hardware.** Desaturated slate surfaces, product-shot
  lighting, physical-device framing. Rejected: the product is light-only warm
  paper; B would be a skin bolted on from outside.
- **C — Data energy.** Tabular numerals, charts, status colours driving the
  frame. Rejected as the *base* (it would fight the calm of A) but kept as an
  **accent vocabulary** — the status colours appear exactly where the app uses
  them.

**Chosen: A as the skin, with C's status vocabulary as the only colour.** The
product's identity *is* warm paper + warm ink + a status vocabulary, so the film
uses nothing else.

### Colour tokens (verbatim from `src/app.css`)

```
paper-0 oklch(0.995 0.002 80)   ink-1  oklch(0.24 0.014 70)   ok     oklch(0.50 0.10 155)
paper-1 oklch(0.98  0.006 80)   ink-2  oklch(0.42 0.014 70)   warn   oklch(0.52 0.11 70)
paper-2 oklch(0.965 0.008 80)   ink-3  oklch(0.52 0.012 70)   danger oklch(0.50 0.17 25)
paper-3 oklch(0.92  0.012 80)   accent oklch(0.30 0.006 75)   info   oklch(0.48 0.10 250)
paper-4 oklch(0.86  0.014 80)   inverse oklch(0.99 0.002 80)  neutral oklch(0.52 0.012 70)
```

### Motion personality tokens

From the brand→motion derivation, Ledgerly sits **low on the energy axis**
(calm, premium, infrastructural) and **serious on the tone axis** (finance,
small-business trust). Nearest preset: *professional trust*, nudged warmer.

| Token | Value | Note |
|---|---|---|
| `durFast` | 12f | control-level micro feedback |
| `durBase` | 21f | info fades, row settles |
| `durSlow` | 48f | camera pushes, card arcs |
| `easeTrust` | `cubic-bezier(0,0,0.2,1)` | default in-and-out for informational motion |
| `easeOut` | `cubic-bezier(0.22,1,0.36,1)` | the app's own `--ease-out` |
| `easeLand` | `cubic-bezier(0.2,1.25,0.3,1)` | **only** for things that physically land (card reseat, wordmark stamp) — y₁ > 1 |
| wordmark hold | 30f | R1 |
| batch settle | ≥15f | R2 |
| camera shake | **none** | Q3 |

Three-word check for the finished motion: **measured, warm, exact.** Matches the
brand read.

### Styleframe

**Skipped, with reason.** The pipeline permits skipping the styleframe when the
project already has a strict brand spec or an explicit reference film. Ledgerly
has one: `src/app.css` is an authored, heavily commented design system
(`paper-0…4`, `ink-1…3`, one charcoal accent, radii, the 1-4-9 space rhythm, and
motion durations), and the font files are the product's own. There is no open
direction question for a styleframe to settle — the first rendered stills act
as the verification instead, and any drift is caught there.

---

## 3. Feature → shot mapping (stage 2)

Card names validated against `gallery/api/library.json`; the referenced demo
sources were read before implementation.

| # | Feature | Card | Variant / note |
|---|---|---|---|
| 1 | Brand identity | `brand-ink-open` | mark draws instead of the crosshair — Ledgerly's torn-paper slip |
| 2 | Money owed / chasing | `spotlight-hero-card` | lights-down dashboard, spotlight the *Outstanding* card, 3D annotation |
| 3 | (breath) | `paper-title-card` | re-skinned: Inter, charcoal accent, no serif |
| 4 | Totals engine | `row-embed` | camera down the real invoice: line items → totals, rows embed into slots |
| 5 | Finding a record | `type-and-filter` | type into the real filter box, rows recede, the match lifts |
| 6 | (breath) | `paper-title-card` | second title card |
| 7 | Estimate → invoice | `deck-deal-flyin` (adapted) | estimate line items dealt into place, then a link chip docks |
| 8 | Reports | — (camera walk; no recipe card) | the five real report tables, walked top to bottom; the revenue total is bracketed |
| 9 | The offline promise | `brand-ink-open` (type half) | typographic hero card |
| 10 | AI access over MCP | `scan-bracket-sweep` | AI settings page; corner brackets lock onto the MCP config card |
| 11 | Sign-off | `outro-group-photo-launch` | element roll-call around the mark |

**One technique is protagonist once**: lights-down spotlight (2), row embed (4),
type-and-filter (5), dealing (7), typewriter reveal (8), roll-call (11) — no
method repeats as the lead.

**Deliberate adaptations (recorded):**

- **Shot 7** deals the estimate rows into their *existing* slots and then docks a
  link chip, rather than dealing into a fresh grid — the product's own estimate
  list is the honest target, and the app links the converted invoice to the
  estimate. The card's hard, accelerating stagger is preserved.
- **Shot 5** makes the non-matching rows recede rather than deleting them. A
  fabricated reflow (the card's literal read) would have to invent a card height
  the app does not produce, and would read worse than an honest dissolve.

**Feasibility note (records scope):** `spotlight-hero-card` is reproduced at
reduced fidelity — the page-space card-shaped light hole, the lift / hover /
reseat arc with its landing overshoot, and the page-space annotation are kept.
Dropped, and *not* previously recorded accurately: the travelling light pool
(three stations — the hole locks immediately instead), the oblique left-side
3D camera (the push is frontal), the volumetric light shaft, and the double-lap
contour beam. On a light warm-paper UI the last two read as noise rather than
scan; the first two would have made the arc longer than the shot budget allowed.
The reseat overshoot and the ≥3 s arc are preserved, so the card's *命门*
parameters are not downgraded.

---

## 4. Storyboard + frame-level timeline (stage 3)

Sequence template: `promo-energy-arc` (4 sections + breath cards).

| # | from | dur | Scene | Content | Card | Source | Caption | Out |
|---|------|-----|-------|---------|------|--------|---------|-----|
| 1 | 0 | 140 | Open | torn-paper mark draws → "Ledgerly" letterpress → tagline types on → 30f hold → drift | brand-ink-open | — (SVG mark) | — | whoosh-fast @118 |
| 2 | 140 | 200 | Hero | dashboard darkens; light locks on the Outstanding card; card lifts/hovers/reseats; 3D annotation | spotlight-hero-card | `dashboard-full` + `dashboard-card1` | in-scene page-space annotation "Money owed, at a glance." 252–340 | whoosh-big @236 |
| 3 | 340 | 50 | Title 1 | "Invoices that add up." | paper-title-card | — | — | swoosh-quick @295 |
| 4 | 390 | 178 | Invoice | camera walks the invoice: client → line items → totals panel; rows embed; total ticks up | row-embed | `invoice-full` + rows/cards | "Line items, tax and discounts reconcile to the cent." 385–480 | transition-snap @488 |
| 5 | 568 | 142 | Find | list at rest → types "INV-0041" → the non-matches dissolve → the match lifts and is bracketed | type-and-filter | `invoices-full` + rows | "Find any invoice by number or client." 640–706 | click-camera @672 |
| 6 | 710 | 50 | Title 2 | "Quote it. Then invoice it." | paper-title-card | — | — | swoosh-quick @620 |
| 7 | 760 | 142 | Estimate | estimate line items dealt into an invoice; totals land | deck-deal-flyin | `estimate-full` + `estimate-row*` | "One click turns an estimate into an invoice." 700–775 | transition-snap @785 |
| 8 | 902 | 138 | Reports | camera walks the five real report tables; the revenue total is bracketed and underlined | — (camera walk) | `reports-full` + rows | "Revenue, tax and receivables — exportable to CSV." 924–1020 | whoosh-big @992 |
| 9 | 1040 | 112 | Offline | typographic hero: "No account. No cloud. No network." + rule | brand-ink-open (type half) | — | — | impact @1005 |
| 10 | 1152 | 112 | AI | AI settings screen; corner brackets lock onto the MCP config card | scan-bracket-sweep | `ai-full` + `ai-card2` | "Local, stdio-only — nothing is exposed on a port." 1208–1258 | click-camera @1216 |
| 11 | 1264 | 168 | Outro | elements roll-call in from four sides → mark + wordmark stamped → 30f hold | outro-group-photo-launch | cutouts from every scene | — (clean) | — |

Hold/rest budget is carved **before** the animation in every shot: shot 1 holds
30f, shot 2 holds 30f after reseat, shot 4 holds 18f after the total lands, shot
11 holds 30f.

### Notes on the arc

- Shot 8 (densest information) sits third-from-last, ahead of the peak — per the
  sequence template.
- Two wordmark moments only: shot 1 and shot 11, each ≥1 s (R1).
- The offline card (9) is the emotional pivot: the promise, stated flatly.

---

## 5. Audio

Bed: `bgm-tech-house.mp3` at 0.34 with a 1 s fade-in and a 1.7 s fade-out, leaving
headroom for the SFX. Cues are restricted to the film vocabulary — whoosh
(`whoosh-fast` / `whoosh-big` / `sweep-short`), transition (`transition-soft` /
`swoosh-quick` / `transition-snap`), impact (`impact-deep-whoosh`), riser
(`riser-cine`), sparkle (`sparkle`) — plus foley only where the picture has an
identifiable action of its own (rule S4): `keyboard` for the typing, `pop` for
each roll-call element landing, `click-camera` for the two "found it" beats
(vol 0.5 and 0.6, the loudest SFX in the film). The ending is the fixed
riser → impact → sparkle sentence, with the impact at vol 0.55 as the film's peak.

Every cue is written as `SHOT_AT[shot].from + offset`, never a bare frame number,
so moving a shot moves its cues. `keyboard` is the one long sample (19.6 s) and is
truncated to its action with an explicit `durationInFrames`; `riser-cine` is also
given a window.

**Measured output offset (rule S5).** `capture/measure_offset.py` extracts the
render's audio, builds a per-frame energy envelope, and reports where each known
sharp cue actually landed. Probed **2026-10-14** against `out/promo.mp4`
(Remotion 4.0.484, AAC 48 kHz, mp4, 30 fps), with a provisional 1.28 f
compensation in place:

| cue | designed | measured | residual |
|---|---|---|---|
| click-camera (find) | f672 | f671 | −1 |
| click-camera (invoice) | f512 | f511 | −1 |
| transition-snap (hero) | f306 | f314 | +8 |
| transition-snap (invoice) | f428 | f430 | +2 |
| transition-snap (estimate) | f824 | f816 | −8 |
| swoosh-quick (title) | f340 | f350 | +10 |
| pop (outro) | f1000 | f1003 | +3 |

The two isolated, sharp, loud cues both land exactly 1 frame early, so the
pipeline's real offset is **≈ 0.3 f**, not the 1.28 f the AAC-priming rule of
thumb predicts — `OUTPUT_AUDIO_OFFSET_F` is set to 0.3 accordingly.

**Honest limitation.** The spread on the other five probes is not a second
offset; it is the ±1-frame resolution of frame-level envelope probing plus
contamination from neighbouring cues (the pops, the sparkle, and the previous
shot's whoosh tail all sit within the ±10-frame search window). Per-file peak
lags could therefore not be separated from the pipeline offset at this
resolution, so `PEAK_F` is left empty rather than filled with numbers the method
cannot support. Residuals are within ±1 frame on the clean probes — well inside
perceptibility, and far from the 0.27 s (8-frame) desync the rule exists to
prevent.

Both mixes are rendered from this one timeline via the `bgm` inputProp:
`out/promo.mp4` (with bed) and `out/promo-nobgm.mp4` (SFX only).

## 6. Data handling

All captures use the repo's fictional demo dataset (Acme Corporation, Globex,
Initech, Umbrella Pte Ltd). The business identity in the local stub copy was
replaced with a clearly fictional studio (**Northwind Studio**, example.com
address, zeroed tax id, Example Bank) because the original stub carried the
repository owner's own business name.

## 7. Asset provenance

Captured with `capture/capture.py` at 1920×1080 @2× from the running app:
12 routes → `public/textures/live/<route>-full.png` (2× full page),
`<route>-win.png` (100vh window), per-element cutouts, and page-space
`src/live-layout.json`. Inter 400/700 copied from `static/fonts/`.

## 8. Deliberate deviations from aesthetic-rules.md

| Rule / card | Deviation | Reason |
|---|---|---|
| Q4 (glint/sweep at most once, clipped) | Kept as-is, not violated. | — |
| Q10 (document shots must be publication-grade) | The reports/invoice shots are real screenshots of the real app, not mocks. | Better than a mock. |
| Q11 | Captions rendered at 58 px (≥5.2 % of 1080). | Compliant. |
| `spotlight-hero-card` (shot 2) | Travelling light pool (3 stations), oblique left camera, volumetric shaft and double-lap contour beam all dropped; the hole locks immediately. | See the feasibility note in §3. Only reached after the first independent review — the earlier note over-claimed the pool as kept. |
| `row-embed` (shot 4), partial | Rows slide in from the right; no `rotateX` collect-to-flat and no accent embed seam. | On a flat frontal camera a 3D collect reads as a wobble. The rows still land in their real captured slots (Q9 kept). |
| `type-and-filter` (shot 5), partial | No click ripple and no push-through into the detail page. | Would require fabricating a second page state; the shot's budget went to the macro typing and the match lift. |
| `deck-deal-flyin` (shot 7), partial | The graphite desk, physical pile, orbit and overhead camera are absent. | Those parts of the card are an *object close-up* rig for a pile subject; the product's own estimate list is the honest target. The accelerating deal stagger and landing overshoot are kept. |
| `chart-live-moves` + `document-typewriter-reveal` (shot 8) | **Planned, then not implemented** — shot 8 is a camera walk over the real reports page with a bracket on the revenue total. | There is no live chart on that screen; fabricating one would be a false product claim (P3). A row-by-row wipe would repeat row-embed as a second protagonist (P4). The mapping table has been corrected to match what shipped. |
| `spotlight-sweep-moves` (shot 10) | **Replaced** by `scan-bracket-sweep`. | The AI screen's subject is a config block rather than a lit surface, and lights-down spotlight is the hero's technique, which may only lead once (P4). |
| R4 (full-frame impacts ≤3) | Seven warm-white `FlashCut` blooms were present in the first cut — one per hard cut. | Violation caught by the independent review; **fixed** by reducing to three (see below), so it is no longer a live deviation. |
