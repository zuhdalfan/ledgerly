// Shot 9 — the offline promise, as a typographic hero.
//
// The pivot of the film: the claim that separates Ledgerly from every cloud
// invoicing tool. Deliberately plain — three short lines, one rule, one
// supporting line. No product UI at all.
import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { C, FONT, M } from '../tokens';
import { Letterpress, PaperBg, Rule, clamp01 } from '../lib/ui';

export const OFFLINE_DURATION = 112;

const LINES = ['No account.', 'No cloud.', 'No network.'];

export const Offline: React.FC = () => {
  const f = useCurrentFrame();
  const outT = M.easeTrust(clamp01((f - (OFFLINE_DURATION - 10)) / 10));

  return (
    <AbsoluteFill style={{ opacity: 1 - outT }}>
      <PaperBg light={{ x: 0.32, y: 0.4 }} />
      <AbsoluteFill style={{ justifyContent: 'center', padding: '0 210px' }}>
        <div>
          {LINES.map((line, i) => (
            <div key={line} style={{ marginBottom: 4 }}>
              <Letterpress text={line} start={i * 13} per={2.4} dur={11} size={106} />
            </div>
          ))}
        </div>
        <div style={{ marginTop: 46 }}>
          <Rule width={230} start={68} dur={20} thickness={6} />
        </div>
        <div
          style={{
            marginTop: 36,
            fontFamily: FONT,
            fontSize: 36,
            fontWeight: 400,
            letterSpacing: '-0.012em',
            color: C.ink2,
            opacity: clamp01((f - 78) / 14),
          }}
        >
          Every invoice, client and report — in one file on your own disk.
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
