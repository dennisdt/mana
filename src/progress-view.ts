export type Progress = {
  xp: number;
  level: number;
  rank: number;
  tier: string;
  prestige: number;
  rank_up_eligible: boolean;
  prestige_eligible: boolean;
  level_progress: { current: number; needed: number };
};

/** Rank 0 has no material — it reads better as "Unranked" than "Naked". */
export function tierDisplayName(tier: string): string {
  if (tier === "naked") return "Unranked";
  return tier.charAt(0).toUpperCase() + tier.slice(1);
}

export function levelLabel(p: Progress): string {
  return `Lv ${p.level} · ${tierDisplayName(p.tier)}`;
}

/** needed <= 0 means the curve is exhausted (level cap) — show a full bar. */
export function xpBarFraction(p: Progress): number {
  const { current, needed } = p.level_progress;
  if (needed <= 0) return 1;
  return Math.min(1, Math.max(0, current / needed));
}

/** Badges 1-10 are bespoke; beyond 10 the tenth badge carries a count overlay. */
export function badgeSlots(prestige: number): number[] {
  return Array.from({ length: Math.min(prestige, 10) }, (_, i) => i + 1);
}

export function actionKind(p: Progress): "rank-up" | "prestige" | null {
  if (p.prestige_eligible) return "prestige";
  if (p.rank_up_eligible) return "rank-up";
  return null;
}
