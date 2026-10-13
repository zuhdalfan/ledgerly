"""The walkthrough scripts: one entry per chapter/segment.

Each `script(d)` drives the real UI through `record.Demo`. Keep selectors based
on visible copy or roles where possible; prefer clicking in-app links over a
full page load so the recording never flashes a blank frame mid-segment.
"""


def nav(d, label):
    """Click a sidebar nav link (SPA navigation, no reload)."""
    d.click(f'#sidebar a:text-is("{label}")', settle=900)


# --------------------------------------------------------------------------- #
def dashboard(d):
    d.scroll(360)
    d.hover("text=Outstanding")
    d.scroll_to("text=Needs chasing")
    d.click("button:has-text('Copy')", settle=900)          # reminder built + copied
    d.click("button:has-text('Mark chased')", settle=900)   # writes via IPC
    d.scroll_to("text=Aged receivables")
    d.click("a[href='/invoices?age=31-60']", settle=1100)   # deep-links to a filtered list
    d.click("button:has-text('Clear filter')", settle=900)
    nav(d, "Dashboard")
    d.scroll_to("text=Billed and collected")
    d.click("a[href^='/invoices?issued=']", settle=1000)
    nav(d, "Dashboard")
    # Command palette + keyboard shortcut
    d.key("Control+k", settle=800)
    d.type("input[aria-label='Search commands, clients and invoices']", "client", delay=70)
    d.key("ArrowDown", settle=300)
    d.key("Escape", settle=600)
    d.key("Control+n", settle=1100)                         # global shortcut: new invoice
    nav(d, "Dashboard")


def invoices_list(d):
    nav(d, "Invoices")
    d.type("input[type=search]", "INV-0040", delay=70)
    d.wait(700)
    d.click("input[type=search]")
    d.page.locator("input[type=search]").first.fill("")
    d.wait(600)
    d.select("select", "overdue")
    d.wait(700)
    d.click("button:has-text('Clear filter')", settle=800)
    d.click("a:has-text('INV-0041')", settle=1200)          # open the editor


def invoice_editor(d):
    d.scroll_to("text=Line items")
    d.click("button:has-text('Add from catalog')", settle=1100)
    d.type("input[placeholder='Search the catalog']", "Consult", delay=60)
    d.click("text=Consulting day", settle=900)              # appends a line, live totals
    d.scroll_to("text=Totals")
    discount = d.card("Invoice discount").locator("select").first
    d.click_loc(discount, settle=300)
    discount.select_option("percent")
    d.wait(700)


def payments(d):
    pay = d.card("Record a payment")
    d.scroll_to("text=Record a payment")
    d.type_loc(pay.locator("input[type=date]"), "10/11/2026", delay=40)
    d.type_loc(pay.locator("input[inputmode=decimal]"), "1000.00", delay=60)
    d.type_loc(pay.locator("input[placeholder='bank / cash / card']"), "bank", delay=60)
    d.click("button:has-text('Record payment')", settle=1300)
    # While a balance remains, the chasing card is on screen.
    d.scroll_to("text=Chasing this one")
    d.click("button:has-text('Copy reminder')", settle=900)
    d.click("button:has-text('Mark as chased')", settle=900)
    # Deleting a payment offers an Undo toast, then we put it back.
    d.scroll_to("text=Record a payment")
    d.click("button:has-text('Delete') >> nth=0", settle=1000)
    d.click("button:has-text('Undo')", settle=1000)
    # A settling payment empties the balance and opens the celebration.
    d.type_loc(pay.locator("input[inputmode=decimal]"), "7240.40", delay=50)
    d.click("button:has-text('Record payment')", settle=1500)
    d.click("button:has-text('Close')", settle=900)        # the paid celebration
    d.scroll_to("text=Notes & terms")
    d.click("button:has-text('Save')", settle=1100)
    # The sent/draft control is one toggle whose label flips; click it both ways.
    toggle = "button:has-text('Move to draft'), button:has-text('Mark as sent')"
    d.click(toggle, settle=1200)
    d.click(toggle, settle=1200)


def invoice_create(d):
    d.click("text=New invoice", settle=1200)
    d.select("select >> nth=0", "globex")
    d.wait(500)
    d.type_loc(d.label("PO / reference"), "PO-2026-88", delay=45)
    d.click("button:has-text('Add from catalog')", settle=1100)
    d.type("input[placeholder='Search the catalog']", "Design", delay=60)
    d.click("text=Design hour", settle=900)
    d.click("button:has-text('Add line')", settle=500)
    d.scroll_to("text=Totals")
    d.click("button:has-text('Save')", settle=1600)         # creates + lands on the record


def estimates(d):
    nav(d, "Estimates")
    d.click("a:has-text('EST-0012')", settle=1200)
    d.select(d.label("Status"), "accepted", settle=1000)
    d.scroll_to("text=Line items")
    d.click("button:has-text('Convert to invoice')", settle=1600)  # -> linked invoice


def clients(d):
    nav(d, "Clients")
    d.type("input[type=search]", "Acme", delay=70)
    d.wait(700)
    d.click("input[type=search]")
    d.page.locator("input[type=search]").first.fill("")
    d.wait(500)
    d.click("input[type=checkbox]", settle=800)             # show archived
    d.click("input[type=checkbox]", settle=700)
    d.click("a:has-text('Acme Corporation')", settle=1200)
    d.scroll_to("text=Invoices")
    d.scroll_to("text=Details")
    d.type_loc(d.label("Contact person"), "Dana Lee", delay=45)
    d.click("button:has-text('Save client')", settle=1000)


