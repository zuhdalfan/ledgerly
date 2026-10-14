# Ledgerly Roadmap

This is the direction, not the to-do list. It says where the product is going and
*why the order is what it is*; it deliberately carries no status and no dates.

- **[`PRD.md`](PRD.md)** holds the requirements, the personas and the non-goals —
  the "what".
- **This file** holds the vision, the sequencing reasoning and the revisit
  triggers — the "why now".
- **[`TASKS.json`](../TASKS.json)** holds where each item is right now — the
  "where". Items are referenced here by id only; state lives there and nowhere
  else.

---

## The destination

Ledgerly exists for one sentence: **your data never leaves your machine.**

The app a freelancer installs once and then forgets about — no account, no
subscription, no cloud, working identically with the network cable unplugged for
the rest of its life. It should be *fast and boring*: under a second to open,
under a second to make a PDF, on a laptop from a decade ago. And it should be
*yours*: one portable SQLite file you can copy, inspect and keep, even after you
stop using the tool.

That framing is the whole product. Every feature is judged against it, and most
features worth having are the ones that make the promise easier to trust, not the
ones that add surface area.

**North-star tests.** If any of these stops being true, the roadmap has failed,
not just slipped:

1. A user can run a full month of invoicing with the network never on.
2. A user can take their data file and hand it to someone else — or keep it
   themselves — and lose nothing.
3. The install is small, signed and uneventful, and nothing in it ever phones
   home.

---

## The principles that set the order

These decide sequencing when two things both seem worth doing:

- **Offline is not a feature, it is the spine.** Not negotiable, not "v2".
- **Trust beats capability.** A smaller app that behaves exactly as promised
  outperforms a bigger one that maybe doesn't.
- **The user owns the data.** Portable, transparent, documented formats — no
  lock-in to be engineered, ever.
- **Do less, better.** The explicit non-goals in PRD §3.2 are part of the design,
  not a backlog to grow back into.
- **Free and open.** MIT, no paid tier, no key. Sustainability comes from people
  who value it, not from gatekeeping it.

---

## Where we are

The product works. The core arc — skeleton, clients, invoices, PDF, payments,
estimates, reports, backup and restore, plus app lock and optional AI access over
MCP — is done, offline-verified and covered by the CI gate. What remains is not
new capability. It is **reach, the last mile to a real install, and long-term
stewardship.**

Three themes carry the rest of the journey:

1. **Speak the user's language** — finish localization so the app is coherent end
   to end, not half-translated.
2. **Earn the install** — signed, verified installers on all three platforms, so
   "install it once" is actually true for a stranger.
3. **Last a decade** — recurring invoices, the encryption decision revisited on
   evidence, and only then any thought of other platforms.

## Now — Speak the user's language

### I18N — UI & PDF localization → `TASKS.json#I18N`
The shell, the shared components and Settings already render from the locale
files; the content routes and the PDF document still emit English literals. That
gap is worse than untranslated software — it reads as broken. This is the "Now"
item because it is the only thing standing between the current build and a
coherent whole, and because a translation done halfway in a user-facing document
is a trust cost, not a nice-to-have.

**Done when** every route and the PDF locale render from `locales/*.ts`, with key
parity enforced by the type check rather than by eye.

## Next — Earn the install

### RELEASE-SIGN — Packaging, signing & notarization → `TASKS.json#RELEASE-SIGN`
The last thing between the working app and the promise "install it once". An
unsigned installer that a browser or OS warns about undercuts the entire premise
of a trustworthy local app. Nothing else can ship to the public until this does.

**Done when** signed installers for Windows, macOS and Linux build and pass the
manual checks. **Held by** owner-held credentials — a Windows code-signing cert
and an Apple Developer ID plus notarization secrets. The workflow can be built
now, but cannot be *verified* without them; that is what keeps this in Next
rather than Now.

### RELEASE-MANUAL — Manual pre-release checks → `TASKS.json#RELEASE-MANUAL`
The only P1 that by nature cannot be automated: first-run, app-lock unlock, a
catalog-to-invoice walk, installer size. A human at a GUI on each OS. It sits
beside signing because it is the same milestone from the user's point of view —
the difference between "it builds" and "it installs and works".

## Later — Last a decade

