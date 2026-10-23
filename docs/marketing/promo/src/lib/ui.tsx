// Shared UI for the Ledgerly promo: the product's own brand mark, its font
// loading, and the small typographic primitives the shots compose from.
import React from 'react';
import { AbsoluteFill, continueRender, delayRender, interpolate, staticFile, useCurrentFrame } from 'remotion';
import { C, FONT, M, T, tracking } from '../tokens';

export const clamp01 = (x: number) => (x < 0 ? 0 : x > 1 ? 1 : x);

/** Loads the product's own Inter files before any frame is captured. */
export const Fonts: React.FC = () => {
  const [handle] = React.useState(() => delayRender('ledgerly-inter'));
  React.useEffect(() => {
    let settled = false;
    const finish = () => {
      if (settled) return;
      settled = true;
      continueRender(handle);
    };
    try {
      const faces = [
        new FontFace('Inter', `url(${staticFile('fonts/Inter-Regular.ttf')}) format('truetype')`, { weight: '400' }),
        new FontFace('Inter', `url(${staticFile('fonts/Inter-Bold.ttf')}) format('truetype')`, { weight: '700' }),
      ];
      Promise.all(faces.map((f) => f.load().then((loaded) => document.fonts.add(loaded))))
        .catch(() => undefined)
        .then(finish);
    } catch {
      finish();
    }
    const t = setTimeout(finish, 8000);
    return () => clearTimeout(t);
  }, [handle]);
  return null;
};

// The brand mark: a torn paper slip, the same artifact the mascot is made from.
// Monochrome so it sits at any inline or hero scale.
const SLIP = 'M24 22A10 10 0 0 1 34 12H62A10 10 0 0 1 72 22V62L66 68 60 62 54 68 48 62 42 68 36 62 30 68 24 62Z';
const RULES = ['M36 31H60', 'M36 43H60'];

export const LedgerlyMark: React.FC<{
  size?: number;
  color?: string;
  ruleColor?: string;
  /** 0→1 draws the slip outline. */
  draw?: number;
  /** 0→1 cross-fades the fill in. */
  fill?: number;
  /** 0→1 draws the two rules (the second follows slightly behind). */
  rules?: number;
  style?: React.CSSProperties;
}> = ({ size = 96, color = C.ink1, ruleColor = C.paper0, draw = 1, fill = 1, rules = 1, style }) => (
  <svg width={size} height={size} viewBox="0 0 96 96" style={{ display: 'block', overflow: 'visible', ...style }}>
    <path
      d={SLIP}
      fill={color}
      stroke={color}
      strokeWidth={5}
      strokeLinejoin="round"
      pathLength={1}
      strokeDasharray={1}
      strokeDashoffset={1 - clamp01(draw)}
      opacity={clamp01(fill)}
    />
    {RULES.map((d, i) => {
      const p = clamp01((clamp01(rules) - i * 0.22) / (1 - i * 0.22));
      return (
        <path
          key={d}
          d={d}
          fill="none"
          stroke={ruleColor}
          strokeWidth={6}
          strokeLinecap="round"
          pathLength={1}
          strokeDasharray={1}
          strokeDashoffset={1 - p}
        />
      );
    })}
  </svg>
);

export const LedgerlyWordmark: React.FC<{
  size?: number;
  color?: string;
  gap?: number;
  markSize?: number;
  showMark?: boolean;
  style?: React.CSSProperties;
}> = ({ size = 72, color = C.ink1, gap, markSize, showMark = true, style }) => (
  <div style={{ display: 'inline-flex', alignItems: 'center', gap: gap ?? size * 0.42, ...style }}>
    {showMark && <LedgerlyMark size={markSize ?? size * 1.15} color={color} ruleColor={C.paper0} />}
    <span
      style={{
        fontFamily: FONT,
        fontSize: size,
        fontWeight: 700,
        letterSpacing: '-0.022em',
        lineHeight: 1,
        color,
        whiteSpace: 'nowrap',
      }}
    >
      Ledgerly
    </span>
  </div>
);

/** The paper ground: a warm ramp with one soft key light. */
export const PaperBg: React.FC<{ light?: { x: number; y: number }; dim?: number }> = ({
  light = { x: 0.5, y: 0.22 },
  dim = 0,
}) => (
  <AbsoluteFill
    style={{
      background: `linear-gradient(180deg, ${C.paper0} 0%, ${C.paper1} 54%, ${C.paper2} 100%)`,
    }}
  >
    <AbsoluteFill
      style={{
        background: `radial-gradient(ellipse 62% 58% at ${light.x * 100}% ${light.y * 100}%, ${C.paper0} 0%, rgba(0,0,0,0) 70%)`,
      }}
    />
    {dim > 0 && <AbsoluteFill style={{ background: C.stage, opacity: dim }} />}
    <AbsoluteFill
      style={{
        background: `radial-gradient(ellipse 120% 100% at 50% 45%, rgba(0,0,0,0) 55%, oklch(0.24 0.014 70 / 0.10) 100%)`,
      }}
    />
  </AbsoluteFill>
);

