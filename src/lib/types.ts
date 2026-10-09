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

export type TaxRate = {
  id: string;
  name: string;
  rateBp: number;
  archivedAt: string | null;
  createdAt: string;
  updatedAt: string;
};

export type TaxRateInput = {
  name: string;
  rateBp: number;
};

export type Numbering = {
  kind: string;
  prefix: string;
  nextValue: number;
  padding: number;
  resetYearly: boolean;
  lastYear: number | null;
};

export type NumberingInput = {
  prefix: string;
  nextValue: number;
  padding: number;
  resetYearly: boolean;
};

export type InvoiceItem = {
  id: string;
  sortOrder: number;
  description: string;
  quantity: string;
  unit: string | null;
  unitPriceMinor: number;
  discountType: string | null;
  discountValue: string | null;
  taxRateId: string | null;
  taxRateName: string | null;
  taxInclusive: boolean;
};

export type Invoice = {
  id: string;
  number: string;
  clientId: string | null;
  clientName: string | null;
  status: string;
  issueDate: string;
  dueDate: string | null;
  currency: string;
  poReference: string | null;
  discountType: string | null;
  discountValue: string | null;
  subtotalMinor: number;
  discountMinor: number;
  taxMinor: number;
  totalMinor: number;
  notes: string | null;
  terms: string | null;
  paymentInstructions: string | null;
  voidedAt: string | null;
  sentAt: string | null;
  createdAt: string;
  updatedAt: string;
  paidMinor: number;
  balanceMinor: number;
  daysPastDue: number | null;
  displayStatus: string;
  items: InvoiceItem[];
};

export type Payment = {
  id: string;
  invoiceId: string;
  paidOn: string;
  amountMinor: number;
  method: string | null;
  reference: string | null;
  note: string | null;
  createdAt: string;
  updatedAt: string;
};

export type PaymentInput = {
  paidOn: string;
  amountMinor: number;
  method: string | null;
  reference: string | null;
  note: string | null;
};

/** Line items are identical for invoices and estimates. */
export type LineItemInput = {
  description: string;
  quantity: string;
  unit: string | null;
  unitPriceMinor: number;
  discountType: string | null;
  discountValue: string | null;
  taxRateId: string | null;
  taxInclusive: boolean;
};

export type EstimateItem = {
  id: string;
  sortOrder: number;
  description: string;
  quantity: string;
  unit: string | null;
  unitPriceMinor: number;
  discountType: string | null;
  discountValue: string | null;
  taxRateId: string | null;
  taxRateName: string | null;
  taxRateBp: number | null;
  taxInclusive: boolean;
};

export type Estimate = {
  id: string;
  number: string;
  clientId: string | null;
  clientName: string | null;
  status: string;
  issueDate: string;
  validUntil: string | null;
  currency: string;
  poReference: string | null;
  discountType: string | null;
  discountValue: string | null;
  subtotalMinor: number;
  discountMinor: number;
  taxMinor: number;
  totalMinor: number;
  notes: string | null;
  terms: string | null;
  paymentInstructions: string | null;
  convertedInvoiceId: string | null;
  convertedAt: string | null;
  createdAt: string;
  updatedAt: string;
  daysUntilExpiry: number | null;
  displayStatus: string;
  items: EstimateItem[];
};

export type EstimateInput = {
  clientId: string | null;
  issueDate: string;
  validUntil: string | null;
  currency: string;
  poReference: string | null;
  discountType: string | null;
  discountValue: string | null;
  notes: string | null;
  terms: string | null;
  paymentInstructions: string | null;
  items: LineItemInput[];
};

export type InvoiceInput = {
  clientId: string | null;
  issueDate: string;
  dueDate: string | null;
  currency: string;
  poReference: string | null;
  discountType: string | null;
  discountValue: string | null;
  notes: string | null;
  terms: string | null;
  paymentInstructions: string | null;
  items: LineItemInput[];
};

export type BackupInfo = {
  path: string;
  name: string;
  sizeBytes: number;
  modifiedAt: number | null;
};

export type BackupSettings = {
  enabled: boolean;
  folder: string;
  keep: number;
  lastRun: string | null;
};

export type BackupSettingsInput = {
  enabled: boolean;
  folder: string;
  keep: number;
};

export type PeriodRow = {
  period: string;
  invoiceCount: number;
  invoicedMinor: number;
  taxMinor: number;
  paidMinor: number;
};

export type ClientRevenueRow = {
  clientId: string | null;
  clientName: string;
  invoiceCount: number;
  invoicedMinor: number;
  paidMinor: number;
  outstandingMinor: number;
};

export type AgedRow = {
  label: string;
  invoiceCount: number;
  balanceMinor: number;
};

export type TaxRow = {
  taxRateId: string | null;
  name: string;
  rateBp: number;
  taxableMinor: number;
  taxMinor: number;
};

export type MethodRow = {
  method: string;
  paymentCount: number;
  amountMinor: number;
};

export type LineTotals = {
  grossMinor: number;
  discountMinor: number;
  netMinor: number;
  taxableMinor: number;
  taxMinor: number;
  totalMinor: number;
};

export type InvoiceTotals = {
  lines: LineTotals[];
  subtotalMinor: number;
  discountMinor: number;
  taxMinor: number;
  totalMinor: number;
};
