import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  Business,
  BusinessInput,
  CatalogItem,
  CatalogItemInput,
  Client,
  ClientInput,
  Invoice,
  InvoiceInput,
  InvoiceTotals,
  Numbering,
  NumberingInput,
  TaxRate,
  TaxRateInput,
} from "./types";

export const api = {
  appInfo: () => invoke<AppInfo>("app_info"),

  business: {
    get: () => invoke<Business>("business_get"),
    update: (input: BusinessInput) => invoke<Business>("business_update", { input }),
  },

  clients: {
    list: (query?: string, includeArchived = false) =>
      invoke<Client[]>("clients_list", { query: query ?? null, includeArchived }),
    get: (id: string) => invoke<Client | null>("clients_get", { id }),
    create: (input: ClientInput) => invoke<Client>("clients_create", { input }),
    update: (id: string, input: ClientInput) => invoke<Client>("clients_update", { id, input }),
    setArchived: (id: string, archived: boolean) =>
      invoke<Client>("clients_set_archived", { id, archived }),
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
    list: (query?: string, status?: string) =>
      invoke<Invoice[]>("invoices_list", { query: query ?? null, status: status ?? null }),
    get: (id: string) => invoke<Invoice | null>("invoices_get", { id }),
    totals: (input: InvoiceInput) => invoke<InvoiceTotals>("invoices_totals", { input }),
    create: (input: InvoiceInput) => invoke<Invoice>("invoices_create", { input }),
    update: (id: string, input: InvoiceInput) => invoke<Invoice>("invoices_update", { id, input }),
    duplicate: (id: string) => invoke<Invoice>("invoices_duplicate", { id }),
    setVoid: (id: string, voided: boolean) => invoke<Invoice>("invoices_set_void", { id, voided }),
  },
};
