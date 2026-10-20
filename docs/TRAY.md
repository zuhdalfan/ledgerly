# Tray & background running — implementation plan

Status: **plan only, nothing built.** This document turns the `TRAY` and
`NOTIFY` entries in `TASKS.json` (and `docs/ROADMAP.md`) into a build order. It
does not change the roadmap's direction; it records how to deliver it.

## Why

Ledgerly currently runs only while its window is open. That is the right
default, and it must stay the default: someone who never opts in should see the
same "open, use, close, gone" app after an update.

The reason to offer more is the one proactive thing the app can do — tell a
freelancer that an invoice is past due. A reminder that can only appear while
the app is already open is not a reminder; it is a banner. A resident process
makes it real. A tray gives that process a visible, honest home instead of a
hidden background service.

Both pieces are therefore one feature:

- **TRAY** keeps Ledgerly available after the window is closed.
- **NOTIFY** uses that availability to check once a day for overdue invoices.

Neither is whole alone. Build them together, or not yet.

## The contract

1. **Opt-in, off by default.** A new setting, off until the user turns it on.
   Until then, closing the window quits — exactly today's behaviour.
2. **No start-on-login.** Not part of this work. A resident process is already
   a deliberate choice; starting at login would make it an unrequested one.
3. **Near-idle.** No polling loop, no network, no database churn. The daily
   hook already exists (`backup::auto_backup`) and runs at most once a day. The
   due check rides the same rhythm, plus one timer set for the next local
   midnight — not a repeating interval.
4. **Quit means quit.** Closing hides only while tray mode is on. The tray's
   **Quit** and the app menu's Quit exit the process; they never hide it.
5. **The lock still holds.** Hiding to the tray is not a way around the
   passphrase. If the app lock is enabled, hiding locks the app (FR-SEC-4), the
   tray reveals no client or amount data, and opening the window shows the lock
   gate.
6. **Offline.** Tray, single-instance and notifications are local OS facilities.
   `check:offline` must stay green; any dependency that drags in a network stack
   is disqualified.

## User-facing design

### Setting

Settings > Appearance, in a new **Window** section above Language:

> **Keep Ledgerly running in the tray**
> Closing the window hides it instead of quitting. Use **Quit** in the tray
> menu to exit. Off by default.

Why Appearance: it already answers "how does the app behave?" (motion,
language), and no new Settings page is justified for one toggle. The hint is
the whole explanation; no modal, no first-run tour.

If the app lock is enabled, a second checkbox appears:

> **Lock when hidden to the tray**
> On by default when the app lock is on. FR-SEC-4 calls for it.

If the platform cannot show a tray icon (some Linux desktops without an
AppIndicator host), the setting is disabled with a one-line reason rather than
pretending to work. In that case closing always quits.

### Tray

- **Icon:** the torn-paper mark, sized for the menu bar. On macOS use a template
  image if the mark reads well in monochrome; otherwise the coloured icon with a
  contrasting outline. Do not ship a scaled-down app icon if it turns to mush at
  16 px.
- **Tooltip:** the word "Ledgerly". Never a count, a client name or an amount —
  a tooltip is visible without unlocking.
- **Left click:** show and focus the window (or hide it if already focused).
- **Menu:**
  - **Open Ledgerly** — show, unminimise, focus.
  - **Lock now** — only when the app lock is enabled and the app is unlocked.
  - **Quit Ledgerly** — exit the process.
- No "New invoice" shortcut. Creating a record while the window is hidden needs
  a second, half-built UI; the tray stays a launcher.

### Closing and quitting

- Tray mode off: `CloseRequested` closes the window and exits, unchanged.
- Tray mode on: `CloseRequested` calls `api.prevent_close()` and hides the
  window. The first time this happens, the user has just been told in Settings
  what would happen; the icon appearing is the confirmation.
- Quit from the tray sets a `quitting` flag and exits. The close handler must
  check that flag, or Quit would merely hide the window again.
