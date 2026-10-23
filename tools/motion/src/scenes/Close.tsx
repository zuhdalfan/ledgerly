import type { FC } from "react";
import { useCurrentFrame } from "remotion";
import { EndCard } from "../components/EndCard";
import { progress } from "../lib/anim";
import { SceneProps } from "../timeline";

/** Beat 8 — the promise, restated as three negatives. */
export const Close: FC<SceneProps> = ({ headline }) => {
  const frame = useCurrentFrame();
  return <EndCard tagline={headline} ledRing={progress(frame, 20, 40)} />;
};