### RECURRING — Recurring invoices → `TASKS.json#RECURRING`
The most-requested shape of missing feature: load-bearing for weekly billers, and
the one thing that pushes users back to a cloud tool. It waits because it must
land on stable invoice and PDF paths and must be idempotent — a recurring invoice
generated twice is a real-world billing error, worse than not having the feature.
**Revisit trigger:** two or more users ask, or the next minor-version planning
pass.

### NOTIFY — Due-date notifications → `TASKS.json#NOTIFY`
The one proactive thing Ledgerly will ever do — and the reason many freelancers
leave for a cloud tool, because someone else does the reminding. Doing it locally
keeps the promise intact: no account, no network, no mail relay, just the app the
user already trusts telling them an invoice is late. The PRD already filed this as
FR-REC-3, so it finishes a committed idea rather than starting a new one.

It waits because it introduces the first interruption the app has ever made, and
that is a principle, not a feature. The settled answer: **on by default, but only
once an invoice is past due** — never a nudge on the due date itself, so it stays
a help and not a nag. Mechanically it rides a hook that already exists:
`backup::auto_backup` runs at most once a day on launch, and the due check belongs
on the same path — no daemon, nothing polling, nothing reaching the network.

**Revisit trigger:** whenever RECURRING is picked up, or when users ask to be
reminded rather than to go looking. Building it overturns PRD §18.3's "the app
never pushes anything at the user", so reconcile that section first.

### TRAY — Tray & background running → `TASKS.json#TRAY`
The thing that lets Ledgerly be *resident* without being in the way — there when
you need it, gone when you don't. It is filed beside NOTIFY because neither is
whole alone: a tray without reminders is just a convenience, and reminders without
a resident process barely fire, since today they can only surface at the moment
the user opens the app.

It is **opt-in and off by default**: until someone asks to keep it running in the
tray, the app behaves exactly as it does now — open, use, close, gone — with no
start-on-login. That keeps the contract "it runs only when you open it" true for
everyone who never opts in, and makes the first-ever always-resident process a
deliberate choice rather than something that arrives with an update. Once it does
run it must be near-idle: the same once-a-day hook (`backup::auto_backup`) governs
its work, so nothing polls and nothing reaches the network.

**Revisit trigger:** build it with NOTIFY, when users ask to be reminded rather
than to go looking. Reconciles PRD FR-SEC-4, which already refers to
minimize-to-tray, and needs Tauri's tray feature plus an icon.

### SEC-ENCRYPT — Encrypted database at rest → `TASKS.json#SEC-ENCRYPT`
Deferred on evidence, not neglected. Most users already have OS disk encryption,
and the cost of doing it ourselves is high — a vendored OpenSSL on three
platforms (plus an exemption in the offline guard) and an unrecoverable database
if a passphrase is forgotten. The promise "your data never leaves your machine"
is best served by the machine's own encryption until there is a reason it isn't.
**Revisit trigger:** backups move to the cloud, or the machine is shared or
resold.

### MOBILE-APP — iOS & Android → `TASKS.json#MOBILE-APP`
A v1 non-goal (PRD §3.2). Tauri 2 can target mobile, which makes it tempting, but
it is a different product: its own layouts, storage paths, signing, stores and
release cadence. It should not start until the desktop promise is proven and
retained.
**Revisit trigger:** v1 ships and desktop retention is measured.

## What we will not build

Carried from PRD §3.2 and treated as identity, not scope: no cloud or accounts,
no sending email from the app, no payment collection, no bank feeds, no
double-entry accounting, no multi-user, no telemetry and no network auto-update,
and no anti-tamper arms race. If a future idea needs one of these to work, the
answer is that it belongs in a different product.

## Decision log

Choices already made, kept so they are not silently re-opened:

- **Licensing & freemium — dropped.** The app is free and MIT; the licensing
  tables were removed in `0008_drop_licensing.sql`.
- **Encrypted database at rest — deferred.** The OS already encrypts for most
  users, and vendoring OpenSSL on three platforms (plus risking an unrecoverable
  database) is not worth it yet. Revisit on the trigger above.
- **Virtualized list rendering — declined after measuring.** At 10,000 invoices
  the query returns in ~43 ms; the DOM was the cost, and paging in SQL bought
  nothing (PRD §11).

---

*No dates, no status, no task lists — those live in `TASKS.json`. Every deferred
item here must carry its own revisit trigger, so a decision can only be re-opened
deliberately.*
