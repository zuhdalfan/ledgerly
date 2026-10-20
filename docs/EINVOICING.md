# E-invoicing in Europe: what it means for Ledgerly

Research note, written 2026-10-17. **No code implements any of this.** It exists
so the decision is on record and a future session does not start from scratch.

The short version: mandatory structured e-invoicing is arriving across Europe,
it is **not** a PDF, and the part Ledgerly can honestly do offline is *produce and
validate the file* — not send it. The real work is a data-model gap, not XML.

---

## 1. What changed

The EU's **VAT in the Digital Age** package (ViDA) was adopted by the Council on
**11 March 2025** and published in the Official Journal on 25 March 2025:

- Directive (EU) 2025/516
- Regulation (EU) 2025/517
- Implementing Regulation (EU) 2025/518

Key dates from the Commission's own summary:

| When | What |
|---|---|
| 11 Mar 2025 | Adopted. Member States may now introduce mandatory e-invoicing under defined conditions. |
| 1 Jan 2027 | Minor clarifications for One-Stop Shop (OSS) and IOSS users. |
| 1 Jul 2028 | Platform economy measures (short-term accommodation, passenger transport); Single VAT Registration. |
| **1 Jul 2030** | **Digital Reporting Requirements apply to cross-border B2B transactions.** |
| 1 Jan 2035 | Member States with domestic real-time reporting must align with EU standards. |

So the headline for a small-business tool: **intra-EU B2B e-invoicing becomes the
default in 2030**, and several countries already require it domestically *today*.

## 2. What is already live

National mandates run ahead of the EU timetable, and each has its own format and
often its own network:

| Country | Format | Delivery | Since |
|---|---|---|---|
| Italy | FatturaPA | SDI (national platform) | 2019 |
| Romania | RO e-Factura | national platform | 2024 |
| Germany | XRechnung / ZUGFeRD | Peppol | receive Jan 2025; send phased 2027–2028 |
| Belgium | Peppol BIS Billing 3.0 (UBL) | Peppol | Jan 2026 |
| Poland | KSeF XML | KSeF (national platform) | Feb 2026, phased |
| France | Factur-X / UBL / CII | certified platforms (PDP) | Sep 2026 large & mid, 2027 SMEs |
| Spain | Facturae / SII | national | 2026 |

These dates move. Treat the table as a starting point and re-check before building.

## 3. The standards

- **EN 16931** is the European standard: it defines the *semantic* data model of an
  invoice — which business terms exist (BT-1, BT-2, …) and which are mandatory —
  independently of any file format. A revision adapted for B2B and for digital
  reporting has been reported as approved in 2026; confirm with CEN before
  implementing against it.
- **UBL** and **CII** are the two permitted syntaxes. The same invoice can be
  expressed in either.
- **Peppol BIS Billing 3.0** is a *CIUS* (Core Invoice Usage Specification) of
  EN 16931: a constrained profile in the UBL syntax. It is the most widely
  required flavour, because a Peppol-compliant document is by construction
  EN 16931-compliant.
- Country flavours layer on top: XRechnung (Germany), Factur-X / ZUGFeRD (France —
  a hybrid PDF/A-3 *with* the XML embedded), KSeF (Poland), FatturaPA (Italy).

## 4. A PDF is not an e-invoice

This is the single most misread point, and it directly concerns Ledgerly. An
e-invoice is a **structured, machine-readable** document (the XML). A PDF — even a
very good one — does not qualify. The only hybrid that does is Factur-X/ZUGFeRD,
which is a PDF/A-3 with the XML *inside* it.

So the logo work and the PDF templates, however polished, move nothing towards
compliance.

## 5. The honest constraint: this is a *transmission* mandate

The obligation is to send and receive over an approved channel: a **Peppol access
point**, or a national platform (Italy's SDI, Poland's KSeF, France's PDPs). Every
one of those is a network service.

Ledgerly's core promise is "fully local, fully offline, no network requests ever"
(`scripts/check-offline.mjs` fails the build on an HTTP client crate). Becoming an
access point would mean becoming an online product. That is a product decision,
not an engineering one, and it is out of scope here.

**What is genuinely compatible**: generate the compliant XML locally, validate it
locally, and let the user hand the file to their access point, accountant, or
customer portal — exactly the way PDF export works today. That is a real,
useful feature; it is just not "sending".

## 6. What Ledgerly is missing today

The XML is the easy part. The data model cannot currently express what EN 16931
requires:

| Ledgerly today | EN 16931 needs |
|---|---|
| `tax_rate` = name + rate in basis points | **BT-151** line VAT *category* code is mandatory (`S` standard, `Z` zero, `E` exempt, `AE` reverse charge, `O` out of scope, …), and the document VAT breakdown groups by *(category, rate)* |
| `unit` is free text (`"hour"`) | `unitCode` from UN/ECE Recommendation 20 (`HUR`, `DAY`, `C62`, …) |
| `business.tax_id` / `client.tax_id` free text | VAT identifiers must be country-prefixed (`SG…`); a legal registration identifier needs an ISO/IEC 6523 ICD scheme |
| no electronic address | seller and buyer `EndpointID` with an EAS scheme code, which is how Peppol routes a document |
| no buyer reference | **BT-10** — an invoice needs either a buyer reference or an order reference |
| no bank details | `PaymentMeans` plus an account identifier (**BT-84**) for a credit transfer |
| `country` free text | ISO 3166-1 alpha-2 |

Adding VAT categories and unit codes means schema changes that touch tax rates,
catalog items, invoice lines and the PDF. That is the bulk of the work, and it is
worth doing on its own merits — the PDF would show real VAT categories.

## 7. Recommendation

**Phase 0 — research (this document).** Done.

**Phase 1 — offline Peppol BIS Billing 3.0 UBL export.** First extend the schema:
VAT category on tax rates, unit codes on lines, structured identifiers on business
and client. Then emit one UBL file per invoice, with a small subset of EN 16931
`BR-*` business rules checked locally before export so a document is never handed
over obviously broken. Also document that customers needing a *format* the EU does
not standardise (KSeF, FatturaPA) are out of scope for this phase.

**Phase 2 — Factur-X hybrid PDF/A-3**, if demand appears. `printpdf` has no PDF/A-3
support, so this is a PDF-layer project, not an XML one.

**We will not** become a Peppol access point or integrate a national platform. That
would break the offline promise, and it is the part a dedicated provider does
better anyway.

**Revisit trigger:** a real user who needs to *send* rather than *download*, or a
user in a country whose domestic mandate has just bitten them. Not a date.

## 8. Sources

- European Commission — [Adoption of the VAT in the Digital Age package](https://taxation-customs.ec.europa.eu/news/adoption-vat-digital-age-package-2025-03-11_en) (11 March 2025)
- European Commission — [VAT in the Digital Age (ViDA)](https://taxation-customs.ec.europa.eu/taxation/vat/vat-digital-age-vida_en)
- OpenPeppol — [Peppol BIS Billing 3.0 specification](https://docs.peppol.eu/poacc/billing/3.0/bis/)
- OpenPeppol — [Peppol](https://peppol.org)
- EN 16931 supporting artefacts — [Registry](https://ec.europa.eu/digital-building-blocks/sites/spaces/DIGITAL/pages/467108974/Registry+of+supporting+artefacts+to+implement+EN16931)
- Country timelines and format mapping — [Invoice Navigator: ViDA](https://www.invoicenavigator.eu/vida)
