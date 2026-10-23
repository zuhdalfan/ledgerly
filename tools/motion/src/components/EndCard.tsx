import type { FC } from "react";
import { useCurrentFrame } from "remotion";
import { enter } from "../lib/anim";
import { CTA } from "../lib/format";
import { FONT } from "../fonts";
import { tokens } from "../theme";
import { LogoMark } from "./LogoMark";
import { OfflineLed } from "./OfflineLed";

export const EndCard: FC<{ tagline: string; ledRing?: number }> = ({ tagline, ledRing = 0 }) => {
  const frame = useCurrentFrame();
  const scale = 0.92 + 0.08 * Math.min(1, frame / 24);

  return (
    <div
      style={{
        position: "absolute",
        inset: 0,
        background: tokens.paper1,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: 28,
        fontFamily: FONT,
      }}
    >
      <div style={{ transform: `scale(${scale})` }}>
        <LogoMark size={128} />
      </div>

      <div
        style={{
          fontSize: 30,
          fontWeight: 700,
          color: tokens.ink3,
          letterSpacing: "0.28em",
          textTransform: "uppercase",
          ...enter(frame, 20, 8, 14),
        }}
      >
        Ledgerly
      </div>

      <div
        style={{
          marginTop: 4,
          maxWidth: 780,
          textAlign: "center",
          fontSize: 76,
          fontWeight: 700,
          lineHeight: 1.04,
          letterSpacing: "-0.035em",
          color: tokens.ink1,
          ...enter(frame, 28, 12, 16),
        }}
      >
        {tagline}
      </div>

      <div
        style={{
          marginTop: 10,
          display: "flex",
          alignItems: "center",
          gap: 14,
          fontSize: 24,
          color: tokens.ink2,
          ...enter(frame, 44, 8, 14),
        }}
      >
        <span>Free</span>
        <span style={{ color: tokens.ink3 }}>·</span>
        <span>MIT</span>
        <span style={{ color: tokens.ink3 }}>·</span>
        <span>Windows</span>
        <span style={{ color: tokens.ink3 }}>·</span>
        <span>macOS</span>
        <span style={{ color: tokens.ink3 }}>·</span>
        <span>Linux</span>
      </div>

      <div
        style={{
          marginTop: 26,
          padding: "10px 20px",
          borderRadius: tokens.radiusFull,
          border: `1px solid ${tokens.paper4}`,
          fontSize: 21,
          color: tokens.ink2,
          letterSpacing: "-0.005em",
          ...enter(frame, 54, 8, 14),
        }}
      >
        {CTA.label}
      </div>

      <OfflineLed ring={ledRing} />
    </div>
  );
};
