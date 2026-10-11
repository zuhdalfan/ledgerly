# Screenshots

Interface captures of every Ledgerly screen, at three viewport widths.

| Folder | Width | Represents |
|---|---|---|
| `desktop/` | 1280px | A desktop window. |
| `tablet/` | 768px | A tablet in portrait. |
| `phone/` | 390px | A phone-sized canvas. |

Below the 820px breakpoint the sidebar collapses into a drawer behind a top bar,
which is what the tablet and phone sets show. The app window itself has a 480px
minimum width, so the phone set is narrower than the app normally runs — it is
there to show that the layout holds up, not as a supported size.

## About the data

These are **development captures with sample data**, not a live database. The
app is a Tauri desktop program whose frontend fetches everything from the Rust
core over IPC; the captures were made against the dev server with that IPC layer
stubbed so each screen renders fully. The figures are internally consistent
(for example, a client's balance equals the sum of its open invoices), but they
are illustrative.

## Regenerating

```sh
npm run screenshots
```

One command regenerates every image. It starts the dev server, stubs the Tauri
IPC with sample data (`tools/screenshots/stub.js`), loads each route in a
headless Chromium at the three widths, and writes the PNGs back into this
folder.

It needs Python 3 with Playwright, installed once:

```sh
pip install playwright
playwright install chromium
```

The harness lives in `tools/screenshots/` and is development-only: it is not
part of the app build and ships with nothing.

