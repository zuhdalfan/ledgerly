import type { CSSProperties, FC } from "react";
import { useCurrentFrame } from "remotion";
import { AppWindow } from "../components/AppWindow";
import { Bar } from "../components/Bar";
import { Fit } from "../components/Fit";
import { SceneFrame } from "../components/SceneFrame";
import { enter, progress } from "../lib/anim";
import { money } from "../lib/format";
import { FONT } from "../fonts";
import { SceneProps } from "../timeline";
import { tokens } from "../theme";

const CARDS: CSSProperties = {
  background: tokens.paper1,
  border: `1px solid ${tokens.paper3}`,
  borderRadius: tokens.radiusMd,
  padding: 16,
};

/** Beat 4 — the work happens in the app, not in a second application. */
export const Chase: FC<SceneProps> = ({ headline }) => {
  const frame = useCurrentFrame();
  const copied = progress(frame, 50, 8);
  const chased = progress(frame, 74, 10);
  const barGrow = progress(frame, 16, 44);

  return (
    <SceneFrame headline={headline}>
      <Fit designWidth={1200} designHeight={560}>
        <div style={{ transform: `scale(${0.97 + 0.03 * progress(frame, 0, 12)})` }}>
          <AppWindow title="Dashboard" active="Dashboard" height={560}>
            <div style={{ display: "flex", flexDirection: "column", gap: 14 }}>
              <div style={{ ...CARDS, ...enter(frame, 6, 10, 12) }}>
                <div
                  style={{
                    display: "flex",
                    justifyContent: "space-between",
                    alignItems: "baseline",
                    marginBottom: 10,
                  }}
                >
                  <span style={{ fontSize: 13.5, fontWeight: 700 }}>Needs chasing</span>
                  <span style={{ fontSize: 11.5, color: tokens.ink3 }}>View all overdue</span>
                </div>

                <div
                  style={{
                    display: "grid",
                    gridTemplateColumns: "78px 1fr 86px 66px 108px 70px 190px",
                    alignItems: "center",
                    gap: 8,
                    padding: "8px 0",
                    fontSize: 11,
                    letterSpacing: "0.06em",
                    textTransform: "uppercase",
                    color: tokens.ink3,
                    borderBottom: `1px solid ${tokens.paper3}`,
                  }}
                >
                  <span>Number</span>
                  <span>Client</span>
                  <span>Due</span>
                  <span>Late by</span>
                  <span style={{ textAlign: "right" }}>Balance</span>
                  <span>Chased</span>
                  <span />
                </div>

                <div
                  style={{
                    display: "grid",
                    gridTemplateColumns: "78px 1fr 86px 66px 108px 70px 190px",
                    alignItems: "center",
                    gap: 8,
                    padding: "9px 0",
                    fontSize: 12.5,
                    color: tokens.ink1,
                    fontVariantNumeric: "tabular-nums",
                  }}
                >
                  <span style={{ color: tokens.ink2 }}>INV-0041</span>
                  <span>Acme Corporation</span>
                  <span>14 Aug 2026</span>
                  <span style={{ color: tokens.danger, fontWeight: 700 }}>58 days</span>
                  <span style={{ textAlign: "right", fontWeight: 700, color: tokens.danger }}>
                    {money(8240.4)}
                  </span>
                  <span style={{ color: chased > 0.5 ? tokens.ok : tokens.ink3 }}>
                    {chased > 0.5 ? "Chased" : "Never"}
                  </span>

                  <span style={{ display: "flex", gap: 6, justifyContent: "flex-end" }}>
                    <span
                      style={{
                        display: "inline-flex",
                        alignItems: "center",
                        gap: 6,
                        padding: "5px 10px",
                        borderRadius: tokens.radiusSm,
                        border: `1px solid ${copied > 0.5 ? tokens.ok : tokens.paper4}`,
                        color: copied > 0.5 ? tokens.ok : tokens.ink2,
                        background: tokens.paper0,
                        fontSize: 11.5,
                        fontWeight: 700,
                      }}
                    >
                      {copied > 0.5 ? (
                        <svg width={11} height={11} viewBox="0 0 24 24">
                          <path
                            d="M5 12.5 L10 17.5 L19 6.5"
                            fill="none"
                            stroke={tokens.ok}
                            strokeWidth={3}
                            strokeLinecap="round"
                          />
                        </svg>
                      ) : null}
                      {copied > 0.5 ? "Copied" : "Copy"}
                    </span>
                    <span
                      style={{
                        padding: "5px 10px",
                        borderRadius: tokens.radiusSm,
                        border: `1px solid ${tokens.paper4}`,
                        background: tokens.paper0,
                        fontSize: 11.5,
                        fontWeight: 700,
                        color: tokens.ink2,
                      }}
                    >
                      Mark chased
                    </span>
                  </span>
                </div>
              </div>

              <div style={{ ...CARDS, ...enter(frame, 22, 10, 12), flex: 1 }}>
                <div style={{ fontSize: 13.5, fontWeight: 700, marginBottom: 4 }}>
                  Aged receivables
                </div>
                <div style={{ fontSize: 11.5, color: tokens.ink3, marginBottom: 14 }}>
                  Follow a bucket to the invoices in it.
                </div>

                <div style={{ display: "flex", flexDirection: "column", gap: 12 }}>
                  <BarRow
                    label="Not yet due"
                    count={2}
                    value={5798.8}
                    max={16049}
                    progress={barGrow}
                    color={tokens.ink1}
                  />
                  <BarRow
                    label="31–60 days"
                    count={2}
                    value={16049}
                    max={16049}
                    progress={barGrow}
                    color={tokens.danger}
                  />
                </div>
              </div>
            </div>
          </AppWindow>
        </div>
      </Fit>
    </SceneFrame>
  );
};

const BarRow: FC<{
  label: string;
  count: number;
  value: number;
  max: number;
  progress: number;
  color: string;
}> = ({ label, count, value, max, progress, color }) => (
  <div style={{ display: "grid", gridTemplateColumns: "120px 40px 1fr 120px", alignItems: "center", gap: 12 }}>
    <span style={{ fontFamily: FONT, fontSize: 12.5, color: tokens.ink2 }}>{label}</span>
    <span style={{ fontSize: 12, color: tokens.ink3 }}>{count}</span>
    <Bar value={value} max={max} progress={progress} color={color} width={560} height={11} />
    <span
      style={{
        textAlign: "right",
        fontSize: 12.5,
        color: tokens.ink1,
        fontVariantNumeric: "tabular-nums",
      }}
    >
      {money(value)}
    </span>
  </div>
);
