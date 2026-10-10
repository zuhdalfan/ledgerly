/// Where the "Support this project" button sends people.
///
/// Deliberately empty until the funding page is chosen, so the button is
/// disabled rather than pointing somewhere wrong. Buy Me a Coffee, Trakteer or
/// anything else will do.
///
/// To go live:
///   1. Put the URL here.
///   2. Allowlist that host in `scripts/check-offline.mjs`. The offline check
///      fails on external URLs by design, and this one is a page the user
///      chooses to open in their own browser, not a request the app makes.
///   3. Grant `opener:allow-open-url` for that host in
///      `src-tauri/capabilities/default.json`.
export const SUPPORT_URL = "";
