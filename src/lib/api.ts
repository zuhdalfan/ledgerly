import { invoke } from "@tauri-apps/api/core";
import { errorText } from "./errors";
import type {
  AgedRow,
  AppInfo,
  BackupInfo,
  BackupSettings,
  BackupSettingsInput,
  Business,
  BusinessInput,
  SecurityStatus,
  TrayStatus,
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

/**
 * The one place every command goes through, so a backend failure reaches the
 * UI in the reader's language. `errorText` returns a plain string, which the
 * existing `String(e)` handlers render as-is — rethrowing an `Error` would
 * prefix every message with "Error:".
 */
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error) {
    throw errorText(error);
  }
}

export const api = {
  appInfo: () => call<AppInfo>("app_info"),

  business: {
    get: () => call<Business>("business_get"),
    update: (input: BusinessInput) => call<Business>("business_update", { input }),
    logoDataUrl: () => call<string | null>("business_logo_data_url"),
    setLogo: (path: string) => call<string>("business_logo_set", { path }),
    clearLogo: () => call<void>("business_logo_clear"),
  },

  clients: {
    list: (query?: string, includeArchived = false) =>
      call<Client[]>("clients_list", { query: query ?? null, includeArchived }),
    get: (id: string) => call<Client | null>("clients_get", { id }),
    create: (input: ClientInput) => call<Client>("clients_create", { input }),
    update: (id: string, input: ClientInput) => call<Client>("clients_update", { id, input }),
    setArchived: (id: string, archived: boolean) =>
      call<Client>("clients_set_archived", { id, archived }),
    exportStatement: (id: string, path: string, template?: string) =>
      call<string>("pdf_export_statement", { clientId: id, path, template }),
  },

  catalog: {
    list: (query?: string, includeArchived = false) =>
      call<CatalogItem[]>("catalog_list", { query: query ?? null, includeArchived }),
    create: (input: CatalogItemInput) => call<CatalogItem>("catalog_create", { input }),
    update: (id: string, input: CatalogItemInput) =>
      call<CatalogItem>("catalog_update", { id, input }),
    setArchived: (id: string, archived: boolean) =>
      call<CatalogItem>("catalog_set_archived", { id, archived }),
  },

  taxRates: {
    list: (includeArchived = false) =>
      call<TaxRate[]>("tax_rates_list", { includeArchived }),
    create: (input: TaxRateInput) => call<TaxRate>("tax_rates_create", { input }),
    update: (id: string, input: TaxRateInput) =>
      call<TaxRate>("tax_rates_update", { id, input }),
    setArchived: (id: string, archived: boolean) =>
      call<TaxRate>("tax_rates_set_archived", { id, archived }),
  },

  numbering: {
    get: (kind: string) => call<Numbering>("numbering_get", { kind }),
    update: (kind: string, input: NumberingInput) =>
      call<Numbering>("numbering_update", { kind, input }),
  },

  invoices: {
    list: (query?: string, status?: string, clientId?: string, age?: string, issuedMonth?: string) =>
      call<Invoice[]>("invoices_list", {
        query: query ?? null,
        status: status ?? null,
        clientId: clientId ?? null,
        age: age ?? null,
        issuedMonth: issuedMonth ?? null,
      }),
    get: (id: string) => call<Invoice | null>("invoices_get", { id }),
    totals: (input: InvoiceInput) => call<InvoiceTotals>("invoices_totals", { input }),
    create: (input: InvoiceInput) => call<Invoice>("invoices_create", { input }),
    update: (id: string, input: InvoiceInput) => call<Invoice>("invoices_update", { id, input }),
    duplicate: (id: string) => call<Invoice>("invoices_duplicate", { id }),
    setVoid: (id: string, voided: boolean) => call<Invoice>("invoices_set_void", { id, voided }),
    setSent: (id: string, sent: boolean) => call<Invoice>("invoices_set_sent", { id, sent }),
    setChased: (id: string, chased: boolean) =>
      call<Invoice>("invoices_set_chased", { id, chased }),
    exportPdf: (id: string, path: string, template: string) =>
      call<string>("pdf_export_invoice", { id, path, template }),
  },

  reports: {
    revenue: (from: string, to: string, granularity: string) =>
      call<PeriodRow[]>("reports_revenue", { from, to, granularity }),
    revenueByClient: (from: string, to: string) =>
      call<ClientRevenueRow[]>("reports_revenue_by_client", { from, to }),
    agedReceivables: () => call<AgedRow[]>("reports_aged_receivables"),
    tax: (from: string, to: string) => call<TaxRow[]>("reports_tax", { from, to }),
    paymentsByMethod: (from: string, to: string) =>
      call<MethodRow[]>("reports_payments_by_method", { from, to }),
  },

  exportCsv: (path: string, headers: string[], rows: string[][]) =>
    call<string>("export_csv", { path, headers, rows }),
  exportJson: (path: string) => call<string>("export_json", { path }),

  settings: {
    get: (key: string) => call<string | null>("settings_get", { key }),
    set: (key: string, value: string) => call<void>("settings_set", { key, value }),
  },

  mcp: {
    info: () => call<McpInfo>("mcp_info"),
  },

  tray: {
    status: () => call<TrayStatus>("tray_status"),
    setEnabled: (enabled: boolean, lockOnHide?: boolean) =>
      call<TrayStatus>("tray_set_enabled", {
        enabled,
        lockOnHide: lockOnHide ?? null,
      }),
  },

  reminders: {
    setEnabled: (enabled: boolean) =>
      call<TrayStatus>("reminders_set_enabled", { enabled }),
    test: () => call<void>("reminders_test"),
  },

  security: {
    status: () => call<SecurityStatus>("security_status"),
    unlock: (passphrase: string) => call<SecurityStatus>("security_unlock", { passphrase }),
    lock: () => call<SecurityStatus>("security_lock"),
    enable: (passphrase: string, autoLockMinutes: number) =>
      call<SecurityStatus>("security_enable", {
        input: { passphrase, autoLockMinutes },
      }),
    disable: (passphrase: string) => call<SecurityStatus>("security_disable", { passphrase }),
    setAutoLock: (minutes: number) =>
      call<SecurityStatus>("security_set_auto_lock", { minutes }),
  },

  backup: {
    now: (path?: string) => call<string>("backup_now", { path: path ?? null }),
    restore: (path: string) => call<string>("backup_restore", { path }),
    list: () => call<BackupInfo[]>("backup_list"),
    getSettings: () => call<BackupSettings>("backup_settings_get"),
    saveSettings: (input: BackupSettingsInput) =>
      call<BackupSettings>("backup_settings_set", { input }),
  },

  estimates: {
    list: (query?: string, status?: string, clientId?: string) =>
      call<Estimate[]>("estimates_list", {
        query: query ?? null,
        status: status ?? null,
        clientId: clientId ?? null,
      }),
    get: (id: string) => call<Estimate | null>("estimates_get", { id }),
    totals: (input: EstimateInput) => call<InvoiceTotals>("estimates_totals", { input }),
    create: (input: EstimateInput) => call<Estimate>("estimates_create", { input }),
    update: (id: string, input: EstimateInput) =>
      call<Estimate>("estimates_update", { id, input }),
    duplicate: (id: string) => call<Estimate>("estimates_duplicate", { id }),
    setStatus: (id: string, status: string) =>
      call<Estimate>("estimates_set_status", { id, status }),
    convert: (id: string) => call<Invoice>("estimates_convert", { id }),
    exportPdf: (id: string, path: string, template?: string) =>
      call<string>("pdf_export_estimate", { id, path, template }),
  },

  payments: {
    list: (invoiceId: string) => call<Payment[]>("payments_list", { invoiceId }),
    create: (invoiceId: string, input: PaymentInput) =>
      call<Payment>("payments_create", { invoiceId, input }),
    update: (id: string, input: PaymentInput) =>
      call<Payment>("payments_update", { id, input }),
    remove: (id: string) => call<void>("payments_delete", { id }),
  },

  openInViewer: (path: string) => call<void>("open_in_viewer", { path }),
};
