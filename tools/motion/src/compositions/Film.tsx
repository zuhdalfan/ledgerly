import type { FC } from "react";
import { AbsoluteFill, Sequence } from "remotion";
import { SCENES, type Beat } from "../timeline";
import { tokens } from "../theme";
import { Soundtrack } from "../components/Soundtrack";

/**
 * Renders a beat list. The master and the cutdown differ only in that list, and
 * `audio` is what makes a silent variant without a second edit.
 */
export const Film: FC<{ beats: Beat[]; audio?: boolean }> = ({ beats, audio = true }) => (
  <AbsoluteFill style={{ backgroundColor: tokens.paper1 }}>
    {audio ? <Soundtrack beats={beats} /> : null}

    {beats.map((beat) => {
      const Scene = SCENES[beat.id];
      return (
        <Sequence
          key={beat.id}
          from={beat.from}
          durationInFrames={beat.duration}
          name={beat.id}
        >
          <Scene headline={beat.headline} />
        </Sequence>
      );
    })}
  </AbsoluteFill>
);
