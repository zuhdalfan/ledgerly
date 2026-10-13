# Preview feature audit

Coverage matrix for the Ledgerly walkthrough previews. Every feature discovered
in the audit gets an id, so each one can be traced to a preview segment (or
marked as not yet covered / blocked).

**Status legend** — `covered`: demonstrated in a segment; `partial`: the screen
or control appears but the full interaction is not exercised; `not covered`:
exists, not yet recorded; `blocked`: cannot be demonstrated with the current
harness (see reasons); `n/a`: not implemented in the app.

**Segments** live in `docs/preview/` (`NN-*.mp4`; a few also have a `.gif`).
Segment scripts: `tools/preview-record/segments.py`. Regenerate: see
`docs/preview/README.md`.

> Honesty note: the point of this table is to be true, not flattering. Rows
> marked *partial* or *not covered* are real gaps, not claimed wins.

## How the previews are produced

The app is a Tauri program whose frontend fetches everything over IPC. The
walkthrough drives the **real frontend** in headless Chromium with a stateful
IPC stub (`tools/preview-record/demo-stub.js`) and records video
(`tools/preview-record/record.py`). It is real UI behaviour — dropdowns open,
forms fill, the command palette filters, totals recompute — not a mocked
slideshow. Two categories therefore cannot be shown and are marked *blocked*:
anything needing the packaged desktop runtime (OS file dialogs, Argon2id
hashing, the OS PDF viewer) and the MCP server, which is a separate stdio
process an external client launches.

## 1. Shell and navigation — `src/routes/+layout.svelte`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| NAV-01 | Sidebar navigation | Click a nav link | all | covered |
| NAV-02 | Command palette open | Click Search, or `Ctrl/Cmd+K` | 01 | covered |
| NAV-03 | Command palette search + keyboard | Type, `ArrowDown`, `Escape` | 01 | partial — `Enter`-to-navigate not shown |
| NAV-04 | Shortcut `Ctrl/Cmd+N` (new invoice) | Press | 01 | covered |
| NAV-05 | Shortcut `/` (open palette) | Press | — | not covered |
| NAV-06 | Mobile drawer (`<820px`): hamburger, scrim, Escape | Resize + click | — | not covered (desktop viewport only) |
| NAV-07 | Skip-to-content link | Tab to it | — | not covered |
| NAV-08 | App-lock gate (+ error/checking states) | Start locked | 13 | partial — enable shown, unlock gate not |
| NAV-09 | Auto-lock on idle | Wait idle | — | blocked (multi-minute timer) |

## 2. Dashboard — `src/routes/+page.svelte`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| DASH-01 | Outstanding hero (total, overdue, collected, clients) | View | 01 | covered |
| DASH-02 | "Finish setting up" checklist | View when incomplete | — | not covered (seed data completes all steps) |
| DASH-03 | Needs chasing — Copy reminder (clipboard) | Click Copy | 01 | covered |
| DASH-04 | Needs chasing — Mark chased / Unmark | Click | 01 | covered |
| DASH-05 | "View all overdue" deep link | Click | 01 | partial (via aged link instead) |
| DASH-06 | Aged receivables bucket links | Click a bucket | 01 | covered |
| DASH-07 | Billed & collected month links | Click a column | 01 | covered |
| DASH-08 | Your data (data directory) | View | 01 | covered |

## 3. Invoices

### List — `src/routes/invoices/+page.svelte`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| INV-L01 | Search by number/client | Type | 02 | covered |
| INV-L02 | Status filter | Select | 02 | covered |
| INV-L03 | Clear filter | Click | 01, 02 | covered |
| INV-L04 | Deep links (status/age/issued described) | Follow a link | 01 | covered |
| INV-L05 | Row → detail | Click a number | 02 | covered |
| INV-L06 | Empty / no-match states | Filter to nothing | — | not covered |
| INV-L07 | 250-row truncation note | >250 rows | — | not covered |

