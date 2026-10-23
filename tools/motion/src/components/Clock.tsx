import type { FC } from "react";
import { FONT } from "../fonts";
import { tokens } from "../theme";

/**
 * Digits flip rather than crossfade, so the overnight jump is unmistakable and
 * the stillness around it reads as idle time, not a pause in the render.
 */
export const Clock: FC<{
  from: string;
  to: string;
  flip: number;
  size?: number;
}> = ({ from, to, flip, size = 54 }) => (
  <div
    style={{
      position: "relative",
      height: size * 1.2,
      width: size * 3.2,
      fontFamily: FONT,
      fontSize: size,
      fontWeight: 700,
      letterSpacing: "-0.02em",
      color: tokens.ink1,
      fontVariantNumeric: "tabular-nums",
      overflow: "hidden",
    }}
  >
    <span
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        opacity: 1 - flip,
        transform: `translateY(${-flip * size * 0.35}px)`,
      }}
    >
      {from}
    </span>
    <span
      style={{
        position: "absolute",
        inset: 0,
        display: "flex",
        alignItems: "center",
        justifyContent: "center",
        opacity: flip,
        transform: `translateY(${(1 - flip) * size * 0.35}px)`,
      }}
    >
      {to}
    </span>
  </div>
);
