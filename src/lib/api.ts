import { invoke } from "@tauri-apps/api/core";
import type {
  AgedRow,
  AppInfo,
  BackupInfo,
  BackupSettings,
  BackupSettingsInput,
  Business,
  BusinessInput,
  SecurityStatus,
  ClientRevenueRow,
  MethodRow,
  PeriodRow,
  TaxRow,
  McpInfo,
  CatalogItem,
  CatalogItemInput,
  Client,
  ClientInput,
  Estimate,
  EstimateInput,
  Invoice,
  InvoiceInput,
  InvoiceTotals,
  Numbering,
  NumberingInput,
  Payment,
  PaymentInput,
  TaxRate,
  TaxRateInput,
} from "./types";

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),

  business: {
    get: () => invoke<Business>("business_get"),
    update: (input: BusinessInput) => invoke<Business>("business_update", { input }),
    logoDataUrl: () => invoke<string | null>("business_logo_data_url"),
    setLogo: (path: string) => invoke<string>("business_logo_set", { path }),
    clearLogo: () => invoke<void>("business_logo_clear"),
  },

  clients: {
    list: (query?: string, includeArchived = false) =>
      invoke<Client[]>("clients_list", { query: query ?? null, includeArchived }),
    get: (id: string) => invoke<Client | null>("clients_get", { id }),
    create: (input: ClientInput) => invoke<Client>("clients_create", { input }),
    update: (id: string, input: ClientInput) => invoke<Client>("clients_update", { id, input }),
    setArchived: (id: string, archived: boolean) =>
      invoke<Client>("clients_set_archived", { id, archived }),
    exportStatement: (id: string, path: string, template?: string) =>
      invoke<string>("pdf_export_statement", { clientId: id, path, template }),
  },

  catalog: {
    list: (query?: string, includeArchived = false) =>
      invoke<CatalogItem[]>("catalog_list", { query: query ?? null, includeArchived }),
    create: (input: CatalogItemInput) => invoke<CatalogItem>("catalog_create", { input }),
    update: (id: string, input: CatalogItemInput) =>
      invoke<CatalogItem>("catalog_update", { id, input }),
    setArchived: (id: string, archived: boolean) =>
      invoke<CatalogItem>("catalog_set_archived", { id, archived }),
  },

  taxRates: {
    list: (includeArchived = false) =>
      invoke<TaxRate[]>("tax_rates_list", { includeArchived }),
    create: (input: TaxRateInput) => invoke<TaxRate>("tax_rates_create", { input }),
    update: (id: string, input: TaxRateInput) =>
      invoke<TaxRate>("tax_rates_update", { id, input }),
    setArchived: (id: string, archived: boolean) =>
      invoke<TaxRate>("tax_rates_set_archived", { id, archived }),
  },

  numbering: {
    get: (kind: string) => invoke<Numbering>("numbering_get", { kind }),
    update: (kind: string, input: NumberingInput) =>
      invoke<Numbering>("numbering_update", { kind, input }),
  },

  invoices: {
    list: (query?: string, status?: string, clientId?: string, age?: string, issuedMonth?: string) =>
      invoke<Invoice[]>("invoices_list", {
        query: query ?? null,
        status: status ?? null,
        clientId: clientId ?? null,
        age: age ?? null,
        issuedMonth: issuedMonth ?? null,
      }),
    get: (id: string) => invoke<Invoice | null>("invoices_get", { id }),
    totals: (input: InvoiceInput) => invoke<InvoiceTotals>("invoices_totals", { input }),
    create: (input: InvoiceInput) => invoke<Invoice>("invoices_create", { input }),
    update: (id: string, input: InvoiceInput) => invoke<Invoice>("invoices_update", { id, input }),
    duplicate: (id: string) => invoke<Invoice>("invoices_duplicate", { id }),
    setVoid: (id: string, voided: boolean) => invoke<Invoice>("invoices_set_void", { id, voided }),
    setSent: (id: string, sent: boolean) => invoke<Invoice>("invoices_set_sent", { id, sent }),
    setChased: (id: string, chased: boolean) =>
      invoke<Invoice>("invoices_set_chased", { id, chased }),
    exportPdf: (id: string, path: string, template: string) =>
      invoke<string>("pdf_export_invoice", { id, path, template }),
  },

  reports: {
    revenue: (from: string, to: string, granularity: string) =>
      invoke<PeriodRow[]>("reports_revenue", { from, to, granularity }),
    revenueByClient: (from: string, to: string) =>
      invoke<ClientRevenueRow[]>("reports_revenue_by_client", { from, to }),
    agedReceivables: () => invoke<AgedRow[]>("reports_aged_receivables"),
    tax: (from: string, to: string) => invoke<TaxRow[]>("reports_tax", { from, to }),
    paymentsByMethod: (from: string, to: string) =>
      invoke<MethodRow[]>("reports_payments_by_method", { from, to }),
  },

  exportCsv: (path: string, headers: string[], rows: string[][]) =>
    invoke<string>("export_csv", { path, headers, rows }),
  exportJson: (path: string) => invoke<string>("export_json", { path }),

  settings: {
    get: (key: string) => invoke<string | null>("settings_get", { key }),
    set: (key: string, value: string) => invoke<void>("settings_set", { key, value }),
  },

  mcp: {
    info: () => invoke<McpInfo>("mcp_info"),
  },

  security: {
    status: () => invoke<SecurityStatus>("security_status"),
    unlock: (passphrase: string) => invoke<SecurityStatus>("security_unlock", { passphrase }),
    lock: () => invoke<SecurityStatus>("security_lock"),
    enable: (passphrase: string, autoLockMinutes: number) =>
      invoke<SecurityStatus>("security_enable", {
        input: { passphrase, autoLockMinutes },
      }),
    disable: (passphrase: string) => invoke<SecurityStatus>("security_disable", { passphrase }),
    setAutoLock: (minutes: number) =>
      invoke<SecurityStatus>("security_set_auto_lock", { minutes }),
  },

  backup: {
    now: (path?: string) => invoke<string>("backup_now", { path: path ?? null }),
    restore: (path: string) => invoke<string>("backup_restore", { path }),
    list: () => invoke<BackupInfo[]>("backup_list"),
    getSettings: () => invoke<BackupSettings>("backup_settings_get"),
    saveSettings: (input: BackupSettingsInput) =>
      invoke<BackupSettings>("backup_settings_set", { input }),
  },

  estimates: {
    list: (query?: string, status?: string, clientId?: string) =>
      invoke<Estimate[]>("estimates_list", {
        query: query ?? null,
        status: status ?? null,
        clientId: clientId ?? null,
      }),
    get: (id: string) => invoke<Estimate | null>("estimates_get", { id }),
    totals: (input: EstimateInput) => invoke<InvoiceTotals>("estimates_totals", { input }),
    create: (input: EstimateInput) => invoke<Estimate>("estimates_create", { input }),
    update: (id: string, input: EstimateInput) =>
      invoke<Estimate>("estimates_update", { id, input }),
    duplicate: (id: string) => invoke<Estimate>("estimates_duplicate", { id }),
    setStatus: (id: string, status: string) =>
      invoke<Estimate>("estimates_set_status", { id, status }),
    convert: (id: string) => invoke<Invoice>("estimates_convert", { id }),
    exportPdf: (id: string, path: string, template?: string) =>
      invoke<string>("pdf_export_estimate", { id, path, template }),
  },

  payments: {
    list: (invoiceId: string) => invoke<Payment[]>("payments_list", { invoiceId }),
    create: (invoiceId: string, input: PaymentInput) =>
      invoke<Payment>("payments_create", { invoiceId, input }),
    update: (id: string, input: PaymentInput) =>
      invoke<Payment>("payments_update", { id, input }),
    remove: (id: string) => invoke<void>("payments_delete", { id }),
  },

  openInViewer: (path: string) => invoke<void>("open_in_viewer", { path }),
};
