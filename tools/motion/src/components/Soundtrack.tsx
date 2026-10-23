import type { FC } from "react";
import { Audio, Sequence, interpolate, staticFile, useVideoConfig } from "remotion";
import { MUSIC, voClip } from "../audio";
import { VO_LINES } from "../vo";
import type { Beat } from "../timeline";

/**
 * Music and voiceover, both optional.
 *
 * The bed is mixed down under each VO line rather than side-chained, so the
 * result is deterministic: the same frame always produces the same gain. Beat 5b
 * has no line by design, which is where the music is allowed back up — the
 * silence before the notification.
 */
export const Soundtrack: FC<{ beats: Beat[] }> = ({ beats }) => {
  const { durationInFrames } = useVideoConfig();
  const starts = new Map(beats.map((beat) => [beat.id, beat.from]));

  const speechWindows: Array<[number, number]> = [];
  for (const line of VO_LINES) {
    const start = starts.get(line.beat);
    if (start === undefined || !voClip(line.id)) continue;
    speechWindows.push([start + line.local, start + line.local + line.duration]);
  }

  return (
    <>
      {MUSIC ? (
        <Audio
          src={staticFile(MUSIC)}
          volume={(frame) => {
            const bed = interpolate(
              frame,
              [0, 15, durationInFrames - 45, durationInFrames],
              [0, 0.32, 0.32, 0],
              { extrapolateLeft: "clamp", extrapolateRight: "clamp" }
            );
            const speaking = speechWindows.some(
              ([from, to]) => frame >= from - 6 && frame <= to + 6
            );
            return bed * (speaking ? 0.42 : 1);
          }}
        />
      ) : null}

      {VO_LINES.map((line) => {
        const start = starts.get(line.beat);
        const clip = voClip(line.id);
        if (start === undefined || !clip) return null;
        const from = start + line.local;
        // Never let a line run past the composition: a shorter beat (the cutdown)
        // must clip the tail, not spill audio past the last frame.
        const available = durationInFrames - from;
        if (available <= 0) return null;
        return (
          <Sequence
            key={line.id}
            from={from}
            durationInFrames={Math.min(line.duration, available)}
            name={`vo:${line.id}`}
          >
            <Audio src={staticFile(clip)} />
          </Sequence>
        );
      })}
    </>
  );
};
