import type { FC } from "react";
import { FONT } from "../fonts";
import { tokens } from "../theme";

/** An aged-receivables bar. Grows left to right; the track is the empty paper. */
export const Bar: FC<{
  value: number;
  max: number;
  progress: number;
  color?: string;
  height?: number;
  width?: number;
  label?: string;
}> = ({ value, max, progress, color = tokens.ink1, height = 12, width = 420, label }) => {
  const ratio = max <= 0 ? 0 : Math.min(1, value / max);
  return (
    <div style={{ display: "flex", alignItems: "center", gap: 12 }}>
      {label ? (
        <span style={{ width: 92, fontFamily: FONT, fontSize: 13, color: tokens.ink3 }}>{label}</span>
      ) : null}
      <div
        style={{
          width,
          height,
          borderRadius: tokens.radiusFull,
          background: tokens.paper3,
          overflow: "hidden",
        }}
      >
        <div
          style={{
            width: `${ratio * progress * 100}%`,
            height: "100%",
            borderRadius: tokens.radiusFull,
            background: color,
          }}
        />
      </div>
    </div>
  );
};
