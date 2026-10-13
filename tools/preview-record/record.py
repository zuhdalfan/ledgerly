"""Record real, cursor-driven walkthrough clips of Ledgerly.

    python tools/preview-record/record.py                 # every segment
    python tools/preview-record/record.py --only invoices
    python tools/preview-record/record.py --list

Starts the Vite dev server, drives the *real* frontend in headless Chromium
with a stateful IPC stub (`demo-stub.js`) and a visible cursor overlay
(`cursor.js`), and records one webm per segment via Playwright. `encode.py`
then turns those into mp4/gif.

Development tooling only: nothing here ships with the app.
"""

from __future__ import annotations

import argparse
import json
import os
import pathlib
import signal
import socket
import subprocess
import sys
import time

# The app's copy contains glyphs (☰, –, ·) that the Windows console codepage
# cannot encode; without this a stray print would crash a recording run.
try:
    sys.stdout.reconfigure(encoding="utf-8", errors="replace")
    sys.stderr.reconfigure(encoding="utf-8", errors="replace")
except Exception:
    pass

ROOT = pathlib.Path(__file__).resolve().parents[2]
HERE = pathlib.Path(__file__).resolve().parent
STUB = HERE / "demo-stub.js"
CURSOR = HERE / "cursor.js"
OUT = HERE / "out"
VIDEO = OUT / "video"

HOST, PORT = "127.0.0.1", 1420
BASE = f"http://{HOST}:{PORT}"
VIEWPORT = {"width": 1280, "height": 800}


