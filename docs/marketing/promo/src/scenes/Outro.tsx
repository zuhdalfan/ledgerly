// Shot 11 — sign-off (outro-group-photo-launch, re-skinned).
//
// Every feature the film showed sends one representative element; they fly in
// from four sides and settle in a ring, then the wordmark stamps in the middle
// and they step back. The energy peak, and the last wordmark hold (rule R1).
import React from 'react';
import { AbsoluteFill, Img, staticFile, useCurrentFrame } from 'remotion';
import { C, FONT, M } from '../tokens';
import { cut, tex } from '../layout';
import { LedgerlyMark, clamp01 } from '../lib/ui';

export const OUTRO_DURATION = 168;

// One representative per feature shown earlier in the film.
const PIECES: { file: string; w: number; h: number }[] = [
  { file: 'dashboard-card1.png', w: 1628, h: 193.5 },
  { file: 'invoice-card3.png', w: 1628, h: 359.5 },
  { file: 'reports-card3.png', w: 1628, h: 434 },
  { file: 'ai-card2.png', w: 1628, h: 494 },
  { file: 'estimate-card3.png', w: 1628, h: 267.5 },
  { file: 'catalog-card2.png', w: 1628, h: 285.5 },
  { file: 'client-card1.png', w: 1628, h: 227.5 },
  { file: 'invoices-card1.png', w: 1628, h: 411.5 },
];

const ANGLES = [-90, -45, 0, 45, 90, 135, 180, 225];
const RX = 740;
const RY = 330;
const CX = 960;
const CY = 470;
const PIECE_W = 352;

const hash = (n: number) => {
  const x = Math.sin(n * 91.345 + 47.853) * 43758.5453;
  return x - Math.floor(x);
};

