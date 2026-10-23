"""Stage 4 asset capture for the Ledgerly promo.

Produces the three-piece set the 2.5D page camera needs:

  1. full-page 2x texture per route   (public/textures/live/<name>-full.png)
  2. a 1920x1080 window shot per route (<name>-win.png) for framed shots
  3. element cutouts + layout.json     (page-space bbox for every element)

Ledgerly is a Tauri app: its frontend calls the Rust core over IPC, so a plain
browser cannot render it. This harness serves the frontend with `npm run dev`
and stubs the IPC bridge with fictional sample data (capture/stub.js), exactly
like tools/screenshots/capture.py does for the still screenshots.

Requires: Python 3 + Playwright (pip install playwright; playwright install chromium).

    python capture/capture.py
"""

import json
import os
import pathlib
import signal
import socket
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[4]  # the ledgerly repo root
HERE = pathlib.Path(__file__).resolve().parent
PROJECT = HERE.parent
STUB = HERE / "stub.js"
OUT = PROJECT / "public" / "textures" / "live"
LAYOUT = PROJECT / "src" / "live-layout.json"

HOST = "127.0.0.1"
PORT = 1420
BASE = f"http://{HOST}:{PORT}"

VIEWPORT = {"width": 1920, "height": 1080}
SCALE = 2

# Selectors the scenes fly in or mask. `all` collects every match.
BOXES = [
    {"key": "headings", "selector": "main h1, main h2", "all": True},
    {"key": "cards", "selector": "main .card", "all": True, "max": 14},
    {"key": "stats", "selector": "main .stat", "all": True},
    {"key": "rows", "selector": "main table tbody tr", "all": True, "max": 12},
    {"key": "badges", "selector": "main .badge", "all": True, "max": 12},
    {"key": "toolbar", "selector": "main .toolbar", "all": True, "max": 4},
    {"key": "search", "selector": "main .toolbar input, main input[placeholder]", "all": True, "max": 4},
    {"key": "selects", "selector": "main select", "all": True, "max": 4},
    {"key": "buttons", "selector": "main .btn", "all": True, "max": 8},
]

CUTOUTS = [
    {"name": "card", "selector": "main .card", "all": True, "max": 6},
    {"name": "row", "selector": "main table tbody tr", "all": True, "max": 6},
    {"name": "stat", "selector": "main .stat", "all": True},
    {"name": "aside", "selector": "aside"},
    {"name": "head", "selector": "main .page-head"},
    {"name": "toolbar", "selector": "main .toolbar"},
]

PAGES = [
    {"name": "dashboard", "path": "/"},
    {"name": "invoices", "path": "/invoices"},
    {"name": "editor", "path": "/invoices/new"},
    {"name": "invoice", "path": "/invoices/inv-0041"},
    {"name": "clients", "path": "/clients"},
    {"name": "client", "path": "/clients/acme"},
    {"name": "catalog", "path": "/catalog"},
    {"name": "estimates", "path": "/estimates"},
    {"name": "estimate", "path": "/estimates/est-0012"},
    {"name": "reports", "path": "/reports"},
    {"name": "ai", "path": "/settings/ai"},
    {"name": "settings", "path": "/settings"},
]

# The app grid is 100vh with the page content scrolling inside <main>. To get a
# single tall texture we let the document grow instead.
EXPAND_CSS = """
html, body { height: auto !important; }
.app { height: auto !important; min-height: 100vh !important; grid-template-rows: auto !important; }
main { overflow: visible !important; height: auto !important; }
aside { position: static !important; transform: none !important; height: auto !important; }
"""


def start_dev_server():
    if os.name == "nt":
        return subprocess.Popen(
            "npm run dev",
            cwd=ROOT,
            shell=True,
            creationflags=subprocess.CREATE_NEW_PROCESS_GROUP,
        )
    return subprocess.Popen(["npm", "run", "dev"], cwd=ROOT, start_new_session=True)


