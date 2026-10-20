// Throwaway capture harness: stubs the Tauri IPC bridge with sample data so
// every Ledgerly screen renders in a plain browser. Not part of the repo.
(() => {
  const APP_INFO = {
    name: "ledgerly",
    version: "0.1.0",
    dataDir: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop",
    databasePath: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\ledgerly.db",
  };

  const BUSINESS = {
    id: "biz",
    displayName: "Zuhdi Studio",
    legalName: "Zuhdi Studio Pte Ltd",
    email: "hello@zuhdistudio.com",
    phone: "+65 9123 4567",
    website: "zuhdistudio.com",
    addressLine1: "21 Tanjong Pagar Road",
    addressLine2: "#04-01",
    city: "Singapore",
    region: null,
    postalCode: "088444",
    country: "Singapore",
    taxId: "GST-2026-1234567",
    defaultCurrency: "SGD",
    defaultTerms: "Net 30",
    defaultNotes: "Thank you for your business.",
    defaultPaymentInstructions: "Bank transfer to DBS 012-345678-9 (Zuhdi Studio Pte Ltd)",
    updatedAt: "2026-10-11T08:00:00Z",
  };

  const client = (over) => ({
    id: "",
    displayName: "",
    company: null,
    contactPerson: null,
    email: null,
    phone: null,
    addressLine1: null,
    addressLine2: null,
    city: null,
    region: null,
    postalCode: null,
    country: null,
    taxId: null,
    currency: "SGD",
    notes: null,
    archivedAt: null,
    createdAt: "2026-01-05T09:00:00Z",
    updatedAt: "2026-09-28T09:00:00Z",
    ...over,
  });

  const CLIENTS = [
    client({
      id: "acme",
      displayName: "Acme Corporation",
      company: "Acme Corporation",
      contactPerson: "Dana Lee",
      email: "ap@acme.example",
      phone: "+65 6222 1111",
      addressLine1: "1 Raffles Place",
      city: "Singapore",
      postalCode: "048616",
      country: "Singapore",
      currency: "SGD",
    }),
    client({
      id: "globex",
      displayName: "Globex",
      company: "Globex",
      email: "billing@globex.example",
      currency: "SGD",
    }),
    client({
      id: "initech",
      displayName: "Initech",
      company: "Initech",
      email: "ap@initech.example",
      currency: "USD",
    }),
    client({
      id: "umbrella",
      displayName: "Umbrella Pte Ltd",
      company: "Umbrella Pte Ltd",
      email: "accounts@umbrella.example",
      currency: "SGD",
    }),
  ];

  const TAX_RATES = [
    { id: "gst9", name: "GST 9%", rateBp: 900, archivedAt: null, createdAt: "2026-01-05T09:00:00Z", updatedAt: "2026-01-05T09:00:00Z" },
    { id: "zero", name: "Zero-rated", rateBp: 0, archivedAt: null, createdAt: "2026-01-05T09:00:00Z", updatedAt: "2026-01-05T09:00:00Z" },
  ];

  const CATALOG = [
    { id: "cat1", name: "Design hour", description: "Product and interface design.", unit: "hour", unitPriceMinor: 18000, currency: null, sku: null, archivedAt: null, createdAt: "2026-01-05T09:00:00Z", updatedAt: "2026-01-05T09:00:00Z" },
    { id: "cat2", name: "Development hour", description: "Implementation work.", unit: "hour", unitPriceMinor: 22000, currency: null, sku: null, archivedAt: null, createdAt: "2026-01-05T09:00:00Z", updatedAt: "2026-01-05T09:00:00Z" },
    { id: "cat3", name: "Consulting day", description: "Strategy and review.", unit: "day", unitPriceMinor: 150000, currency: null, sku: null, archivedAt: null, createdAt: "2026-01-05T09:00:00Z", updatedAt: "2026-01-05T09:00:00Z" },
  ];

  const line = (over) => ({
    id: "li-" + Math.random().toString(36).slice(2, 8),
    sortOrder: 0,
    description: "",
    quantity: "1",
    unit: null,
    unitPriceMinor: 0,
    discountType: null,
    discountValue: null,
    taxRateId: null,
    taxRateName: null,
    taxInclusive: false,
    ...over,
  });

  const invoice = (over) => ({
    id: "",
    number: "",
    clientId: null,
    clientName: null,
    status: "draft",
    issueDate: "2026-01-01",
    dueDate: null,
    currency: "SGD",
    poReference: null,
    discountType: null,
    discountValue: null,
    subtotalMinor: 0,
    discountMinor: 0,
    taxMinor: 0,
    totalMinor: 0,
    notes: null,
    terms: "Net 30",
    paymentInstructions: "Bank transfer to DBS 012-345678-9 (Zuhdi Studio Pte Ltd)",
    voidedAt: null,
    sentAt: null,
    createdAt: "2026-01-01T09:00:00Z",
    updatedAt: "2026-10-01T09:00:00Z",
    paidMinor: 0,
    balanceMinor: 0,
    daysPastDue: null,
    displayStatus: "draft",
    lastChasedAt: null,
    chaseCount: 0,
    items: [],
    ...over,
  });

  const INVOICES = [
    invoice({
      id: "inv-0041",
      number: "INV-0041",
      clientId: "acme",
      clientName: "Acme Corporation",
      status: "sent",
      issueDate: "2026-07-15",
      dueDate: "2026-08-14",
      currency: "SGD",
      subtotalMinor: 756000,
      taxMinor: 68040,
      totalMinor: 824040,
      balanceMinor: 824040,
      daysPastDue: 58,
      displayStatus: "overdue",
      items: [
        line({ sortOrder: 0, description: "Product design", quantity: "24", unit: "hour", unitPriceMinor: 18000, taxRateId: "gst9", taxRateName: "GST 9%" }),
        line({ sortOrder: 1, description: "Interface design", quantity: "18", unit: "hour", unitPriceMinor: 18000, taxRateId: "gst9", taxRateName: "GST 9%" }),
      ],
    }),
    invoice({
      id: "inv-0040",
      number: "INV-0040",
      clientId: "globex",
      clientName: "Globex",
      status: "sent",
      issueDate: "2026-08-02",
      dueDate: "2026-09-01",
      subtotalMinor: 854000,
      taxMinor: 76860,
      totalMinor: 930860,
      paidMinor: 150000,
      balanceMinor: 780860,
      daysPastDue: 40,
      displayStatus: "partially_paid",
      items: [line({ description: "Retainer — August", quantity: "1", unitPriceMinor: 854000, taxRateId: "gst9", taxRateName: "GST 9%" })],
    }),
    invoice({
      id: "inv-0039",
      number: "INV-0039",
      clientId: "initech",
      clientName: "Initech",
      status: "paid",
      issueDate: "2026-06-10",
      dueDate: "2026-07-10",
      subtotalMinor: 300000,
      taxMinor: 27000,
      totalMinor: 327000,
      paidMinor: 327000,
      balanceMinor: 0,
      displayStatus: "paid",
      items: [line({ description: "Monthly support", quantity: "1", unitPriceMinor: 300000, taxRateId: "gst9", taxRateName: "GST 9%" })],
    }),
    invoice({
      id: "inv-0038",
      number: "INV-0038",
      clientId: "umbrella",
      clientName: "Umbrella Pte Ltd",
      status: "sent",
      issueDate: "2026-09-20",
      dueDate: "2026-10-20",
      subtotalMinor: 352000,
      taxMinor: 31680,
      totalMinor: 383680,
      balanceMinor: 383680,
      displayStatus: "sent",
      items: [line({ description: "Brand consulting", quantity: "1", unitPriceMinor: 352000, taxRateId: "gst9", taxRateName: "GST 9%" })],
    }),
    invoice({
      id: "inv-0037",
      number: "INV-0037",
      clientId: "acme",
      clientName: "Acme Corporation",
      status: "draft",
      issueDate: "2026-10-08",
      dueDate: "2026-11-07",
      subtotalMinor: 180000,
      taxMinor: 16200,
      totalMinor: 196200,
      balanceMinor: 196200,
      displayStatus: "draft",
      items: [line({ description: "Discovery workshop", quantity: "1", unitPriceMinor: 180000, taxRateId: "gst9", taxRateName: "GST 9%" })],
    }),
    invoice({
      id: "inv-0036",
      number: "INV-0036",
      clientId: "initech",
      clientName: "Initech",
      status: "void",
      issueDate: "2026-05-04",
      dueDate: "2026-06-03",
      subtotalMinor: 450000,
      taxMinor: 40500,
      totalMinor: 490500,
      balanceMinor: 490500,
      voidedAt: "2026-05-20T09:00:00Z",
      displayStatus: "void",
      items: [line({ description: "Hardware procurement", quantity: "1", unitPriceMinor: 450000, taxRateId: "gst9", taxRateName: "GST 9%" })],
    }),
  ];

  const estimate = (over) => ({
    id: "",
    number: "",
    clientId: null,
    clientName: null,
    status: "draft",
    issueDate: "2026-01-01",
    validUntil: null,
    currency: "SGD",
    poReference: null,
    discountType: null,
    discountValue: null,
    subtotalMinor: 0,
    discountMinor: 0,
    taxMinor: 0,
    totalMinor: 0,
    notes: null,
    terms: "Valid 30 days.",
    paymentInstructions: null,
    convertedInvoiceId: null,
    convertedAt: null,
    createdAt: "2026-01-01T09:00:00Z",
    updatedAt: "2026-10-01T09:00:00Z",
    daysUntilExpiry: null,
    displayStatus: "draft",
    items: [],
    ...over,
  });

  const ESTIMATES = [
    estimate({
      id: "est-0012",
      number: "EST-0012",
      clientId: "acme",
      clientName: "Acme Corporation",
      status: "sent",
      issueDate: "2026-09-28",
      validUntil: "2026-10-28",
      subtotalMinor: 360000,
      taxMinor: 32400,
      totalMinor: 392400,
      daysUntilExpiry: 17,
      displayStatus: "sent",
      items: [
        { id: "ei-1", sortOrder: 0, description: "Redesign retainer", quantity: "2", unit: "day", unitPriceMinor: 180000, discountType: null, discountValue: null, taxRateId: "gst9", taxRateName: "GST 9%", taxRateBp: 900, taxInclusive: false },
      ],
    }),
    estimate({
      id: "est-0011",
      number: "EST-0011",
      clientId: "globex",
      clientName: "Globex",
      status: "accepted",
      issueDate: "2026-08-15",
      validUntil: "2026-09-15",
      subtotalMinor: 704000,
      taxMinor: 63360,
      totalMinor: 767360,
      daysUntilExpiry: -26,
      displayStatus: "accepted",
      items: [
        { id: "ei-2", sortOrder: 0, description: "Platform build", quantity: "1", unit: null, unitPriceMinor: 704000, discountType: null, discountValue: null, taxRateId: "gst9", taxRateName: "GST 9%", taxRateBp: 900, taxInclusive: false },
      ],
    }),
    estimate({
      id: "est-0010",
      number: "EST-0010",
      clientId: "initech",
      clientName: "Initech",
      status: "draft",
      issueDate: "2026-10-05",
      validUntil: "2026-11-04",
      currency: "USD",
      subtotalMinor: 300000,
      taxMinor: 0,
      totalMinor: 300000,
      daysUntilExpiry: 24,
      displayStatus: "draft",
      items: [
        { id: "ei-3", sortOrder: 0, description: "Migration scoping", quantity: "2", unit: "day", unitPriceMinor: 150000, discountType: null, discountValue: null, taxRateId: "zero", taxRateName: "Zero-rated", taxRateBp: 0, taxInclusive: false },
      ],
    }),
  ];

  const SETTINGS = {
    "ui.motion": "standard",
    "pdf.locale": "en",
    "pdf.template": "classic",
  };

  const SECURITY = {
    locked: false,
    settings: { enabled: false, autoLockMinutes: 15 },
    minPassphraseLength: 8,
  };

  const MCP = {
    enabled: true,
    allowWrite: false,
    serverPath: "C:\\Program Files\\Ledgerly\\ledgerly-mcp.exe",
    lockEnabled: false,
  };

  const BACKUP_SETTINGS = {
    enabled: true,
    folder: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups",
    keep: 10,
    lastRun: "2026-10-11 08:00:00",
  };

  const BACKUPS = [
    { path: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups\\ledgerly-20261011-080000.ledgerly", name: "ledgerly-20261011-080000.ledgerly", sizeBytes: 184320, modifiedAt: 1791676800 },
    { path: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups\\ledgerly-20261010-080000.ledgerly", name: "ledgerly-20261010-080000.ledgerly", sizeBytes: 182272, modifiedAt: 1791590400 },
    { path: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups\\ledgerly-20261009-080000.ledgerly", name: "ledgerly-20261009-080000.ledgerly", sizeBytes: 180224, modifiedAt: 1791504000 },
  ];

  const NUMBERING = { kind: "invoice", prefix: "INV-", nextValue: 42, padding: 4, resetYearly: false, lastYear: null };

  const REVENUE = [
    { period: "2026-05", invoiceCount: 8, invoicedMinor: 1240000, taxMinor: 111600, paidMinor: 980000 },
    { period: "2026-06", invoiceCount: 6, invoicedMinor: 980000, taxMinor: 88200, paidMinor: 1120000 },
    { period: "2026-07", invoiceCount: 9, invoicedMinor: 1470000, taxMinor: 132300, paidMinor: 900000 },
    { period: "2026-08", invoiceCount: 7, invoicedMinor: 1120000, taxMinor: 100800, paidMinor: 1310000 },
    { period: "2026-09", invoiceCount: 5, invoicedMinor: 860000, taxMinor: 77400, paidMinor: 1040000 },
    { period: "2026-10", invoiceCount: 4, invoicedMinor: 640000, taxMinor: 57600, paidMinor: 480000 },
  ];

  const REVENUE_BY_CLIENT = [
    { clientId: "acme", clientName: "Acme Corporation", invoiceCount: 5, invoicedMinor: 1980000, paidMinor: 900000, outstandingMinor: 1180000 },
    { clientId: "globex", clientName: "Globex", invoiceCount: 3, invoicedMinor: 1120000, paidMinor: 970000, outstandingMinor: 150000 },
    { clientId: "umbrella", clientName: "Umbrella Pte Ltd", invoiceCount: 2, invoicedMinor: 640000, paidMinor: 640000, outstandingMinor: 0 },
    { clientId: "initech", clientName: "Initech", invoiceCount: 4, invoicedMinor: 900000, paidMinor: 900000, outstandingMinor: 0 },
  ];

  const AGED = [
    { label: "Not yet due", invoiceCount: 2, balanceMinor: 579880 },
    { label: "1–30 days", invoiceCount: 0, balanceMinor: 0 },
    { label: "31–60 days", invoiceCount: 2, balanceMinor: 1604900 },
    { label: "61–90 days", invoiceCount: 0, balanceMinor: 0 },
    { label: "90+ days", invoiceCount: 0, balanceMinor: 0 },
  ];

  const TAX = [
    { taxRateId: "gst9", name: "GST 9%", rateBp: 900, taxableMinor: 4820000, taxMinor: 433800 },
    { taxRateId: "zero", name: "No tax", rateBp: 0, taxableMinor: 900000, taxMinor: 0 },
  ];

  const METHODS = [
    { method: "bank", paymentCount: 12, amountMinor: 3980000 },
    { method: "cash", paymentCount: 2, amountMinor: 260000 },
  ];

  const clone = (value) => JSON.parse(JSON.stringify(value));
  const rateFor = (id) => TAX_RATES.find((rate) => rate.id === id)?.rateBp ?? 0;

  function computeTotals(input) {
    const items = (input && input.items) || [];
    let subtotal = 0;
    let tax = 0;
    const lines = items.map((item) => {
      const qty = Number(item.quantity) || 0;
      const price = Number(item.unitPriceMinor) || 0;
      const gross = Math.round(qty * price);
      let discount = 0;
      if (item.discountType === "percent") discount = Math.round((gross * (Number(item.discountValue) || 0)) / 100);
      else if (item.discountType === "fixed") discount = Number(item.discountValue) || 0;
      const net = gross - discount;
      const bp = rateFor(item.taxRateId);
      const lineTax = bp ? Math.round(net * bp / (item.taxInclusive ? 10000 + bp : 10000)) : 0;
      subtotal += net;
      tax += lineTax;
      return { grossMinor: gross, discountMinor: discount, netMinor: net, taxableMinor: bp ? net : 0, taxMinor: lineTax, totalMinor: net + lineTax };
    });
    return { lines, subtotalMinor: subtotal, discountMinor: 0, taxMinor: tax, totalMinor: subtotal + tax };
  }

  if (new URLSearchParams(location.search).has("lock")) SECURITY.locked = true;

  const handlers = {
    app_info: () => APP_INFO,
    business_get: () => BUSINESS,
    business_update: (a) => ({ ...BUSINESS, ...(a.input || {}) }),
    // No logo in the captures, so the settings page shows its placeholder.
    business_logo_data_url: () => null,
    business_logo_set: () => null,
    business_logo_clear: () => null,

    clients_list: (a) => {
      const q = (a.query || "").toLowerCase();
      return CLIENTS.filter((c) => !q || `${c.displayName} ${c.company || ""} ${c.email || ""}`.toLowerCase().includes(q));
    },
    clients_get: (a) => CLIENTS.find((c) => c.id === a.id) || null,
    clients_create: (a) => ({ ...CLIENTS[0], ...(a.input || {}), id: "new-client" }),
    clients_update: () => CLIENTS[0],
    clients_set_archived: () => CLIENTS[0],

    catalog_list: () => CATALOG,
    catalog_create: () => CATALOG[0],
    catalog_update: () => CATALOG[0],
    catalog_set_archived: () => CATALOG[0],

    tax_rates_list: () => TAX_RATES,
    tax_rates_create: () => TAX_RATES[0],
    tax_rates_set_archived: () => TAX_RATES[0],
    numbering_get: () => NUMBERING,
    numbering_update: () => NUMBERING,

    invoices_list: (a) => INVOICES.filter((i) => !a.clientId || i.clientId === a.clientId),
    invoices_get: (a) => INVOICES.find((i) => i.id === a.id) || null,
    invoices_totals: (a) => computeTotals(a.input),
    invoices_create: (a) => ({ ...INVOICES[0], id: "new-invoice", number: "INV-0042" }),
    invoices_update: () => INVOICES[0],
    invoices_duplicate: () => INVOICES[0],
    invoices_set_void: () => INVOICES[0],
    invoices_set_sent: () => INVOICES[0],
    invoices_set_chased: () => INVOICES[0],

    estimates_list: (a) => ESTIMATES.filter((e) => !a.clientId || e.clientId === a.clientId),
    estimates_get: (a) => ESTIMATES.find((e) => e.id === a.id) || null,
    estimates_totals: (a) => computeTotals(a.input),
    estimates_create: (a) => ({ ...ESTIMATES[0], id: "new-estimate", number: "EST-0013" }),
    estimates_update: () => ESTIMATES[0],
    estimates_set_status: () => ESTIMATES[0],

    payments_list: () => [],
    payments_create: () => ({}),
    payments_delete: () => undefined,

    reports_revenue: () => REVENUE,
    reports_revenue_by_client: () => REVENUE_BY_CLIENT,
    reports_aged_receivables: () => AGED,
    reports_tax: () => TAX,
    reports_payments_by_method: () => METHODS,

    mcp_info: () => MCP,

    backup_settings_get: () => BACKUP_SETTINGS,
    backup_settings_set: () => BACKUP_SETTINGS,
    backup_list: () => BACKUPS,

    settings_get: (a) => (a.key in SETTINGS ? SETTINGS[a.key] : null),
    settings_set: () => undefined,

    security_status: () => SECURITY,
    security_unlock: () => ({ ...SECURITY, locked: false }),
    security_lock: () => ({ ...SECURITY, locked: true }),

    open_in_viewer: () => undefined,
    export_csv: () => "",
    export_json: () => "",
  };

  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args) => {
      const handler = handlers[cmd];
      if (!handler) {
        console.warn("[stub] unhandled command:", cmd, args);
        return Promise.resolve(null);
      }
      try {
        return Promise.resolve(clone(handler(args || {})));
      } catch (error) {
        return Promise.reject(error);
      }
    },
    transformCallback: (cb) => {
      const id = Math.floor(Math.random() * 1e9);
      window[`_${id}`] = cb;
      return id;
    },
    unregisterCallback: () => {},
    convertFileSrc: (path) => path,
    metadata: {},
  };
})();
