export function manaLeft(usedPercent: number): number {
  return Math.min(100, Math.max(0, 100 - usedPercent));
}

export function fmtCountdown(resetsAt: number | null, nowMs: number): string {
  if (resetsAt == null) return "";
  const s = Math.round(resetsAt - nowMs / 1000);
  if (s <= 0) return "now";
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  if (h > 0) return `${h}h ${m}m`;
  if (m > 0) return `${m}m`;
  return "<1m";
}

export function fmtAge(fetchedAt: number, nowMs: number): string {
  const m = Math.floor((nowMs / 1000 - fetchedAt) / 60);
  return m <= 0 ? "just now" : `${m}m ago`;
}

const PLAN_LABELS: Record<string, string> = {
  prolite: "Pro Lite",
  plus: "Plus",
  pro: "Pro",
  max: "Max",
  free: "Free",
  team: "Team",
  enterprise: "Enterprise",
};

export function planLabel(plan: string | null): string {
  if (!plan) return "";
  return PLAN_LABELS[plan.toLowerCase()] ?? plan;
}
