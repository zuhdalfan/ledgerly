import { Easing, interpolate } from "remotion";

/** The app's own --ease-out. Nothing in this film uses a different curve. */
export const EASE_OUT = Easing.bezier(0.22, 1, 0.36, 1);

export const progress = (frame: number, start: number, duration: number): number =>
  interpolate(frame, [start, start + duration], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
    easing: EASE_OUT,
  });

export const linearProgress = (frame: number, start: number, duration: number): number =>
  interpolate(frame, [start, start + duration], [0, 1], {
    extrapolateLeft: "clamp",
    extrapolateRight: "clamp",
  });

/** The app's entrance: 8px rise, no bounce. `distance` is in design pixels. */
export const enter = (frame: number, start = 0, distance = 8, duration = 12) => {
  const p = progress(frame, start, duration);
  return {
    opacity: p,
    transform: `translateY(${(1 - p) * distance}px)`,
  };
};

export const fadeIn = (frame: number, start: number, duration = 10) => ({
  opacity: progress(frame, start, duration),
});

export const fadeOut = (frame: number, start: number, duration = 8) => ({
  opacity: 1 - progress(frame, start, duration),
});

/** A short overshoot, used only for the paid check — never on a money table. */
export const pop = (frame: number, start: number, duration = 14): number => {
  const p = linearProgress(frame, start, duration);
  return 1 - Math.pow(1 - p, 3);
};
