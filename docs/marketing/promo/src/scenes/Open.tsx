// Shot 1 — brand open (brand-ink-open, re-skinned).
//
// The crosshair of the recipe card is replaced by Ledgerly's own torn-paper
// slip, drawn on, then filled; the wordmark letterpresses in; the line types
// itself on. The wordmark then holds still for a full second before drifting
// out (rule R1).
import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { M } from '../tokens';
import { LedgerlyMark, Letterpress, PaperBg, Typewriter, clamp01 } from '../lib/ui';

export const OPEN_DURATION = 140;

export const Open: React.FC = () => {
  const f = useCurrentFrame();
  const draw = M.easeOut(clamp01((f - 2) / 26));
  const fill = M.easeOut(clamp01((f - 22) / 16));
  const rules = clamp01((f - 26) / 22);
  // Drift out only after the wordmark has held (lands f63, exit f122 => 59f).
  const exit = M.easeTrust(clamp01((f - 122) / 16));

  return (
    <AbsoluteFill>
      <PaperBg light={{ x: 0.5, y: 0.34 }} />
      <AbsoluteFill
        style={{
          justifyContent: 'center',
          alignItems: 'center',
          transform: `translateY(${-exit * 46}px) scale(${(1 - exit * 0.1).toFixed(4)})`,
          opacity: 1 - exit,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 32 }}>
          <LedgerlyMark size={148} draw={draw} fill={fill} rules={rules} />
          <Letterpress text="Ledgerly" start={30} per={3} dur={12} size={148} />
        </div>
        <div style={{ height: 50 }} />
        <Typewriter
          text="Invoicing that never leaves your machine."
          start={66}
          per={0.8}
          size={42}
          stopBlink={116}
        />
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
