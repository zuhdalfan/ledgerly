"""Record short, captioned marketing cuts of Ledgerly.

    python tools/preview-record/marketing.py --list
    python tools/preview-record/marketing.py --format landscape
    python tools/preview-record/marketing.py --format portrait --only hero

Unlike the README walkthrough (complete, uncaptioned, one screen per idea),
these are payoff-first: one message each, captioned, ~8-12s, with a brand chip
and an end card — ready for a landing hero, feature cards or social.

Navigation goes through the command palette, so the same script works at every
viewport (the sidebar is hidden in portrait).

Development tooling only.
"""

from __future__ import annotations

import argparse
import json
import pathlib
import sys
import time

HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import record as walk  # noqa: E402

STUB = HERE / "demo-stub.js"
CURSOR = HERE / "cursor.js"
OVERLAY = HERE / "marketing.js"
OUT = HERE / "out-marketing"

# One script, three frames: landscape for the web, square for the feed, portrait
# for Reels/TikTok/Shorts.
FORMATS = {
    "landscape": {"width": 1280, "height": 720},
    "square": {"width": 1080, "height": 1080},
    "portrait": {"width": 720, "height": 1280},
}

BRAND = "Ledgerly"
CTA = "Free · MIT · github.com/zuhdalfan/ledgerly"


def go(d, query: str, settle: int = 900):
    """Jump somewhere via the command palette — works at any viewport."""
    d.key("Control+k", settle=650)
    d.page.keyboard.type(query, delay=35)
    d.wait(350)
    d.key("Enter", settle=settle)


# --------------------------------------------------------------------------- #
def hero(d):
    """The whole product in one beat: an overdue invoice gets paid."""
    d.brand(True)
    d.caption("Get paid, without the busywork", "Record a payment in two fields.")
    # Straight to the invoice by palette: the list table scrolls sideways at
    # narrow widths, so a row link is not reliably clickable in portrait.
    go(d, "INV-0041")
    d.scroll_to("text=Record a payment")
    pay = d.card("Record a payment")
    d.type_loc(pay.locator("input[inputmode=decimal]"), "8240.40", delay=45)
    d.type_loc(pay.locator("input[placeholder='bank / cash / card']"), "bank", delay=45)
    d.click("button:has-text('Record payment')", settle=1600)
    d.wait(1400)  # let the celebration land
    d.end_card(BRAND, "Invoicing that stays on your machine.", CTA)


def create(d):
    """Speed: an invoice from nothing, on camera."""
    d.brand(True)
    d.caption("An invoice in seconds")
    go(d, "New invoice")
    d.select("select >> nth=0", "globex")
    d.click("button:has-text('Add from catalog')", settle=1100)
    d.click("text=Design hour", settle=900)
    d.scroll_to("text=Totals")
    d.caption("Line items, tax and totals — recomputed as you type.")
    d.wait(900)
    d.end_card(BRAND, "Built for freelancers and small studios.", CTA)


def offline(d):
    """The differentiator: it is a file on your machine."""
    d.brand(True)
    d.caption("No account. No cloud. No network.", "Everything lives in one local file.")
    d.wait(1200)
    d.end_card(BRAND, "Offline by design — the build fails on any network code.", CTA)


def estimate(d):
    """Quote → invoice in one click."""
    d.brand(True)
    d.caption("Quote it, then invoice it", "One click turns an estimate into an invoice.")
    d.click("button:has-text('Convert to invoice')", settle=1600)
    d.wait(700)
    d.end_card(BRAND, "Estimates and invoices, linked.", CTA)


def chase(d):
    """The pain it removes: chasing late payers."""
    d.brand(True)
    d.scroll_to("text=Chasing this one")
    d.caption("Chase late payments in one click", "The reminder writes itself — copy and send.")
    d.click("button:has-text('Copy reminder')", settle=1400)
    d.end_card(BRAND, "No mail server. No network. Just the words.", CTA)


def mcp(d):
    """The differentiator that surprises people."""
    d.brand(True)
    d.caption("Your AI assistant can read your books", "Over MCP, on standard input and output.")
    d.click("input[type=checkbox] >> nth=0", settle=900)
    d.click("input[type=checkbox] >> nth=1", settle=900)
    d.scroll_to("text=What the assistant can do")
    d.wait(800)
    d.end_card(BRAND, "Still offline. The assistant runs on your machine.", CTA)