/** Per-character letterpress: scale down into place, blur to zero. */
export const Letterpress: React.FC<{
  text: string;
  start?: number;
  per?: number;
  dur?: number;
  size?: number;
  weight?: number;
  color?: string;
  style?: React.CSSProperties;
}> = ({ text, start = 0, per = 3, dur = 12, size = 92, weight = 700, color = C.ink1, style }) => {
  const frame = useCurrentFrame();
  return (
    <span
      style={{
        fontFamily: FONT,
        fontSize: size,
        fontWeight: weight,
        letterSpacing: tracking(size),
        color,
        lineHeight: 1.02,
        display: 'inline-block',
        ...style,
      }}
    >
      {Array.from(text).map((ch, i) => {
        const p = M.easeOut(clamp01((frame - (start + i * per)) / dur));
        return (
          <span
            key={i}
            style={{
              display: 'inline-block',
              opacity: p,
              transform: `scale(${(1.35 - 0.35 * p).toFixed(4)})`,
              filter: p < 1 ? `blur(${((1 - p) * 7).toFixed(2)}px)` : undefined,
              transformOrigin: 'center bottom',
              whiteSpace: 'pre',
            }}
          >
            {ch}
          </span>
        );
      })}
    </span>
  );
};

/** Per-character typewriter with a block cursor that stops once the text lands. */
export const Typewriter: React.FC<{
  text: string;
  start?: number;
  per?: number;
  size?: number;
  weight?: number;
  color?: string;
  cursorColor?: string;
  stopBlink?: number;
  style?: React.CSSProperties;
}> = ({ text, start = 0, per = 0.9, size = 40, weight = 400, color = C.ink2, cursorColor = C.accent, stopBlink, style }) => {
  const frame = useCurrentFrame();
  const shown = clamp01((frame - start) / per);
  const count = Math.floor(shown * text.length);
  const complete = count >= text.length;
  const blinkOff = !complete && Math.floor(frame / 2) % 2 === 1;
  const done = stopBlink !== undefined && frame >= stopBlink;
  return (
    <span
      style={{
        fontFamily: FONT,
        fontSize: size,
        fontWeight: weight,
        color,
        letterSpacing: '-0.011em',
        lineHeight: 1.35,
        ...style,
      }}
    >
      {text.slice(0, count)}
      {!done && (
        <span
          style={{
            display: 'inline-block',
            width: size * 0.5,
            height: size * 1.02,
            marginLeft: 3,
            verticalAlign: '-0.16em',
            background: cursorColor,
            opacity: blinkOff ? 0.15 : 1,
          }}
        />
      )}
    </span>
  );
};

/** Whole-word letterpress. Unlike Letterpress (per character), this animates a
 *  word as one unit — the recipe the title cards are tuned to. */
export const WordStamp: React.FC<{
  text: string;
  start?: number;
  dur?: number;
  color?: string;
  style?: React.CSSProperties;
}> = ({ text, start = 0, dur = 10, color, style }) => {
  const frame = useCurrentFrame();
  const p = M.easeOut(clamp01((frame - start) / dur));
  return (
    <span
      style={{
        display: 'inline-block',
        whiteSpace: 'pre',
        color,
        opacity: p,
        transform: `scale(${(1.28 - 0.28 * p).toFixed(4)})`,
        filter: p < 1 ? `blur(${((1 - p) * 7).toFixed(2)}px)` : undefined,
        transformOrigin: 'center bottom',
        ...style,
      }}
    >
      {text}
    </span>
  );
};

/** A hairline rule that grows. */
export const Rule: React.FC<{ width: number; start?: number; dur?: number; color?: string; thickness?: number }> = ({
  width,
  start = 0,
  dur = 18,
  color = C.accent,
  thickness = 4,
}) => {
  const frame = useCurrentFrame();
  const p = M.easeOut(clamp01((frame - start) / dur));
  return (
    <div style={{ width, height: thickness, background: color, transform: `scaleX(${p})`, transformOrigin: 'left center' }} />
  );
};

/**
 * Bottom-strip narration. Sized at 58px (>=5.2% of 1080) so it survives a
 * phone-sized replay; a paper scrim keeps contrast over busy UI.
 */
export const Narration: React.FC<{
  text: string;
  duration: number;
  bottom?: number;
  color?: string;
  scrim?: boolean;
}> = ({ text, duration, bottom = 58, color = C.ink1, scrim = true }) => {
  const frame = useCurrentFrame();
  const inT = interpolate(frame, [0, 9], [0, 1], { extrapolateLeft: 'clamp', extrapolateRight: 'clamp', easing: M.easeOut });
  const outT = interpolate(frame, [duration - 9, duration], [1, 0], {
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });
  return (
    <div
      style={{
        position: 'absolute',
        left: 0,
        right: 0,
        bottom,
        display: 'flex',
        justifyContent: 'center',
        pointerEvents: 'none',
        opacity: inT * outT,
        transform: `translateY(${(1 - inT) * 12}px)`,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 18,
          padding: scrim ? '14px 30px' : 0,
          background: scrim ? `oklch(0.995 0.002 80 / 0.86)` : undefined,
          border: scrim ? `1px solid ${C.paper3}` : undefined,
          borderRadius: 999,
          backdropFilter: scrim ? 'blur(6px)' : undefined,
        }}
      >
        <span style={{ width: 12, height: 12, background: C.accent, display: 'inline-block', borderRadius: 2 }} />
        <span
          style={{
            fontFamily: FONT,
            fontSize: T.caption,
            fontWeight: 600,
            letterSpacing: '-0.018em',
            color,
            whiteSpace: 'nowrap',
          }}
        >
          {text}
        </span>
      </div>
    </div>
  );
};
