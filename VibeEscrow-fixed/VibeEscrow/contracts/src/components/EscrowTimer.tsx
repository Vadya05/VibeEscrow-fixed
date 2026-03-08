import React from "react";

interface Props {
  seconds: number;
}

function formatTime(seconds: number): string {
  if (seconds <= 0) return "Expired";
  const d = Math.floor(seconds / 86400);
  const h = Math.floor((seconds % 86400) / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  const s = seconds % 60;

  if (d > 0) return `${d}d ${h}h ${m}m`;
  if (h > 0) return `${h}h ${m}m ${s}s`;
  return `${m}m ${s}s`;
}

export const EscrowTimer: React.FC<Props> = ({ seconds }) => (
  <div className={`timer ${seconds <= 0 ? "timer-expired" : ""}`}>
    <span className="timer-label">⏱ Time left</span>
    <span className="timer-value">{formatTime(seconds)}</span>
  </div>
);