def wait_for_port(port, timeout=120.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        with socket.socket() as sock:
            sock.settimeout(1.0)
            if sock.connect_ex((HOST, port)) == 0:
                return
        time.sleep(0.5)
    raise SystemExit(f"the dev server did not open port {port} within {timeout:.0f}s")


def stop_dev_server(server):
    if server.poll() is not None:
        return
    if os.name == "nt":
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(server.pid)], capture_output=True)
    else:
        try:
            os.killpg(os.getpgid(server.pid), signal.SIGTERM)
        except ProcessLookupError:
            pass


def main():
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        raise SystemExit("Playwright missing. Run:\n    pip install playwright\n    playwright install chromium")

    if not STUB.exists():
        raise SystemExit(f"missing stub at {STUB}")

    OUT.mkdir(parents=True, exist_ok=True)
    LAYOUT.parent.mkdir(parents=True, exist_ok=True)

    server = start_dev_server()
    layout = {"pageW": VIEWPORT["width"]}
    try:
        wait_for_port(PORT)
        with sync_playwright() as pw:
            browser = pw.chromium.launch(headless=True)
            context = browser.new_context(viewport=VIEWPORT, device_scale_factor=SCALE)
            context.add_init_script(path=str(STUB))
            page = context.new_page()

            for pg in PAGES:
                page.goto(f"{BASE}{pg['path']}", wait_until="load")
                page.wait_for_timeout(1500)
                page.evaluate("document.fonts.ready")

                # window shot: the app exactly as it runs (100vh)
                page.screenshot(path=str(OUT / f"{pg['name']}-win.png"))

                # expand the inner scroller, then capture the whole page
                page.add_style_tag(content=EXPAND_CSS)
                page.wait_for_timeout(400)
                page.evaluate("document.fonts.ready")
                pageH = page.evaluate("document.documentElement.scrollHeight")
                page.screenshot(path=str(OUT / f"{pg['name']}-full.png"), full_page=True)
                entry = {"pageH": pageH}

                # page-space bbox (Playwright's bounding_box() is viewport-relative
                # and element.screenshot() scrolls, so measure via evaluate).
                page.evaluate("window.scrollTo(0, 0)")

                def page_box(el):
                    return el.evaluate(
                        "e => { const r = e.getBoundingClientRect();"
                        " return {x: r.x + window.scrollX, y: r.y + window.scrollY,"
                        " width: r.width, height: r.height}; }"
                    )

                entry["boxes"] = {}
                for b in BOXES:
                    els = page.query_selector_all(b["selector"])
                    picked = els[: b.get("max", len(els))] if b["all"] else els[:1]
                    got = [page_box(el) for el in picked]
                    entry["boxes"][b["key"]] = got if b["all"] else (got[0] if got else None)

                entry["cutouts"] = []
                for c in CUTOUTS:
                    els = page.query_selector_all(c["selector"])
                    is_all = bool(c.get("all"))
                    picked = els[: c.get("max", len(els))] if is_all else els[:1]
                    for i, el in enumerate(picked):
                        fname = f"{c['name']}{i + 1}.png" if is_all else f"{c['name']}.png"
                        bb = page_box(el)
                        if not bb:
                            continue
                        try:
                            el.screenshot(path=str(OUT / f"{pg['name']}-{fname}"), omit_background=True)
                        except Exception as exc:  # noqa: BLE001
                            print(f"  cutout miss {fname}: {exc}")
                            continue
                        entry["cutouts"].append({"file": f"{pg['name']}-{fname}", **bb})
                    if not picked:
                        print(f"  cutout miss {c['name']} (no match: {c['selector']})")

                layout[pg["name"]] = entry
                print(f"captured {pg['name']} pageH={pageH} cutouts={len(entry['cutouts'])}", flush=True)

            browser.close()
    finally:
        stop_dev_server(server)

    LAYOUT.write_text(json.dumps(layout, indent=1), encoding="utf8")
    print(f"\nwrote {LAYOUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
