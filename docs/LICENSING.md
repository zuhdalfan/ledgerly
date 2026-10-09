# Issuing Ledgerly licences

Ledgerly is freemium. Free is the default and is fully usable. Pro is unlocked
by an **offline key** signed by you and bound to one installation on one
machine. The app never contacts a server — you and the customer exchange the
request code and the key out of band.

All of this uses the vendor tool, which is a **separate crate** and is never
built into or shipped with the app:

```bash
cargo run --manifest-path tools/licensectl/Cargo.toml -- <command>
```

## The keys

- **Private key** — signs licences. Lives only on your machine, offline, backed
  up. It must never be committed, shipped, or copied to a customer machine.
  `.vendor/` is gitignored for this reason.
- **Public key** — committed in `src-tauri/assets/license_public_key.txt`. The
  app embeds it at build time and can only ever verify.

Generate a keypair:

```bash
cargo run --manifest-path tools/licensectl/Cargo.toml -- keygen --out .vendor/ledgerly.key
```

The private key goes to the file; the public key is printed for embedding.

## Issuing a licence to a customer

1. The customer opens **Settings → Licence** and copies the **request code**
   (or saves it to a file) and sends it to you. It contains their installation
   serial, a salted machine fingerprint, the SKU, app version and OS — no
   personal data.
2. You issue a key:

   ```bash
   cargo run --manifest-path tools/licensectl/Cargo.toml -- issue \
     --key .vendor/ledgerly.key \
     --request @request.txt \
     --plan perpetual \
     --out acme.lic
   ```

   Options:

   | Flag | Meaning |
   |---|---|
   | `--plan perpetual` | never expires (default) |
   | `--plan term --expires 2027-01-01` | expires on a date |
   | `--features reports,pdf_templates,branding,scheduled_backups,batch_export,multi_currency` | defaults to all of them |
   | `--key-id <id>` | which signing key made it (for rotation) |
   | `--out <path>` | omit to print to stdout |

3. Send `acme.lic` back. The customer loads it in **Settings → Licence**
   (paste or "Load .lic file…") and Pro unlocks immediately.

The key is bound to the request code's machine. Loading it on any other machine
fails with *"issued for a different installation or machine"*.

## Checking a key

```bash
cargo run --manifest-path tools/licensectl/Cargo.toml -- verify \
  --file acme.lic --pubkey <public key hex>
```

## Re-issuing

Hardware changes, OS reinstalls and machine migrations change the fingerprint,
so the old key stops verifying. The app says so plainly and offers a fresh
request code. Issue a new key from the new code; the old one is harmless.

Re-issue is the one place to watch for abuse: it is cheap to do, so it is also
how someone might try to move one licence across many machines. Bound it with
your own policy rather than in the app.

## Rotation

`key_id` is inside the signed payload, and the app embeds a public key. To
rotate:

1. `keygen` a new pair.
2. Issue new licences with `--key-id <new id>`.
3. Ship an app release that embeds the new public key — and, at least for the
   first release after rotating, keep verifying old keys too. Right now only one
   public key is embedded (`licensing::ticket::embedded_key`), so supporting two
   at once means extending that function to try a list.

## What offline licensing cannot do

- **Revoke.** Once a key is out, the app cannot be told to distrust it. A
  revocation list can only take effect in a *future release* that ships with it.
- **Stop a patched binary.** Someone determined can bypass a local check. The
  design accepts that: cryptography stops forgery and casual sharing, and the
  answer to piracy is a fair price and a good free tier, not an arms race.

## Free versus Pro

Pro features are `reports`, `pdf_templates`, `branding`, `scheduled_backups`,
`batch_export` and `multi_currency`. Everything else — clients, invoices,
estimates, payments, PDF export, manual backups, CSV/JSON export — is free, and
**app lock and encryption are deliberately never paywalled**.

A 14-day trial grants all Pro features once per installation.
