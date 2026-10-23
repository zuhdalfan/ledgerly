import type { FC } from "react";
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import { AppWindow } from "../components/AppWindow";
import { Clock } from "../components/Clock";
import { Cursor } from "../components/Cursor";
import { Fit } from "../components/Fit";
import { MenuBar } from "../components/MenuBar";
import { Money } from "../components/Money";
import { NotificationCard } from "../components/NotificationCard";
import { OfflineLed } from "../components/OfflineLed";
import { enter, progress } from "../lib/anim";
import { DEMO, money } from "../lib/format";
import { FONT } from "../fonts";
import { SceneProps } from "../timeline";
import { tokens } from "../theme";

/**
 * Beat 5 — the app outlives its window.
 *
 * 5a  0–48   the window folds into the tray; the menu is shown, not explained
 * 5b 48–90   empty desktop; the clock flips overnight; almost nothing moves
 * 5c 90–150  one notification, carrying a count and nothing else
 * 5d 150–192 click, and the window is back on the overdue figure
 *
 * Tray mode is opt-in, so the caption says so. The notification is shown only
 * while the tray icon is present — they are one feature, not two.
 */
export const TrayReminder: FC<SceneProps> = ({ headline }) => {
  const frame = useCurrentFrame();
  const { width, height } = useVideoConfig();
  const vertical = height > width;

  const close = progress(frame, 10, 34);
  const open = progress(frame, 150, 30);
  const shrink = close * (1 - open);

  const trayOpacity = Math.max(close, open);
  const pulse = progress(frame, 44, 16) * (1 - progress(frame, 62, 12));

  const clockOpacity = progress(frame, 48, 10) * (1 - progress(frame, 88, 10));
  const flip = progress(frame, 56, 12);

  const appear = progress(frame, 92, 12);
  const dismiss = progress(frame, 152, 10);
  const notificationOpacity = appear * (1 - dismiss);

  const menuOpacity = progress(frame, 28, 6) * (1 - progress(frame, 44, 6));

  const cursorMove = progress(frame, 122, 16);
  const cursorOpacity = progress(frame, 120, 8) * (1 - progress(frame, 144, 8));
  const press = progress(frame, 136, 6);

  return (
    <AbsoluteFill
      style={{
        backgroundColor: tokens.paper1,
        padding: vertical ? 56 : 80,
        display: "flex",
        flexDirection: "column",
        fontFamily: FONT,
      }}
    >
      <div style={{ flex: "0 0 auto", maxWidth: vertical ? "100%" : 1120 }}>
        <div
          style={{
            fontSize: vertical ? 56 : 58,
            fontWeight: 700,
            lineHeight: 1.06,
            letterSpacing: "-0.03em",
            color: tokens.ink1,
            ...enter(frame, 0, 10, 12),
          }}
        >
          {headline}
        </div>
        <CaptionSwap frame={frame} />
      </div>

      <div
        style={{
          flex: 1,
          minHeight: 0,
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
          paddingTop: vertical ? 8 : 12,
        }}
      >
        <Fit designWidth={1300} designHeight={700}>
          <div
            style={{
              position: "relative",
              width: 1300,
              height: 700,
              background: tokens.paper0,
              border: `1px solid ${tokens.paper3}`,
              borderRadius: 22,
              overflow: "hidden",
            }}
          >
            <MenuBar
              width={1300}
              trayOpacity={trayOpacity}
              trayPulse={pulse}
              time={frame < 60 ? "18:40" : "09:41"}
            />

            {/* 5a — the window, folding into the tray icon. */}
            <div
              style={{
                position: "absolute",
                left: 50,
                top: 78,
                width: 1200,
                height: 620,
                transformOrigin: "top right",
                transform: `translate(${-95 * shrink}px, ${-100 * shrink}px) scale(${1 - 0.95 * shrink})`,
                opacity: 1 - 0.9 * shrink,
              }}
            >
              <AppWindow title="Dashboard" active="Dashboard">
                <div style={{ display: "flex", flexDirection: "column", gap: 16 }}>
                  <div
                    style={{
                      padding: 22,
                      background: tokens.paper1,
                      border: `1px solid ${tokens.paper3}`,
                      borderRadius: tokens.radiusLg,
                      display: "flex",
                      justifyContent: "space-between",
                      alignItems: "flex-end",
                    }}
                  >
                    <div>
                      <div style={{ fontSize: 11.5, letterSpacing: "0.08em", textTransform: "uppercase", color: tokens.ink3 }}>
                        Outstanding
                      </div>
                      <Money value={21847.8} size={42} />
                    </div>
                    <div style={{ textAlign: "right" }}>
                      <div style={{ fontSize: 11.5, color: tokens.ink3 }}>Overdue</div>
                      <span style={{ fontSize: 22, fontWeight: 700, color: tokens.danger, fontVariantNumeric: "tabular-nums" }}>
                        {money(8240.4)}
                      </span>
                    </div>
                  </div>

                  <div
                    style={{
                      padding: 18,
                      background: tokens.paper1,
                      border: `1px solid ${tokens.paper3}`,
                      borderRadius: tokens.radiusMd,
                      display: "flex",
                      justifyContent: "space-between",
                      fontSize: 13,
                      color: tokens.ink2,
                    }}
                  >
                    <span>{DEMO.client}</span>
                    <span style={{ color: tokens.danger, fontWeight: 700 }}>58 days late</span>
                    <span style={{ fontWeight: 700 }}>{money(8240.4)}</span>
                  </div>
                </div>
              </AppWindow>
            </div>

            {/* 5a — the tray menu, shown once so the affordance exists. */}
            <div
              style={{
                position: "absolute",
                right: 14,
                top: 56,
                width: 190,
                padding: 6,
                background: tokens.paper0,
                border: `1px solid ${tokens.paper3}`,
                borderRadius: tokens.radiusMd,
                boxShadow: tokens.shadow2,
                opacity: menuOpacity,
                transform: `translateY(${(1 - menuOpacity) * -6}px)`,
              }}
            >
              {["Open Ledgerly", "Lock now", "Quit Ledgerly"].map((item) => (
                <div
                  key={item}
                  style={{
                    padding: "7px 10px",
                    borderRadius: tokens.radiusSm,
                    fontSize: 12.5,
                    color: tokens.ink2,
                  }}
                >
                  {item}
                </div>
              ))}
            </div>

            {/* 5b — the overnight gap. The clock is the only thing that moves. */}
            <div
              style={{
                position: "absolute",
                inset: "48px 0 0 0",
                display: "flex",
                alignItems: "center",
                justifyContent: "center",
                opacity: clockOpacity,
              }}
            >
              <Clock from="18:40" to="09:41" flip={flip} size={86} />
            </div>

            {/* 5c — one notification, no data. */}
            <div
              style={{
                position: "absolute",
                right: 24,
                top: 64,
                opacity: notificationOpacity,
                transform: `translateY(${(1 - appear) * -30}px)`,
              }}
            >
              <NotificationCard />
            </div>

            {/* 5d — the click that brings it back. */}
            <Cursor
              x={700 + 300 * cursorMove}
              y={520 - 400 * cursorMove}
              opacity={cursorOpacity}
              press={press}
            />
          </div>
        </Fit>
      </div>

      <OfflineLed />
    </AbsoluteFill>
  );
};

/** Captions change with the sub-beat; the headline above stays put. */
const CaptionSwap: FC<{ frame: number }> = ({ frame }) => {
  const captions = [
    { text: "Opt in to keep it running.", from: 4, to: 46 },
    { text: "Once a day. Not once a minute.", from: 50, to: 90 },
    { text: "No names. No amounts.", from: 94, to: 150 },
    { text: "Click it. You're back.", from: 156, to: 192 },
  ];

  return (
    <div style={{ position: "relative", height: 40, marginTop: 14 }}>
      {captions.map((caption) => {
        const inOpacity = progress(frame, caption.from, 8);
        const outOpacity = 1 - progress(frame, caption.to - 8, 8);
        return (
          <div
            key={caption.text}
            style={{
              position: "absolute",
              inset: 0,
              fontSize: 25,
              color: tokens.ink3,
              opacity: inOpacity * outOpacity,
            }}
          >
            {caption.text}
          </div>
        );
      })}
    </div>
  );
};