export const Outro: React.FC = () => {
  const f = useCurrentFrame();
  const crane = M.easeTrust(clamp01(f / 40));
  const stepBack = M.easeTrust(clamp01((f - 52) / 14));

  const markIn = M.easeLand(clamp01((f - 46) / 14));
  const rule = M.easeOut(clamp01((f - 74) / 18));
  const subIn = M.easeOut(clamp01((f - 88) / 16));
  const stage = interpolate01(clamp01((f - 42) / 20));

  return (
    <AbsoluteFill style={{ backgroundColor: C.paper1, overflow: 'hidden' }}>
      {/* The product recedes: a blurred page becomes depth-of-field. */}
      <AbsoluteFill style={{ filter: 'blur(16px)', opacity: 0.4, transform: 'scale(1.06)' }}>
        <Img src={staticFile(tex('dashboard'))} style={{ position: 'absolute', left: 0, top: -220, width: 1920, height: 1567 }} />
      </AbsoluteFill>
      <AbsoluteFill style={{ background: `linear-gradient(180deg, ${'oklch(0.995 0.002 80 / 0.82)'} 0%, ${'oklch(0.98 0.006 80 / 0.94)'} 60%, ${'oklch(0.965 0.008 80 / 0.98)'} 100%)` }} />

      <AbsoluteFill style={{ perspective: 1400, perspectiveOrigin: '960px 540px' }}>
        <AbsoluteFill
          style={{
            transformStyle: 'preserve-3d',
            transform: `rotateX(${(4 * (1 - crane)).toFixed(2)}deg) scale(${(1.06 - 0.06 * crane).toFixed(4)})`,
          }}
        >
          {PIECES.map((p, i) => {
            const a = (ANGLES[i] * Math.PI) / 180;
            const x = CX + RX * Math.cos(a);
            const y = CY + RY * Math.sin(a);
            const w = PIECE_W;
            const h = (p.h / p.w) * PIECE_W;
            const start = 4 + i * 3;
            const t = M.easeLand(clamp01((f - start) / 13));
            const q = 1 - t;
            const dirX = Math.cos(a) > 0.15 ? 1 : Math.cos(a) < -0.15 ? -1 : 0;
            const dirY = Math.sin(a) > 0.15 ? 1 : Math.sin(a) < -0.15 ? -1 : 0;
            const badge = 1 - stepBack * 0.14;
            return (
              <div
                key={p.file}
                style={{
                  position: 'absolute',
                  left: x - w / 2,
                  top: y - h / 2,
                  width: w,
                  height: h,
                  borderRadius: 10,
                  overflow: 'hidden',
                  border: `1px solid ${C.paper4}`,
                  outline: `1px solid oklch(0.24 0.014 70 / 0.06)`,
                  opacity: clamp01(t * 2.2) * badge,
                  transform: `translate(${(q * dirX * 420).toFixed(1)}px, ${(q * dirY * 320).toFixed(1)}px) rotate(${(q * (dirX || 1) * 6).toFixed(2)}deg) scale(${(1.12 - 0.12 * t) * (1 - stepBack * 0.04)})`,
                  filter: q > 0.03 ? `blur(${(q * 9).toFixed(2)}px) saturate(${(1 - stepBack * 0.08).toFixed(3)})` : `saturate(${(1 - stepBack * 0.08).toFixed(3)})`,
                  boxShadow: `0 ${(20 + q * 18).toFixed(1)}px ${(44 + q * 24).toFixed(1)}px oklch(0.24 0.014 70 / ${(0.22 * t).toFixed(3)})`,
                  background: C.paper0,
                }}
              >
                <Img src={staticFile(cut(p.file))} style={{ width: w, height: h, display: 'block' }} />
              </div>
            );
          })}

          {/* Stage light behind the wordmark. */}
          <div
            style={{
              position: 'absolute',
              left: CX - 620,
              top: CY - 300,
              width: 1240,
              height: 600,
              background: `radial-gradient(ellipse 50% 50% at 50% 50%, oklch(0.995 0.004 80 / 0.95) 0%, rgba(0,0,0,0) 70%)`,
              opacity: stage,
            }}
          />

          {/* The wordmark. */}
          <div
            style={{
              position: 'absolute',
              left: 0,
              right: 0,
              top: CY - 90,
              display: 'flex',
              flexDirection: 'column',
              alignItems: 'center',
            }}
          >
            <div style={{ transform: `scale(${(0.9 + 0.1 * markIn).toFixed(4)})`, opacity: markIn }}>
              <LedgerlyMark size={104} />
            </div>
            <div
              style={{
                marginTop: 14,
                fontFamily: FONT,
                fontSize: 118,
                fontWeight: 700,
                letterSpacing: '-0.03em',
                color: C.ink1,
                lineHeight: 1,
                display: 'flex',
              }}
            >
              {Array.from('Ledgerly').map((ch, i) => {
                const t = M.easeOut(clamp01((f - (50 + i * 1.8)) / 8));
                return (
                  <span
                    key={i}
                    style={{
                      display: 'inline-block',
                      opacity: t,
                      transform: `scale(${(1.3 - 0.3 * t).toFixed(4)})`,
                      filter: t < 1 ? `blur(${((1 - t) * 6).toFixed(2)}px)` : undefined,
                      transformOrigin: 'center bottom',
                    }}
                  >
                    {ch}
                  </span>
                );
              })}
            </div>
            <div
              style={{
                marginTop: 26,
                width: 300,
                height: 6,
                background: C.accent,
                transform: `scaleX(${rule})`,
              }}
            />
            <div
              style={{
                marginTop: 28,
                fontFamily: FONT,
                fontSize: 36,
                fontWeight: 400,
                letterSpacing: '-0.012em',
                color: C.ink2,
                opacity: subIn,
              }}
            >
              MIT licensed — free, and yours to keep.
            </div>
          </div>
        </AbsoluteFill>
      </AbsoluteFill>

      {/* Deterministic dust: warm motes drifting up, no two frames alike. */}
      <AbsoluteFill style={{ pointerEvents: 'none' }}>
        {Array.from({ length: 18 }, (_, i) => {
          const z = hash(i * 7.13 + 3.1);
          const size = 3 + z * 5;
          const x = hash(i * 3.71 + 11.3) * 1920;
          const y0 = hash(i * 5.31 + 2.7) * 1080;
          const y = ((y0 - f * (0.22 + z * 0.3)) % 1240 + 1240) % 1240 - 80;
          const tw = 0.55 + 0.45 * Math.sin(f / (16 + z * 22) + i);
          return (
            <div
              key={i}
              style={{
                position: 'absolute',
                left: x,
                top: y,
                width: size,
                height: size,
                borderRadius: '50%',
                background: `radial-gradient(circle, oklch(0.72 0.09 75 / ${(0.75 * tw).toFixed(3)}) 0%, rgba(0,0,0,0) 70%)`,
                opacity: clamp01((f - 18) / 16) * (0.35 + z * 0.4),
              }}
            />
          );
        })}
      </AbsoluteFill>
    </AbsoluteFill>
  );
};

function interpolate01(t: number) {
  return t;
}
