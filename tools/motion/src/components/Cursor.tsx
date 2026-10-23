import type { FC } from "react";
import { tokens } from "../theme";

/** A pointer, drawn rather than borrowed, so it sits in the same ink as the UI. */
export const Cursor: FC<{
  x: number;
  y: number;
  opacity?: number;
  press?: number;
}> = ({ x, y, opacity = 1, press = 0 }) => (
  <div
    style={{
      position: "absolute",
      left: x,
      top: y,
      opacity,
      display: "flex",
      alignItems: "flex-start",
      justifyContent: "flex-start",
      pointerEvents: "none",
      zIndex: 20,
    }}
  >
    <svg width={26} height={30} viewBox="0 0 26 30" style={{ display: "block" }}>
      <path
        d="M2 1.5 L2 24 L7.6 18.8 L11 27 L15.6 25 L12.2 16.9 L20 16.9 Z"
        fill={tokens.ink1}
        stroke={tokens.paper0}
        strokeWidth={1.6}
        strokeLinejoin="round"
      />
    </svg>
    {press > 0 ? (
      <span
        style={{
          position: "absolute",
          left: -6,
          top: -4,
          width: 22,
          height: 22,
          borderRadius: tokens.radiusFull,
          border: `1.5px solid ${tokens.ink1}`,
          opacity: Math.max(0, 0.55 - press * 0.55),
          transform: `scale(${0.4 + press * 1.6})`,
        }}
      />
    ) : null}
  </div>
);
