import type { FC, ReactNode } from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { enter } from "../lib/anim";
import { FONT } from "../fonts";
import { tokens } from "../theme";
import { OfflineLed } from "./OfflineLed";

/**
 * Landscape: headline top-left, visual fills the rest.
 * Vertical and square: headline on top, visual below, both scaled by <Fit>.
 */
export const SceneFrame: FC<{
  headline: ReactNode;
  sub?: ReactNode;
  delay?: number;
  children: ReactNode;
  ledRing?: number;
}> = ({ headline, sub, delay = 0, children, ledRing = 0 }) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const vertical = height > width;
  const pad = vertical ? 64 : 88;

  return (
    <AbsoluteFill
      style={{
        backgroundColor: tokens.paper1,
        padding: pad,
        display: "flex",
        flexDirection: "column",
        gap: vertical ? 32 : 20,
        overflow: "hidden",
        fontFamily: FONT,
      }}
    >
      <div style={{ flex: "0 0 auto", maxWidth: vertical ? "100%" : 1160 }}>
        <div
          style={{
            margin: 0,
            fontWeight: 700,
            fontSize: vertical ? 62 : 60,
            lineHeight: 1.06,
            letterSpacing: "-0.03em",
            color: tokens.ink1,
            textWrap: "balance",
            ...enter(frame, delay, 10, 14),
          }}
        >
          {headline}
        </div>
        {sub ? (
          <p
            style={{
              margin: "18px 0 0",
              fontSize: vertical ? 27 : 24,
              lineHeight: 1.4,
              color: tokens.ink3,
              ...enter(frame, delay + 8, 8, 14),
            }}
          >
            {sub}
          </p>
        ) : null}
      </div>

      <div
        style={{
          flex: 1,
          minHeight: 0,
          position: "relative",
          display: "flex",
          // Centred on both axes. Portrait currently reuses the landscape stage,
          // so a dedicated 9:16 layout is a follow-up, not a scaling trick.
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        {children}
      </div>

      <OfflineLed ring={ledRing} />
    </AbsoluteFill>
  );
};
