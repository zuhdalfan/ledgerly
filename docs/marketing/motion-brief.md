# Motion brief — Ledgerly "Your machine. Your money."

| | |
|---|---|
| **Status** | Ready to build — audio wired, VO scripted |
| **Master** | 35.0s · 1920×1080 · 30 fps · 1050 frames |
| **Tool** | Remotion, isolated in `tools/motion/` |
| **Audio** | Music bed + English VO (no Indonesian cut) |
| **CTA** | `github.com/zuhdalfan/ledgerly` |
| **Last updated** | 2026-10-23 |

This sits **above** the per-feature cuts in `docs/marketing/README.md`. Those stay as
feature cards; this is the brand piece. Everything below is grounded in shipped
behaviour and shipped copy — the strings in the film are the strings in the product.

---

## 1. The one thing

> **Everything a freelancer needs to get paid — on a computer that never touches
> the internet.**

The film is not a feature tour. It makes one idea undeniable in 35 seconds: *this is
real invoicing software, and it is offline on purpose.* Features are the proof, not
the pitch.

## 2. Audience and placement

Solo freelancers and 1–5 person shops who are tired of subscriptions, logins and
"your trial ends in 3 days".

- Landing-page hero (autoplay, muted, loops cleanly into the end card)
- X / LinkedIn, YouTube pre-roll
- Reels, TikTok, Shorts (vertical master)
- The end card is the only place a URL or CTA appears

## 3. Deliverables

| Asset | Spec | Use |
|---|---|---|
| `ledgerly-master.mp4` | 1920×1080, 30 fps, 35.0s (1050f), H.264 CRF 17, yuv420p, AAC 48 kHz | landing hero, YouTube, LinkedIn |
| `ledgerly-master-silent.mp4` | same edit, no music or VO | muted-first placements |
| `ledgerly-vertical.mp4` | 1080×1920, same 35.0s, re-laid-out | Reels, TikTok, Shorts |
| `ledgerly-square.mp4` | 1080×1080, same 35.0s, reframed | feed posts |
| `ledgerly-17s.mp4` | 1920×1080, 17.0s (510f): promise → invoice → tray → end card | paid social |
| Captions | The on-screen headlines are the burned-in captions; a separate `.srt` is only needed if closed captions are required | silent autoplay |
| Poster frames | 3 PNGs at 0.5s / 17s / 31s | thumbnails |

Audio is part of the master: one music bed plus nine voiceover lines. English only —
no Indonesian variant. Most platforms autoplay muted, so the silent master is the
same edit with the audio layer switched off, not a different cut.

## 4. Message ladder

1. **Hook:** offline is the feature.
2. **Competence:** invoice → payment → chase → quote → PDF.
3. **Surprise A:** it keeps working after you close it.
4. **Surprise B:** the AI reads your books over a wire that isn't there.
5. **Close:** free, MIT, install once.

## 5. Storyboard

The spine: a small **`network: off` LED** sits in the corner of every scene. It never
lights. It is the only recurring graphic device, and it pays off in the end card.

| # | Time | On screen | Visual | Motion | Sound |
|---|---|---|---|---|---|
| 1 | 0.0–3.0 (90f) | **"Invoicing that never phones home."** | Warm paper field; the torn-paper mark draws itself | Logo stroke-draw, then hold; text rises 8px on `--ease-out` | Room tone, one paper tick |
| 2 | 3.0–8.0 (150f) | **"An invoice in 42 seconds."** | Editor assembles: catalog picker, three line items, totals panel | Window 0.96→1; rows drop in; digits roll; `0:42` chip counts up | 3 pops, row ticks, total whoosh |
| 3 | 8.0–11.8 (114f) | **"Paid, or partially paid. It just knows."** | Payment panel → balance counts down → `Sent → Partially paid → Paid` | Pill morph + digits roll; green check with 6px overshoot | Soft cash chime |
| 4 | 11.8–15.4 (108f) | **"Chase it without leaving the app."** | "Needs chasing" row, red overdue, aged-receivables bars | Bars grow; `Copy` → clipboard check; row → "Chased" | Bar ticks, clipboard click |
| 5 | 15.4–21.8 (192f) | **"Or close it. It still remembers."** | Window shrinks into a tray icon; clock winds 18:40 → 09:41; notification card; click reopens | See §5.1 | Two-note reminder chime, soft click |
| 6 | 21.8–26.2 (132f) | **"Quote it. Invoice it. Export it."** | Estimate → one-click convert → linked invoice → PDF with 3 templates cycling | Connector line, page slides up, accent crossfades | Whoosh, page-turn |
| 7 | 26.2–31.0 (144f) | **"An AI that reads your books — over a wire that isn't there."** | Assistant panel: *"Who owes me the most?"* → answer matching the dashboard | Query types; answer streams; dashed `stdio` connector draws; LED gets a ring | Typing, three blips |
| 8 | 31.0–35.0 (120f) | End card: logo, **"No account. No cloud. No network."**, `Free · MIT · Windows · macOS · Linux` | Paper field returns; UI folds into the mark; the LED's ring becomes the full stop | Everything collapses into the logo | Music resolves on one note |

