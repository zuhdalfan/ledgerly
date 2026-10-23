import type { CSSProperties, FC, ReactNode } from "react";
import { useCurrentFrame } from "remotion";
import { AppWindow } from "../components/AppWindow";
import { Fit } from "../components/Fit";
import { Money } from "../components/Money";
import { SceneFrame } from "../components/SceneFrame";
import { StatusPill } from "../components/StatusPill";
import { enter, linearProgress, progress } from "../lib/anim";
import { DEMO, money } from "../lib/format";
import { FONT } from "../fonts";
import { SceneProps } from "../timeline";
import { tokens } from "../theme";

const ROWS = [
  { description: "Brand identity system", qty: 1, unit: 3200, amount: 3200, at: 18 },
  { description: "Landing page build", qty: 12, unit: 145, amount: 1740, at: 30 },
  { description: "Copywriting", qty: 4, unit: 220, amount: 880, at: 42 },
];

const SUBTOTAL = ROWS.reduce((sum, row) => sum + row.amount, 0);
const TAX_RATE = 0.09;
const TAX = SUBTOTAL * TAX_RATE;
const TOTAL = SUBTOTAL + TAX;

const HeadCell: FC<{ children: ReactNode; align?: "left" | "right" }> = ({
  children,
  align = "left",
}) => (
  <th
    style={{
      padding: "0 0 8px",
      textAlign: align,
      fontSize: 11,
      fontWeight: 700,
      letterSpacing: "0.08em",
      textTransform: "uppercase",
      color: tokens.ink3,
      borderBottom: `1px solid ${tokens.paper3}`,
    }}
  >
    {children}
  </th>
);

const Cell: FC<{ children: ReactNode; align?: "left" | "right" }> = ({
  children,
  align = "left",
}) => (
  <td
    style={{
      padding: "11px 0",
      textAlign: align,
      fontSize: 14,
      color: tokens.ink1,
      borderBottom: `1px solid ${tokens.paper3}`,
      fontVariantNumeric: "tabular-nums",
    }}
  >
    {children}
  </td>
);

/** Beat 2 — competence. The editor assembles and the money reconciles. */
export const Invoice: FC<SceneProps> = ({ headline }) => {
  const frame = useCurrentFrame();

  return (
    <SceneFrame headline={headline}>
      <Fit designWidth={1200} designHeight={500}>
        <div
          style={{
            transform: `scale(${0.96 + 0.04 * progress(frame, 0, 14)})`,
            opacity: progress(frame, 0, 10),
          }}
        >
          <AppWindow title={`Invoice ${DEMO.invoice}`} active="Invoices" height={500}>
            <div style={{ display: "flex", gap: 20, height: "100%" }}>
              <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column" }}>
                <div style={headerRow}>
                  <div>
                    <div style={{ fontSize: 19, fontWeight: 700, letterSpacing: "-0.01em" }}>
                      {DEMO.client}
                    </div>
                    <div style={{ marginTop: 3, fontSize: 12.5, color: tokens.ink3 }}>
                      {DEMO.invoice} · Issued 23 Oct 2026 · Due 22 Nov 2026
                    </div>
                  </div>
                  <div style={{ display: "flex", alignItems: "center", gap: 10 }}>
                    <StatusPill status="draft" />
                    <TimerChip frame={frame} />
                  </div>
                </div>

                <table style={{ width: "100%", borderCollapse: "collapse", marginTop: 16 }}>
                  <thead>
                    <tr>
                      <HeadCell>Description</HeadCell>
                      <HeadCell align="right">Qty</HeadCell>
                      <HeadCell align="right">Unit price</HeadCell>
                      <HeadCell align="right">Amount</HeadCell>
                    </tr>
                  </thead>
                  <tbody>
                    {ROWS.map((row) => (
                      <tr key={row.description} style={enter(frame, row.at, 10, 12)}>
                        <Cell>{row.description}</Cell>
                        <Cell align="right">{row.qty}</Cell>
                        <Cell align="right">{money(row.unit)}</Cell>
                        <Cell align="right">{money(row.amount)}</Cell>
                      </tr>
                    ))}
                  </tbody>
                </table>

                <div
                  style={{
                    marginTop: "auto",
                    paddingTop: 14,
                    fontSize: 12,
                    color: tokens.ink3,
                    ...enter(frame, 62, 8, 12),
                  }}
                >
                  Payment terms: Net 30 · Bank transfer
                </div>
              </div>

              <div
                style={{
                  width: 284,
                  flex: "0 0 auto",
                  alignSelf: "flex-start",
                  padding: 16,
                  background: tokens.paper1,
                  border: `1px solid ${tokens.paper3}`,
                  borderRadius: tokens.radiusMd,
                  display: "flex",
                  flexDirection: "column",
                  gap: 10,
                  ...enter(frame, 50, 10, 14),
                }}
              >
                <TotalRow label="Subtotal" value={SUBTOTAL} at={56} />
                <TotalRow label="Discount" value={0} at={60} muted />
                <TotalRow label="Tax (9%)" value={TAX} at={64} />
                <div style={{ height: 1, background: tokens.paper3, margin: "4px 0" }} />
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
                  <span style={{ fontSize: 12.5, fontWeight: 700, color: tokens.ink2 }}>Total</span>
                  <Money value={roll(frame, 68, 34, TOTAL)} size={23} />
                </div>
                <div
                  style={{
                    marginTop: 6,
                    display: "flex",
                    justifyContent: "space-between",
                    fontSize: 12,
                    color: tokens.ink3,
                  }}
                >
                  <span>Balance due</span>
                  <span style={{ fontVariantNumeric: "tabular-nums" }}>
                    {money(roll(frame, 74, 30, TOTAL))}
                  </span>
                </div>
              </div>
            </div>

            <CatalogPopover frame={frame} />
          </AppWindow>
        </div>
      </Fit>
    </SceneFrame>
  );
};

