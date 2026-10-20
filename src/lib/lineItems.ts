import { emptyToNull, minorToInput, normalizeDecimal, toMinor } from "./format";
import type { LineItemInput } from "./types";

/** The string-based shape the editor binds to. */
export type ItemForm = {
  description: string;
  quantity: string;
  unit: string;
  price: string;
  discountType: string;
  discountValue: string;
  taxRateId: string;
  taxInclusive: boolean;
};

export function emptyItem(): ItemForm {
  return {
    description: "",
    quantity: "1",
    unit: "",
    price: "",
    discountType: "",
    discountValue: "",
    taxRateId: "",
    taxInclusive: false,
  };
}

type StoredItem = {
  description: string;
  quantity: string;
  unit: string | null;
  unitPriceMinor: number;
  discountType: string | null;
  discountValue: string | null;
  taxRateId: string | null;
  taxInclusive: boolean;
};

/** Turn a stored line item into the editor's string form. Fixed discounts are
 *  stored in minor units, so they are converted back for display. */
export function toItemForm(item: StoredItem): ItemForm {
  return {
    description: item.description,
    quantity: item.quantity,
    unit: item.unit ?? "",
    price: minorToInput(item.unitPriceMinor),
    discountType: item.discountType ?? "",
    discountValue:
      item.discountType === "fixed" && item.discountValue
        ? minorToInput(Number(item.discountValue))
        : (item.discountValue ?? ""),
    taxRateId: item.taxRateId ?? "",
    taxInclusive: item.taxInclusive,
  };
}

/** Turn the editor form into the payload shape. */
export function toItemInput(item: ItemForm): LineItemInput {
  return {
    description: item.description,
    quantity: normalizeDecimal(item.quantity.trim() || "1"),
    unit: emptyToNull(item.unit),
    unitPriceMinor: toMinor(item.price),
    discountType: item.discountType || null,
    discountValue:
      item.discountType === ""
        ? null
        : item.discountType === "fixed"
          ? String(toMinor(item.discountValue))
          : normalizeDecimal(item.discountValue),
    taxRateId: item.taxRateId || null,
    taxInclusive: item.taxInclusive,
  };
}

export function toItemInputs(items: ItemForm[]): LineItemInput[] {
  return items.map(toItemInput);
}
