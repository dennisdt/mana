import { probeImage } from "./cosmetics";

export const RANK_TIERS = [
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
] as const;

export type RankTier = (typeof RANK_TIERS)[number];
export type FrameSide = "top" | "right" | "bottom" | "left";
export type FrameCorner = "tl" | "tr" | "bl" | "br";

export type FramePieceSet = {
  key: string;
  rails: Record<FrameSide, string>;
  corners: Record<FrameCorner, string>;
  ornaments: Partial<Record<FrameSide, string>>;
  crestTop?: string;
};

export type ResolvedFrameDecoration = {
  requestedTier: RankTier;
  resolvedTier: RankTier;
  rank: FramePieceSet | null;
  prestige: FramePieceSet | null;
  /** @deprecated Compatibility metadata; prestige identity renders from crestTop. */
  prestigeText: string;
  diagnostics: string[];
};

type Probe = (url: string) => Promise<boolean>;

const BASE_PIECES = [
  "rail-h",
  "rail-v",
  "corner-tl",
  "corner-tr",
  "corner-bl",
  "corner-br",
] as const;

const RANK_EXTRAS: Partial<
  Record<RankTier, readonly ("crest-top" | "ornament-h" | "ornament-v")[]>
> = {
  gold: ["crest-top"],
  platinum: ["crest-top"],
  emerald: ["crest-top", "ornament-h", "ornament-v"],
  diamond: ["crest-top", "ornament-h", "ornament-v"],
  master: ["crest-top", "ornament-h", "ornament-v"],
  legend: ["crest-top", "ornament-h", "ornament-v"],
  champion: ["crest-top", "ornament-h", "ornament-v"],
  godlike: ["crest-top", "ornament-h", "ornament-v"],
};

const PRESTIGE_PIECES = [...BASE_PIECES, "crest-top"] as const;
const ROMAN_PRESTIGE = [
  "",
  "I",
  "II",
  "III",
  "IV",
  "V",
  "VI",
  "VII",
  "VIII",
  "IX",
  "X",
] as const;

function normalizeTier(tier: string): RankTier {
  const candidate = typeof tier === "string" ? tier.trim().toLowerCase() : "";
  return (RANK_TIERS as readonly string[]).includes(candidate)
    ? (candidate as RankTier)
    : "naked";
}

function normalizePrestige(prestige: number): number {
  return Number.isSafeInteger(prestige) && prestige > 0 ? prestige : 0;
}

function rankPieceSet(tier: Exclude<RankTier, "naked">): {
  model: FramePieceSet;
  required: string[];
} {
  const root = `/frames/ranks/${tier}`;
  const extras = RANK_EXTRAS[tier] ?? [];
  const ornamentH = extras.includes("ornament-h")
    ? `${root}/ornament-h.png`
    : undefined;
  const ornamentV = extras.includes("ornament-v")
    ? `${root}/ornament-v.png`
    : undefined;
  const crestTop = extras.includes("crest-top") ? `${root}/crest-top.png` : undefined;

  return {
    model: {
      key: `rank-${tier}`,
      rails: {
        top: `${root}/rail-h.png`,
        right: `${root}/rail-v.png`,
        bottom: `${root}/rail-h.png`,
        left: `${root}/rail-v.png`,
      },
      corners: {
        tl: `${root}/corner-tl.png`,
        tr: `${root}/corner-tr.png`,
        bl: `${root}/corner-bl.png`,
        br: `${root}/corner-br.png`,
      },
      ornaments: {
        ...(ornamentH ? { top: ornamentH, bottom: ornamentH } : {}),
        ...(ornamentV ? { right: ornamentV, left: ornamentV } : {}),
      },
      ...(crestTop ? { crestTop } : {}),
    },
    required: [...BASE_PIECES, ...extras].map((piece) => `${root}/${piece}.png`),
  };
}

function prestigePieceSet(level: number): {
  model: FramePieceSet;
  required: string[];
} {
  const root = `/frames/prestige/${level}`;
  return {
    model: {
      key: `prestige-${level}`,
      rails: {
        top: `${root}/rail-h.png`,
        right: `${root}/rail-v.png`,
        bottom: `${root}/rail-h.png`,
        left: `${root}/rail-v.png`,
      },
      corners: {
        tl: `${root}/corner-tl.png`,
        tr: `${root}/corner-tr.png`,
        bl: `${root}/corner-bl.png`,
        br: `${root}/corner-br.png`,
      },
      ornaments: {},
      crestTop: `${root}/crest-top.png`,
    },
    required: PRESTIGE_PIECES.map((piece) => `${root}/${piece}.png`),
  };
}

async function completeKit(paths: readonly string[], probe: Probe): Promise<boolean> {
  const results = await Promise.all(
    paths.map(async (path) => {
      try {
        return (await probe(path)) === true;
      } catch {
        return false;
      }
    }),
  );
  return results.every(Boolean);
}

/** @deprecated Compatibility formatter; prestige UI uses the generated crest. */
export function prestigeLabel(prestige: number): string {
  const normalized = normalizePrestige(prestige);
  if (normalized === 0) return "";
  return normalized <= 10 ? ROMAN_PRESTIGE[normalized] : `P${normalized}`;
}

export async function resolveFrameDecoration(
  tier: string,
  prestige: number,
  probe: Probe = probeImage,
): Promise<ResolvedFrameDecoration> {
  const requestedTier = normalizeTier(tier);
  const normalizedPrestige = normalizePrestige(prestige);
  const diagnostics: string[] = [];
  let resolvedTier: RankTier = "naked";
  let rank: FramePieceSet | null = null;

  for (let index = RANK_TIERS.indexOf(requestedTier); index >= 1; index -= 1) {
    const candidate = RANK_TIERS[index] as Exclude<RankTier, "naked">;
    const kit = rankPieceSet(candidate);
    if (await completeKit(kit.required, probe)) {
      resolvedTier = candidate;
      rank = kit.model;
      break;
    }
    diagnostics.push(`Rank frame "${candidate}" is incomplete.`);
  }

  let resolvedPrestige: FramePieceSet | null = null;
  for (let level = Math.min(normalizedPrestige, 10); level >= 1; level -= 1) {
    const kit = prestigePieceSet(level);
    if (await completeKit(kit.required, probe)) {
      resolvedPrestige = kit.model;
      break;
    }
    diagnostics.push(`Prestige frame "${level}" is incomplete.`);
  }

  return {
    requestedTier,
    resolvedTier,
    rank,
    prestige: resolvedPrestige,
    prestigeText: resolvedPrestige ? prestigeLabel(normalizedPrestige) : "",
    diagnostics,
  };
}