def catalog(d):
    nav(d, "Catalog")
    d.type_loc(d.label("Name"), "Discovery sprint", delay=50)
    d.type_loc(d.label("Unit price"), "4800.00", delay=50)
    d.type_loc(d.label("Unit"), "day", delay=50)
    d.type_loc(d.label("Description"), "Two-week product discovery sprint.", delay=25)
    d.click("button:has-text('Add item')", settle=1100)
    d.type("input[type=search]", "Discovery", delay=60)
    d.wait(800)
    d.click("input[type=search]")
    d.page.locator("input[type=search]").first.fill("")
    d.wait(600)
    d.click("button:has-text('Edit')", settle=800)          # inline row editing
    d.click("button:has-text('Cancel')", settle=600)


def reports(d):
    nav(d, "Reports")
    d.select("select", "month", settle=400)
    d.click("button:has-text('Run')", settle=1500)
    d.scroll_to("text=Revenue by client")
    d.scroll_to("text=Aged receivables")
    d.click("button:has-text('Export CSV') >> nth=0", settle=1200)
    d.scroll_to("text=Tax collected")
    d.scroll_to("text=Payments received by method")
    d.click("button:has-text('Export all data')", settle=1200)


def settings(d):
    nav(d, "Settings")
    d.type_loc(d.label("Phone"), "+65 9123 4567", delay=40)
    d.click("button:has-text('Save business details')", settle=1100)
    d.click("a:text-is('Invoicing')", settle=900)
    d.type_loc(d.label("Name"), "VAT 20%", delay=50)
    d.type_loc(d.label("Rate %"), "20", delay=50)
    d.click("button:has-text('Add tax rate')", settle=1100)
    d.scroll_to("text=Invoice numbering")
    d.type_loc(d.label("Prefix"), "INV-2026-", delay=45)
    d.click("button:has-text('Save numbering')", settle=1000)


def settings_data(d):
    d.click("button:has-text('Back up now')", settle=1600)
    d.scroll_to("text=Automatic backups")
    d.click("button:has-text('Save backup settings')", settle=1000)
    d.scroll_to("text=Recent backups")
    d.click("button:has-text('Restore') >> nth=0", settle=1600)


def settings_more(d):
    pw = d.page.locator("input[type=password]")
    d.type_loc(pw.first, "correct horse battery", delay=35)
    d.type_loc(pw.nth(1), "correct horse battery", delay=35)
    d.click("button:has-text('Enable app lock')", settle=1200)
    d.click("a:text-is('Appearance')", settle=900)
    d.click("text=Enhanced", settle=1000)                   # motion preference
    d.click("a:text-is('About')", settle=900)
    d.scroll_to("text=Where things are")
    d.click("a:text-is('Support')", settle=900)
    d.scroll_to("text=Licence")


def ai_mcp(d):
    d.click("input[type=checkbox] >> nth=0", settle=1100)   # enable AI access
    d.click("input[type=checkbox] >> nth=1", settle=1100)   # allow changes
    d.click("button:has-text('Copy') >> nth=0", settle=900)
    d.scroll_to("text=What the assistant can do")
    d.click("button:has-text('Copy') >> nth=-1", settle=900)


# --------------------------------------------------------------------------- #
SEGMENTS = [
    {"id": "01-dashboard", "chapter": "Dashboard & navigation", "title": "Dashboard, chasing and the command palette", "script": dashboard, "start": "/"},
    {"id": "02-invoices-list", "chapter": "Invoices", "title": "Finding an invoice", "script": invoices_list, "start": "/"},
    {"id": "03-invoice-editor", "chapter": "Invoices", "title": "The invoice editor", "script": invoice_editor, "start": "/invoices/inv-0041"},
    {"id": "04-payments", "chapter": "Invoices", "title": "Payments, chasing and sending", "script": payments, "start": "/invoices/inv-0041"},
    {"id": "05-invoice-create", "chapter": "Invoices", "title": "Creating an invoice from scratch", "script": invoice_create, "start": "/"},
    {"id": "06-estimates", "chapter": "Estimates", "title": "Estimates and convert-to-invoice", "script": estimates, "start": "/"},
    {"id": "07-clients", "chapter": "Clients", "title": "Clients and their history", "script": clients, "start": "/"},
    {"id": "08-catalog", "chapter": "Catalog", "title": "The catalog", "script": catalog, "start": "/"},
    {"id": "09-reports", "chapter": "Reports", "title": "Reports and exports", "script": reports, "start": "/"},
    {"id": "10-settings", "chapter": "Settings", "title": "Business profile and invoicing", "script": settings, "start": "/"},
    {"id": "11-settings-data", "chapter": "Settings", "title": "Backups and restore", "script": settings_data, "start": "/settings/data"},
    {"id": "12-ai-mcp", "chapter": "AI access (MCP)", "title": "AI access over MCP", "script": ai_mcp, "start": "/settings/ai"},
    {"id": "13-settings-more", "chapter": "Settings", "title": "App lock, appearance, about and support", "script": settings_more, "start": "/settings/security"},
]
