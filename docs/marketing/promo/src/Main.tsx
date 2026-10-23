// The film's single source of truth: shot order and durations. Every derived
// table (captions, SFX cues, flash cuts) reads its frames from here, so moving
// the timeline moves everything with it.
import React from 'react';
import { AbsoluteFill, Audio, Sequence, interpolate, staticFile } from 'remotion';
import { TitleCard } from './lib/TitleCard';
import { Narration, PaperBg } from './lib/ui';
import { FlashCut } from './lib/FlashCut';
import { Open, OPEN_DURATION } from './scenes/Open';
import { Hero, HERO_DURATION } from './scenes/Hero';
import { Invoice, INVOICE_DURATION } from './scenes/Invoice';
import { Find, FIND_DURATION } from './scenes/Find';
import { Estimate, ESTIMATE_DURATION } from './scenes/Estimate';
import { Reports, REPORTS_DURATION } from './scenes/Reports';
import { Offline, OFFLINE_DURATION } from './scenes/Offline';
import { Ai, AI_DURATION } from './scenes/Ai';
import { Outro, OUTRO_DURATION } from './scenes/Outro';

export type Shot = {
  id: string;
  scene: React.FC;
  duration: number;
  /** 0 = the shot fades out on its own; 1 = the next shot hard-cuts in. */
  flash: 0 | 1;
};

export const SHOTS: Shot[] = [
  { id: 'open', scene: Open, duration: OPEN_DURATION, flash: 0 },
  { id: 'hero', scene: Hero, duration: HERO_DURATION, flash: 1 },
  { id: 'title1', scene: TitleCardShot('Invoices that *add up*.', 50), duration: 50, flash: 0 },
  { id: 'invoice', scene: Invoice, duration: INVOICE_DURATION, flash: 0 },
  { id: 'find', scene: Find, duration: FIND_DURATION, flash: 0 },
  { id: 'title2', scene: TitleCardShot('Quote it. Then *invoice it*.', 50), duration: 50, flash: 0 },
  { id: 'estimate', scene: Estimate, duration: ESTIMATE_DURATION, flash: 1 },
  { id: 'reports', scene: Reports, duration: REPORTS_DURATION, flash: 0 },
  { id: 'offline', scene: Offline, duration: OFFLINE_DURATION, flash: 0 },
  { id: 'ai', scene: Ai, duration: AI_DURATION, flash: 0 },
  { id: 'outro', scene: Outro, duration: OUTRO_DURATION, flash: 1 },
];

function TitleCardShot(text: string, duration: number): React.FC {
  const C: React.FC = () => <TitleCard text={text} duration={duration} />;
  return C;
}

/** Absolute start frame of every shot. */
export const SHOT_AT: Record<string, { from: number; duration: number }> = (() => {
  const at: Record<string, { from: number; duration: number }> = {};
  let cursor = 0;
  for (const s of SHOTS) {
    at[s.id] = { from: cursor, duration: s.duration };
    cursor += s.duration;
  }
  return at;
})();

export const TOTAL_FRAMES = SHOTS.reduce((n, s) => n + s.duration, 0);

/** Narration. Frames are relative to each shot's start, so they ride along. */
export type Caption = { shot: string; from: number; duration: number; text: string };

export const CAPTIONS: Caption[] = [
  { shot: 'invoice', from: 34, duration: 104, text: 'Line items, tax and discounts reconcile to the cent.' },
  { shot: 'find', from: 72, duration: 66, text: 'Find any invoice by number or client.' },
  { shot: 'estimate', from: 46, duration: 84, text: 'One click turns an estimate into a linked invoice.' },
  { shot: 'reports', from: 22, duration: 96, text: 'Revenue, tax and receivables — exportable to CSV.' },
  { shot: 'ai', from: 56, duration: 50, text: 'Local, stdio-only — nothing is exposed on a port.' },
];

// ---------------------------------------------------------------------------
// Sound.
//
// Vocabulary is the film's (whoosh / impact / riser / sparkle / transition),
// plus foley only where the picture has an identifiable action of its own
// (typing, rows landing, a click) — rule S4. Everything is pinned to a shot
// start, never a bare frame number, so the timeline can move without re-pinning.
// ---------------------------------------------------------------------------