const headerRow: CSSProperties = {
  display: "flex",
  alignItems: "flex-start",
  justifyContent: "space-between",
  gap: 16,
};

const roll = (frame: number, start: number, duration: number, target: number): number =>
  target * progress(frame, start, duration);

const TotalRow: FC<{ label: string; value: number; at: number; muted?: boolean }> = ({
  label,
  value,
  at,
  muted,
}) => {
  const frame = useCurrentFrame();
  return (
    <div
      style={{
        display: "flex",
        justifyContent: "space-between",
        alignItems: "baseline",
        fontSize: 12.5,
        color: muted ? tokens.ink3 : tokens.ink2,
        ...enter(frame, at, 6, 10),
      }}
    >
      <span>{label}</span>
      <span style={{ fontVariantNumeric: "tabular-nums", color: tokens.ink1 }}>
        {money(value)}
      </span>
    </div>
  );
};

const TimerChip: FC<{ frame: number }> = ({ frame }) => {
  const seconds = Math.round(42 * linearProgress(frame, 12, 120));
  return (
    <span
      style={{
        display: "inline-flex",
        alignItems: "center",
        gap: 6,
        padding: "5px 10px",
        borderRadius: tokens.radiusFull,
        background: tokens.accentSoft,
        border: `1px solid ${tokens.paper3}`,
        fontSize: 11.5,
        fontWeight: 700,
        color: tokens.ink2,
        fontVariantNumeric: "tabular-nums",
      }}
    >
      <span style={{ width: 5, height: 5, borderRadius: 999, background: tokens.ink2 }} />
      0:{String(seconds).padStart(2, "0")}
    </span>
  );
};

const CATALOG = [
  { name: "Brand identity system", price: 3200 },
  { name: "Landing page build", price: 145 },
  { name: "Copywriting", price: 220 },
];

const CatalogPopover: FC<{ frame: number }> = ({ frame }) => {
  const show = progress(frame, 14, 10) * (1 - progress(frame, 46, 8));
  if (show <= 0.001) return null;

  return (
    <div
      style={{
        position: "absolute",
        left: 18,
        top: 92,
        width: 320,
        padding: 12,
        background: tokens.paper0,
        border: `1px solid ${tokens.paper3}`,
        borderRadius: tokens.radiusMd,
        boxShadow: tokens.shadow2,
        opacity: show,
        transform: `translateY(${(1 - show) * 8}px)`,
        zIndex: 10,
        fontFamily: FONT,
      }}
    >
      <div
        style={{
          fontSize: 10.5,
          fontWeight: 700,
          letterSpacing: "0.08em",
          textTransform: "uppercase",
          color: tokens.ink3,
          padding: "0 4px 8px",
        }}
      >
        Catalog
      </div>
      {CATALOG.map((item, index) => (
        <div
          key={item.name}
          style={{
            display: "flex",
            justifyContent: "space-between",
            gap: 12,
            padding: "7px 8px",
            borderRadius: tokens.radiusSm,
            fontSize: 12.5,
            background: index === 0 ? tokens.paper2 : "transparent",
            color: tokens.ink1,
          }}
        >
          <span>{item.name}</span>
          <span style={{ color: tokens.ink3, fontVariantNumeric: "tabular-nums" }}>
            {money(item.price)}
          </span>
        </div>
      ))}
    </div>
  );
};
