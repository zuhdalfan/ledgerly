import type { FC } from "react";
import { useCurrentFrame } from "remotion";
import { AppWindow } from "../components/AppWindow";
import { Fit } from "../components/Fit";
import { Money } from "../components/Money";
import { SceneFrame } from "../components/SceneFrame";
import { StatusPill, type Status } from "../components/StatusPill";
import { enter, pop, progress } from "../lib/anim";
import { DEMO, money } from "../lib/format";
import { SceneProps } from "../timeline";
import { tokens } from "../theme";

const TOTAL = 6343.8;

/**
 * Beat 3 — it just knows. One payment, one roll to zero, and a status that
 * changes itself. The pill passes through `Partially paid` so the derivation is
 * visible rather than asserted.
 */
export const Paid: FC<SceneProps> = ({ headline }) => {
  const frame = useCurrentFrame();

  // The balance drains only after Save is pressed, and the pill turns Paid at
  // the exact frame the balance reaches zero — never before.
  const balance = TOTAL * (1 - progress(frame, 36, 44));
  const settled = progress(frame, 80, 10);
  const checkScale = pop(frame, 82, 14);
  const phase: Status = frame < 38 ? "sent" : frame < 80 ? "partially_paid" : "paid";

  return (
    <SceneFrame headline={headline}>
      <Fit designWidth={1200} designHeight={560}>
        <div
          style={{
            transform: `scale(${0.97 + 0.03 * progress(frame, 0, 12)})`,
          }}
        >
          <AppWindow title={`Invoice ${DEMO.invoice}`} active="Invoices" height={560}>
            <div style={{ display: "flex", gap: 20, height: "100%" }}>
              <div style={{ flex: 1, minWidth: 0, display: "flex", flexDirection: "column", gap: 14 }}>
                <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
                  <div>
                    <div style={{ fontSize: 19, fontWeight: 700, letterSpacing: "-0.01em" }}>
                      {DEMO.client}
                    </div>
                    <div style={{ marginTop: 3, fontSize: 12.5, color: tokens.ink3 }}>
                      {DEMO.invoice} · Due 22 Nov 2026
                    </div>
                  </div>
                  <StatusPill status={phase} size={13} />
                </div>

                <div
                  style={{
                    marginTop: 4,
                    padding: 18,
                    background: tokens.paper1,
                    border: `1px solid ${tokens.paper3}`,
                    borderRadius: tokens.radiusMd,
                  }}
                >
                  <div
                    style={{
                      fontSize: 11.5,
                      letterSpacing: "0.08em",
                      textTransform: "uppercase",
                      color: tokens.ink3,
                    }}
                  >
                    Balance due
                  </div>
                  <Money value={balance} size={38} />
                </div>

                <div style={{ display: "flex", gap: 8 }}>
                  {(["Sent", "Partially paid", "Paid"] as const).map((label, index) => {
                    const reached =
                      (index === 0 && frame >= 0) ||
                      (index === 1 && frame >= 38) ||
                      (index === 2 && frame >= 80);
                    const current =
                      phase === "sent" ? 0 : phase === "partially_paid" ? 1 : 2;
                    const isCurrent = index === current;
                    return (
                      <div
                        key={label}
                        style={{
                          flex: 1,
                          padding: "9px 10px",
                          borderRadius: tokens.radiusMd,
                          background: isCurrent ? tokens.paper0 : tokens.paper1,
                          border: `1px solid ${isCurrent ? tokens.paper4 : tokens.paper3}`,
                          fontSize: 11.5,
                          textAlign: "center",
                          color: isCurrent ? tokens.ink1 : reached ? tokens.ink2 : tokens.ink3,
                          fontWeight: isCurrent ? 700 : 400,
                        }}
                      >
                        {label}
                      </div>
                    );
                  })}
                </div>

                <div style={{ marginTop: "auto", fontSize: 12, color: tokens.ink3 }}>
                  Payments are recorded locally · no card processor involved
                </div>
              </div>

              <div
                style={{
                  width: 330,
                  flex: "0 0 auto",
                  padding: 18,
                  background: tokens.paper1,
                  border: `1px solid ${tokens.paper3}`,
                  borderRadius: tokens.radiusMd,
                  display: "flex",
                  flexDirection: "column",
                  gap: 12,
                  ...enter(frame, 8, 10, 12),
                }}
              >
                <div style={{ fontSize: 16, fontWeight: 700 }}>Record payment</div>
                <div style={{ fontSize: 11.5, color: tokens.ink3 }}>
                  Bank transfer · 23 Oct 2026
                </div>

                <div
                  style={{
                    marginTop: 6,
                    paddingBottom: 12,
                    borderBottom: `1px solid ${tokens.paper3}`,
                    display: "flex",
                    alignItems: "baseline",
                    justifyContent: "space-between",
                    gap: 10,
                  }}
                >
                  <span style={{ fontSize: 12, color: tokens.ink2 }}>Amount</span>
                  <Money value={TOTAL * progress(frame, 14, 12)} size={25} />
                </div>

                <div
                  style={{
                    padding: "9px 16px",
                    borderRadius: tokens.radiusMd,
                    background: tokens.ink1,
                    color: tokens.inkInverse,
                    fontSize: 12.5,
                    fontWeight: 700,
                    textAlign: "center",
                    transform: `scale(${1 - 0.03 * Math.sin(Math.PI * progress(frame, 34, 8))})`,
                  }}
                >
                  Save payment
                </div>

                <div style={{ fontSize: 11.5, color: tokens.ink3 }}>
                  {frame < 34 ? "Settles the balance in full" : "Balance settled"}
                </div>

                <div
                  style={{
                    marginTop: "auto",
                    display: "flex",
                    alignItems: "center",
                    gap: 10,
                    opacity: settled,
                  }}
                >
                  <span
                    style={{
                      display: "flex",
                      alignItems: "center",
                      justifyContent: "center",
                      width: 40,
                      height: 40,
                      borderRadius: 999,
                      border: `2px solid ${tokens.ok}`,
                      transform: `scale(${checkScale})`,
                    }}
                  >
                    <svg width={20} height={20} viewBox="0 0 24 24">
                      <path
                        d="M5 12.5 L10 17.5 L19 6.5"
                        fill="none"
                        stroke={tokens.ok}
                        strokeWidth={2.8}
                        strokeLinecap="round"
                        strokeLinejoin="round"
                      />
                    </svg>
                  </span>
                  <div>
                    <div style={{ fontSize: 15, fontWeight: 700, color: tokens.ok }}>Paid</div>
                    <div style={{ fontSize: 11, color: tokens.ink3 }}>{money(TOTAL)} received</div>
                  </div>
                </div>
              </div>
            </div>
          </AppWindow>
        </div>
      </Fit>
    </SceneFrame>
  );
};
