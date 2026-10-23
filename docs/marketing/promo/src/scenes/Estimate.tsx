// Shot 7 — quote, then invoice (deck-deal-flyin, adapted).
//
// The estimate rows are dealt into their slots with a hard, accelerating
// rhythm, then the Accepted estimate gains a link chip: that row is now an
// invoice. Recorded adaptation: the rows are dealt in place rather than into a
// fresh grid, because the product's own estimate list is the real target.
import React from 'react';
import { AbsoluteFill, Img, staticFile, useCurrentFrame } from 'remotion';
import { C, FONT, M } from '../tokens';
import { PageCam } from '../lib/PageCam';
import { boxes, cut, tex } from '../layout';
import { clamp01 } from '../lib/ui';

export const ESTIMATE_DURATION = 142;

const ROW_FILES = ['estimates-row1.png', 'estimates-row2.png', 'estimates-row3.png'];

export const Estimate: React.FC = () => {
  const f = useCurrentFrame();
  const rows = boxes('estimates', 'rows');

  const chipP = M.easeLand(clamp01((f - 64) / 16));
  const lift = M.easeOut(clamp01((f - 52) / 14));

  return (
    <AbsoluteFill style={{ backgroundColor: C.paper1 }}>
      <PageCam
        src={tex('estimates')}
        pageH={1080}
        keys={[
          { frame: 0, cx: 1080, cy: 360, zoom: 1.0 },
          { frame: 34, cx: 1080, cy: 360, zoom: 1.0 },
          { frame: 62, cx: 1080, cy: 348, zoom: 1.08 },
          { frame: ESTIMATE_DURATION, cx: 1080, cy: 348, zoom: 1.08 },
        ]}
      >
        {/* Empty slots first (deck edge), then the rows deal in. The gaps
            shorten — the deal accelerates. */}
        <div style={{ position: 'absolute', left: 274, top: 218, width: 1592, height: 174, background: C.paper0 }} />

        {rows.slice(0, 3).map((r, i) => {
          const start = 6 + i * (i === 2 ? 3 : 5);
          const p = M.easeLand(clamp01((f - start) / 13));
          const q = 1 - p;
          const accepted = i === 1;
          return (
            <div
              key={r.y}
              style={{
                position: 'absolute',
                left: r.x,
                top: r.y,
                width: r.width,
                height: r.height,
                background: C.paper0,
                opacity: clamp01(p * 2.2),
                transform: `translateX(${(q * 1180).toFixed(1)}px) rotate(${(q * 5).toFixed(2)}deg) translateY(${(accepted ? -lift * 3 : 0).toFixed(2)}px)`,
                transformOrigin: 'right center',
                filter: q > 0.02 ? `blur(${(q * 7).toFixed(2)}px)` : undefined,
                boxShadow: `0 ${(q * 22).toFixed(1)}px ${(q * 40).toFixed(1)}px oklch(0.24 0.014 70 / ${(q * 0.16).toFixed(3)})`,
              }}
            >
              <Img src={staticFile(cut(ROW_FILES[i]))} style={{ width: r.width, height: r.height, display: 'block' }} />
            </div>
          );
        })}

        {/* The link: the Accepted estimate is now an invoice. The chip carries
            the read on its own — a connector bar at this x would land on the
            row below's number. */}
        <div
          style={{
            position: 'absolute',
            left: 281,
            top: 424,
            display: 'flex',
            alignItems: 'center',
            gap: 16,
            padding: '14px 26px',
            borderRadius: 999,
            background: C.accent,
            color: C.inkInverse,
            opacity: chipP,
            transform: `translateX(${((1 - chipP) * 90).toFixed(1)}px)`,
          }}
        >
          <span style={{ fontFamily: FONT, fontSize: 34, fontWeight: 700, letterSpacing: '-0.012em' }}>
            → linked invoice INV-0042
          </span>
        </div>
      </PageCam>
    </AbsoluteFill>
  );
};
