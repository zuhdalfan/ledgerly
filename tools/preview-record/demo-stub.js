// Deterministic, stateful IPC stub for recording the Ledgerly walkthrough.
//
// Why this exists: tools/screenshots/stub.js only ever *reads* canned data, so
// a "create an invoice" flow would save and then land on an empty editor. This
// stub mutates an in-memory store (persisted to sessionStorage so it survives a
// full page load) and answers the Tauri *dialog* plugin too, so export, backup
// and PDF flows complete and show their success state.
//
// Determinism: the store is seeded from fixed data and "today" is pinned, so a
// run always produces the same numbers. sessionStorage is per-context, so each
// recording starts from the same seed.
//
// Development tooling only. Ships with nothing.

(() => {
  const KEY = "ledgerly.preview.store.v1";
  // Pinned clock: the seed dates are chosen around this day so aging, due dates
  // and "collected this month" are stable run to run.
  const TODAY = "2026-10-11";

  const minor = (n) => Math.round(n * 100);

  const seed = () => {
    let counter = 0;
    const uid = (p) => `${p}-${(++counter).toString().padStart(3, "0")}`;

    const business = {
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

    const clients = [
      ["acme", "Acme Corporation", "Dana Lee", "ap@acme.example", "+65 6222 1111", "SGD"],
      ["globex", "Globex", null, "billing@globex.example", null, "SGD"],
      ["initech", "Initech", null, "ap@initech.example", null, "USD"],
      ["umbrella", "Umbrella Pte Ltd", null, "accounts@umbrella.example", null, "SGD"],
      ["hooli", "Hooli", "Russ Hanneman", "finance@hooli.example", null, "SGD"],
    ].map(([id, displayName, contactPerson, email, phone, currency]) => ({
      id,
      displayName,
      company: displayName,
      contactPerson,
      email,
      phone,
      addressLine1: id === "acme" ? "1 Raffles Place" : null,
      addressLine2: null,
      city: id === "acme" ? "Singapore" : null,
      region: null,
      postalCode: id === "acme" ? "048616" : null,
      country: null,
      taxId: null,
      currency,
      notes: null,
      archivedAt: null,
      createdAt: "2026-01-05T09:00:00Z",
      updatedAt: "2026-09-28T09:00:00Z",
    }));

    const catalog = [
      ["Design hour", "Product and interface design.", "hour", minor(180)],
      ["Development hour", "Implementation work.", "hour", minor(220)],
      ["Consulting day", "Strategy and review.", "day", minor(1500)],
      ["Retainer — monthly", "Ongoing availability.", "month", minor(2500)],
      ["Print & stationery", "Printed collateral.", "item", minor(95)],
    ].map(([name, description, unit, unitPriceMinor]) => ({
      id: uid("cat"),
      name,
      description,
      unit,
      unitPriceMinor,
      currency: null,
      sku: null,
      archivedAt: null,
      createdAt: "2026-01-05T09:00:00Z",
      updatedAt: "2026-01-05T09:00:00Z",
    }));

    const taxRates = [
      { id: "gst9", name: "GST 9%", rateBp: 900 },
      { id: "zero", name: "Zero-rated", rateBp: 0 },
    ].map((r) => ({ ...r, archivedAt: null, createdAt: "2026-01-05T09:00:00Z", updatedAt: "2026-01-05T09:00:00Z" }));

    const numbering = {
      invoice: { kind: "invoice", prefix: "INV-", nextValue: 42, padding: 4, resetYearly: false, lastYear: null },
      estimate: { kind: "estimate", prefix: "EST-", nextValue: 13, padding: 4, resetYearly: false, lastYear: null },
    };

    const line = (description, quantity, unit, unitPriceMinor, taxRateId, extra = {}) => ({
      id: uid("li"),
      sortOrder: 0,
      description,
      quantity: String(quantity),
      unit,
      unitPriceMinor,
      discountType: null,
      discountValue: null,
      taxRateId,
      taxInclusive: false,
      ...extra,
    });

    const invoice = (over) => ({
      id: "", number: "", clientId: null, clientName: null, status: "draft",
      issueDate: TODAY, dueDate: null, currency: "SGD", poReference: null,
      discountType: null, discountValue: null, subtotalMinor: 0, discountMinor: 0,
      taxMinor: 0, totalMinor: 0, notes: null, terms: "Net 30",
      paymentInstructions: business.defaultPaymentInstructions,
      voidedAt: null, sentAt: null, createdAt: "2026-01-01T09:00:00Z",
      updatedAt: "2026-10-01T09:00:00Z", items: [], payments: [], lastChasedAt: null,
      chaseCount: 0, ...over,
    });

    const invoiceSeed = [
      ["INV-0041", "acme", "sent", "2026-07-15", "2026-08-14", "SGD", [["Product design", 24, "hour", minor(180), "gst9"], ["Interface design", 18, "hour", minor(180), "gst9"]]],
      ["INV-0040", "globex", "sent", "2026-08-02", "2026-09-01", "SGD", [["Retainer — August", 1, null, minor(8540), "gst9"]]],
      ["INV-0039", "initech", "sent", "2026-06-10", "2026-07-10", "SGD", [["Monthly support", 1, null, minor(3000), "gst9"]]],
      ["INV-0038", "umbrella", "sent", "2026-09-20", "2026-10-20", "SGD", [["Brand consulting", 1, null, minor(3520), "gst9"]]],
      ["INV-0037", "acme", "draft", "2026-10-08", "2026-11-07", "SGD", [["Discovery workshop", 1, null, minor(1800), "gst9"]]],
      ["INV-0036", "initech", "void", "2026-05-04", "2026-06-03", "SGD", [["Hardware procurement", 1, null, minor(4500), "gst9"]]],
      ["INV-0035", "hooli", "sent", "2026-09-01", "2026-10-01", "SGD", [["Consulting day", 3, "day", minor(1500), "gst9"]]],
      ["INV-0034", "globex", "sent", "2026-06-18", "2026-07-18", "SGD", [["Development hour", 40, "hour", minor(220), "gst9"]]],
    ].map(([number, clientId, status, issueDate, dueDate, currency, items], i) =>
      invoice({
        id: `inv-${number.slice(-4)}`,
        number,
        clientId,
        clientName: clients.find((c) => c.id === clientId).displayName,
        status,
        issueDate,
        dueDate,
        currency,
        sentAt: status === "draft" ? null : `${issueDate}T09:00:00Z`,
        voidedAt: status === "void" ? "2026-05-20T09:00:00Z" : null,
        items: items.map(([d, q, u, p, t], k) => line(d, q, u, p, t, { sortOrder: k })),
        payments:
          number === "INV-0039"
            ? [{ id: uid("pay"), date: "2026-07-08", amountMinor: minor(3270), method: "bank", reference: "TRF-8891", note: null }]
            : number === "INV-0040"
              ? [{ id: uid("pay"), date: "2026-09-04", amountMinor: minor(1500), method: "bank", reference: "TRF-9012", note: "Part payment" }]
              : [],
      })
    );

    const estimate = (over) => ({
      id: "", number: "", clientId: null, clientName: null, status: "draft",
      issueDate: TODAY, validUntil: null, currency: "SGD", poReference: null,
      discountType: null, discountValue: null, subtotalMinor: 0, discountMinor: 0,
      taxMinor: 0, totalMinor: 0, notes: null, terms: "Valid 30 days.",
      paymentInstructions: null, convertedInvoiceId: null, convertedAt: null,
      createdAt: "2026-01-01T09:00:00Z", updatedAt: "2026-10-01T09:00:00Z", items: [], ...over,
    });

    const estimates = [
      ["EST-0012", "acme", "sent", "2026-09-28", "2026-10-28", "SGD", [["Redesign retainer", 2, "day", minor(1800), "gst9"]]],
      ["EST-0011", "globex", "accepted", "2026-08-15", "2026-09-15", "SGD", [["Platform build", 1, null, minor(7040), "gst9"]]],
      ["EST-0010", "initech", "draft", "2026-10-05", "2026-11-04", "USD", [["Migration scoping", 2, "day", minor(1500), "zero"]]],
      ["EST-0009", "umbrella", "declined", "2026-07-01", "2026-07-31", "SGD", [["Roadmap workshop", 1, null, minor(2200), "gst9"]]],
    ].map(([number, clientId, status, issueDate, validUntil, currency, items]) =>
      estimate({
        id: `est-${number.slice(-4)}`,
        number,
        clientId,
        clientName: clients.find((c) => c.id === clientId).displayName,
        status,
        issueDate,
        validUntil,
        currency,
        items: items.map(([d, q, u, p, t], k) => line(d, q, u, p, t, { sortOrder: k })),
      })
    );

    return {
      counter,
      business,
      clients,
      catalog,
      taxRates,
      numbering,
      invoices: invoiceSeed,
      estimates,
      settings: {
        "ui.motion": "standard",
        "pdf.locale": "en",
        "pdf.template": "classic",
        "mcp.enabled": "false",
        "mcp.allowWrite": "false",
        "backup.enabled": "true",
        "backup.folder": "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups",
        "backup.keep": "10",
        "backup.lastRun": "2026-10-11 08:00:00",
      },
      security: { locked: false, settings: { enabled: false, autoLockMinutes: 15 }, minPassphraseLength: 8 },
      backups: [
        { path: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups\\ledgerly-20261011-080000.ledgerly", name: "ledgerly-20261011-080000.ledgerly", sizeBytes: 184320, modifiedAt: 1791676800 },
        { path: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups\\ledgerly-20261010-080000.ledgerly", name: "ledgerly-20261010-080000.ledgerly", sizeBytes: 182272, modifiedAt: 1791590400 },
      ],
    };
  };

  let store;
  try {
    const raw = sessionStorage.getItem(KEY);
    store = raw ? JSON.parse(raw) : seed();
  } catch {
    store = seed();
  }
  const persist = () => {
    try {
      sessionStorage.setItem(KEY, JSON.stringify(store));
    } catch {
      /* recording only; losing the store just re-seeds */
    }
  };

  const APP_INFO = {
    name: "ledgerly",
    version: "0.1.0",
    dataDir: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop",
    databasePath: "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\ledgerly.db",
  };
  const MCP = {
    enabled: true,
    allowWrite: false,
    serverPath: "C:\\Program Files\\Ledgerly\\ledgerly-mcp.exe",
    lockEnabled: false,
  };

  const rateFor = (id) => store.taxRates.find((r) => r.id === id && !r.archivedAt)?.rateBp ?? 0;
  const round = (v) => Math.round(v);

  function totalsFor(input) {
    const items = (input && input.items) || [];
    let subtotal = 0;
    let tax = 0;
    const lines = items.map((it) => {
      const gross = round((Number(it.quantity) || 0) * (Number(it.unitPriceMinor) || 0));
      let discount = 0;
      if (it.discountType === "percent") discount = round((gross * (Number(it.discountValue) || 0)) / 100);
      else if (it.discountType === "fixed") discount = Number(it.discountValue) || 0;
      const net = gross - discount;
      const bp = rateFor(it.taxRateId);
      const lineTax = bp ? round((net * bp) / (it.taxInclusive ? 10000 + bp : 10000)) : 0;
      subtotal += net;
      tax += lineTax;
      return { grossMinor: gross, discountMinor: discount, netMinor: net, taxableMinor: bp ? net : 0, taxMinor: lineTax, totalMinor: net + lineTax };
    });
    // Invoice-level discount applied after the lines, matching the Rust engine.
    let invDiscount = 0;
    const dt = input && input.discountType;
    if (dt === "percent") invDiscount = round((subtotal * (Number(input.discountValue) || 0)) / 100);
    else if (dt === "fixed") invDiscount = Number(input.discountValue) || 0;
    const total = subtotal - invDiscount + tax;
    return { lines, subtotalMinor: subtotal, discountMinor: invDiscount, taxMinor: tax, totalMinor: total };
  }

  function daysBetween(a, b) {
    return Math.round((Date.parse(b) - Date.parse(a)) / 86400000);
  }

  function decorateInvoice(inv) {
    const t = totalsFor(inv);
    const paid = inv.payments.reduce((s, p) => s + p.amountMinor, 0);
    const balance = t.totalMinor - paid;
    const daysPastDue = inv.dueDate && inv.sentAt && !inv.voidedAt ? Math.max(0, daysBetween(inv.dueDate, TODAY)) : null;
    let displayStatus;
    if (inv.voidedAt) displayStatus = "void";
    else if (t.totalMinor > 0 && balance <= 0) displayStatus = "paid";
    else if (paid > 0) displayStatus = "partially_paid";
    else if (inv.sentAt && daysPastDue > 0) displayStatus = "overdue";
    else if (inv.sentAt) displayStatus = "sent";
    else displayStatus = "draft";
    return {
      ...inv,
      status: inv.status,
      subtotalMinor: t.subtotalMinor,
      discountMinor: t.discountMinor,
      taxMinor: t.taxMinor,
      totalMinor: t.totalMinor,
      paidMinor: paid,
      balanceMinor: balance,
      daysPastDue: displayStatus === "overdue" || displayStatus === "partially_paid" ? daysPastDue : null,
      displayStatus,
      items: inv.items.map((it, i) => ({ ...it, ...t.lines[i] })),
    };
  }

  function decorateEstimate(est) {
    const t = totalsFor(est);
    let displayStatus = est.status;
    if (est.convertedInvoiceId) displayStatus = "converted";
    else if (["draft", "sent"].includes(est.status) && est.validUntil && daysBetween(est.validUntil, TODAY) > 0) displayStatus = "expired";
    return {
      ...est,
      subtotalMinor: t.subtotalMinor,
      discountMinor: t.discountMinor,
      taxMinor: t.taxMinor,
      totalMinor: t.totalMinor,
      daysUntilExpiry: est.validUntil ? -daysBetween(TODAY, est.validUntil) : null,
      displayStatus,
      items: est.items.map((it, i) => ({ ...it, ...t.lines[i], taxRateBp: rateFor(it.taxRateId) })),
    };
  }

  const clientName = (id) => store.clients.find((c) => c.id === id)?.displayName ?? null;

  function nextNumber(kind) {
    const n = store.numbering[kind];
    const year = n.resetYearly ? TODAY.slice(0, 4) : null;
    const value = String(n.nextValue).padStart(n.padding, "0");
    return `${n.prefix}${year ? year + "-" : ""}${value}`;
  }

  function normalizedItems(input) {
    return (input.items || []).map((it, i) => ({
      id: it.id || `li-new-${i}`,
      sortOrder: i,
      description: it.description ?? "",
      quantity: String(it.quantity ?? "1"),
      unit: it.unit || null,
      unitPriceMinor: it.unitPriceMinor ?? 0,
      discountType: it.discountType || null,
      discountValue: it.discountValue ?? null,
      taxRateId: it.taxRateId || null,
      taxInclusive: !!it.taxInclusive,
    }));
  }

  function createInvoiceFrom(input) {
    const kind = "invoice";
    const number = nextNumber(kind);
    const inv = {
      id: `inv-new-${++store.counter}`,
      number,
      clientId: input.clientId || null,
      clientName: clientName(input.clientId),
      status: "draft",
      issueDate: input.issueDate || TODAY,
      dueDate: input.dueDate || null,
      currency: input.currency || store.business.defaultCurrency,
      poReference: input.poReference || null,
      discountType: input.discountType || null,
      discountValue: input.discountValue ?? null,
      notes: input.notes || null,
      terms: input.terms || null,
      paymentInstructions: input.paymentInstructions || null,
      voidedAt: null,
      sentAt: null,
      createdAt: `${TODAY}T09:00:00Z`,
      updatedAt: `${TODAY}T09:00:00Z`,
      items: normalizedItems(input),
      payments: [],
      lastChasedAt: null,
      chaseCount: 0,
    };
    store.numbering[kind].nextValue += 1;
    store.invoices.unshift(inv);
    persist();
    return inv;
  }

  const findInvoice = (id) => store.invoices.find((i) => i.id === id);
  const findEstimate = (id) => store.estimates.find((e) => e.id === id);

  const handlers = {
    app_info: () => APP_INFO,
    business_get: () => store.business,
    business_update: (a) => {
      store.business = { ...store.business, ...(a.input || {}), updatedAt: `${TODAY}T09:00:00Z` };
      persist();
      return store.business;
    },

    clients_list: (a) => {
      const q = (a.query || "").toLowerCase();
      return store.clients.filter((c) => (a.includeArchived || !c.archivedAt) && (!q || `${c.displayName} ${c.company || ""} ${c.email || ""}`.toLowerCase().includes(q)));
    },
    clients_get: (a) => store.clients.find((c) => c.id === a.id) || null,
    clients_create: (a) => {
      const c = { ...a.input, id: `client-new-${++store.counter}`, archivedAt: null, createdAt: `${TODAY}T09:00:00Z`, updatedAt: `${TODAY}T09:00:00Z` };
      store.clients.unshift(c);
      persist();
      return c;
    },
    clients_update: (a) => {
      const c = store.clients.find((x) => x.id === a.id);
      Object.assign(c, a.input, { updatedAt: `${TODAY}T09:00:00Z` });
      persist();
      return c;
    },
    clients_set_archived: (a) => {
      const c = store.clients.find((x) => x.id === a.id);
      c.archivedAt = a.archived ? `${TODAY}T09:00:00Z` : null;
      persist();
      return c;
    },

    catalog_list: (a) => {
      const q = (a.query || "").toLowerCase();
      return store.catalog.filter((c) => (a.includeArchived || !c.archivedAt) && (!q || `${c.name} ${c.sku || ""}`.toLowerCase().includes(q)));
    },
    catalog_create: (a) => {
      const c = { ...a.input, id: `cat-new-${++store.counter}`, archivedAt: null, createdAt: `${TODAY}T09:00:00Z`, updatedAt: `${TODAY}T09:00:00Z` };
      store.catalog.unshift(c);
      persist();
      return c;
    },
    catalog_update: (a) => {
      const c = store.catalog.find((x) => x.id === a.id);
      Object.assign(c, a.input, { updatedAt: `${TODAY}T09:00:00Z` });
      persist();
      return c;
    },
    catalog_set_archived: (a) => {
      const c = store.catalog.find((x) => x.id === a.id);
      c.archivedAt = a.archived ? `${TODAY}T09:00:00Z` : null;
      persist();
      return c;
    },

    tax_rates_list: (a) => store.taxRates.filter((r) => a.includeArchived || !r.archivedAt),
    tax_rates_create: (a) => {
      const r = { ...a.input, id: `tax-new-${++store.counter}`, archivedAt: null, createdAt: `${TODAY}T09:00:00Z`, updatedAt: `${TODAY}T09:00:00Z` };
      store.taxRates.push(r);
      persist();
      return r;
    },
    tax_rates_update: (a) => {
      const r = store.taxRates.find((x) => x.id === a.id);
      Object.assign(r, a.input);
      persist();
      return r;
    },
    tax_rates_set_archived: (a) => {
      const r = store.taxRates.find((x) => x.id === a.id);
      r.archivedAt = a.archived ? `${TODAY}T09:00:00Z` : null;
      persist();
      return r;
    },

    numbering_get: (a) => store.numbering[a.kind],
    numbering_update: (a) => {
      store.numbering[a.kind] = { ...store.numbering[a.kind], ...a.input };
      persist();
      return store.numbering[a.kind];
    },

    invoices_list: (a) => {
      const q = (a.query || "").toLowerCase();
      return store.invoices
        .map(decorateInvoice)
        .filter((i) => !a.clientId || i.clientId === a.clientId)
        .filter((i) => !q || `${i.number} ${i.clientName || ""}`.toLowerCase().includes(q))
        .filter((i) => {
          if (!a.status) return true;
          if (a.status === "unpaid") return i.balanceMinor > 0 && !i.voidedAt;
          return i.displayStatus === a.status;
        })
        .filter((i) => {
          if (!a.age) return true;
          const d = i.daysPastDue ?? 0;
          const bucket = d <= 0 ? "not_due" : d <= 30 ? "1-30" : d <= 60 ? "31-60" : d <= 90 ? "61-90" : "90plus";
          return bucket === a.age;
        })
        .filter((i) => !a.issuedMonth || i.issueDate.startsWith(a.issuedMonth));
    },
    invoices_get: (a) => (findInvoice(a.id) ? decorateInvoice(findInvoice(a.id)) : null),
    invoices_totals: (a) => totalsFor(a.input),
    invoices_create: (a) => decorateInvoice(createInvoiceFrom(a.input)),
    invoices_update: (a) => {
      const inv = findInvoice(a.id);
      Object.assign(inv, a.input, { items: normalizedItems(a.input), updatedAt: `${TODAY}T09:00:00Z` });
      persist();
      return decorateInvoice(inv);
    },
    invoices_duplicate: (a) => {
      const src = findInvoice(a.id);
      const copy = createInvoiceFrom({ ...src, items: src.items });
      return decorateInvoice(copy);
    },
    invoices_set_void: (a) => {
      const inv = findInvoice(a.id);
      inv.voidedAt = a.voided ? `${TODAY}T09:00:00Z` : null;
      persist();
      return decorateInvoice(inv);
    },
    invoices_set_sent: (a) => {
      const inv = findInvoice(a.id);
      inv.sentAt = a.sent ? `${TODAY}T09:00:00Z` : null;
      persist();
      return decorateInvoice(inv);
    },
    invoices_set_chased: (a) => {
      const inv = findInvoice(a.id);
      inv.lastChasedAt = a.chased ? `${TODAY}T09:00:00Z` : null;
      inv.chaseCount = a.chased ? (inv.chaseCount || 0) + 1 : 0;
      persist();
      return decorateInvoice(inv);
    },

    payments_list: (a) => findInvoice(a.invoiceId)?.payments ?? [],
    payments_create: (a) => {
      const inv = findInvoice(a.invoiceId);
      const p = { id: `pay-new-${++store.counter}`, date: a.input.date || TODAY, amountMinor: a.input.amountMinor, method: a.input.method || null, reference: a.input.reference || null, note: a.input.note || null };
      inv.payments.push(p);
      persist();
      return p;
    },
    payments_update: (a) => {
      for (const inv of store.invoices) {
        const p = inv.payments.find((x) => x.id === a.id);
        if (p) {
          Object.assign(p, a.input);
          persist();
          return p;
        }
      }
      return null;
    },
    payments_delete: (a) => {
      for (const inv of store.invoices) {
        const i = inv.payments.findIndex((x) => x.id === a.id);
        if (i >= 0) {
          inv.payments.splice(i, 1);
          persist();
          return;
        }
      }
    },

    estimates_list: (a) => {
      const q = (a.query || "").toLowerCase();
      return store.estimates
        .map(decorateEstimate)
        .filter((e) => !a.clientId || e.clientId === a.clientId)
        .filter((e) => !q || `${e.number} ${e.clientName || ""}`.toLowerCase().includes(q))
        .filter((e) => !a.status || e.displayStatus === a.status);
    },
    estimates_get: (a) => (findEstimate(a.id) ? decorateEstimate(findEstimate(a.id)) : null),
    estimates_totals: (a) => totalsFor(a.input),
    estimates_create: (a) => {
      const number = nextNumber("estimate");
      const est = {
        id: `est-new-${++store.counter}`, number, clientId: a.input.clientId || null,
        clientName: clientName(a.input.clientId), status: "draft",
        issueDate: a.input.issueDate || TODAY, validUntil: a.input.validUntil || null,
        currency: a.input.currency || store.business.defaultCurrency, poReference: a.input.poReference || null,
        discountType: a.input.discountType || null, discountValue: a.input.discountValue ?? null,
        notes: a.input.notes || null, terms: a.input.terms || null,
        paymentInstructions: a.input.paymentInstructions || null,
        convertedInvoiceId: null, convertedAt: null,
        createdAt: `${TODAY}T09:00:00Z`, updatedAt: `${TODAY}T09:00:00Z`,
        items: normalizedItems(a.input),
      };
      store.numbering.estimate.nextValue += 1;
      store.estimates.unshift(est);
      persist();
      return decorateEstimate(est);
    },
    estimates_update: (a) => {
      const est = findEstimate(a.id);
      Object.assign(est, a.input, { items: normalizedItems(a.input), updatedAt: `${TODAY}T09:00:00Z` });
      persist();
      return decorateEstimate(est);
    },
    estimates_duplicate: (a) => {
      const src = findEstimate(a.id);
      return handlers.estimates_create({ input: { ...src, items: src.items, clientId: src.clientId } });
    },
    estimates_set_status: (a) => {
      const est = findEstimate(a.id);
      est.status = a.status;
      persist();
      return decorateEstimate(est);
    },
    estimates_convert: (a) => {
      const est = findEstimate(a.id);
      const inv = createInvoiceFrom({ ...est, items: est.items });
      est.convertedInvoiceId = inv.id;
      est.convertedAt = `${TODAY}T09:00:00Z`;
      persist();
      return decorateInvoice(inv);
    },

    reports_revenue: (a) => {
      const rows = new Map();
      for (const inv of store.invoices) {
        if (inv.voidedAt) continue;
        if (inv.issueDate < a.from || inv.issueDate > a.to) continue;
        const key = a.granularity === "year" ? inv.issueDate.slice(0, 4) : a.granularity === "month" ? inv.issueDate.slice(0, 7) : inv.issueDate;
        const t = totalsFor(inv);
        const paid = inv.payments.reduce((s, p) => s + p.amountMinor, 0);
        const cur = rows.get(key) || { period: key, invoiceCount: 0, invoicedMinor: 0, taxMinor: 0, paidMinor: 0 };
        cur.invoiceCount += 1;
        cur.invoicedMinor += t.totalMinor;
        cur.taxMinor += t.taxMinor;
        cur.paidMinor += paid;
        rows.set(key, cur);
      }
      return [...rows.values()].sort((x, y) => (x.period < y.period ? -1 : 1));
    },
    reports_revenue_by_client: (a) =>
      store.clients.map((c) => {
        const invs = store.invoices.filter((i) => i.clientId === c.id && !i.voidedAt && i.issueDate >= a.from && i.issueDate <= a.to);
        const t = invs.reduce((s, i) => s + totalsFor(i).totalMinor, 0);
        const paid = invs.reduce((s, i) => s + i.payments.reduce((x, p) => x + p.amountMinor, 0), 0);
        return { clientId: c.id, clientName: c.displayName, invoiceCount: invs.length, invoicedMinor: t, paidMinor: paid, outstandingMinor: t - paid };
      }).filter((r) => r.invoiceCount > 0),
    reports_aged_receivables: () => {
      const buckets = { "Not yet due": 0, "1–30 days": 0, "31–60 days": 0, "61–90 days": 0, "90+ days": 0 };
      const counts = { ...buckets };
      for (const inv of store.invoices.map(decorateInvoice)) {
        if (inv.voidedAt || inv.balanceMinor <= 0) continue;
        const d = inv.daysPastDue ?? 0;
        const label = d <= 0 ? "Not yet due" : d <= 30 ? "1–30 days" : d <= 60 ? "31–60 days" : d <= 90 ? "61–90 days" : "90+ days";
        buckets[label] += inv.balanceMinor;
        counts[label] += 1;
      }
      return Object.keys(buckets).map((label) => ({ label, invoiceCount: counts[label], balanceMinor: buckets[label] }));
    },
    reports_tax: (a) => {
      const rows = new Map();
      for (const inv of store.invoices) {
        if (inv.voidedAt || inv.issueDate < a.from || inv.issueDate > a.to) continue;
        for (const it of inv.items) {
          const bp = rateFor(it.taxRateId);
          const key = it.taxRateId || "none";
          const t = totalsFor({ items: [it] });
          const cur = rows.get(key) || { taxRateId: key, name: bp ? store.taxRates.find((r) => r.id === it.taxRateId)?.name ?? "Tax" : "No tax", rateBp: bp, taxableMinor: 0, taxMinor: 0 };
          cur.taxableMinor += bp ? t.subtotalMinor : 0;
          cur.taxMinor += t.taxMinor;
          rows.set(key, cur);
        }
      }
      return [...rows.values()];
    },
    reports_payments_by_method: (a) => {
      const rows = new Map();
      for (const inv of store.invoices) {
        for (const p of inv.payments) {
          const key = p.method || "unspecified";
          const cur = rows.get(key) || { method: key, paymentCount: 0, amountMinor: 0 };
          cur.paymentCount += 1;
          cur.amountMinor += p.amountMinor;
          rows.set(key, cur);
        }
      }
      return [...rows.values()];
    },

    export_csv: () => "",
    export_json: () => "",
    open_in_viewer: () => undefined,
    pdf_export_invoice: (a) => a.path,
    pdf_export_estimate: (a) => a.path,
    pdf_export_statement: (a) => a.path,

    settings_get: (a) => (a.key in store.settings ? store.settings[a.key] : null),
    settings_set: (a) => {
      store.settings[a.key] = a.value;
      persist();
    },
    mcp_info: () => ({
      ...MCP,
      enabled: store.settings["mcp.enabled"] === "true",
      allowWrite: store.settings["mcp.allowWrite"] === "true",
      lockEnabled: store.security.settings.enabled,
    }),

    security_status: () => ({ ...store.security, settings: { ...store.security.settings } }),
    security_unlock: () => {
      store.security.locked = false;
      persist();
      return handlers.security_status();
    },
    security_lock: () => {
      store.security.locked = true;
      persist();
      return handlers.security_status();
    },
    security_enable: (a) => {
      store.security.settings.enabled = true;
      store.security.settings.autoLockMinutes = a.input?.autoLockMinutes ?? 15;
      persist();
      return handlers.security_status();
    },
    security_disable: () => {
      store.security.settings.enabled = false;
      persist();
      return handlers.security_status();
    },
    security_set_auto_lock: (a) => {
      store.security.settings.autoLockMinutes = a.minutes;
      persist();
      return handlers.security_status();
    },

    backup_settings_get: () => ({
      enabled: store.settings["backup.enabled"] === "true",
      folder: store.settings["backup.folder"],
      keep: Number(store.settings["backup.keep"]),
      lastRun: store.settings["backup.lastRun"],
    }),
    backup_settings_set: (a) => {
      store.settings["backup.enabled"] = String(!!a.input.enabled);
      store.settings["backup.folder"] = a.input.folder;
      store.settings["backup.keep"] = String(a.input.keep);
      persist();
      return handlers.backup_settings_get();
    },
    backup_list: () => store.backups,
    backup_now: (a) => {
      const stamp = "20261011-120000";
      const path = a.path || `C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups\\ledgerly-${stamp}.ledgerly`;
      store.backups.unshift({ path, name: path.split("\\").pop(), sizeBytes: 186368, modifiedAt: 1791691200 });
      persist();
      return path;
    },
    backup_restore: () => "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\pre-restore-20261011.sqlite",
  };

  // The Tauri dialog plugin: the app guards on a returned path, so these have
  // to answer for export / backup flows to complete in the browser.
  const dialogHandlers = {
    save: (a) => a?.options?.defaultPath || "C:\\Users\\you\\Desktop\\ledgerly-export.pdf",
    open: () => "C:\\Users\\you\\AppData\\Roaming\\com.ledgerly.desktop\\backups\\ledgerly-20261011-080000.ledgerly",
    ask: () => true,
    message: () => null,
    confirm: () => true,
  };

  const clone = (v) => JSON.parse(JSON.stringify(v));

  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args) => {
      const name = String(cmd);
      if (name.startsWith("plugin:dialog|")) {
        const kind = name.split("|")[1];
        const h = dialogHandlers[kind];
        return h ? Promise.resolve(h(args || {})) : Promise.resolve(null);
      }
      const handler = handlers[name];
      if (!handler) {
        console.warn("[demo-stub] unhandled command:", name, args);
        return Promise.resolve(null);
      }
      try {
        return Promise.resolve(clone(handler(args || {}) ?? null));
      } catch (error) {
        return Promise.reject(error);
      }
    },
    transformCallback: (cb) => {
      const id = Math.floor(1e6 + Math.random() * 1e9);
      window[`_${id}`] = cb;
      return id;
    },
    unregisterCallback: () => {},
    convertFileSrc: (p) => p,
    metadata: {},
  };
})();
