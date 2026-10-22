# Releasing Ledgerly

Everything here is done from the repository root.

## Version

Three files must agree:

| File | Field |
|---|---|
| `package.json` | `version` |
| `src-tauri/Cargo.toml` | `[package] version` |
| `src-tauri/tauri.conf.json` | `version` |

## Build

```bash
npm ci
npm run check:offline      # must pass: no network access anywhere
npm run check              # frontend typecheck
npm run build              # frontend
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

Artifacts land in `src-tauri/target/release/bundle/`:

- Windows — `msi/` and `nsis/`
- macOS — `dmg/` and `macos/`
- Linux — `appimage/` and `deb/`

## Signing and notarization

These need credentials that are not in this repository.

**Windows.** An OV or EV code-signing certificate. Configure Tauri's
`bundle.windows.certificateThumbprint` (or a custom `signCommand`) and sign the
installer and the executable. Without it, SmartScreen will warn until the
binary builds reputation.

**macOS.** A Developer ID Application certificate plus notarization. Set
`APPLE_CERTIFICATE`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD` and
`APPLE_TEAM_ID` in the environment and let Tauri sign and notarize. Unsigned or
un-notarized builds are refused on modern macOS.

**Linux.** No OS-level signing. Publish SHA-256 checksums alongside the
AppImage and `.deb`, and optionally sign the checksums with GPG.

Release builds must be produced on a machine or CI runner for each target OS;
there is no cross-compilation story for Tauri bundles.

## Before announcing a release

- **Audit the dependency licences.** Ledgerly is MIT, but a distributed binary
  also carries the licences of its ~300 dependencies. Run `cargo deny check
  licenses` (or `cargo license`) and confirm nothing copyleft has crept in.
  This is the one legal check that actually matters here, and it is not covered
  by CI yet.
- **Add a `THIRD-PARTY` notice** listing the bundled font and any dependency
  that requires attribution. Inter is already covered by
  `static/fonts/OFL.txt`.

## One-time hardening still to do

- **Content Security Policy.** `tauri.conf.json` currently sets `"csp": null`.
  Tighten it to something like:

  ```json
  "csp": "default-src 'self'; connect-src 'self' ipc: http://ipc.localhost; img-src 'self' data: asset: http://asset.localhost; style-src 'self' 'unsafe-inline'; font-src 'self' data:; script-src 'self'; object-src 'none'; base-uri 'self'; frame-ancestors 'none'",
  "devCsp": null
  ```

  **Smoke-test the built app after changing this** — the window must load and
  every screen must still talk to the backend. A CSP that breaks IPC makes the
  app look empty, which is why it is not switched on blindly.
- **Encrypted database at rest.** Not implemented. SQLCipher needs a vendored
  OpenSSL build (perl + nasm); the app lock protects the running app, not the
  file.

## Manual checks CI cannot do

CI runs on all three platforms, but it does not drive the GUI. Before publishing:

1. Launch the app and complete first-run: create a business profile, a client,
   an invoice — then export a PDF and open it.
2. Turn the app lock on, restart, unlock, and confirm the wrong passphrase is
   refused.
3. Add a catalog item, then create an invoice and pull that item in from the
   catalog picker. Export the invoice as a PDF, and export a quote as one too.
4. Confirm the installer size is still in the expected range (roughly 4–20 MB).
5. Shrink the window below ~820px (or open the app on a tablet) and confirm the
   sidebar collapses into a drawer: the menu button opens it, and Escape, the
   backdrop or a nav link closes it.
6. Confirm `npm run tauri build` bundles the `ledgerly-mcp` sidecar next to the
   app. It is a second crate bin, so the bundler installs it once; do not add it
   to `bundle.externalBin` as well or the Windows MSI fails with ICE30.
7. In **Settings → AI**, switch access on, add the shown program to an MCP client,
   and confirm a read ("list unpaid invoices") and a write ("create a client")
   both land in the app. Confirm writes are refused with **Allow changes** off,
   and that the app lock refuses everything without the passphrase.

## Rollback

Releases are plain installers. If a release is bad, publish the previous
version again; because the app is offline and update-free, no user is forced
onto a new build. Data is migrated forward only, so tell users not to install
an older build over a newer one without restoring a backup.
