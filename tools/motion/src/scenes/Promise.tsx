import type { FC } from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { OfflineLed } from "../components/OfflineLed";
import { LogoMark } from "../components/LogoMark";
import { SceneProps } from "../timeline";
import { enter, progress } from "../lib/anim";
import { FONT } from "../fonts";
import { tokens } from "../theme";

/** Beat 1 — the promise. Nothing but the mark, the sentence and the dead LED. */
export const Promise: FC<SceneProps> = ({ headline }) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const vertical = height > width;
  const draw = progress(frame, 4, 26);

  return (
    <AbsoluteFill
      style={{
        backgroundColor: tokens.paper1,
        display: "flex",
        flexDirection: "column",
        alignItems: "center",
        justifyContent: "center",
        gap: vertical ? 46 : 40,
        padding: vertical ? 64 : 88,
        fontFamily: FONT,
      }}
    >
      <LogoMark size={vertical ? 132 : 148} draw={draw} />

      <div
        style={{
          maxWidth: 1320,
          textAlign: "center",
          fontSize: vertical ? 66 : 82,
          fontWeight: 700,
          lineHeight: 1.04,
          letterSpacing: "-0.035em",
          color: tokens.ink1,
          textWrap: "balance",
          ...enter(frame, 14, 12, 16),
        }}
      >
        {headline}
      </div>

      <div
        style={{
          fontSize: vertical ? 27 : 26,
          color: tokens.ink3,
          ...enter(frame, 34, 8, 14),
        }}
      >
        Fully local invoicing for freelancers and small studios.
      </div>

      <OfflineLed />
    </AbsoluteFill>
  );
};