- macOS: keep the dock icon for now. Hiding it adds activation-policy handling
  and a second way to get lost; the tray already provides the affordance.
  Cmd+Q must quit, not hide.
- If a second copy is launched while the first is resident, focus the existing
  window and exit the second process. Without this, two processes share one
  SQLite file and two tray icons appear. `tauri-plugin-single-instance` is the
  low-risk route (local IPC only), subject to the offline and licence checks.

## Reminders

### What is sent

At most one OS notification per day, only when at least one invoice is past due
(never on the due date itself). The body is short and factual:

> **3 invoices are past due** — Open Ledgerly to see them.

No client names, no amounts, no invoice numbers. This keeps the notification
useful on a lock screen or over a shoulder without becoming a data leak.

### When it runs

- After launch, once the app is **unlocked** (or immediately if no lock is
  configured).
- After an unlock, if the launch check was deferred.
- At the next local midnight while resident, via a single `tokio`/Tauri timer
  that is rescheduled after it fires. No minute-by-minute polling.
- When the window is shown or focused, if the local date has changed since the
  last check — event-driven catch-up after sleep, not a poll.

The date gate uses the local date (`date('now', 'localtime')`), matching the
backup hook, and is stored separately from the backup's `last_run` so enabling
one does not suppress the other.

### While locked

Recommended: **suppress** the notification while the app is locked, and run the
deferred check on unlock. A locked app that announces "3 invoices are past due"
is still telling anyone near the screen something about the business. The cost
is that a user who locks the app and leaves it resident gets no reminder until
they unlock; that is the honest reading of "the lock holds".

Alternative, if that proves too quiet: allow a generic notification with no
count ("Invoices are past due in Ledgerly"). This needs an explicit decision;
do not quietly pick one.

### Clicking it

Show and focus the main window. A deep link to the overdue filter is nice but
optional; the dashboard already surfaces aging and overdue totals, so land
there first and add navigation later if it earns its keep.

### Permission

The first notification attempt asks the OS. If permission is denied:

- Keep the setting on, but show a quiet line in Settings > Appearance: "Your
  system is blocking notifications." Do not re-prompt on every launch.
- The daily check still runs; only the OS delivery is blocked.

## Backend shape

Keep the platform glue out of the domain modules.

| Piece | Responsibility |
|---|---|
| `src-tauri/src/tray.rs` | Build the tray and menu, handle clicks, show/hide/quit, create or remove the tray when the setting changes. |
| `src-tauri/src/reminders.rs` | The overdue query, the once-a-day gate, notification text and permission state. No UI. |
| `security` / `AppState` | A `quitting` flag; `set_unlocked(false)` on hide when lock-on-hide is on; emit `security:changed`. |
| `lib.rs` | Wire the window-event handler, tray setup, single-instance plugin and the post-unlock hook. |

`backup::auto_backup` stays where it is. Do not fold reminders into it: backup
is a startup concern that must work before the app is unlocked, while reminders
must not query invoice data until it is. Sharing a `daily` façade is fine; sharing
the function is not.

### Settings keys

All live in the existing `settings` table (strings; no migration):

| Key | Default | Meaning |
|---|---|---|
| `ui.tray` | `false` | Close hides instead of quitting. |
| `security.lockOnHide` | `true` | Lock when hiding, when the lock is enabled. |
| `notify.overdue` | `true` | Send the daily overdue notification once the tray exists. |
| `notify.lastRun` | — | Local date of the last delivered check (internal). |

`notify.overdue` defaults to true because the settled NOTIFY decision is
"on by default". It has no effect until tray mode is enabled and the OS grants
permission.

### Events

- `security:changed` — emitted by Rust after locking on hide; the shell reloads
  its security status so the gate appears the next time the window is shown.
- `tray:open` (optional) — if the frontend ever needs to navigate on show.
- Notification clicks can simply call the same show/focus helper.

The frontend currently owns the idle-lock timer. Leave that alone; hide-locking
is a Rust concern because only Rust sees the window event. The shell must listen
for `security:changed` or it will keep rendering an unlocked UI after Rust has
locked.

