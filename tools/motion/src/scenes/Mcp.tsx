import type { FC } from "react";
import { useCurrentFrame } from "remotion";
import { Fit } from "../components/Fit";
import { LogoMark } from "../components/LogoMark";
import { Money } from "../components/Money";
import { SceneFrame } from "../components/SceneFrame";
import { enter, progress } from "../lib/anim";
import { DEMO, money } from "../lib/format";
import { FONT } from "../fonts";
import { SceneProps } from "../timeline";
import { tokens } from "../theme";

const MONO = 'ui-monospace, "Cascadia Code", Consolas, monospace';
const QUERY = "Who owes me the most?";
const ANSWERS = [
  "3 invoices are past due.",
  "Acme Corporation — SGD 8,240.40",
  "Oldest: 58 days late",
];

/** Beat 7 — the surprise. Local data, a stdio pipe, and no socket anywhere. */
export const Mcp: FC<SceneProps> = ({ headline }) => {
  const frame = useCurrentFrame();
  const typed = Math.floor(progress(frame, 16, 40) * QUERY.length);
  const link = progress(frame, 62, 20);

  return (
    <SceneFrame headline={headline} ledRing={progress(frame, 110, 18)}>
      <Fit designWidth={1280} designHeight={520}>
        <div style={{ position: "relative", width: 1280, height: 520, fontFamily: FONT }}>
          {/* Assistant panel */}
          <div
            style={{
              position: "absolute",
              left: 0,
              top: 60,
              width: 590,
              height: 380,
              background: tokens.paper0,
              border: `1px solid ${tokens.paper3}`,
              borderRadius: tokens.radiusLg,
              boxShadow: tokens.shadow2,
              padding: 20,
              display: "flex",
              flexDirection: "column",
              ...enter(frame, 4, 10, 12),
            }}
          >
            <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between" }}>
              <span style={{ fontSize: 12, fontWeight: 700, letterSpacing: "0.06em", textTransform: "uppercase", color: tokens.ink3 }}>
                Assistant
              </span>
              <span
                style={{
                  fontSize: 10.5,
                  padding: "3px 8px",
                  border: `1px solid ${tokens.paper4}`,
                  borderRadius: tokens.radiusFull,
                  color: tokens.ink3,
                }}
              >
                read-only
              </span>
            </div>

            <div
              style={{
                marginTop: 16,
                padding: "12px 14px",
                borderRadius: tokens.radiusMd,
                background: tokens.paper1,
                fontFamily: MONO,
                fontSize: 14,
                color: tokens.ink1,
                minHeight: 46,
              }}
            >
              <span style={{ color: tokens.ink3 }}>&gt; </span>
              {QUERY.slice(0, typed)}
              {typed < QUERY.length ? (
                <span
                  style={{
                    display: "inline-block",
                    width: 7,
                    height: 15,
                    marginLeft: 3,
                    background: tokens.ink1,
                    verticalAlign: "text-bottom",
                  }}
                />
              ) : null}
            </div>

            <div style={{ marginTop: 18, display: "flex", flexDirection: "column", gap: 10 }}>
              {ANSWERS.map((line, index) => (
                <div
                  key={line}
                  style={{
                    fontFamily: MONO,
                    fontSize: 13.5,
                    lineHeight: 1.5,
                    color: index === 1 ? tokens.ink1 : tokens.ink2,
                    fontWeight: index === 1 ? 700 : 400,
                    paddingLeft: index === 1 ? 10 : 0,
                    borderLeft: index === 1 ? `2px solid ${tokens.ink1}` : "none",
                    ...enter(frame, 74 + index * 10, 8, 10),
                  }}
                >
                  {line}
                </div>
              ))}
            </div>

            <div
              style={{
                marginTop: "auto",
                display: "flex",
                gap: 8,
                fontSize: 11,
                color: tokens.ink3,
                ...enter(frame, 108, 6, 10),
              }}
            >
              <span>40 tools</span>
              <span>·</span>
              <span>writes require a second switch</span>
            </div>
          </div>

          {/* The pipe. Dashed on purpose: nothing is listening on a port. */}
          <svg
            width={120}
            height={520}
            style={{ position: "absolute", left: 590, top: 0, pointerEvents: "none" }}
          >
            <line
              x1={2}
              y1={250}
              x2={118}
              y2={250}
              stroke={tokens.ink3}
              strokeWidth={2}
              strokeDasharray="7 6"
              strokeDashoffset={120 * (1 - link)}
              opacity={link}
            />
          </svg>

          <div
            style={{
              position: "absolute",
              left: 592,
              top: 262,
              width: 116,
              textAlign: "center",
              fontSize: 9.5,
              letterSpacing: "0.06em",
              textTransform: "uppercase",
              color: tokens.ink3,
              opacity: link,
              lineHeight: 1.4,
            }}
          >
            stdio
            <br />
            no socket
          </div>

          {/* App panel — the same numbers the assistant just spoke. */}
          <div
            style={{
              position: "absolute",
              left: 710,
              top: 60,
              width: 570,
              height: 380,
              background: tokens.paper0,
              border: `1px solid ${tokens.paper3}`,
              borderRadius: tokens.radiusLg,
              boxShadow: tokens.shadow2,
              overflow: "hidden",
              ...enter(frame, 10, 10, 12),
            }}
          >
            <div
              style={{
                height: 46,
                padding: "0 18px",
                borderBottom: `1px solid ${tokens.paper3}`,
                display: "flex",
                alignItems: "center",
                gap: 8,
              }}
            >
              <LogoMark size={18} paper={tokens.paper0} />
              <span style={{ fontSize: 13, fontWeight: 700 }}>Ledgerly</span>
              <span style={{ marginLeft: "auto", fontSize: 11.5, color: tokens.ink3 }}>
                same database
              </span>
            </div>

            <div style={{ padding: 22, display: "flex", flexDirection: "column", gap: 14 }}>
              <div
                style={{
                  padding: 18,
                  background: tokens.paper1,
                  borderRadius: tokens.radiusMd,
                  border: `1px solid ${tokens.paper3}`,
                }}
              >
                <div style={{ fontSize: 11, letterSpacing: "0.08em", textTransform: "uppercase", color: tokens.ink3 }}>
                  Outstanding
                </div>
                <Money value={21847.8} size={34} />
              </div>

              <div
                style={{
                  display: "flex",
                  justifyContent: "space-between",
                  alignItems: "center",
                  padding: "12px 14px",
                  borderRadius: tokens.radiusMd,
                  background: tokens.paper1,
                  border: `1px solid ${tokens.paper3}`,
                  fontSize: 12.5,
                }}
              >
                <span style={{ color: tokens.ink2 }}>{DEMO.client}</span>
                <span style={{ color: tokens.danger, fontWeight: 700 }}>{money(8240.4)}</span>
              </div>

              <div style={{ fontSize: 11.5, color: tokens.ink3, marginTop: 2 }}>
                The tray, the app and the assistant share one SQLite file.
              </div>
            </div>
          </div>
        </div>
      </Fit>
    </SceneFrame>
  );
};
