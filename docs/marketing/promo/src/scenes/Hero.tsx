// Shot 2 — hero (spotlight-hero-card, re-skinned and reduced).
//
// The theatre move: the dashboard goes dark, the frame pushes to the
// Outstanding card, the card lifts off the page, hovers, and reseats. One
// protagonist, one complete arc, >=3s (rule Q5/R3).
//
// Recorded deviation (see DESIGN-SPEC §3): the volumetric light shaft and the
// double-lap contour beam are dropped — on a light warm-paper UI they read as
// noise. The card-shaped light hole, the lift/hover/reseat arc with its landing
// overshoot, the page-space 3D annotation and the single clipped sheen are kept.
import React from 'react';
import { AbsoluteFill, Img, staticFile, useCurrentFrame } from 'remotion';
import { C, FONT, M } from '../tokens';
import { PageCam } from '../lib/PageCam';
import { box, cut, tex } from '../layout';
import { clamp01 } from '../lib/ui';

export const HERO_DURATION = 200;

const CARD_CUTOUT = 'dashboard-card1.png';

export const Hero: React.FC = () => {
  const f = useCurrentFrame();
  const card = box('dashboard', 'cards', 0);
  const cx = card.x + card.width / 2;
  const cy = card.y + card.height / 2;

  const dark = M.easeTrust(clamp01((f - 28) / 58)) * 0.93;

  const rise = M.easeLand(clamp01((f - 96) / 15));
  const reseat = M.easeTrust(clamp01((f - 150) / 16));
  const liftT = clamp01(rise - reseat);
  const hover = clamp01((f - 111) / 8) * (1 - clamp01((f - 150) / 8));
  const bob = Math.sin(((f - 111) / 40) * Math.PI * 2) * 4 * hover;

  const liftPx = liftT * 34 + bob;
  const zLift = liftT * 150;
  const cardScale = 1 + liftT * 0.016;

  const annIn = M.easeOut(clamp01((f - 112) / 12));
  const annRule = M.easeOut(clamp01((f - 122) / 18));
  const sheenP = clamp01((f - 112) / 30);

  return (
    <AbsoluteFill style={{ backgroundColor: C.paper1 }}>
      <PageCam
        src={tex('dashboard')}
        pageH={1567}
        keys={[
          { frame: 0, cx: 960, cy: 560, zoom: 0.82 },
          { frame: 40, cx: 960, cy: 560, zoom: 0.82 },
          { frame: 104, cx: 1090, cy: 300, zoom: 1.12 },
          { frame: HERO_DURATION, cx: 1090, cy: 300, zoom: 1.12 },
        ]}
      >
        {/* 1. The lights: a page-space field with a card-shaped hole. It lives in
            page space so it follows the camera and the card's own geometry. */}
        <div
          style={{
            position: 'absolute',
            left: card.x,
            top: card.y,
            width: card.width,
            height: card.height,
            borderRadius: 14,
            boxShadow: `0 0 150px 2200px oklch(0.14 0.012 70 / ${dark.toFixed(3)})`,
          }}
        />

        {/* 2. The slot the card leaves behind, so the page reads as a page and
            not a print of a card that walked off. */}
        <div
          style={{
            position: 'absolute',
            left: card.x,
            top: card.y,
            width: card.width,
            height: card.height,
            borderRadius: 14,
            background: C.paper0,
            border: `1px solid ${C.paper3}`,
            opacity: Math.min(1, liftT * 2.2),
          }}
        />

        {/* 3. The card itself, lifted on the page plane. */}
        <div
          style={{
            position: 'absolute',
            left: card.x,
            top: card.y,
            width: card.width,
            height: card.height,
            borderRadius: 14,
            overflow: 'hidden',
            transform: `translateY(${-liftPx.toFixed(2)}px) translateZ(${zLift.toFixed(1)}px) scale(${cardScale.toFixed(4)})`,
            transformOrigin: 'center center',
            boxShadow: `0 ${(12 + zLift * 0.16).toFixed(1)}px ${(34 + zLift * 0.46).toFixed(1)}px oklch(0.24 0.014 70 / ${(0.1 + 0.2 * liftT).toFixed(3)})`,
          }}
        >
          <Img src={staticFile(cut(CARD_CUTOUT))} style={{ width: card.width, height: card.height, display: 'block' }} />
          {/* One light effect, on the protagonist only, clipped to the card. */}
          {sheenP > 0 && sheenP < 1 && (
            <div
              style={{
                position: 'absolute',
                inset: 0,
                mixBlendMode: 'soft-light',
                background: `linear-gradient(108deg, transparent ${(sheenP * 100 - 26).toFixed(1)}%, oklch(1 0 0 / 0.5) ${(sheenP * 100).toFixed(1)}%, transparent ${(sheenP * 100 + 26).toFixed(1)}%)`,
              }}
            />
          )}
        </div>

        {/* 4. The 3D annotation: same page coordinate system as the card, drawn
            above the lights, so it is 'self-illuminated'. Its dark backing keeps
            the neighbouring page text from reading through. */}
        <div
          style={{
            position: 'absolute',
            left: card.x + 25,
            top: card.y + card.height + 34,
            width: 'fit-content',
            maxWidth: 1180,
            padding: '26px 34px 30px',
            borderRadius: 16,
            background: 'oklch(0.14 0.012 70 / 0.88)',
            opacity: annIn,
            transform: `translateY(${(1 - annIn) * 16}px)`,
          }}
        >
          <div
            style={{
              fontFamily: FONT,
              fontSize: 52,
              fontWeight: 700,
              letterSpacing: '-0.022em',
              color: C.inkInverse,
              lineHeight: 1.1,
            }}
          >
            Money owed, at a glance.
          </div>
          <div
            style={{
              marginTop: 12,
              height: 6,
              width: 210,
              background: C.inkInverse,
              opacity: 0.55,
              transform: `scaleX(${annRule})`,
              transformOrigin: 'left center',
            }}
          />
          <div
            style={{
              marginTop: 18,
              fontFamily: FONT,
              fontSize: 34,
              fontWeight: 400,
              color: `oklch(0.99 0.002 80 / 0.82)`,
            }}
          >
            Outstanding, overdue and collected — derived, never hand-set.
          </div>
        </div>
      </PageCam>
    </AbsoluteFill>
  );
};
