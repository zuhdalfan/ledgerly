import type { FC, ReactNode } from "react";
import { FONT } from "../fonts";
import { tokens } from "../theme";
import { LogoMark } from "./LogoMark";

const NAV = ["Dashboard", "Invoices", "Estimates", "Reports", "Clients", "Settings"];

/** A neutral window: no OS chrome, no traffic lights, no platform tells. */
export const AppWindow: FC<{
  title: string;
  width?: number;
  height?: number;
  active?: string;
  children: ReactNode;
}> = ({ title, width = 1200, height = 620, active = "Invoices", children }) => (
  <div
    style={{
      width,
      height,
      display: "flex",
      background: tokens.paper0,
      border: `1px solid ${tokens.paper3}`,
      borderRadius: tokens.radiusXl,
      boxShadow: tokens.shadow2,
      overflow: "hidden",
      fontFamily: FONT,
    }}
  >
    <div
      style={{
        width: 186,
        flex: "0 0 auto",
        padding: "20px 12px",
        borderRight: `1px solid ${tokens.paper3}`,
        display: "flex",
        flexDirection: "column",
        gap: 26,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 8, padding: "0 8px" }}>
        <LogoMark size={22} paper={tokens.paper0} />
        <span style={{ fontSize: 16, fontWeight: 700, letterSpacing: "-0.01em" }}>Ledgerly</span>
      </div>

      <div style={{ display: "flex", flexDirection: "column", gap: 3 }}>
        {NAV.map((item) => {
          const isActive = item === active;
          return (
            <div
              key={item}
              style={{
                padding: "8px 12px",
                borderRadius: tokens.radiusMd,
                fontSize: 13.5,
                fontWeight: isActive ? 700 : 400,
                color: isActive ? tokens.inkInverse : tokens.ink2,
                background: isActive ? tokens.accentFill : "transparent",
              }}
            >
              {item}
            </div>
          );
        })}
      </div>
    </div>

    <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
      <div
        style={{
          height: 56,
          flex: "0 0 auto",
          padding: "0 22px",
          borderBottom: `1px solid ${tokens.paper3}`,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <span style={{ fontSize: 15, fontWeight: 700, letterSpacing: "-0.01em" }}>{title}</span>
        <span
          style={{
            padding: "7px 14px",
            borderRadius: tokens.radiusMd,
            background: tokens.ink1,
            color: tokens.inkInverse,
            fontSize: 12.5,
            fontWeight: 700,
          }}
        >
          New invoice
        </span>
      </div>

      <div style={{ flex: 1, minHeight: 0, padding: 22, position: "relative" }}>{children}</div>
    </div>
  </div>
);
