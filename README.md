# Ledgerly

A fully local, fully offline desktop invoicing app for freelancers and small businesses.
Manage clients, create invoices and estimates, track payments, and export professional PDFs —
with no account, no cloud, and no network dependency. Pro is unlocked by an offline,
machine-bound license key.

Built with **Rust + Tauri 2** and a **Svelte + TypeScript** frontend, storing data in a local
SQLite file.

> The product name "Ledgerly" is a placeholder for now.

## Status

Early scaffold — see the product requirements in [`docs/PRD.md`](docs/PRD.md).

## Layout

```
docs/         Product requirements and design docs
src/          Svelte frontend
src-tauri/    Rust core (Tauri backend)
```

## Prerequisites

- Rust (stable, MSVC toolchain on Windows)
- Node.js 20+
- Platform Tauri prerequisites (WebView2 on Windows)

## Develop

```bash
npm install
npm run tauri dev
```

## Build

```bash
npm run tauri build
```