### 5.1 Beat 5, broken down (192f / 6.4s)

The densest beat in the film, so it carries its own timing.

| Sub | Frames | Action |
|---|---|---|
| 5a | 15.4–17.0 (48f) | Window scales down and *into* the tray icon; the icon pulses once on arrival. Tray menu opens for ~0.7s — **Open Ledgerly / Lock now / Quit Ledgerly** — then closes without a selection. Affordance, not tutorial. Caption: **"Opt in to keep it running."** |
| 5b | 17.0–18.4 (42f) | Desktop empty; clock digits flip **18:40 → 09:41**; caption **"Once a day. Not once a minute."** Nothing else moves. The stillness is the point. |
| 5c | 18.4–20.4 (60f) | Notification card slides in top-right: mark, **Ledgerly**, body `3 invoices are past due — Open Ledgerly to see them.` Caption beside it: **"No names. No amounts."** |
| 5d | 20.4–21.8 (42f) | Cursor clicks the card; window expands with the overdue figure already visible; notification dismisses. |

### 5.2 Voiceover script

English only. Nine lines, about 67 words, at a relaxed **2.2 words a second** — a
person talking to one freelancer, not an announcer. The VO never reads a headline
verbatim except the last line, where the tagline should land exactly with the
picture.

| # | In | Max len | Words | Line |
|---|---|---|---|---|
| 1 | 0:00.3 | 2.7s | 6 | "Invoicing that never touches the internet." |
| 2 | 0:03.2 | 4.8s | 10 | "An invoice in under a minute. Line items, tax, totals." |
| 3 | 0:08.3 | 3.6s | 7 | "Record a payment. The status updates itself." |
| 4 | 0:12.1 | 3.3s | 7 | "Chase what's late without another app." |
| 5a | 0:15.5 | 1.5s | 3 | "Close the window." |
| 5c | 0:18.5 | 3.2s | 5 | "Daily, it flags what's overdue." |
| 6 | 0:22.1 | 4.2s | 7 | "Quotes become invoices. Invoices become clean PDFs." |
| 7 | 0:26.5 | 4.6s | 9 | "Your AI can read it all, without a network." |
| 8 | 0:31.3 | 3.5s | 6 | "No account. No cloud. No network." |

**Direction.** Warm, level, unhurried; a slight downward inflection on each
sentence end. No rising "upsell" tone, no smiley delivery. The product is calm and
boring on purpose and the read should match it.

**Silence is written in.** Line 5a ends before 17.0s and 5c starts after 18.4s, so
beat 5b — the overnight gap — has no speech at all. Do not fill it.

**Recording.** 48 kHz / 24-bit mono WAV, one file per line, room-treated, no
compression or reverb. Leave 100ms of head and 250ms of tail; the edit trims to the
reserved window. Peak around −6 dBFS, no limiting.

**Delivery naming.** `public/audio/vo/<id>.wav` with the ids from
`tools/motion/src/vo.ts` (`01-promise`, `02-invoice`, `03-paid`, `04-chase`,
`05a-close`, `05c-remind`, `06-quote`, `07-mcp`, `08-close`). A real read wins over
the generated scratch read automatically.

## 6. Accuracy guardrails

The film must not over-claim. These are product truths, not creative preferences.

- **Tray mode is opt-in and off by default.** Never imply it ships on. Beat 5a says
  "Opt in to keep it running" for exactly this reason.
- **Reminders are on by default, but only fire once tray mode is on and the OS grants
  permission.** Never show the notification without the tray icon present — the docs
  are explicit that TRAY and NOTIFY are one feature, and the film should read that way.
- **Use the shipping copy verbatim.** Title `Ledgerly`; body
  `3 invoices are past due — Open Ledgerly to see them.` It was written to survive a
  lock screen, so it also reads well as a graphic. Indonesian variant exists
  (`3 faktur sudah jatuh tempo — Buka Ledgerly untuk melihatnya.`).