### Editor — `src/routes/invoices/[id]/+page.svelte`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| INV-E01 | Details: client, PO, dates, currency, derived status | View/edit | 05 | covered |
| INV-E02 | Inline "New client" from the form | Toggle + add | — | not covered |
| INV-E03 | Add line | Click | 05 | covered |
| INV-E04 | Add from catalog picker (+ in-picker search) | Open, filter, pick | 03, 05 | covered |
| INV-E05 | Remove a line | Click × | — | not covered |
| INV-E06 | Per-line discount / tax rate / tax-inclusive | Change | 03 | partial — discount shown; tax-inclusive checkbox not toggled |
| INV-E07 | Invoice-level discount | Select % / fixed | 03 | covered |
| INV-E08 | Live totals recompute | Type in a line | 03, 05 | covered |
| INV-E09 | Record payment (full / partial) | Fill + record | 04 | covered |
| INV-E10 | Paid celebration modal | Settle balance | 04 | covered |
| INV-E11 | Delete payment → Undo toast | Delete, Undo | 04 | covered |
| INV-E12 | Chasing: Copy reminder, Mark chased | Click | 04 | covered |
| INV-E13 | Notes & terms edit + Save | Type, save | 04 | covered |
| INV-E14 | Duplicate invoice | Click | — | partial (button visible in 04) |
| INV-E15 | Mark sent / Move to draft | Click toggle | 04 | covered |
| INV-E16 | Void / Restore (+ confirm dialog) | Click Void | — | partial (button visible; dialog + action not shown) |
| INV-E17 | PDF template + Export PDF | Select, export | — | partial (card visible in 04; export not clicked) |
| INV-E18 | Keyboard `Ctrl+S` / `Ctrl+P` | Press | — | not covered |
| INV-E19 | Unsaved-changes guard | Navigate away dirty | — | not covered |

## 4. Estimates — `src/routes/estimates/**`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| EST-01 | List + search + status filter | Type/select | 06 | partial — list shown via nav; filters not exercised |
| EST-02 | Detail: line items, totals | View | 06 | covered |
| EST-03 | Status change (Draft/Sent/Accepted/Declined) | Select | 06 | covered |
| EST-04 | Convert to invoice | Click | 06 | covered |
| EST-05 | Expiry banner / converted banner | State | 06 | partial — converted link visible |
| EST-06 | Duplicate / Export PDF | Click | — | not covered |
| EST-07 | `?client=` deep link presets client | Via client page | — | not covered |

## 5. Clients — `src/routes/clients/**`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| CLI-01 | List + search | Type | 07 | covered |
| CLI-02 | Show archived | Toggle | 07 | covered |
| CLI-03 | Detail: balance by currency, overdue notice | View | 07 | covered |
| CLI-04 | Detail: invoices & estimates tables | View | 07 | covered |
| CLI-05 | New client (name, contact, address, billing, notes) | Fill + save | 07 | covered |
| CLI-06 | Archive / Restore | Click | — | not covered |
| CLI-07 | Statement PDF export | Click | — | partial (button visible) |
| CLI-08 | New invoice / New estimate from client | Click | — | not covered |
| CLI-09 | Unsaved-changes guard | Navigate away | — | not covered |

## 6. Catalog — `src/routes/catalog/+page.svelte`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| CAT-01 | Add item (name, price, unit, description) | Fill + add | 08 | covered |
| CAT-02 | Search by name/SKU | Type | 08 | covered |
| CAT-03 | Show archived | Toggle | — | not covered |
| CAT-04 | Inline row edit (Edit / Save / Cancel) | Click | 08 | partial — Edit then Cancel; Save not shown |
| CAT-05 | Archive / Restore | Click | — | not covered |

## 7. Reports — `src/routes/reports/+page.svelte`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| REP-01 | Date range + granularity controls | Change + Run | 09 | covered |
| REP-02 | Revenue table + totals | View | 09 | covered |
| REP-03 | Revenue by client | View | 09 | covered |
| REP-04 | Aged receivables | View | 09 | covered |
| REP-05 | Tax collected | View | 09 | covered |
| REP-06 | Payments by method | View | 09 | covered |
| REP-07 | Per-report Export CSV | Click | 09 | covered |
| REP-08 | Export all data (JSON) | Click | 09 | covered |