## Platform notes

- **Windows:** tray icons live in the notification area; an overflow flyout may
  hide it. Notifications work from a dev build but are more reliable from an
  installed build with the bundle identifier. Test both.
- **macOS:** menu-bar icons should be template images where possible; the app
  stays in the dock. Notification permission is per-app and survives updates.
- **Linux:** Tauri's tray needs an AppIndicator host (`libayatana-appindicator`
  at runtime for `.deb`/AppImage). GNOME without the extension shows nothing.
  Detect the failure and disable the toggle rather than leaving a resident
  process with no way to quit. Test the AppImage, not just `tauri dev`.
- **All:** closing the last window with tray mode on must not trigger the
  default "exit when no windows" behaviour. Verify `RunEvent::ExitRequested`
  does not end the process on macOS when the window is hidden.

## Build order

### Phase 1 — tray and close-to-tray

1. Add the `tray-icon` feature and `tauri-plugin-single-instance`; confirm
   `check:offline` and `check:licenses`.
2. Settings toggle + typed read/write through the existing settings commands;
   disable with a reason where no tray host exists.
3. `tray.rs`: icon, tooltip, menu, show/focus/quit; `quitting` flag.
4. Window close handler; hide vs exit; second-launch focus.
5. Lock-on-hide + `security:changed`; shell listens and refreshes.
6. Tests for the pure decisions (`hide_on_close`, settings defaults) and manual
   per-OS checks.

### Phase 2 — reminders

1. `tauri-plugin-notification` + capability; permission state surfaced in
   Settings.
2. `reminders.rs`: overdue query reusing invoice balances/status, local-date
   gate, notification text.
3. Post-unlock check, midnight timer, show/focus catch-up.
4. Click-to-focus.
5. Tests for the gate and query; manual notification checks per OS.

### Phase 3 — finish

1. Settings copy and screenshots (Appearance gains a Window section).
2. README status: remove tray/reminders from the known-gaps list.
3. PRD: FR-SEC-4 and FR-REC-3 no longer say "when the tray exists"; §18.3's
   notification reversal points at the shipped plugin.
4. Release checklist: Linux AppIndicator dependency, notification permission,
   tray icon at 1x/2x, quit-from-tray on every platform.
5. `TASKS.json`: TRAY and NOTIFY move to done; roadmap is not edited as a status
   mirror.

## Tests and checks

Automated (Rust):

- Tray mode off means close exits; on means hide; Quit always exits.
- The daily gate is once per local date, and a deferred check runs on unlock.
- The overdue query excludes paid, void, draft and not-yet-due invoices.
- Notification copy uses counts correctly (0/1/n) and never includes names or
  amounts.
- Settings defaults survive an older database (no migration required).

Manual, per OS:

- Toggle on → close hides → icon remains → Open restores → Quit exits.
- Relaunch while resident focuses the existing window; one process, one icon.
- With the app lock on: hide locks; Open shows the gate; tray menu has no data.
- Notification arrives once, clicking focuses, denied permission is reported
  without nagging.
- Lock screen / notification centre preview contains no client data.
- `npm run verify`; `npm run screenshots` after the Appearance change.

## Non-goals

- Start-on-login, auto-update, background sync, or any network call.
- Polling the database on a timer shorter than a day.
- Per-invoice reminders, snooze, or a notification centre inside the app.
- Running data commands while locked just to build a richer notification.
- A second mini-UI in the tray menu.

## Open questions

1. **Locked reminders:** suppress until unlock (recommended) or send a generic
   no-count notification? Decide before Phase 2.
2. **Linux without a tray host:** disable the setting, or keep running with the
   window minimised and a warning? Recommended: disable the setting.
3. **macOS dock icon:** keep it (recommended for now) or hide it while resident?
4. **First-close feedback:** is the tray icon enough, or does the first hide
   deserve a one-time OS notification? Recommended: icon only; the Settings hint
   already set the expectation.
5. **Notification click:** dashboard (recommended) or the overdue filter? Start
   with the dashboard.
