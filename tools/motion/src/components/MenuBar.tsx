import type { FC } from "react";
import { FONT } from "../fonts";
import { tokens } from "../theme";
import { LogoMark } from "./LogoMark";

/**
 * A tray icon, not an app icon: small, square, legible at 16px. The pulse is the
 * only attention-seeking motion in the film.
 */
export const TrayIcon: FC<{ size?: number; pulse?: number; opacity?: number }> = ({
  size = 30,
  pulse = 0,
  opacity = 1,
}) => (
  <div
    style={{
      position: "relative",
      width: size,
      height: size,
      opacity,
      borderRadius: size * 0.26,
      background: tokens.paper0,
      border: `1px solid ${tokens.paper3}`,
      display: "flex",
      alignItems: "center",
      justifyContent: "center",
    }}
  >
    <LogoMark size={size * 0.68} paper={tokens.paper0} />
    {pulse > 0 ? (
      <span
        style={{
          position: "absolute",
          inset: -5,
          borderRadius: size * 0.34,
          border: `1px solid ${tokens.ink3}`,
          opacity: Math.max(0, 0.7 - pulse * 0.7),
          transform: `scale(${1 + pulse * 0.5})`,
        }}
      />
    ) : null}
  </div>
);

/** A neutral desktop bar. No platform is implied, so the film ages well. */
export const MenuBar: FC<{
  trayOpacity?: number;
  trayPulse?: number;
  time?: string;
  timeOpacity?: number;
  width?: number;
}> = ({ trayOpacity = 1, trayPulse = 0, time = "09:41", timeOpacity = 1, width = 1400 }) => (
  <div
    style={{
      width,
      height: 48,
      display: "flex",
      alignItems: "center",
      justifyContent: "flex-end",
      gap: 16,
      padding: "0 18px",
      background: tokens.paper0,
      borderBottom: `1px solid ${tokens.paper3}`,
      fontFamily: FONT,
      boxSizing: "border-box",
    }}
  >
    <TrayIcon opacity={trayOpacity} pulse={trayPulse} />
    <span
      style={{
        fontSize: 14,
        color: tokens.ink2,
        fontVariantNumeric: "tabular-nums",
        opacity: timeOpacity,
        width: 44,
        textAlign: "right",
      }}
    >
      {time}
    </span>
  </div>
);
