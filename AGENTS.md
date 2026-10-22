# AGENTS.md

How to pick up work on Ledgerly in a fresh session — human or agent.

## What this is

Ledgerly is a **fully local, fully offline** desktop invoicing app (MIT, free,
`v0.2.0`). A **Rust + Tauri 2** core owns all data and logic; a **Svelte 5 +
TypeScript** frontend talks to it over Tauri IPC; state lives in a single local
**SQLite** file. It makes no network requests, ever.

## Start here, every session

1. **Read `TASKS.json` first.** It is the cross-session source of truth: `current`
   holds the id in progress (`null` when idle), `nextUp` is the queue, and every
   task has a status and priority. Keep `updated` in `YYYY-MM-DD`, set `current`
   while you work, and move finished items to status `done` — don't delete them.
   Milestone ids (`M0`–`M10`) track `docs/PRD.md` §14.
2. **`git status` and `git log`** — the tree often carries uncommitted work.
3. **Skim `docs/PRD.md`** for the intent behind the area you're touching.

## The roadmap is gated

`docs/ROADMAP.md` carries the vision and the reasoning, not the work. Treat every
edit as deliberate — the reasoning comes first, the edit second:

- **Reason first, then edit.** Before changing `ROADMAP.md`, state the
  vision-level case for the change — which theme and principle it serves, and why
  now — and get it agreed. Never edit it as a side effect of finishing a task.
- **It is not a status mirror.** Do not sync it from `TASKS.json`. State, dates
  and task lists live in `TASKS.json` alone; the roadmap holds direction. A change
  that only reflects progress belongs there, not here.
- **Hold the standard.** Every deferred item keeps its own revisit trigger, every
  item references a `TASKS.json` id for state (never inlines it), and the "what we
  will not build" section stays intact.
- **Direction changes are top-down.** If the direction really shifts, revise the
  vision and principles first, then the horizons — never the reverse.

## Run, build, test

Prerequisites: Rust (stable; MSVC toolchain on Windows), Node 20+, and the
platform Tauri prerequisites (WebView2 on Windows, `libwebkit2gtk-4.1-dev` etc.
on Linux).

```bash
npm install
npm run tauri dev        # the app (builds the MCP sidecar, then vite)
npm run tauri build      # installers → src-tauri/target/release/bundle/
npm run verify           # the full CI gate — run before calling anything done
```

`npm run verify` chains: `check:offline` → `check:licenses` → `check:contrast` →
`check` (svelte-check) → `build` → `cargo fmt --check` → `cargo clippy -D warnings`
→ `cargo test`. Individual pieces:

```bash
npm run check            # svelte-check (types)
npm run check:offline    # no network access anywhere
npm run check:licenses   # nothing copyleft in the shipped graph
npm run check:contrast   # WCAG AA for every colour pair the UI renders
cargo test --manifest-path src-tauri/Cargo.toml
npm run screenshots      # regenerate docs/screenshots (needs Python + Playwright)
```

> `npm run dev` on its own serves only the Svelte frontend, so the Rust IPC is
> missing and the app shows the "could not open" gate. Use `tauri dev` to run the
> app; `npm run dev` is only useful behind the screenshot stub.

`npm run screenshots` needs `pip install playwright && playwright install chromium`
once. It drives a headless Chromium with the IPC stubbed (`tools/screenshots/`).

## Hard invariants — do not break these

1. **Offline, always.** This is the product's core promise. `check:offline` fails
   on any HTTP client crate, any browser networking API (`fetch`, `XMLHttpRequest`,
   `WebSocket`, `EventSource`, `sendBeacon`), or a remote URL in `src/` or
   `src-tauri/src/`. **Before adding a dependency, confirm it pulls in no network
   stack.** External URLs are permitted only for a small doc/donation allowlist.
2. **Tokens, not literals.** All colour, spacing and type come from the CSS
   variables in `src/app.css`, whose comments carry the accessibility rationale.
   When you introduce a foreground/background pair, add it to the pair list in
   `scripts/check-contrast.mjs` — that script measures every pair the UI renders.
3. **Licences.** `check:licenses` fails the build on copyleft in the shipped
   graph (the binary is MIT). Weak-copyleft (`MPL-2.0`) crates are reported as
   `WATCH` for a human eye, not blocked.
4. **Two binaries.** The crate builds `ledgerly` and `ledgerly-mcp`.
   `default-run = "ledgerly"` in `src-tauri/Cargo.toml` is what keeps `cargo run`
   and `tauri dev` working — do not remove it.
5. **Derive state in Rust, not Svelte.** Invoice/estimate status, totals, aging
   and balances are computed in the core. The frontend displays them; it never
   recomputes them.

## Layout

```
TASKS.json          Cross-session task tracker (read this first)
docs/               PRD, roadmap, release checklist, AI/MCP guide, screenshots
src/                Svelte 5 frontend
  lib/api.ts        The single IPC surface — every backend call lives here
  lib/components/   Shared UI (Logo, Mascot, LineItems, AsyncState, …)
  routes/           Pages (SvelteKit file-based routing)
src-tauri/          Rust core (Tauri backend)
  src/commands.rs   #[tauri::command] handlers exposed over IPC
  src/{db,domain,pdf,security,mcp}/
  src/bin/ledgerly-mcp.rs   The MCP sidecar binary
scripts/            CI helpers (offline, licences, contrast) + sidecar build
tools/screenshots/  Screenshot harness (stub + Playwright capture)
static/             Fonts, favicon, logo.svg
```

## Conventions

- **Commits:** short, imperative subject. No AI co-author trailer.
- **Svelte 5 runes** (`$state`, `$derived`, `$props`) and TypeScript throughout.
- **Comments explain *why*.** The codebase is deliberately heavy on rationale
  comments (see `src/app.css`, `src/lib/list.ts`). Match that tone.
- **Screenshots are deliverables.** After a UI change, run `npm run screenshots`
  and keep `docs/screenshots/` current — the README links to it.
- Brand: the mark is the torn paper slip (`src/lib/components/Logo.svelte`,
  `static/logo.svg`); the mascot is "Tally" (`Mascot.svelte`).

## Gotchas

- **Port 1420 is strict.** A leftover dev server blocks `tauri dev`; kill whatever
  holds it first.
- **Stale cargo cache** after moving the checkout causes a `tauri_build` failure
  looking for files under the old path — fix with `cargo clean` in `src-tauri`.
- **The sidecar** is a second crate bin (`src/bin/ledgerly-mcp.rs`).
  `scripts/build-mcp-sidecar.mjs` builds it for `tauri dev`; `tauri build`
  compiles and bundles it next to the app with the rest of the crate. Do not
  also list it in `bundle.externalBin` — since tauri-cli 2.12 that ships it
  twice, and WiX refuses the MSI (ICE30).

## Releasing

See `docs/RELEASING.md`. The version must agree in `package.json`,
`src-tauri/Cargo.toml` and `src-tauri/tauri.conf.json`. Bundles are built per-OS
(no cross-compilation). Signing and notarization are blocked on owner-held
credentials and there is no release workflow yet; the manual, GUI-only checks in
`RELEASING.md` still have to be done by hand.