/**
 * Output-audio offset for THIS render pipeline (Remotion 4.0.484, AAC 48 kHz,
 * .mp4). Measured, not assumed — capture/measure_offset.py pins two isolated
 * sharp cues (click-camera at f512 and f672) and both land 1 frame early against
 * a provisional 1.28f, so the true offset is ~0.3f. Frame-level envelope probing
 * is ±1f, i.e. this pipeline's offset is at the floor of what the method can
 * resolve. Per-file peak lags were not separable from neighbouring cues at this
 * resolution; see DESIGN-SPEC §5 for the probe table and residuals.
 */
const OUTPUT_AUDIO_OFFSET_F = 0.3;

/** Per-file peak lag (frames) for samples whose energy peak is not at the head. */
const PEAK_F: Record<string, number> = {
  // all chosen samples are sharp attacks; measured peaks land at the head
};

const sfxFrom = (targetPeakF: number, src: string) =>
  Math.max(0, Math.round(targetPeakF - (PEAK_F[src] ?? 0) - OUTPUT_AUDIO_OFFSET_F));

const at = (shot: string, offset: number) => SHOT_AT[shot].from + offset;

export type Cue = {
  from: number;
  src: string;
  volume: number;
  durationInFrames?: number;
  note: string;
};

const cue = (shot: string, offset: number, src: string, volume: number, note: string, durationInFrames?: number): Cue => ({
  from: sfxFrom(at(shot, offset), src),
  src,
  volume,
  durationInFrames,
  note,
});

const OUTRO_LANDINGS = [17, 20, 23, 26, 29, 32, 35, 38];
const OUTRO_POPS = [0.4, 0.37, 0.34, 0.31, 0.28, 0.27, 0.26, 0.25];

export const SFX: Cue[] = [
  // 1 brand open
  cue('open', 6, 'transition-soft.mp3', 0.3, 'the slip draws itself on'),
  cue('open', 30, 'swoosh-quick.mp3', 0.26, 'wordmark letterpress'),
  cue('open', 66, 'keyboard.mp3', 0.3, 'tagline types on', 44),
  cue('open', 124, 'whoosh-fast.mp3', 0.4, 'brand → dashboard'),

  // 2 hero
  cue('hero', 6, 'transition-soft.mp3', 0.3, 'dashboard settles'),
  cue('hero', 96, 'whoosh-big.mp3', 0.5, 'the outstanding card pops up'),
  cue('hero', 112, 'sparkle.mp3', 0.32, 'contour sheen crosses the card'),
  cue('hero', 166, 'transition-snap.mp3', 0.5, 'card reseats into its slot'),

  // 3 title card
  cue('title1', 0, 'swoosh-quick.mp3', 0.28, 'title card 1'),

  // 4 totals engine
  cue('invoice', 8, 'whoosh-fast.mp3', 0.36, 'line item 1 flies in'),
  cue('invoice', 22, 'sweep-short.mp3', 0.3, 'line item 2 flies in'),
  cue('invoice', 38, 'transition-snap.mp3', 0.42, 'both rows embed'),
  cue('invoice', 108, 'transition-soft.mp3', 0.34, 'camera arrives at the totals'),
  cue('invoice', 122, 'click-camera.mp3', 0.5, 'the totals land'),

  // 5 find
  cue('find', 2, 'transition-soft.mp3', 0.28, 'into the invoice list'),
  cue('find', 34, 'keyboard.mp3', 0.42, 'typing the invoice number', 40),
  cue('find', 58, 'swoosh-quick.mp3', 0.24, 'typing ends'),
  cue('find', 96, 'whoosh-fast.mp3', 0.32, 'pull back to the list'),
  cue('find', 104, 'click-camera.mp3', 0.6, 'the one match is found'),

  // 6 title card
  cue('title2', 0, 'swoosh-quick.mp3', 0.28, 'title card 2'),

  // 7 estimate → invoice
  cue('estimate', 6, 'whoosh-fast.mp3', 0.36, 'estimate row 1 dealt in'),
  cue('estimate', 11, 'sweep-short.mp3', 0.33, 'estimate row 2 dealt in'),
  cue('estimate', 14, 'whoosh-fast.mp3', 0.3, 'estimate row 3 dealt in'),
  cue('estimate', 26, 'transition-snap.mp3', 0.34, 'the deal settles'),
  cue('estimate', 58, 'sweep-short.mp3', 0.28, 'the link is drawn'),
  cue('estimate', 64, 'transition-snap.mp3', 0.46, 'the invoice chip docks'),

  // 8 reports
  cue('reports', 2, 'transition-soft.mp3', 0.3, 'into the reports'),
  cue('reports', 30, 'transition-snap.mp3', 0.34, 'revenue total brackets in'),
  cue('reports', 90, 'whoosh-big.mp3', 0.3, 'long pan down the report'),

  // 9 the promise
  cue('offline', 2, 'transition-soft.mp3', 0.32, 'the promise appears'),
  cue('offline', 62, 'transition-snap.mp3', 0.34, 'the rule draws'),
  cue('offline', 76, 'sparkle.mp3', 0.3, 'residue'),

  // 10 AI over MCP
  cue('ai', 2, 'transition-soft.mp3', 0.3, 'into AI access'),
  cue('ai', 44, 'whoosh-fast.mp3', 0.3, 'push to the config card'),
  cue('ai', 64, 'click-camera.mp3', 0.4, 'the brackets lock'),

  // 11 sign-off — the fixed riser → impact → sparkle sentence
  cue('outro', 4, 'riser-cine.mp3', 0.4, 'roll-call assembles', 110),
  cue('outro', 50, 'impact-deep-whoosh.mp3', 0.55, 'wordmark stamps — film SFX peak'),
  cue('outro', 78, 'sparkle.mp3', 0.45, 'the rule and its residue'),
  ...OUTRO_LANDINGS.map((f, i) =>
    cue('outro', f, 'pop.mp3', OUTRO_POPS[i], `roll-call element ${i + 1} lands`),
  ),
];

