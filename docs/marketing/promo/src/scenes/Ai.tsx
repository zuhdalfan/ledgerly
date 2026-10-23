// Shot 10 — AI access over MCP (scan-bracket-sweep, adapted).
//
// Bracket corners sweep in and lock around the "Connect an assistant" card.
// Deliberately NOT the lights-down spotlight: that technique is the hero's and
// may only be the protagonist once (rules P4/Q4).
import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { C, M } from '../tokens';
import { PageCam } from '../lib/PageCam';
import { box, tex } from '../layout';
import { clamp01 } from '../lib/ui';

export const AI_DURATION = 112;

const Corner: React.FC<{ x: number; y: number; dx: number; dy: number; p: number; len?: number }> = ({
  x,
  y,
  dx,
  dy,
  p,
  len = 46,
}) => {
  const w = len * p;
  const t = 5;
  return (
    <>
      <div style={{ position: 'absolute', left: dx > 0 ? x : x - w, top: dy > 0 ? y : y - t, width: w, height: t, background: C.accent }} />
      <div style={{ position: 'absolute', left: dx > 0 ? x : x - t, top: dy > 0 ? y : y - w, width: t, height: w, background: C.accent }} />
    </>
  );
};

export const Ai: React.FC = () => {
  const f = useCurrentFrame();
  const card = box('ai', 'cards', 1);
  const lock = M.easeTrust(clamp01((f - 44) / 22));

  return (
    <AbsoluteFill style={{ backgroundColor: C.paper1 }}>
      <PageCam
        src={tex('ai')}
        pageH={1479}
        keys={[
          { frame: 0, cx: 1080, cy: 520, zoom: 0.9 },
          { frame: 44, cx: 1080, cy: 900, zoom: 1.08 },
          { frame: AI_DURATION, cx: 1080, cy: 900, zoom: 1.08 },
        ]}
      >
        <Corner x={card.x} y={card.y} dx={1} dy={1} p={lock} />
        <Corner x={card.x + card.width} y={card.y} dx={-1} dy={1} p={lock} />
        <Corner x={card.x} y={card.y + card.height} dx={1} dy={-1} p={lock} />
        <Corner x={card.x + card.width} y={card.y + card.height} dx={-1} dy={-1} p={lock} />
      </PageCam>
    </AbsoluteFill>
  );
};
