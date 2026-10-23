// Shot 4 — the totals engine (row-embed).
//
// The invoice's line items embed into their real slots, then the camera walks
// down to the totals that reconcile them. The rows land in the coordinates
// captured in live-layout.json, not near them (rule Q9).
import React from 'react';
import { AbsoluteFill, Img, staticFile, useCurrentFrame } from 'remotion';
import { C, M } from '../tokens';
import { PageCam } from '../lib/PageCam';
import { boxes, cut, tex } from '../layout';
import { clamp01 } from '../lib/ui';

export const INVOICE_DURATION = 178;

const ROWS = ['invoice-row1.png', 'invoice-row2.png'];
const TOTALS = boxes('invoice', 'rows').filter((r) => r.x > 1000);

export const Invoice: React.FC = () => {
  const f = useCurrentFrame();
  const rows = boxes('invoice', 'rows').filter((r) => r.x < 1000);

  const totalsTop = TOTALS.length ? Math.min(...TOTALS.map((r) => r.y)) : 664;
  const totalsBottom = TOTALS.length ? Math.max(...TOTALS.map((r) => r.y + r.height)) : 860;
  const wash = M.easeOut(clamp01((f - 108) / 22));

  return (
    <AbsoluteFill style={{ backgroundColor: C.paper1 }}>
      <PageCam
        src={tex('invoice')}
        pageH={2520}
        keys={[
          { frame: 0, cx: 1080, cy: 320, zoom: 1.14 },
          { frame: 58, cx: 1080, cy: 320, zoom: 1.14 },
          { frame: 108, cx: 1080, cy: 800, zoom: 1.18 },
          { frame: INVOICE_DURATION, cx: 1080, cy: 800, zoom: 1.18 },
        ]}
      >
        {/* The line-item slots start empty; the rows below fly into them. */}
        <div
          style={{
            position: 'absolute',
            left: 274,
            top: 392,
            width: 1592,
            height: 136,
            background: C.paper0,
          }}
        />

        {rows.slice(0, 2).map((r, i) => {
          const start = 10 + i * 14;
          const p = M.easeLand(clamp01((f - start) / 14));
          const q = 1 - p; // remaining travel
          return (
            <div
              key={r.y}
              style={{
                position: 'absolute',
                left: r.x,
                top: r.y,
                width: r.width,
                height: r.height,
                opacity: clamp01(p * 2.4),
                transform: `translate(${(q * 168).toFixed(1)}px, ${(q * 24).toFixed(1)}px) rotate(${(q * 1.1).toFixed(3)}deg)`,
                transformOrigin: 'right center',
                filter: q > 0.02 ? `blur(${(q * 4).toFixed(2)}px)` : undefined,
                boxShadow: `0 ${(q * 16).toFixed(1)}px ${(q * 30).toFixed(1)}px oklch(0.24 0.014 70 / ${(q * 0.14).toFixed(3)})`,
                background: C.paper0,
              }}
            >
              <Img src={staticFile(cut(ROWS[i]))} style={{ width: r.width, height: r.height, display: 'block' }} />
            </div>
          );
        })}

        {/* The totals block, emphasised once the camera arrives. A left bracket
            and an underline, not a fill — a fill would bury the numbers. */}
        <div
          style={{
            position: 'absolute',
            left: 1054,
            top: totalsTop - 14,
            width: 830,
            height: totalsBottom - totalsTop + 28,
            borderRadius: 12,
            background: C.accentSoft,
            opacity: wash * 0.42,
          }}
        />
        <div
          style={{
            position: 'absolute',
            left: 1040,
            top: totalsTop - 14,
            width: 7,
            height: totalsBottom - totalsTop + 28,
            background: C.accent,
            borderRadius: 4,
            transform: `scaleY(${wash})`,
            transformOrigin: 'top center',
          }}
        />
        <div
          style={{
            position: 'absolute',
            left: 1054,
            top: totalsBottom + 16,
            width: 830,
            height: 5,
            background: C.accent,
            transform: `scaleX(${wash})`,
            transformOrigin: 'left center',
          }}
        />
      </PageCam>
    </AbsoluteFill>
  );
};
