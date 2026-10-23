import type { FC } from "react";
import { FONT } from "../fonts";
import { tokens } from "../theme";
import { LogoMark } from "./LogoMark";

const LockGlyph: FC<{ size?: number }> = ({ size = 13 }) => (
  <svg width={size} height={size} viewBox="0 0 16 16" style={{ display: "block" }}>
    <rect x={3} y={7} width={10} height={7} rx={1.6} fill={tokens.ink3} />
    <path
      d="M5.4 7V5.2a2.6 2.6 0 0 1 5.2 0V7"
      fill="none"
      stroke={tokens.ink3}
      strokeWidth={1.5}
      strokeLinecap="round"
    />
  </svg>
);

/**
 * The shipping copy, verbatim: a count and nothing else. No client name, no
 * amount, no invoice number — the notification is readable on a lock screen.
 */
export const NotificationCard: FC<{
  title?: string;
  body?: string;
  opacity?: number;
  offsetY?: number;
  scale?: number;
}> = ({
  title = "Ledgerly",
  body = "3 invoices are past due — Open Ledgerly to see them.",
  opacity = 1,
  offsetY = 0,
  scale = 1,
}) => (
  <div
    style={{
      width: 470,
      padding: "18px 20px",
      background: tokens.paper0,
      border: `1px solid ${tokens.paper3}`,
      borderRadius: tokens.radiusLg,
      boxShadow: tokens.shadow2,
      display: "flex",
      gap: 14,
      fontFamily: FONT,
      opacity,
      transform: `translateY(${offsetY}px) scale(${scale})`,
      transformOrigin: "top right",
    }}
  >
    <div style={{ flex: "0 0 auto", marginTop: 2 }}>
      <LogoMark size={30} paper={tokens.paper0} />
    </div>

    <div style={{ flex: 1, minWidth: 0 }}>
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          gap: 10,
        }}
      >
        <span style={{ fontSize: 16, fontWeight: 700, color: tokens.ink1 }}>{title}</span>
        <span style={{ fontSize: 12, color: tokens.ink3, fontVariantNumeric: "tabular-nums" }}>
          now
        </span>
      </div>
      <div style={{ marginTop: 4, fontSize: 15, lineHeight: 1.38, color: tokens.ink2 }}>{body}</div>
      <div
        style={{
          marginTop: 10,
          display: "flex",
          alignItems: "center",
          gap: 6,
          fontSize: 11.5,
          letterSpacing: "0.06em",
          textTransform: "uppercase",
          color: tokens.ink3,
        }}
      >
        <LockGlyph />
        no names, no amounts
      </div>
    </div>
  </div>
);
