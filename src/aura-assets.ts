export type Provider = "claude" | "codex";
export type AuraBand = "low" | "mid" | "high";

export type AuraDescriptor = {
  provider: Provider;
  band: AuraBand;
  atlasUrl: string;
  frameCount: 2 | 4 | 8;
  cellSizeCss: 96;
  frameHoldsMs: readonly number[];
  phaseOffsetMs: number;
  spriteXOffsetPx: number;
  spriteYOffsetPx: number;
};

const TIER_BANDS: Readonly<Record<string, AuraBand>> = Object.freeze({
  bronze: "low",
  silver: "low",
  gold: "low",
  platinum: "mid",
  emerald: "mid",
  diamond: "mid",
  master: "high",
  legend: "high",
  champion: "high",
  godlike: "high",
});

const FRAME_HOLDS: Readonly<
  Record<Provider, Readonly<Record<AuraBand, readonly number[]>>>
> = Object.freeze({
  claude: Object.freeze({
    low: Object.freeze([1_200, 1_600]),
    mid: Object.freeze([700, 820, 760, 920]),
    high: Object.freeze([352, 384, 384, 384, 384, 384, 384, 544]),
  }),
  codex: Object.freeze({
    low: Object.freeze([1_425, 1_825]),
    mid: Object.freeze([830, 960, 780, 1_080]),
    high: Object.freeze([511, 401, 474, 401, 474, 401, 438, 550]),
  }),
});

const FRAME_COUNTS: Readonly<Record<AuraBand, 2 | 4 | 8>> = Object.freeze({
  low: 2,
  mid: 4,
  high: 8,
});

const PROVIDER_REGISTRATION = Object.freeze({
  claude: Object.freeze({ phase: 0, x: -4, y: 10 }),
  codex: Object.freeze({ phase: 1_380, x: -3, y: 8 }),
});

export function auraBandForTier(tier: string): AuraBand | null {
  if (typeof tier !== "string") return null;
  return TIER_BANDS[tier.trim().toLowerCase()] ?? null;
}

function normalizedPrestige(prestige: number): number {
  return Number.isSafeInteger(prestige) && prestige >= 0 ? prestige : 0;
}

export function resolveAura(
  provider: Provider,
  tier: string,
  prestige: number,
): AuraDescriptor | null {
  const band =
    normalizedPrestige(prestige) >= 7 ? "high" : auraBandForTier(tier);
  if (!band) return null;

  const registration = PROVIDER_REGISTRATION[provider];
  const frameHoldsMs = Object.freeze([...FRAME_HOLDS[provider][band]]);
  return Object.freeze({
    provider,
    band,
    atlasUrl: `/effects/${provider}-aura-${band}.png`,
    frameCount: FRAME_COUNTS[band],
    cellSizeCss: 96,
    frameHoldsMs,
    phaseOffsetMs: registration.phase,
    spriteXOffsetPx: registration.x,
    spriteYOffsetPx: registration.y,
  });
}
