import type { FC } from "react";
import { tokens } from "../theme";

// The app's mark: a torn paper slip. Copied from src/lib/components/Logo.svelte so
// the film draws the same artifact, not an approximation.
const SLIP =
  "M24 22A10 10 0 0 1 34 12H62A10 10 0 0 1 72 22V62L66 68 60 62 54 68 48 62 42 68 36 62 30 68 24 62Z";

export type LogoMarkProps = {
  size?: number;
  /** 0..1 — draws the two rules inside the slip. */
  draw?: number;
  color?: string;
  paper?: string;
};

export const LogoMark: FC<LogoMarkProps> = ({
  size = 96,
  draw = 1,
  color = tokens.ink1,
  paper = tokens.paper1,
}) => {
  const ruleLength = 24;
  const offset = ruleLength * (1 - draw);
  const scale = 0.94 + 0.06 * draw;

  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 96 96"
      style={{ display: "block", transform: `scale(${scale})`, transformOrigin: "center" }}
    >
      <path d={SLIP} fill={color} opacity={0.2 + 0.8 * draw} />
      <path
        d="M36 31H60"
        stroke={paper}
        strokeWidth={6}
        strokeLinecap="round"
        strokeDasharray={ruleLength}
        strokeDashoffset={offset}
      />
      <path
        d="M36 43H60"
        stroke={paper}
        strokeWidth={6}
        strokeLinecap="round"
        strokeDasharray={ruleLength}
        strokeDashoffset={offset}
      />
    </svg>
  );
};
