"""Capture the Ledgerly interface at three viewport widths.

    npm run screenshots        (equivalently: python tools/screenshots/capture.py)

The app is a Tauri desktop program whose frontend fetches everything from the
Rust core over IPC, so a plain browser cannot render it. This harness serves the
frontend with `npm run dev`, stubs the IPC layer with sample data
(`stub.js`), loads every screen in a headless Chromium, and writes one PNG per
screen under `docs/screenshots/`.

Requirements:

    pip install playwright
    playwright install chromium

Development tooling only: it is not part of the app build and ships with
nothing.
"""

import os
import pathlib
import signal
import socket
import subprocess
import sys
import time

ROOT = pathlib.Path(__file__).resolve().parents[2]
STUB = pathlib.Path(__file__).with_name("stub.js")
OUT = ROOT / "docs" / "screenshots"

HOST = "127.0.0.1"
PORT = 1420
BASE = f"http://{HOST}:{PORT}"

SCREENS = [
    ("01-dashboard", "/"),
    ("02-invoices", "/invoices"),
    ("03-invoice-detail", "/invoices/inv-0041"),
    ("04-invoice-editor", "/invoices/new"),
    ("05-clients", "/clients"),
    ("06-client-detail", "/clients/acme"),
    ("07-catalog", "/catalog"),
    ("08-estimates", "/estimates"),
    ("09-estimate-detail", "/estimates/est-0012"),
    ("10-reports", "/reports"),
    ("11-settings-business", "/settings"),
    ("12-settings-invoicing", "/settings/invoicing"),
    ("13-settings-security", "/settings/security"),
    ("14-settings-ai", "/settings/ai"),
    ("15-settings-data", "/settings/data"),
    ("16-settings-appearance", "/settings/appearance"),
    ("17-settings-about", "/settings/about"),
    ("18-settings-support", "/settings/support"),
    ("19-lock-screen", "/?lock=1"),
]

# Matches the widths documented in docs/screenshots/README.md.
VIEWPORTS = [
    ("desktop", {"width": 1280, "height": 2000}, 1),
    ("tablet", {"width": 768, "height": 1600}, 2),
    ("phone", {"width": 390, "height": 1400}, 2),
]


def start_dev_server() -> subprocess.Popen:
    if os.name == "nt":
        return subprocess.Popen(
            "npm run dev",
            cwd=ROOT,
            shell=True,
            creationflags=subprocess.CREATE_NEW_PROCESS_GROUP,
        )
    return subprocess.Popen(["npm", "run", "dev"], cwd=ROOT, start_new_session=True)


def wait_for_port(port: int, timeout: float = 90.0) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        with socket.socket() as sock:
            sock.settimeout(1.0)
            if sock.connect_ex((HOST, port)) == 0:
                return
        time.sleep(0.5)
    raise SystemExit(f"the dev server did not open port {port} within {timeout:.0f}s")


def stop_dev_server(server: subprocess.Popen) -> None:
    if server.poll() is not None:
        return
    # A shell-spawned `npm run dev` leaves vite as a grandchild, so kill the
    # whole tree rather than just the shell.
    if os.name == "nt":
        subprocess.run(["taskkill", "/F", "/T", "/PID", str(server.pid)], capture_output=True)
    else:
        try:
            os.killpg(os.getpgid(server.pid), signal.SIGTERM)
        except ProcessLookupError:
            pass


def main() -> int:
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        raise SystemExit(
            "Playwright is not installed. Run:\n"
            "    pip install playwright\n"
            "    playwright install chromium"
        )

    if not STUB.exists():
        raise SystemExit(f"missing stub at {STUB}")

    server = start_dev_server()
    try:
        wait_for_port(PORT)
        with sync_playwright() as playwright:
            browser = playwright.chromium.launch(headless=True)
            for folder, viewport, scale in VIEWPORTS:
                context = browser.new_context(viewport=viewport, device_scale_factor=scale)
                context.add_init_script(path=str(STUB))
                page = context.new_page()
                for name, route in SCREENS:
                    page.goto(f"{BASE}{route}", wait_until="load")
                    page.wait_for_timeout(1400)
                    page.evaluate("document.fonts.ready")
                    target = OUT / folder / f"{name}.png"
                    target.parent.mkdir(parents=True, exist_ok=True)
                    page.screenshot(path=str(target))
                    print(f"{folder}/{name}.png", flush=True)
                context.close()
            browser.close()
    finally:
        stop_dev_server(server)

    print(f"\nWrote {len(SCREENS) * len(VIEWPORTS)} screenshots to {OUT}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
