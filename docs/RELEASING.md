# Releasing Ledgerly

Everything here is done from the repository root.

## Before anything else: replace the development signing key

The repo ships a **development** licence key. The matching private key was left
outside version control in `.vendor/` (gitignored). Anyone holding it can mint
Pro licences, so before the first real release:

```bash
cargo run --manifest-path tools/licensectl/Cargo.toml -- keygen --out /safe/place/ledgerly.key
```

Put the printed public key into `src-tauri/assets/license_public_key.txt`
(replacing the existing line), then keep the private key somewhere safe and
backed up — offline, or in a password manager. Losing it means you can no longer
issue licences for that key id.

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
  file. See `docs/PRD.md` §13 / the M9 notes.

## Manual checks CI cannot do

CI runs on all three platforms, but it does not drive the GUI. Before publishing:

1. Launch the app and complete first-run: create a business profile, a client,
   an invoice — then export a PDF and open it.
2. Turn the app lock on, restart, unlock, and confirm the wrong passphrase is
   refused.
3. Activate a licence with a key issued by `licensectl`, restart, and confirm
   Pro stays active; then deactivate and confirm it returns to Free.
4. Two tests are `#[ignore]`d because they need a real OS keychain and the
   vendor dev key. Run them locally with:

   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml -- --ignored
   ```

5. Confirm the installer size is still in the expected range (roughly 8–20 MB).

## Rollback

Releases are plain installers. If a release is bad, publish the previous
version again; because the app is offline and update-free, no user is forced
onto a new build. Data is migrated forward only, so tell users not to install
an older build over a newer one without restoring a backup.