## 8. Settings — `src/routes/settings/**`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| SET-01 | Business profile + defaults | Edit + save | 10 | covered |
| SET-02 | Tax rates (add) | Add | 10 | covered |
| SET-03 | Tax rates (archive/restore) | Click | — | not covered |
| SET-04 | Invoice numbering + live preview | Edit + save | 10 | covered |
| SET-05 | Document language | Select | — | partial (on page in 10) |
| SET-06 | Security: enable app lock | Fill + enable | 13 | covered |
| SET-07 | Security: auto-lock minutes, Lock now, disable | Change | — | not covered |
| SET-08 | AI access (MCP) — see §9 | — | 12 | covered |
| SET-09 | Data: data location + Open folder | View | 11 | covered |
| SET-10 | Data: Back up now | Click | 11 | covered |
| SET-11 | Data: automatic backup settings | Edit + save | 11 | covered |
| SET-12 | Data: Restore from file (+ confirm + message) | Click | 11 | covered |
| SET-13 | Appearance: motion level | Select | 13 | covered |
| SET-14 | About (version, paths, source link) | View | 13 | covered |
| SET-15 | Support (free/MIT, support link, licence) | View | 13 | covered |

## 9. AI access (MCP) — `src/routes/settings/ai`, `src-tauri/src/mcp`

| ID | Feature | Action | Segment | Status |
|---|---|---|---|---|
| MCP-01 | Enable AI access toggle | Toggle | 12 | covered |
| MCP-02 | Allow changes toggle (write gate) | Toggle | 12 | covered |
| MCP-03 | Resolved server path + Copy | Click | 12 | covered |
| MCP-04 | Generated client config + Copy | Click | 12 | covered |
| MCP-05 | Read / Change / Never summary | View | 12 | covered |
| MCP-06 | 40 MCP tools over stdio (read + write) | Run a tool | — | blocked — separate `ledgerly-mcp` process, launched by an external client; not reachable from the app UI |
| MCP-07 | PASS-REQUIRED gate when the lock is on | Call a tool | — | blocked — same reason as MCP-06 |
| MCP-08 | `allowWrite` refusal ("Changes are off") | Call a write tool | — | blocked — same reason |

## 10. Backend capabilities surfaced in the UI

| ID | Feature | Segment | Status |
|---|---|---|---|
| BE-01 | PDF export, 3 templates | — | blocked — native save dialog + `std::fs` write; the stub returns a path so the UI flow completes but no file is produced |
| BE-02 | Statement PDF | — | blocked — as BE-01 |
| BE-03 | Backup / restore (.ledgerly) | 11 | blocked — UI flow demonstrated; real archive needs the Tauri runtime |
| BE-04 | CSV / JSON export | 09 | blocked — as BE-01 (UI + success message shown) |
| BE-05 | App lock (Argon2id verifier) | 13 | blocked — hashing + gating need the Rust core |
| BE-06 | Migrations / offline guarantee | — | n/a (no UI) |

## 11. Cross-cutting / minor

| ID | Feature | Segment | Status |
|---|---|---|---|
| X-01 | Optimistic "Saved" / inline error messages | 07, 10 | covered |
| X-02 | Toasts / undo | 04 | covered |
| X-03 | Confirmation dialogs (void, restore) | 11 | partial — restore confirm's dialog is auto-answered |
| X-04 | Empty states (lists) | — | not covered |
| X-05 | Loading skeletons | — | not covered (data resolves too fast) |
| X-06 | Responsive layouts at tablet/phone widths | — | not covered (see `docs/screenshots/`) |
| X-07 | Keyboard focus states | — | not covered |

## Summary

- **Covered** end to end: dashboard (except the setup checklist), invoice list,
  invoice editor core, payments + celebration + undo, invoice creation,
  estimates conversion, clients create/edit, catalog add/search/edit, all five
  reports + CSV/JSON export, business/numbering/tax settings, AI access, app-lock
  enable, appearance, about, support, backups UI.
- **Notable gaps** (*not covered*): empty/loading states, the invoice void
  confirmation, PDF export, per-line tax-inclusive, archive/restore actions,
  the mobile drawer and responsive widths, and several keyboard shortcuts.
- **Blocked by design**: the MCP tool calls (external stdio process) and the
  true file-writing / OS-dialog / crypto paths (need the packaged desktop app).
- Known app bugs found while auditing are listed in the PR notes: the estimates
  "Clear filter" clears only the status param, and `statusLabel("unpaid")`
  renders as "Draft".