export const BGM_TRACK = 'audio/bgm-tech-house.mp3';

export const LedgerlyPromo: React.FC<{ bgm: boolean }> = ({ bgm }) => {
  return (
    <AbsoluteFill style={{ backgroundColor: 'oklch(0.98 0.006 80)' }}>
      <PaperBg />

      {bgm && (
        <Audio
          src={staticFile(BGM_TRACK)}
          volume={(f) =>
            interpolate(f, [0, 30, TOTAL_FRAMES - 50, TOTAL_FRAMES], [0, 0.34, 0.34, 0], {
              extrapolateLeft: 'clamp',
              extrapolateRight: 'clamp',
            })
          }
        />
      )}

      {SHOTS.map((s) => {
        const Comp = s.scene;
        const at2 = SHOT_AT[s.id];
        return (
          <Sequence key={s.id} from={at2.from} durationInFrames={s.duration} name={s.id}>
            <Comp />
          </Sequence>
        );
      })}

      {/* Warm-white bloom over every hard cut (never a decorative sweep). */}
      {SHOTS.map((s) => {
        if (!s.flash) return null;
        const a = SHOT_AT[s.id];
        return (
          <Sequence key={`f-${s.id}`} from={Math.max(0, a.from - 5)} durationInFrames={10} name={`flash-${s.id}`}>
            <FlashCut duration={10} />
          </Sequence>
        );
      })}

      {CAPTIONS.map((c, i) => {
        const a = SHOT_AT[c.shot];
        if (!a) return null;
        return (
          <Sequence key={i} from={a.from + c.from} durationInFrames={c.duration} name={`caption-${i}`}>
            <Narration text={c.text} duration={c.duration} />
          </Sequence>
        );
      })}

      {SFX.map((s, i) => (
        <Sequence key={`sfx-${i}`} from={s.from} durationInFrames={s.durationInFrames ?? 90} name={`sfx-${i}`}>
          <Audio src={staticFile(`audio/${s.src}`)} volume={s.volume} />
        </Sequence>
      ))}
    </AbsoluteFill>
  );
};
