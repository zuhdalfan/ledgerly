// Workbench manifest — how the film decomposes into editable clips.
//
// The shotcraft workbench (node workbench/scripts/open.mjs <this project>) reads
// WORKBENCH and rebuilds the film as tracks: shots / transitions / captions /
// sound, each unit exactly where its <Sequence> sits in Main.tsx. Every table is
// imported from Main.tsx, so editing the timeline there moves the workbench too —
// there is no second copy of the timing to drift.
//
// Editability: title cards and narration expose their copy as props (with
// schemas, below). The live scenes expose timing and layer transforms; their
// in-shot annotations are still constants, so those are edited in code.
import React from 'react';
import { SHOTS, SHOT_AT, TOTAL_FRAMES, CAPTIONS, SFX, LedgerlyPromo } from './Main';
import { TitleCard } from './lib/TitleCard';
import { Narration } from './lib/ui';
import { FlashCut } from './lib/FlashCut';

type Field = Record<string, unknown> & { type: string; key: string; label: string; default: unknown };
const text = (key: string, label: string, def: string): Field => ({ type: 'text', key, label, default: def });
const textarea = (key: string, label: string, def: string): Field => ({ type: 'textarea', key, label, default: def });
const slider = (key: string, label: string, def: number, min: number, max: number): Field => ({
  type: 'slider',
  key,
  label,
  default: def,
  min,
  max,
  step: 1,
  unit: 'px',
});

// The copy for each title card, kept in one place next to the shot that renders it.
export const TITLE_COPY: Record<string, string> = {
  title1: 'Invoices that *add up*.',
  title2: 'Quote it. Then *invoice it*.',
};

type TitleProps = { text?: string; duration?: number };
const TitleCardUnit: React.FC<TitleProps> = ({ text: copy = '', duration = 50 }) =>
  React.createElement(TitleCard, { text: copy, duration });
TitleCardUnit.displayName = 'TitleCard';

type CaptionProps = { text?: string; duration?: number };
const CaptionUnit: React.FC<CaptionProps> = ({ text: copy = '', duration = 60 }) =>
  React.createElement(Narration, { text: copy, duration });
CaptionUnit.displayName = 'Narration';

const FlashUnit: React.FC<{ duration?: number }> = ({ duration = 10 }) =>
  React.createElement(FlashCut, { duration });
FlashUnit.displayName = 'FlashCut';

const TITLE_SCHEMA: Field[] = [textarea('text', 'Copy (*word* = emphasis)', ''), slider('duration', 'Duration', 50, 20, 200)];
const CAPTION_SCHEMA: Field[] = [textarea('text', 'Narration', ''), slider('duration', 'Duration', 60, 20, 300)];

/** Flash cuts are derived from the shots, mirroring Main.tsx exactly. */
export const FLASH_CUTS = SHOTS.filter((s) => s.flash).map((s) => SHOT_AT[s.id].from);

const ProjOriginal: React.FC = () => React.createElement(LedgerlyPromo, { bgm: true });
ProjOriginal.displayName = 'LedgerlyPromo';

export const WORKBENCH = {
  name: 'Ledgerly · promo',
  fps: 30,
  width: 1920,
  height: 1080,
  total: TOTAL_FRAMES,
  background: '#faf7f2',
  shots: SHOTS.map((s) => {
    const at = SHOT_AT[s.id];
    if (s.id === 'title1' || s.id === 'title2') {
      return {
        id: s.id,
        label: `${s.id} · title card`,
        from: at.from,
        duration: s.duration,
        component: TitleCardUnit as React.FC<Record<string, unknown>>,
        props: { text: TITLE_COPY[s.id], duration: s.duration },
        schema: TITLE_SCHEMA,
        durationProp: 'duration',
        cardId: 'title-card',
        cardName: 'Title card',
      };
    }
    return {
      id: s.id,
      label: `${s.id} · ${s.duration}f`,
      from: at.from,
      duration: s.duration,
      component: s.scene as React.FC<Record<string, unknown>>,
    };
  }),
  transitions: FLASH_CUTS.map((from, i) => ({
    id: `flash-${i + 1}`,
    label: `flash cut @${from}`,
    from: Math.max(0, from - 5),
    duration: 10,
    component: FlashUnit as React.FC<Record<string, unknown>>,
    durationProp: 'duration',
    cardId: 'flash-cut',
    cardName: 'Warm flash cut',
  })),
  captions: CAPTIONS.map((c, i) => {
    const at = SHOT_AT[c.shot];
    return {
      id: `caption-${i + 1}`,
      label: c.text,
      from: at.from + c.from,
      duration: c.duration,
      component: CaptionUnit as React.FC<Record<string, unknown>>,
      props: { text: c.text, duration: c.duration },
      schema: CAPTION_SCHEMA,
      durationProp: 'duration',
      cardId: 'caption',
      cardName: 'Narration',
    };
  }),
  sfx: SFX.map((s) => ({
    from: s.from,
    duration: s.durationInFrames ?? 90,
    src: `audio/${s.src}`,
    volume: s.volume,
  })),
  bgm: [{ from: 0, duration: TOTAL_FRAMES, src: 'audio/bgm-tech-house.mp3', volume: 0.34 }],
  order: ['transitions', 'captions', 'overlays'] as const,
  original: ProjOriginal as React.FC<Record<string, unknown>>,
};
