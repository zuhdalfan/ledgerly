export type AppInfo = {
  name: string;
  version: string;
  dataDir: string;
  databasePath: string;
};

export type Business = {
  id: string;
  displayName: string;
  legalName: string | null;
  email: string | null;
  phone: string | null;
  website: string | null;
  addressLine1: string | null;
  addressLine2: string | null;
  city: string | null;
  region: string | null;
  postalCode: string | null;
  country: string | null;
  taxId: string | null;
  defaultCurrency: string;
  defaultTerms: string | null;
  defaultNotes: string | null;
  defaultPaymentInstructions: string | null;
  updatedAt: string;
};

export type BusinessInput = Omit<Business, "id" | "updatedAt">;

export type Client = {
  id: string;
  displayName: string;
  company: string | null;
  contactPerson: string | null;
  email: string | null;
  phone: string | null;
  addressLine1: string | null;
  addressLine2: string | null;
  city: string | null;
  region: string | null;
  postalCode: string | null;
  country: string | null;
  taxId: string | null;
  currency: string | null;
  notes: string | null;
  archivedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type ClientInput = Omit<Client, "id" | "archivedAt" | "createdAt" | "updatedAt">;

export type CatalogItem = {
  id: string;
  name: string;
  description: string | null;
  unit: string | null;
  unitPriceMinor: number;
  currency: string | null;
  sku: string | null;
  archivedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type CatalogItemInput = Omit<
  CatalogItem,
  "id" | "archivedAt" | "createdAt" | "updatedAt"
>;
