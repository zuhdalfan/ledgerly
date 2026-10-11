/// Where the "Support this project" button sends people.
///
/// Opened in the user's own browser by the Tauri opener plugin, never by the
/// app: the host is allowlisted in `scripts/check-offline.mjs`, and
/// `opener:default` already permits `https:` URLs, so no network request is
/// made from the app itself.
export const SUPPORT_URL = "https://ganknow.com/zuhdalfan";
