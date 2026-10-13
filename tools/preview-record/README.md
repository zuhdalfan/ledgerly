# preview-record

Records the Ledgerly feature walkthrough in `docs/preview/`.

The app is a Tauri desktop program whose frontend talks to the Rust core over
IPC. This harness runs that **real frontend** in headless Chromium, answers the
IPC from a stateful stub, drives it with a scripted cursor, and records video.
That is why the previews show real UI behaviour — menus opening, totals
recomputing, dialogs appearing — rather than a mocked slideshow.

Development tooling only: it is not part of the app build and ships with nothing.

## Files

| File | Purpose |
|---|---|
| `demo-stub.js` | Stateful, deterministic IPC stub. Seeds a rich demo dataset and *mutates* it (create/update/delete, totals, statuses, reports), so save-then-see-it flows work. Also answers the Tauri **dialog** plugin (`save`/`open`/`ask`/`message`) so export and backup flows complete. |
| `cursor.js` | A visible cursor + click ripple that follows Playwright's mouse events (the video recorder does not draw the OS cursor). |
| `record.py` | Starts the dev server, runs each segment's script in a fresh browser context, and records one webm per segment. Also `--list`, `--only <ids>`, `--scale`. |
| `segments.py` | The walkthrough scripts — one per chapter — plus the segment table (id, chapter, title, start route). |
| `encode.py` | Trims each clip's page-load lead-in and writes mp4 (all) and gif (the focused ones) into `docs/preview/`. |
| `out/` | Recorded webm + `manifest.json` (gitignored). |

## Run it

```sh
python record.py                 # every segment
python record.py --only 04-payments
python record.py --list
python encode.py                 # -> ../../docs/preview/
```

Requires Python 3 with Playwright (`pip install playwright && playwright install
chromium`) and `ffmpeg` on PATH. `record.py` starts and stops the Vite dev server
itself (`--no-server` to bring your own).

## How a segment is written

A segment is a small function that drives `record.Demo`:

```python
def catalog(d):
    nav(d, "Catalog")
    d.type_loc(d.label("Name"), "Discovery sprint", delay=50)
    d.click("button:has-text('Add item')", settle=1100)
    d.click("button:has-text('Edit')", settle=800)
    d.click("button:has-text('Cancel')", settle=600)
```

Every pointer action glides the mouse to the target first, so the recorded
cursor travels on screen. Two things to know when editing:

- **The app scrolls inside `main`, not the window.** `Demo` scrolls elements
  into view through their own ancestors before clicking; `d.scroll(px)` moves the
  wheel over `main`.
- **The stub resets per run, not per navigation.** State lives in
  `sessionStorage`, so it survives a full page load within a segment but each
  segment starts from the same seed.

## Adding a chapter

1. Add a function to `segments.py` and an entry to `SEGMENTS`.
2. `python record.py --only <id>` to iterate, `--only` merges into
   `out/manifest.json` so it never drops other chapters.
3. Add the id to `GIF_IDS` in `encode.py` if it should also get a gif.
4. Update `docs/preview/README.md`, the index.

## Limits

The stub cannot make the app do things that need the desktop runtime: writing
files, the OS file dialogs and PDF viewer, and the Argon2id lock are simulated at
the UI boundary. The MCP server is a separate stdio process and is never part of
the app UI. These are recorded as *blocked* in
`docs/preview-feature-audit.md`, never faked as working.
