# Using Ledgerly with a desktop AI assistant

Ledgerly ships a small companion program, **`ledgerly-mcp`**, that lets a desktop
AI assistant (Claude Desktop, Cursor, Windsurf, LM Studio, …) read and, if you
allow it, change your data.

It speaks the **Model Context Protocol (MCP)** over **standard input and output**
only. There is no server, no port and no network call: your assistant launches
the program, they talk over pipes, and Ledgerly's promise that it never touches
the network stays literally true.

## Turning it on

1. Open Ledgerly and go to **Settings → AI**.
2. Switch on **Enable AI access**.
3. Optionally switch on **Allow changes** to let the assistant create and edit
   records. Leave it off to keep the assistant read-only.
4. Copy the program path and the ready-made config the page shows.

The assistant talks to the *same database* the app uses. You can keep both open
at once.

## Connecting an assistant

### Claude Desktop

1. Open **Settings → Developer → Edit Config** (this opens
   `claude_desktop_config.json`).
2. Add Ledgerly under `mcpServers`, replacing the path with the one from
   Ledgerly's Settings → AI page:

   ```json
   {
     "mcpServers": {
       "ledgerly": {
         "command": "C:\\Program Files\\Ledgerly\\ledgerly-mcp.exe"
       }
     }
   }
   ```

3. Restart Claude Desktop. Ledgerly's tools now appear in the tools menu.

### Other clients

Any client that supports **local (stdio) MCP servers** works the same way: give
it the `ledgerly-mcp` program path as the command. See
<https://modelcontextprotocol.io> for client-specific instructions.

## The app lock

If you have the app lock on, the assistant needs your passphrase. Add it to the
client config as an environment variable:

```json
{
  "mcpServers": {
    "ledgerly": {
      "command": "C:\\Program Files\\Ledgerly\\ledgerly-mcp.exe",
      "env": { "LEDGERLY_MCP_PASSPHRASE": "your passphrase" }
    }
  }
}
```

Without a correct passphrase, every request is refused while the lock is on.
Note that the passphrase is then stored in the client's config file in plain
text; if that is a concern, run Ledgerly without the app lock.

## What the assistant can do

**Read (always, when enabled):** clients, catalog items, tax rates, invoices,
estimates, payments, the business profile, settings, and every report (revenue,
revenue by client, aged receivables, tax collected, payments by method).

**Change (only when "Allow changes" is on):** create and update clients, catalog
items and tax rates; create, update, duplicate, mark sent or chased invoices;
create, update, set status on or convert estimates; record and update payments;
update the business profile; take a backup.

**Never:** restore a backup, void an invoice, delete a payment, change the app
lock, open files, or dump the database. Destructive and identity-changing
actions are deliberately left to you in the app.

## Where the data lives

Both the app and `ledgerly-mcp` resolve the same folder:

- **Windows:** `%APPDATA%\com.ledgerly.desktop`
- **macOS:** `~/Library/Application Support/com.ledgerly.desktop`
- **Linux:** `$XDG_DATA_HOME/com.ledgerly.desktop` (or `~/.local/share/...`)

Set `LEDGERLY_DATA_DIR` to point the assistant at a different folder, which is
useful for testing. The app honours it too, so both processes move together.

Debug builds (`tauri dev`) use `com.ledgerly.desktop-dev` instead, so developing
never writes into the database the installed app reads.

## Troubleshooting

- **"AI access is off"** — switch it on in **Settings → AI**.
- **"Ledgerly is locked"** — add `LEDGERLY_MCP_PASSPHRASE` to the client config.
- **"Changes are off"** — switch on **Allow changes** in **Settings → AI**.
- **Nothing appears in the assistant** — check the command path is exactly the
  one shown in **Settings → AI**, then restart the assistant.
