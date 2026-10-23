import type { FC } from "react";
import { Composition } from "remotion";
import "./fonts";
import { Film } from "./compositions/Film";
import {
  CUTDOWN_BEATS,
  CUTDOWN_TOTAL,
  FPS,
  HEIGHT,
  MASTER_BEATS,
  MASTER_TOTAL,
  WIDTH,
} from "./timeline";

const Master: FC = () => <Film beats={MASTER_BEATS} />;
const MasterSilent: FC = () => <Film beats={MASTER_BEATS} audio={false} />;
const Vertical: FC = () => <Film beats={MASTER_BEATS} />;
const Square: FC = () => <Film beats={MASTER_BEATS} />;
const Cutdown: FC = () => <Film beats={CUTDOWN_BEATS} />;

/**
 * Five compositions over one scene set. Scenes lay themselves out at design
 * size and <Fit> scales them, so only the frame changes between these.
 *
 * MasterSilent exists because most placements autoplay muted; it is the same
 * edit with no music or voiceover, not a different cut.
 */
export const RemotionRoot: FC = () => (
  <>
    <Composition
      id="Master"
      component={Master}
      durationInFrames={MASTER_TOTAL}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
    />
    <Composition
      id="MasterSilent"
      component={MasterSilent}
      durationInFrames={MASTER_TOTAL}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
    />
    <Composition
      id="Vertical"
      component={Vertical}
      durationInFrames={MASTER_TOTAL}
      fps={FPS}
      width={1080}
      height={1920}
    />
    <Composition
      id="Square"
      component={Square}
      durationInFrames={MASTER_TOTAL}
      fps={FPS}
      width={1080}
      height={1080}
    />
    <Composition
      id="Cutdown"
      component={Cutdown}
      durationInFrames={CUTDOWN_TOTAL}
      fps={FPS}
      width={WIDTH}
      height={HEIGHT}
    />
  </>
);