# --------------------------------------------------------------------------- #
# Driving helpers
# --------------------------------------------------------------------------- #
class Demo:
    """Small, readable action vocabulary for a segment script.

    Every pointer action glides the mouse first so the injected cursor overlay
    travels on screen the way a person would move it.
    """

    def __init__(self, page, base: str, scale: float = 1.0):
        self.page = page
        self.base = base
        self.scale = scale

    def wait(self, ms: int):
        self.page.wait_for_timeout(int(ms * self.scale))

    def goto(self, path: str, settle: int = 1400):
        self.page.goto(f"{self.base}{path}", wait_until="load")
        self.page.wait_for_selector("#sidebar, .gate", timeout=25000)
        self.wait(settle)

    def _center_loc(self, locator):
        loc = locator.first
        try:
            handle = loc.element_handle(timeout=15000)
        except Exception as exc:
            raise RuntimeError(f"element not found ({repr(loc)[:180]}): {exc}") from exc
        if handle is None:
            raise RuntimeError(f"element not found ({repr(loc)[:180]})")
        # The app scrolls inside `main`, not the window, so bring the element
        # into view through its own ancestors before reading a click point.
        self.page.evaluate("el => el.scrollIntoView({block: 'center', inline: 'nearest'})", handle)
        self.wait(160)
        box = loc.bounding_box()
        if not box:
            raise RuntimeError("element has no box after scrolling into view")
        return box["x"] + box["width"] / 2, box["y"] + box["height"] / 2

    def _center(self, target):
        return self._center_loc(self.page.locator(target) if isinstance(target, str) else target)

    def move_to(self, selector: str, steps: int = 18, settle: int = 260):
        x, y = self._center(selector)
        self.page.mouse.move(x, y, steps=steps)
        self.wait(settle)

    def move(self, x: int, y: int, steps: int = 18, settle: int = 220):
        self.page.mouse.move(x, y, steps=steps)
        self.wait(settle)

    def click(self, selector: str, settle: int = 700, steps: int = 18):
        x, y = self._center(selector)
        self.page.mouse.move(x, y, steps=steps)
        self.wait(240)
        self.page.mouse.click(x, y)
        self.wait(settle)

    def click_text(self, text: str, settle: int = 700, exact: bool = False):
        self.click(f"text={text}" if not exact else f"text=\"{text}\"", settle=settle)

    def hover(self, selector: str, settle: int = 500):
        self.move_to(selector, settle=settle)

    def type(self, selector: str, text: str, clear: bool = True, delay: int = 45):
        loc = self.page.locator(selector).first
        loc.click()
        self.wait(200)
        if clear:
            loc.fill("")
        loc.type(text, delay=int(delay * self.scale))
        self.wait(280)

    def select(self, target, value: str, settle: int = 650):
        if isinstance(target, str):
            self.move_to(target, settle=180)
            loc = self.page.locator(target).first
        else:
            x, y = self._center_loc(target)
            self.page.mouse.move(x, y, steps=12)
            self.wait(180)
            loc = target.first
        loc.select_option(value)
        self.wait(settle)

    def scroll(self, dy: int, settle: int = 700):
        self.page.mouse.move(760, 420)  # over `main`, so the wheel scrolls it
        self.page.mouse.wheel(0, dy)
        self.wait(settle)

    def scroll_to(self, selector: str, settle: int = 700):
        try:
            handle = self.page.locator(selector).first.element_handle(timeout=8000)
        except Exception as exc:
            raise RuntimeError(f"scroll target not found: {selector!r} ({exc})") from exc
        if handle is not None:
            self.page.evaluate("el => el.scrollIntoView({block: 'start', inline: 'nearest'})", handle)
        self.wait(settle)

    def key(self, combo: str, settle: int = 500):
        self.page.keyboard.press(combo)
        self.wait(settle)

    def assert_visible(self, selector: str, note: str = ""):
        if self.page.locator(selector).count() == 0:
            raise RuntimeError(f"expected {selector!r} to be present {note}".strip())

    # --- locator-based helpers (for fields with no stable CSS selector) ---
    def label(self, name: str):
        return self.page.get_by_label(name, exact=False).first

    def role(self, role: str, name: str | None = None):
        return self.page.get_by_role(role, name=name).first if name else self.page.get_by_role(role).first

    def click_loc(self, locator, settle: int = 700, steps: int = 18):
        x, y = self._center_loc(locator)
        self.page.mouse.move(x, y, steps=steps)
        self.wait(240)
        self.page.mouse.click(x, y)
        self.wait(settle)

    def type_loc(self, locator, text: str, clear: bool = True, delay: int = 45):
        loc = locator.first
        self.click_loc(loc, settle=200)
        if clear:
            loc.fill("")
        loc.type(text, delay=int(delay * self.scale))
        self.wait(280)

    def card(self, text: str):
        """Scope to a card by a heading/text it contains — avoids the many
        same-shaped inputs (dates, decimals) that appear on one page."""
        return self.page.locator(".card", has_text=text).first


# --------------------------------------------------------------------------- #
# Server
# --------------------------------------------------------------------------- #
def wait_port(port: int, timeout: float = 120.0):
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        with socket.socket() as s:
            s.settimeout(1.0)
            if s.connect_ex((HOST, port)) == 0:
                return
        time.sleep(0.5)
    raise SystemExit(f"the dev server did not open port {port} in {timeout:.0f}s")


def start_server():
    if os.name == "nt":
        proc = subprocess.Popen(
            "npm run dev", cwd=ROOT, shell=True,
            creationflags=subprocess.CREATE_NEW_PROCESS_GROUP,
        )
    else:
        proc = subprocess.Popen(["npm", "run", "dev"], cwd=ROOT, start_new_session=True)
    wait_port(PORT)
    return proc


def stop_server(proc):
    if proc is None or proc.poll() is not None:
        return
    if os.name == "nt":
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(proc.pid)], capture_output=True)
    else:
        try:
            os.killpg(os.getpgid(proc.pid), signal.SIGTERM)
        except ProcessLookupError:
            pass


