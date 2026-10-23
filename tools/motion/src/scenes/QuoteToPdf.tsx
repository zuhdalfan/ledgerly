import type { CSSProperties, FC } from "react";
import { useCurrentFrame } from "remotion";
import { Fit } from "../components/Fit";
import { LogoMark } from "../components/LogoMark";
import { Money } from "../components/Money";
import { SceneFrame } from "../components/SceneFrame";
import { StatusPill } from "../components/StatusPill";
import { enter, progress } from "../lib/anim";
import { DEMO, money } from "../lib/format";
import { FONT } from "../fonts";
import { SceneProps } from "../timeline";
import { tokens } from "../theme";

const TOTAL = 4200;

/** Beat 6 — one click carries the line items across, then the PDF leaves. */
export const QuoteToPdf: FC<SceneProps> = ({ headline }) => {
  const frame = useCurrentFrame();
  const link = progress(frame, 34, 18);
  const page = progress(frame, 64, 22);
  const cycle = (start: number) => progress(frame, start, 8) * (1 - progress(frame, start + 12, 8));

  return (
    <SceneFrame headline={headline}>
      <Fit designWidth={1200} designHeight={540}>
        <div style={{ position: "relative", width: 1200, height: 540 }}>
          <div style={{ position: "absolute", left: 0, top: 130 }}>
            <DocumentCard
              kind="Estimate"
              number={DEMO.estimate}
              status="accepted"
              total={TOTAL}
              style={enter(frame, 4, 10, 12)}
            />
          </div>

          <div style={{ position: "absolute", left: 410, top: 130 }}>
            <DocumentCard
              kind="Invoice"
              number={DEMO.invoice}
              status="draft"
              total={TOTAL}
              style={enter(frame, 44, 10, 12)}
            />
          </div>

          <svg
            width={1200}
            height={540}
            style={{ position: "absolute", inset: 0, pointerEvents: "none" }}
          >
            <path
              d="M 348 248 C 372 248, 386 248, 408 248"
              fill="none"
              stroke={tokens.ink3}
              strokeWidth={2}
              strokeDasharray={80}
              strokeDashoffset={80 * (1 - link)}
            />
            <circle cx={378} cy={248} r={4} fill={tokens.ink1} opacity={link} />
          </svg>

          <div
            style={{
              position: "absolute",
              left: 350,
              top: 262,
              width: 58,
              textAlign: "center",
              fontSize: 9.5,
              letterSpacing: "0.04em",
              textTransform: "uppercase",
              color: tokens.ink3,
              opacity: link,
            }}
          >
            1 click
          </div>

          <div
            style={{
              position: "absolute",
              left: 820,
              top: 30 + 40 * (1 - page),
              opacity: page,
              transform: `rotate(${-1.2 + 0.4 * page}deg)`,
            }}
          >
            <div style={{ position: "relative", width: 340, height: 460 }}>
              {[0, 1, 2].map((index) => {
                const starts = [64, 96, 110];
                const visibility = index === 0 ? page : cycle(starts[index] + 4);
                return (
                  <div key={index} style={{ position: "absolute", inset: 0, opacity: visibility }}>
                    <PdfPage variant={index} />
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </Fit>
    </SceneFrame>
  );
};

const DocumentCard: FC<{
  kind: string;
  number: string;
  status: "accepted" | "draft";
  total: number;
  style: CSSProperties;
}> = ({ kind, number, status, total, style }) => (
  <div
    style={{
      width: 348,
      padding: 20,
      background: tokens.paper0,
      border: `1px solid ${tokens.paper3}`,
      borderRadius: tokens.radiusLg,
      boxShadow: tokens.shadow2,
      fontFamily: FONT,
      ...style,
    }}
  >
    <div
      style={{
        fontSize: 10.5,
        fontWeight: 700,
        letterSpacing: "0.1em",
        textTransform: "uppercase",
        color: tokens.ink3,
      }}
    >
      {kind}
    </div>
    <div style={{ marginTop: 6, display: "flex", justifyContent: "space-between", alignItems: "center" }}>
      <span style={{ fontSize: 18, fontWeight: 700 }}>{number}</span>
      <StatusPill status={status} />
    </div>
    <div style={{ marginTop: 14, fontSize: 13, color: tokens.ink2 }}>{DEMO.client}</div>
    <div style={{ marginTop: 18, display: "flex", justifyContent: "space-between", alignItems: "baseline" }}>
      <span style={{ fontSize: 11.5, color: tokens.ink3 }}>Total</span>
      <Money value={total} size={22} />
    </div>
  </div>
);

const ACCENT = [tokens.ink1, tokens.accentState, tokens.ink3];

const PdfPage: FC<{ variant: number }> = ({ variant }) => (
  <div
    style={{
      width: 340,
      height: 460,
      padding: 22,
      background: "#fffefb",
      border: `1px solid ${tokens.paper3}`,
      borderRadius: 4,
      boxShadow: tokens.shadow2,
      fontFamily: FONT,
      display: "flex",
      flexDirection: "column",
    }}
  >
    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "flex-start" }}>
      <LogoMark size={26} paper="#fffefb" color={ACCENT[variant]} />
      <span
        style={{
          fontSize: 9,
          letterSpacing: "0.12em",
          textTransform: "uppercase",
          color: tokens.ink3,
        }}
      >
        {["Classic", "Modern", "Minimal"][variant]}
      </span>
    </div>

    <div style={{ marginTop: 14, fontSize: 15, fontWeight: 700, color: tokens.ink1 }}>
      Invoice
    </div>
    <div style={{ fontSize: 9, color: tokens.ink3, marginTop: 2 }}>{DEMO.invoice}</div>

    <div style={{ marginTop: 16, height: 3, width: 60, background: ACCENT[variant] }} />

    <div style={{ marginTop: 16, display: "flex", flexDirection: "column", gap: 7 }}>
      {[90, 74, 84, 60].map((width, index) => (
        <div key={index} style={{ display: "flex", gap: 6 }}>
          <div style={{ width: width, height: 5, background: tokens.paper4, borderRadius: 2 }} />
          <div style={{ flex: 1, height: 5, background: tokens.paper3, borderRadius: 2 }} />
          <div style={{ width: 28, height: 5, background: tokens.paper4, borderRadius: 2 }} />
        </div>
      ))}
    </div>

    <div style={{ marginTop: "auto" }}>
      <div style={{ height: 1, background: tokens.paper3, marginBottom: 10 }} />
      <div style={{ display: "flex", justifyContent: "space-between", fontSize: 9, color: tokens.ink3 }}>
        <span>Total</span>
        <span style={{ fontWeight: 700, color: tokens.ink1 }}>{money(TOTAL)}</span>
      </div>
      <div style={{ marginTop: 10, fontSize: 7.5, color: tokens.ink3 }}>Page 1 of 1</div>
    </div>
  </div>
);
