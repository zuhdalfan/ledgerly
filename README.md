# Ledgerly

A fully local, fully offline desktop invoicing app for freelancers and small
businesses. Manage clients, create invoices and estimates, track payments, run
reports and export professional PDFs — with no account, no cloud and no network
dependency.

Built with **Rust + Tauri 2**, a **Svelte 5 + TypeScript** frontend, and a local
**SQLite** file. Pro is unlocked by an offline key bound to the machine.

> The product name "Ledgerly" is a placeholder.

## What it does

- **Clients and a reusable catalog** of products and services.
- **Invoices** with line items, per-line and invoice-level discounts, multiple
  tax rates, tax-inclusive or exclusive pricing, and a totals engine that
  reconciles to the cent.
- **Payments and lifecycle** — record full or partial payments; draft, sent,
  partially paid, paid, overdue and void are derived, not hand-set. Aged
  receivables on the dashboard.
- **Estimates** that convert into a linked invoice in one click.
- **PDF export** with three templates, an embedded font, pagination and a
  void watermark.
- **Reports** — revenue by period and client, aged receivables, tax collected
  per rate, payments by method — each exportable to CSV, plus a whole-database
  JSON export.
- **Backups** to a single portable `.ledgerly` file, with restore, a
  pre-restore safety copy, and rotating automatic backups.
- **App lock** with an Argon2id passphrase, enforced in the Rust core, with
  auto-lock on idle.
- **Offline licensing** — an Ed25519 key bound to this install and machine.

## The promise

**Ledgerly makes no network requests.** No telemetry, no analytics, no update
checks, no licence server. `npm run check:offline` enforces it in CI by failing
the build if an HTTP client crate, a browser networking API, or a remote URL
appears anywhere in the dependency graph or source.

## Layout

```
docs/          PRD, release checklist, licensing runbook
src/           Svelte frontend
src-tauri/     Rust core (Tauri backend)
licensing-format/  Licence wire format shared by the app and the vendor tool
tools/licensectl/  Vendor signing tool (never shipped with the app)
scripts/       CI helpers
```

## Develop

Prerequisites: Rust (stable, MSVC toolchain on Windows), Node 20+, and the
platform Tauri prerequisites (WebView2 on Windows, `libwebkit2gtk-4.1-dev` etc.
on Linux).

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```

## Test

```bash
cargo test --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path licensing-format/Cargo.toml
npm run check          # svelte-check
npm run check:offline  # no network access
```

Two tests are `#[ignore]`d because they need a real OS keychain and the vendor
development key; run them with `-- --ignored`.

## Docs

- [`docs/PRD.md`](docs/PRD.md) — the product requirements and design.
- [`docs/RELEASING.md`](docs/RELEASING.md) — release checklist, signing, and the
  one-time hardening still to do.
- [`docs/LICENSING.md`](docs/LICENSING.md) — how to issue and re-issue Pro keys.

## Status

Feature-complete against the PRD milestones M0–M8 and the security/i18n parts of
M9. Known gaps: the database is **not encrypted at rest** (SQLCipher needs a
vendored OpenSSL build), reports have no PDF output, and the app UI is
English-only with document labels localized. See `docs/RELEASING.md`.
