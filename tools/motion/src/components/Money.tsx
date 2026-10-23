import type { FC } from "react";
import { FONT } from "../fonts";
import { money } from "../lib/format";
import { tokens } from "../theme";

/** Money rolls, never scrambles, and never jitters: digits are tabular. */
export const Money: FC<{
  value: number;
  size?: number;
  color?: string;
  weight?: 400 | 700;
}> = ({ value, size = 30, color = tokens.ink1, weight = 700 }) => (
  <span
    style={{
      fontFamily: FONT,
      fontSize: size,
      fontWeight: weight,
      letterSpacing: "-0.02em",
      color,
      fontVariantNumeric: "tabular-nums",
      whiteSpace: "nowrap",
    }}
  >
    {money(value)}
  </span>
);
