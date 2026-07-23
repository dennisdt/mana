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

export function actionKind(p: Progress): "rank-up" | "prestige" | null {
  if (p.prestige_eligible) return "prestige";
  if (p.rank_up_eligible) return "rank-up";
  return null;
}

/** Mirror of the backend TIERS table; index = rank. */
export const TIERS = [
  "naked",
  "plastic",
  "wood",
  "iron",
  "bronze",
  "silver",
  "gold",
  "platinum",
  "emerald",
  "diamond",
  "master",
  "legend",
  "champion",
  "godlike",
];

const ROMAN_TENS = ["", "X", "XX"];
const ROMAN_ONES = ["", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX"];

/** Prestige counts stay small, so subtractive pairs beyond XX never render. */
export function romanNumeral(n: number): string {
  if (!Number.isInteger(n) || n < 1 || n > 20) return String(n);
  return ROMAN_TENS[Math.floor(n / 10)] + ROMAN_ONES[n % 10];
}

export function nextTier(p: Progress): string | null {
  return TIERS[p.rank + 1] ?? null;
}

export function dialogCopy(
  kind: "rank-up" | "prestige",
  p: Progress,
): { title: string; body: string; confirm: string } {
  if (kind === "prestige") {
    const n = p.prestige + 1;
    return {
      title: `PRESTIGE ${romanNumeral(n)}`,
      body: `The curve steepens. Begin again at Level 1 — Prestige ${n} badge is yours forever.`,
      confirm: "Prestige",
    };
  }
  const next = nextTier(p) ?? p.tier;
  return {
    title: `ASCEND TO ${tierDisplayName(next).toUpperCase()}`,
    body: `Level ${p.level} · ${tierDisplayName(p.tier)} → ${tierDisplayName(next)}`,
    confirm: "Rank Up",
  };
}
