// paper-title-card, re-skinned for Ledgerly.
//
// The recipe card's per-word letterpress and single emphasised phrase are kept.
// The skin is the product's: Inter (not a serif), and emphasis as a soft
// charcoal wash with a growing underline — Ledgerly's accent is a neutral, so
// emphasis comes from weight and placement rather than hue.
import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { C, FONT, M } from '../tokens';
import { PaperBg, Rule, WordStamp, clamp01 } from './ui';

type Word = { w: string; accent: boolean };
type Run = { accent: boolean; words: string[] };

const PUNCT_ONLY = /^[^\p{L}\p{N}]+$/u;

const tokenize = (s: string): Word[] => {
  const out: Word[] = [];
  let accent = false;
  let cur = '';
  let curAccent = false;
  let started = false;
  for (const ch of s) {
    if (ch === '*') {
      if (started) {
        out.push({ w: cur, accent: curAccent });
        cur = '';
        started = false;
      }
      accent = !accent;
      continue;
    }
    if (ch === ' ') {
      if (started) {
        out.push({ w: cur, accent: curAccent });
        cur = '';
        started = false;
      }
      continue;
    }
    if (!started) {
      started = true;
      curAccent = accent;
    }
    cur += ch;
  }
  if (started) out.push({ w: cur, accent: curAccent });

  // Trailing punctuation belongs to the phrase it follows ("add up." is one
  // emphasised run, not "add up" plus a floating full stop).
  for (let i = 1; i < out.length; i++) {
    if (!out[i].accent && PUNCT_ONLY.test(out[i].w) && out[i - 1].accent) out[i].accent = true;
  }
  return out;
};

const toRuns = (words: Word[]): Run[] => {
  const runs: Run[] = [];
  for (const { w, accent } of words) {
    const last = runs[runs.length - 1];
    if (last && last.accent === accent) last.words.push(w);
    else runs.push({ accent, words: [w] });
  }
  return runs;
};

export const TitleCard: React.FC<{
  text: string;
  duration: number;
  size?: number;
}> = ({ text, duration, size = 92 }) => {
  const frame = useCurrentFrame();
  const runs = toRuns(tokenize(text));
  const outT = M.easeTrust(clamp01((frame - (duration - 9)) / 9));
  const underline = M.easeOut(clamp01((frame - 18) / 18));

  let wordIndex = 0;
  return (
    <AbsoluteFill style={{ opacity: 1 - outT, transform: `translateY(${-outT * 26}px)` }}>
      <PaperBg light={{ x: 0.5, y: 0.42 }} />
      <AbsoluteFill style={{ justifyContent: 'center', alignItems: 'center', padding: '0 180px' }}>
        <div
          style={{
            maxWidth: 1420,
            textAlign: 'center',
            fontFamily: FONT,
            fontSize: size,
            fontWeight: 700,
            letterSpacing: '-0.028em',
            lineHeight: 1.16,
            color: C.ink1,
          }}
        >
          {runs.map((run, ri) => {
            const inner = run.words.map((w, wi) => {
              const idx = wordIndex++;
              const needsSpace = wi > 0 && !PUNCT_ONLY.test(w[0]);
              return (
                <React.Fragment key={wi}>
                  {needsSpace ? ' ' : null}
                  <WordStamp text={w} start={4 + idx * 4} dur={9} />
                </React.Fragment>
              );
            });
            return (
              <React.Fragment key={ri}>
                {ri > 0 ? ' ' : null}
                {run.accent ? (
                  <span
                    style={{
                      display: 'inline-block',
                      whiteSpace: 'pre',
                      padding: '0 18px 8px',
                      backgroundColor: C.accentSoft,
                      backgroundImage: `linear-gradient(${C.accent}, ${C.accent})`,
                      backgroundRepeat: 'no-repeat',
                      backgroundPosition: 'left 90%',
                      backgroundSize: `${(underline * 100).toFixed(1)}% 6px`,
                      borderRadius: 12,
                    }}
                  >
                    {inner}
                  </span>
                ) : (
                  inner
                )}
              </React.Fragment>
            );
          })}
        </div>
        <div style={{ marginTop: 54 }}>
          <Rule width={190} start={26} dur={20} thickness={5} />
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
