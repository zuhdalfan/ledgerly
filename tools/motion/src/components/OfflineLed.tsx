import type { FC } from "react";
import { tokens } from "../theme";
import { FONT } from "../fonts";

/**
 * The spine of the film. It sits in every scene, never lights, and never says
 * anything but the truth: the app is not on the network. The ring only appears
 * on the last beat, where it becomes the end card's full stop.
 */
export const OfflineLed: FC<{ ring?: number }> = ({ ring = 0 }) => (
  <div
    style={{
      position: "absolute",
      right: 48,
      bottom: 38,
      display: "flex",
      alignItems: "center",
      gap: 10,
      fontFamily: FONT,
      fontSize: 17,
      fontWeight: 400,
      letterSpacing: "0.1em",
      textTransform: "uppercase",
      color: tokens.ink3,
    }}
  >
    <span style={{ position: "relative", width: 10, height: 10, display: "block" }}>
      <span
        style={{
          position: "absolute",
          inset: 0,
          borderRadius: tokens.radiusFull,
          background: tokens.ink3,
          opacity: 0.6,
        }}
      />
      {ring > 0 ? (
        <span
          style={{
            position: "absolute",
            inset: -7,
            borderRadius: tokens.radiusFull,
            border: `1px solid ${tokens.ink3}`,
            opacity: Math.max(0, 1 - ring),
            transform: `scale(${1 + ring * 1.6})`,
          }}
        />
      ) : null}
    </span>
    network: off
  </div>
);
