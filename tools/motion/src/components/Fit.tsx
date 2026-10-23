import type { FC, ReactNode } from "react";
import { useVideoConfig } from "remotion";

/**
 * Lays a scene out at its design size, then scales it to whatever frame the
 * composition asked for. One scene implementation therefore serves 16:9, 9:16
 * and 1:1 — the same trick the marketing recordings use with vmin.
 */
export const Fit: FC<{
  designWidth: number;
  designHeight: number;
  children: ReactNode;
}> = ({ designWidth, designHeight, children }) => {
  const { width, height } = useVideoConfig();
  const vertical = height > width;
  const pad = vertical ? 64 : 88;
  const headerAllowance = vertical ? 300 : 220;

  const availableWidth = width - pad * 2;
  const availableHeight = height - pad * 2 - headerAllowance;
  const scale = Math.min(availableWidth / designWidth, availableHeight / designHeight, 1.2);

  return (
    <div
      style={{
        width: designWidth,
        height: designHeight,
        transform: `scale(${scale})`,
        transformOrigin: "center",
        flex: "0 0 auto",
      }}
    >
      {children}
    </div>
  );
};
