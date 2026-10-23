import type { FC } from "react";
import { FONT } from "../fonts";
import { tokens } from "../theme";

export type Status =
  | "draft"
  | "sent"
  | "partially_paid"
  | "paid"
  | "overdue"
  | "void"
  | "accepted"
  | "declined"
  | "expired"
  | "converted";

const STYLE: Record<Status, { label: string; color: string }> = {
  draft: { label: "Draft", color: tokens.neutral },
  sent: { label: "Sent", color: tokens.info },
  partially_paid: { label: "Partially paid", color: tokens.warn },
  paid: { label: "Paid", color: tokens.ok },
  overdue: { label: "Overdue", color: tokens.danger },
  void: { label: "Void", color: tokens.neutral },
  accepted: { label: "Accepted", color: tokens.ok },
  declined: { label: "Declined", color: tokens.danger },
  expired: { label: "Expired", color: tokens.danger },
  converted: { label: "Converted", color: tokens.accentState },
};

/** Every badge carries its word, never colour alone — matching the app. */
export const StatusPill: FC<{
  status: Status;
  size?: number;
  opacity?: number;
  scale?: number;
}> = ({ status, size = 12, opacity = 1, scale = 1 }) => {
  const { label, color } = STYLE[status];
  return (
    <span
      style={{
        display: "inline-block",
        padding: `${size * 0.35}px ${size * 0.6}px`,
        border: `1px solid ${color}`,
        borderRadius: tokens.radiusFull,
        color,
        fontFamily: FONT,
        fontSize: size,
        fontWeight: 700,
        whiteSpace: "nowrap",
        opacity,
        transform: `scale(${scale})`,
        textDecoration: status === "void" ? "line-through" : "none",
      }}
    >
      {label}
    </span>
  );
};
