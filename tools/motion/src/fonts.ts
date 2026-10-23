// Fonts come from the repo, never a CDN — the same rule the app lives by.
import { loadFont } from "@remotion/fonts";
import { staticFile } from "remotion";

export const FONT = "Inter";

loadFont({
  family: FONT,
  url: staticFile("fonts/Inter-Regular.ttf"),
  weight: "400",
});

loadFont({
  family: FONT,
  url: staticFile("fonts/Inter-Bold.ttf"),
  weight: "700",
});