# --------------------------------------------------------------------------- #
# Runner
# --------------------------------------------------------------------------- #
def record(segments, scale: float, browser, manifest: dict):
    for seg in segments:
        sid = seg["id"]
        print(f"\n=== recording {sid} — {seg['title']}")
        video_dir = VIDEO
        video_dir.mkdir(parents=True, exist_ok=True)
        ctx = browser.new_context(
            viewport=VIEWPORT,
            record_video_dir=str(video_dir),
            record_video_size=VIEWPORT,
            device_scale_factor=1,
            # The reminder/config "Copy" buttons write to the clipboard; without
            # permission they fall back to a "copy by hand" message on screen.
            permissions=["clipboard-read", "clipboard-write"],
        )
        t0 = time.monotonic()
        ctx.add_init_script(path=str(STUB))
        ctx.add_init_script(path=str(CURSOR))
        page = ctx.new_page()
        page.set_default_timeout(20000)
        d = Demo(page, BASE, scale=scale)
        status, error, trim = "ok", None, 0.0
        try:
            d.goto(seg.get("start", "/"))
            d.wait(900)
            # Everything before this line is page load; trim it from the clip.
            trim = max(0.0, time.monotonic() - t0 - 0.15)
            seg["script"](d)
            d.wait(900)
        except Exception as exc:  # keep going; report it
            status, error = "failed", f"{type(exc).__name__}: {exc}"
            print(f"    FAILED: {error}")
        finally:
            ctx.close()
        videos = sorted(video_dir.glob("*.webm"), key=lambda p: p.stat().st_mtime)
        target = OUT / f"{sid}.webm"
        if videos:
            newest = videos[-1]
            if target.exists():
                target.unlink()
            newest.replace(target)
        clip = {"id": sid, "title": seg["title"], "chapter": seg["chapter"],
                "status": status, "error": error, "trimStart": round(trim, 3),
                "file": str(target.relative_to(HERE)) if target.exists() else None}
        # Merge by id so recording a subset (--only) never drops the others.
        manifest["segments"] = [s for s in manifest["segments"] if s["id"] != sid] + [clip]
        print(f"    {status}  trim={trim:.2f}s  -> {clip['file']}")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--only", help="comma-separated segment ids")
    ap.add_argument("--scale", type=float, default=1.0, help="pace multiplier (0.5 = twice as fast)")
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--no-server", action="store_true", help="assume a dev server is already up")
    args = ap.parse_args()

    sys.path.insert(0, str(HERE))
    import segments as seg_mod  # noqa: E402

    all_segments = seg_mod.SEGMENTS
    if args.list:
        for s in all_segments:
            print(f"{s['id']:<22} [{s['chapter']}] {s['title']}")
        return 0

    chosen = all_segments
    if args.only:
        wanted = {x.strip() for x in args.only.split(",")}
        chosen = [s for s in all_segments if s["id"] in wanted]
        missing = wanted - {s["id"] for s in chosen}
        if missing:
            raise SystemExit(f"unknown segment id(s): {', '.join(sorted(missing))}")

    OUT.mkdir(parents=True, exist_ok=True)
    if VIDEO.exists():
        for f in VIDEO.glob("*.webm"):
            f.unlink()

    from playwright.sync_api import sync_playwright

    server = None if args.no_server else start_server()
    manifest = {"viewport": VIEWPORT, "segments": []}
    existing = OUT / "manifest.json"
    if existing.exists():
        try:
            manifest = json.loads(existing.read_text(encoding="utf-8"))
        except Exception:
            pass
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            record(chosen, args.scale, browser, manifest)
            browser.close()
    finally:
        stop_server(server)

    order = {s["id"]: i for i, s in enumerate(all_segments)}
    manifest["segments"].sort(key=lambda s: order.get(s["id"], 999))
    (OUT / "manifest.json").write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    ok = sum(1 for s in manifest["segments"] if s["status"] == "ok")
    print(f"\nRecorded {ok}/{len(manifest['segments'])} segments -> {OUT}")
    return 0 if ok == len(manifest["segments"]) else 1


if __name__ == "__main__":
    sys.exit(main())