- **No client names or amounts in the notification.** A Rust test guards this
  (`the_notification_never_names_a_client_or_an_amount`). Captioning it turns an
  implementation detail into a selling point.
- **Do not fake an OS.** Render a neutral menu bar and a notification card in
  Ledgerly's own visual language. A pixel-accurate Windows toast dates the film and
  picks a platform favourite.
- **Fictional data only.** Invent clients and amounts; do not ship the demo business
  name or real balances to a public video.
- **Optional 0.5s insert** in 5d: the window opens to the lock gate first, then
  unlocks. Reinforces "the lock still holds"; costs comprehension. Skipped in the
  master, available to the 16s cut if it needs a privacy angle.

## 7. Art direction

Use the app's own system; invent nothing.

- **Palette:** import the OKLCH tokens from `src/app.css` verbatim —
  `--paper-0/1/2/3`, `--ink-1/2/3`, `--accent`, `--ok`, `--warn`, `--danger`,
  `--info`, `--line-control`. Remotion renders in Chromium, so `oklch()` works as-is.
  `tools/motion/scripts/sync-theme.mjs` generates `src/theme.ts` from that file, so
  the film cannot drift from the product.
- **Type:** Inter 400/700, already in `static/fonts/`. Load locally — never a CDN,
  for the same reason the app never does.
- **Texture:** warm paper, 1px hairlines, `--radius-md/lg`, one elevation only. No
  gradients, no glassmorphism, no floating 3D cards.
- **Brand:** the torn-paper slip (`src/lib/components/Logo.svelte`, `static/logo.svg`).
  **Tally stays out of this film** — the mascot undercuts the "serious tool" read on
  first contact. Tally remains for social replies and docs.
- **Data:** fictional. `Acme Corporation`, `Northwind`, `SGD 21,847.80`-scale numbers.

## 8. Motion language

Match the app's own rules, already written down in `app.css`: short, decelerating,
**no overshoot on money tables**.

- Easing: `Easing.bezier(0.22, 1, 0.36, 1)` — the app's `--ease-out`.
- Entrances 120–180ms, 8px rise + 0.99→1 scale. Exits 100ms, fade only.
- Digits roll, never scramble; always `font-variant-numeric: tabular-nums`.
- Camera: max 1.03 scale, one direction, no pan beyond 40px.
- **The notification is the only element allowed to enter from the frame edge**
  (12px + fade, 200ms). Everything else originates in place, so the edge reads as
  "the outside world" — the one time the app is allowed to interrupt.
- The tray icon's single pulse is the only attention-seeking motion. Nothing blinks,
  nothing breathes.
- Clock digits flip rather than crossfade, so the time jump is unmistakable.
- The `network: off` LED stays unlit through all eight beats, including when the
  notification fires. That is the joke and the proof.

## 9. Sound

**Music — required.** One 35s bed, at `tools/motion/public/audio/music.mp3` (mp3,
m4a, wav or ogg). Warm, sparse, low-mid, no vocals, no "corporate ukulele". It has
to survive playback at 30% volume on a muted-first platform, which mostly means:
leave the low mids alone and do not compete with speech. 90–110 BPM, resolved by
35.0s, no hard ending that would fight the final card.

- **Licensing is a gate, not a detail.** The track must be cleared for commercial
  use and the licence recorded next to the file (composer, source, licence id). The
  project already runs a licence check on the app; hold the film to the same bar.
- The mix is deterministic: the bed sits at 0.32 and drops to 0.42× that under each
  VO line, with a 15-frame fade in and a 45-frame fade out. No side-chaining.
- Beat 5b is where the bed comes back up: no speech from 17.0s to 18.4s.

**Voiceover — required, English only.** Script in §5.2. Nine clips, one per line, so
a re-take or a retime touches one file and nothing else.

**SFX.** Three families only: **tick** (rows, receipts), **soft pop** (inserts),
**paper** (pages, end card). The reminder chime at 5c is a rising fourth —
deliberately different in register from the payment chime at 8.0s so a rewatch does
not confuse them. Nothing plays during 5b.

**Placeholder.** `npm run vo:scratch` generates a Windows SAPI read so the edit can
be timed before a human exists. It is clearly robotic and must not ship.

## 10. Remotion build plan

