import { invoke } from "@tauri-apps/api/core";
import type {
  AppInfo,
  Business,
  BusinessInput,
  CatalogItem,
  CatalogItemInput,
  Client,
  ClientInput,
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
};
