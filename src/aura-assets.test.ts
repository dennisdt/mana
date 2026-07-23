import { describe, expect, it } from "vitest";
import {
  auraBandForTier,
  resolveAura,
  type AuraBand,
  type Provider,
} from "./aura-assets";

describe("aura band resolution", () => {
  it.each([
    ["naked", null],
    ["plastic", null],
    ["wood", null],
    ["iron", null],
    ["bronze", "low"],
    ["silver", "low"],
    ["gold", "low"],
    ["platinum", "mid"],
    ["emerald", "mid"],
    ["diamond", "mid"],
    ["master", "high"],
    ["legend", "high"],
    ["champion", "high"],
    ["godlike", "high"],
  ] as const)("maps %s to %s", (tier, band) => {
    expect(auraBandForTier(tier)).toBe(band);
  });

  it("normalizes tier casing and surrounding whitespace", () => {
    expect(auraBandForTier("  BrOnZe ")).toBe("low");
    expect(auraBandForTier("PLATINUM")).toBe("mid");
    expect(auraBandForTier("GodLike")).toBe("high");
  });

  it("rejects unknown and non-string runtime tiers", () => {
    expect(auraBandForTier("mythic")).toBeNull();
    expect(auraBandForTier("")).toBeNull();
    expect(auraBandForTier(null as unknown as string)).toBeNull();
  });
});

describe("provider aura descriptors", () => {
  const expectations: readonly [
    Provider,
    AuraBand,
    2 | 4 | 8,
    readonly number[],
    number,
    number,
    number,
  ][] = [
    ["claude", "low", 2, [1_200, 1_600], 0, -4, 10],
    ["claude", "mid", 4, [700, 820, 760, 920], 0, -4, 10],
    ["claude", "high", 8, [352, 384, 384, 384, 384, 384, 384, 544], 0, -4, 10],
    ["codex", "low", 2, [1_425, 1_825], 1_380, -3, 8],
    ["codex", "mid", 4, [830, 960, 780, 1_080], 1_380, -3, 8],
    ["codex", "high", 8, [511, 401, 474, 401, 474, 401, 438, 550], 1_380, -3, 8],
  ];

  it.each(expectations)(
    "resolves %s %s to its authored atlas and timing",
    (provider, band, frameCount, holds, phase, x, y) => {
      const tier = band === "low" ? "bronze" : band === "mid" ? "platinum" : "master";
      const descriptor = resolveAura(provider, tier, 0);

      expect(descriptor).not.toBeNull();
      expect(descriptor).toEqual({
        provider,
        band,
        atlasUrl: `/effects/${provider}-aura-${band}.png`,
        frameCount,
        cellSizeCss: 96,
        frameHoldsMs: holds,
        phaseOffsetMs: phase,
        spriteXOffsetPx: x,
        spriteYOffsetPx: y,
      });
    },
  );

  it("preserves exact provider-specific high loop totals and phases", () => {
    const claude = resolveAura("claude", "godlike", 0)!;
    const codex = resolveAura("codex", "godlike", 0)!;

    expect(claude.frameHoldsMs.reduce((sum, hold) => sum + hold, 0)).toBe(3_200);
    expect(codex.frameHoldsMs.reduce((sum, hold) => sum + hold, 0)).toBe(3_650);
    expect(claude.phaseOffsetMs).toBe(0);
    expect(codex.phaseOffsetMs).toBe(1_380);
  });

  it("keeps low and mid motion slower and provider-specific", () => {
    const claudeLow = resolveAura("claude", "bronze", 0)!;
    const codexLow = resolveAura("codex", "bronze", 0)!;
    const claudeMid = resolveAura("claude", "platinum", 0)!;
    const codexMid = resolveAura("codex", "platinum", 0)!;

    expect(claudeLow.frameHoldsMs).not.toEqual(codexLow.frameHoldsMs);
    expect(claudeMid.frameHoldsMs).not.toEqual(codexMid.frameHoldsMs);
    expect(Math.min(...claudeLow.frameHoldsMs)).toBeGreaterThan(
      Math.max(...resolveAura("claude", "godlike", 0)!.frameHoldsMs),
    );
    expect(Math.min(...codexLow.frameHoldsMs)).toBeGreaterThan(
      Math.max(...resolveAura("codex", "godlike", 0)!.frameHoldsMs),
    );
  });

  it("upgrades Prestige VII through X to the high authored footprint", () => {
    for (const provider of ["claude", "codex"] as const) {
      expect(resolveAura(provider, "naked", 6)).toBeNull();
      expect(resolveAura(provider, "iron", 7)?.band).toBe("high");
      expect(resolveAura(provider, "bronze", 8)?.band).toBe("high");
      expect(resolveAura(provider, "platinum", 9)?.band).toBe("high");
      expect(resolveAura(provider, "naked", 10)?.band).toBe("high");
      expect(resolveAura(provider, "godlike", 0)?.band).toBe("high");
    }
  });

  it("ignores invalid prestige values instead of granting an aura", () => {
    for (const prestige of [-1, 7.5, Number.NaN, Number.POSITIVE_INFINITY]) {
      expect(resolveAura("claude", "naked", prestige)).toBeNull();
      expect(resolveAura("codex", "bronze", prestige)?.band).toBe("low");
    }
  });

  it("returns frozen fresh descriptors and hold arrays", () => {
    const first = resolveAura("claude", "master", 0)!;
    const second = resolveAura("claude", "master", 0)!;

    expect(first).not.toBe(second);
    expect(first.frameHoldsMs).not.toBe(second.frameHoldsMs);
    expect(first).toEqual(second);
    expect(Object.isFrozen(first)).toBe(true);
    expect(Object.isFrozen(first.frameHoldsMs)).toBe(true);
  });
});
