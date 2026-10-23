// Shot 8 — reports (chart-live-moves + document-typewriter-reveal, adapted).
//
// A single camera walk down the real reports page: revenue, then revenue by
// client, then tax and payments. The densest information in the film, so it
// sits third-from-last, ahead of the peak (sequence template).
import React from 'react';
import { AbsoluteFill, useCurrentFrame } from 'remotion';
import { C, FONT, M } from '../tokens';
import { PageCam } from '../lib/PageCam';
import { boxes, tex } from '../layout';
import { clamp01 } from '../lib/ui';

export const REPORTS_DURATION = 138;

export const Reports: React.FC = () => {
  const f = useCurrentFrame();
  const rows = boxes('reports', 'rows');
  const totalRow = rows[6] ?? rows[rows.length - 1];

  const wash = M.easeOut(clamp01((f - 26) / 18)) * (1 - clamp01((f - 62) / 20));
  const rule = M.easeOut(clamp01((f - 30) / 18));

  return (
    <AbsoluteFill style={{ backgroundColor: C.paper1 }}>
      <PageCam
        src={tex('reports')}
        pageH={2275}
        keys={[
          { frame: 0, cx: 1080, cy: 330, zoom: 1.15 },
          { frame: 30, cx: 1080, cy: 330, zoom: 1.15 },
          { frame: 90, cx: 1080, cy: 820, zoom: 1.02 },
          { frame: REPORTS_DURATION, cx: 1080, cy: 1760, zoom: 0.95 },
        ]}
      >
        {/* The revenue total: the one number the table exists to produce.
            A bracket and an underline, never a fill over the figures. */}
        {totalRow && (
          <>
            <div
              style={{
                position: 'absolute',
                left: totalRow.x - 10,
                top: totalRow.y - 4,
                width: totalRow.width + 20,
                height: totalRow.height + 8,
                borderRadius: 10,
                background: C.accentSoft,
                opacity: wash * 0.4,
              }}
            />
            <div
              style={{
                position: 'absolute',
                left: totalRow.x - 24,
                top: totalRow.y - 4,
                width: 6,
                height: totalRow.height + 8,
                background: C.accent,
                borderRadius: 3,
                transform: `scaleY(${rule})`,
                transformOrigin: 'top center',
              }}
            />
            <div
              style={{
                position: 'absolute',
                left: totalRow.x - 10,
                top: totalRow.y + totalRow.height + 4,
                width: totalRow.width + 20,
                height: 5,
                background: C.accent,
                transform: `scaleX(${rule})`,
                transformOrigin: 'left center',
              }}
            />
          </>
        )}
      </PageCam>

      {/* A reading rule: it tracks the camera down the page so the walk reads
          as one continuous document, not three cuts. */}
      <div
        style={{
          position: 'absolute',
          left: 210,
          top: 176,
          bottom: 176,
          width: 3,
          background: `linear-gradient(180deg, ${C.accent} 0%, ${C.accent} 22%, oklch(0.30 0.006 75 / 0.05) 100%)`,
          opacity: 0.5,
          transform: `scaleY(${M.easeTrust(clamp01((f - 4) / 26)).toFixed(3)})`,
          transformOrigin: 'top center',
        }}
      />
    </AbsoluteFill>
  );
};
