// Shot 5 — finding a record (type-and-filter).
//
// The camera pushes onto the real search field; the query types itself on at a
// readable size; the list narrows to the one match. Rows recede rather than
// being deleted — a fake reflow would read worse than an honest dissolve.
import React from 'react';
import { AbsoluteFill, Img, staticFile, useCurrentFrame } from 'remotion';
import { C, FONT, M } from '../tokens';
import { PageCam } from '../lib/PageCam';
import { box, boxes, cut, tex } from '../layout';
import { clamp01 } from '../lib/ui';

export const FIND_DURATION = 142;

const QUERY = 'INV-0041';

export const Find: React.FC = () => {
  const f = useCurrentFrame();
  const search = box('invoices', 'search', 0);
  const rows = boxes('invoices', 'rows');
  const match = rows[0];

  // type from f34, 5 frames per character
  const typed = Math.floor(clamp01((f - 34) / (5 * QUERY.length)) * QUERY.length);
  const caretOn = f >= 30 && Math.floor(f / 2) % 2 === 0;

  // the list narrows once the query is complete
  const narrow = M.easeTrust(clamp01((f - 96) / 20));
  const focus = M.easeLand(clamp01((f - 104) / 16));

  return (
    <AbsoluteFill style={{ backgroundColor: C.paper1 }}>
      <PageCam
        src={tex('invoices')}
        pageH={1080}
        keys={[
          { frame: 0, cx: 1080, cy: 300, zoom: 1.06 },
          { frame: 26, cx: 690, cy: 150, zoom: 2.15 },
          { frame: 62, cx: 690, cy: 150, zoom: 2.15 },
          { frame: 96, cx: 1080, cy: 300, zoom: 1.15 },
          { frame: FIND_DURATION, cx: 1080, cy: 300, zoom: 1.15 },
        ]}
      >
        {/* The field's own placeholder is baked into the texture; cover it, then
            draw what was actually typed, on the page plane at the field's type
            size. */}
        <div
          style={{
            position: 'absolute',
            left: search.x + 2,
            top: search.y + 2,
            width: search.width - 4,
            height: search.height - 4,
            borderRadius: 7,
            background: C.paper0,
          }}
        />
        <div
          style={{
            position: 'absolute',
            left: search.x + 13,
            top: search.y,
            width: search.width - 26,
            height: search.height,
            display: 'flex',
            alignItems: 'center',
            fontFamily: FONT,
            fontSize: 15,
            fontWeight: 400,
            color: C.ink1,
            letterSpacing: '-0.006em',
          }}
        >
          {QUERY.slice(0, typed)}
          <span
            style={{
              display: 'inline-block',
              width: 2,
              height: 20,
              marginLeft: 1,
              background: C.ink1,
              opacity: caretOn ? 1 : 0,
            }}
          />
        </div>

        {/* Rows that are not the match dissolve. */}
        {rows.slice(1).map((r) => (
          <div
            key={r.y}
            style={{
              position: 'absolute',
              left: r.x,
              top: r.y,
              width: r.width,
              height: r.height,
              background: C.paper0,
              opacity: narrow,
            }}
          />
        ))}

        {/* The match steps forward: a wash, then the row itself redrawn so the
            highlight sits behind the text instead of over it. */}
        {match && (
          <>
            <div
              style={{
                position: 'absolute',
                left: match.x - 10,
                top: match.y - 4,
                width: match.width + 20,
                height: match.height + 8,
                borderRadius: 10,
                background: C.accentSoft,
                opacity: focus,
              }}
            />
            <div
              style={{
                position: 'absolute',
                left: match.x,
                top: match.y,
                width: match.width,
                height: match.height,
                background: C.paper0,
                opacity: focus,
              }}
            />
            <div
              style={{
                position: 'absolute',
                left: match.x,
                top: match.y,
                width: match.width,
                height: match.height,
                opacity: focus,
                transform: `translateY(${(-3 * focus).toFixed(2)}px) scale(${(1 + 0.012 * focus).toFixed(4)})`,
                transformOrigin: 'left center',
                boxShadow: `0 ${(8 * focus).toFixed(1)}px ${(22 * focus).toFixed(1)}px oklch(0.24 0.014 70 / ${(0.1 * focus).toFixed(3)})`,
              }}
            >
              <Img src={staticFile(cut('invoices-row1.png'))} style={{ width: match.width, height: match.height, display: 'block' }} />
            </div>
            <div
              style={{
                position: 'absolute',
                left: match.x - 24,
                top: match.y + 6,
                width: 6,
                height: match.height - 12,
                background: C.accent,
                borderRadius: 3,
                transform: `scaleY(${focus})`,
                transformOrigin: 'center center',
              }}
            />
          </>
        )}
      </PageCam>
    </AbsoluteFill>
  );
};