SEGMENTS = [
    {"id": "hero", "title": "Get paid, without the busywork", "start": "/", "script": hero},
    {"id": "create", "title": "An invoice in seconds", "start": "/", "script": create},
    {"id": "offline", "title": "No account, no cloud, no network", "start": "/settings/data", "script": offline},
    {"id": "estimate", "title": "Quote it, then invoice it", "start": "/estimates/est-0012", "script": estimate},
    {"id": "chase", "title": "Chase late payments", "start": "/invoices/inv-0041", "script": chase},
    {"id": "mcp", "title": "Your AI assistant can read your books", "start": "/settings/ai", "script": mcp},
]


def record_format(browser, fmt: str, segments, scale: float, manifest: dict):
    viewport = FORMATS[fmt]
    for seg in segments:
        sid = seg["id"]
        print(f"\n=== {fmt} · {sid} — {seg['title']}")
        video_dir = OUT / fmt / "_video"
        video_dir.mkdir(parents=True, exist_ok=True)
        ctx = browser.new_context(
            viewport=viewport,
            record_video_dir=str(video_dir),
            record_video_size=viewport,
            device_scale_factor=1,
            permissions=["clipboard-read", "clipboard-write"],
        )
        t0 = time.monotonic()
        ctx.add_init_script(path=str(STUB))
        ctx.add_init_script(path=str(CURSOR))
        ctx.add_init_script(path=str(OVERLAY))
        page = ctx.new_page()
        page.set_default_timeout(20000)
        d = walk.Demo(page, walk.BASE, scale=scale)
        status, error, trim = "ok", None, 0.0
        try:
            d.goto(seg["start"])
            d.wait(900)
            trim = max(0.0, time.monotonic() - t0 - 0.15)
            seg["script"](d)
            d.wait(600)
        except Exception as exc:
            status, error = "failed", f"{type(exc).__name__}: {exc}"
            print(f"    FAILED: {error}")
        finally:
            ctx.close()
        videos = sorted(video_dir.glob("*.webm"), key=lambda p: p.stat().st_mtime)
        target = OUT / fmt / f"{sid}.webm"
        if videos:
            newest = videos[-1]
            if target.exists():
                target.unlink()
            newest.replace(target)
        clip = {"id": sid, "format": fmt, "title": seg["title"], "status": status,
                "error": error, "trimStart": round(trim, 3),
                "file": str(target.relative_to(HERE)) if target.exists() else None}
        manifest["segments"] = [s for s in manifest["segments"]
                                if not (s["id"] == sid and s["format"] == fmt)] + [clip]
        print(f"    {status}  trim={trim:.2f}s")


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--format", default="landscape", choices=sorted(FORMATS))
    ap.add_argument("--only", help="comma-separated segment ids")
    ap.add_argument("--scale", type=float, default=1.0)
    ap.add_argument("--list", action="store_true")
    ap.add_argument("--no-server", action="store_true")
    args = ap.parse_args()

    if args.list:
        for s in SEGMENTS:
            print(f"{s['id']:<10} {s['title']}")
        return 0

    chosen = SEGMENTS
    if args.only:
        wanted = {x.strip() for x in args.only.split(",")}
        chosen = [s for s in SEGMENTS if s["id"] in wanted]
        missing = wanted - {s["id"] for s in chosen}
        if missing:
            raise SystemExit(f"unknown segment id(s): {', '.join(sorted(missing))}")

    OUT.mkdir(parents=True, exist_ok=True)
    manifest_path = OUT / "manifest.json"
    manifest = {"formats": FORMATS, "segments": []}
    if manifest_path.exists():
        try:
            manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
        except Exception:
            pass

    from playwright.sync_api import sync_playwright

    server = None if args.no_server else walk.start_server()
    try:
        with sync_playwright() as p:
            browser = p.chromium.launch(headless=True)
            record_format(browser, args.format, chosen, args.scale, manifest)
            browser.close()
    finally:
        walk.stop_server(server)

    order = {s["id"]: i for i, s in enumerate(SEGMENTS)}
    fmt_order = {f: i for i, f in enumerate(FORMATS)}
    manifest["segments"].sort(key=lambda s: (fmt_order.get(s["format"], 9), order.get(s["id"], 99)))
    manifest_path.write_text(json.dumps(manifest, indent=2), encoding="utf-8")
    ok = sum(1 for s in manifest["segments"] if s["status"] == "ok")
    print(f"\nRecorded {ok}/{len(manifest['segments'])} marketing cuts -> {OUT}")
    return 0 if ok == len(manifest["segments"]) else 1


if __name__ == "__main__":
    sys.exit(main())