```
tools/motion/                 ← own package.json; never a dependency of the app
  package.json
  remotion.config.ts
  tsconfig.json
  scripts/
    sync-theme.mjs            src/app.css  ->  src/theme.ts
    sync-assets.mjs           static/fonts + logo  ->  public/
  public/                     generated by sync-assets (gitignored)
  src/
    index.ts                  registerRoot
    Root.tsx                  Master · Vertical · Square · Cutdown
    timeline.ts               single TIMELINE table; all compositions derive from it
    theme.ts                  GENERATED — do not hand-edit
    fonts.ts                  loadFont from local TTFs
    lib/anim.ts               enter · progress · clamp
    lib/format.ts             SGD money formatting
    components/               Paper · SceneFrame · Fit · OfflineLed · LogoMark ·
                              AppWindow · Money · StatusPill · Bar · Cursor ·
                              MenuBar · TrayIcon · NotificationCard · Clock · EndCard
    scenes/                   Promise · Invoice · Paid · Chase · TrayReminder ·
                              QuoteToPdf · Mcp · Close
    compositions/Film.tsx     beats -> <Sequence>s
```

- **Build the UI, don't screen-record.** Recreated components stay crisp at any
  resolution and let individual values animate. The captures in `docs/screenshots/`
  are fidelity references only.
- Scenes are laid out at design size and scaled by `<Fit>`, so one scene renders at
  16:9, 9:16 and 1:1 without a second implementation.
- `NotificationCard` takes `title`/`body` props defaulting to the shipping strings,
  so the Indonesian variant is a one-line swap.
- Every duration lives in `timeline.ts`. The 16s cut is a different beat list over the
  same scenes — not a re-edit.

## 11. Hard constraint — keep it out of the product

`check:offline` scans `src/` and `src-tauri/src/`; `check:licenses` reads the Tauri
cargo graph. Neither touches `tools/`. Keep it that way:

- `tools/motion/` has its **own `package.json`**, absent from the root manifest.
- No network API in the app source, no remote font or asset URL anywhere in the film.
- Remotion's renderer downloads a headless Chrome on first use. That is a
  developer-machine concern only; it must never become part of the app build.
- `tools/motion/README.md` states this so nobody hoists the dependency by reflex.

## 12. Cutdown rules

The 17s cut is promise → invoice → tray/reminder → end card. It keeps only **beat 5**
of the two overdue beats: beat 4 (in-app chasing) and beat 5 (still remembering when
closed) are adjacent, and shipping both makes the film feel like it is about debt
collection rather than invoicing. In the 35s master both stay, because one is *in the
app* and the other is *out of it* — that contrast is the point.

## 13. Decisions

Settled 2026-10-23:

- **Music — yes.** One licensed bed in the master. The slot and the ducking mix are
  built; the track itself still has to be chosen and cleared (§9).
- **Voiceover — yes, English only.** Script final in §5.2. No Indonesian cut.
- **CTA — the GitHub repo.** `github.com/zuhdalfan/ledgerly`, read from the git
  remote. It appears once, on the end card, as a quiet pill under the platform line.
- **Silent variant — yes.** `MasterSilent` is the same edit with the audio layer
  off, for muted-first placements.

Still open:

1. **Which track.** Needs a licensing decision before the master is published.

## 14. Build status

The scaffold is in `tools/motion/` and renders: `npm install && npm run studio`
opens Remotion Studio with all five compositions, and `npm run render` writes the
35s master.

**Done**

- Timeline, eight scenes, five compositions (Master / MasterSilent / Vertical /
  Square / Cutdown).
- Token layer generated from `src/app.css`; fonts and the mark copied from the repo.
- The tray/reminder beat at full sub-beat timing, with the shipping notification copy.
- Music + VO layer: music bed with automatic ducking under speech, nine VO clips
  placed by beat so the 17s cutdown reuses them without a second script.
- End-card CTA: `github.com/zuhdalfan/ledgerly`.
- A scratch VO (`npm run vo:scratch`) covering the real script, so the edit can be
  watched with timing today. It is a placeholder and is gitignored.
- Type-check clean (`npm run typecheck`) and verified frame-by-frame with stills.

**Waiting on**

- The music track, for the same reason as above — the slot is `public/audio/music.mp3`.
- A human VO read, dropped into `public/audio/vo/` under the ids in §5.2. It replaces
  the scratch read automatically; nothing else changes.

**Known gap — portrait**

`Vertical` currently reuses the landscape stage, scaled and centred, so a 16:9
window leaves large empty margins in a 9:16 frame. That is a composition problem,
not a scaling problem. The fix is a portrait pass that uses the app's own narrow
layout (`<820px`: top bar + drawer, from `TABLET-UI`) as the reference, plus a
per-scene portrait arrangement — headline, artifact, and one bottom-anchored line.
It should be done before the vertical file is published; the landscape master is
unaffected.

