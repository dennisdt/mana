import { describe, expect, it } from "vitest";
import {
  actionKind,
  dialogCopy,
  levelLabel,
  nextTier,
  romanNumeral,
  tierDisplayName,
  xpBarFraction,
} from "./progress-view";

const base = {
  xp: 850,
  level: 10,
  rank: 5,
  tier: "silver",
  prestige: 0,
  rank_up_eligible: false,
  prestige_eligible: false,
  level_progress: { current: 50, needed: 125 },
};

describe("progress footer", () => {
  it("capitalizes tiers and renames naked", () => {
    expect(tierDisplayName("master")).toBe("Master");
    expect(tierDisplayName("naked")).toBe("Unranked");
  });
  it("labels level and tier", () => {
    expect(levelLabel(base)).toBe("Lv 10 · Silver");
    expect(levelLabel({ ...base, tier: "naked" })).toBe("Lv 10 · Unranked");
  });
  it("clamps the xp bar fraction", () => {
    expect(xpBarFraction(base)).toBeCloseTo(0.4);
    expect(xpBarFraction({ ...base, level_progress: { current: 200, needed: 125 } })).toBe(1);
    expect(xpBarFraction({ ...base, level_progress: { current: 1, needed: 0 } })).toBe(1);
  });
  it("picks the top-right action", () => {
    expect(actionKind(base)).toBeNull();
    expect(actionKind({ ...base, rank_up_eligible: true })).toBe("rank-up");
    expect(actionKind({ ...base, rank: 13, prestige_eligible: true })).toBe("prestige");
  });
});

describe("ceremony copy", () => {
  it("renders roman numerals up to twenty", () => {
    expect(romanNumeral(4)).toBe("IV");
    expect(romanNumeral(9)).toBe("IX");
    expect(romanNumeral(20)).toBe("XX");
    expect(romanNumeral(21)).toBe("21");
  });
  it("walks to the next tier and stops at godlike", () => {
    expect(nextTier({ ...base, rank: 0, tier: "naked" })).toBe("plastic");
    expect(nextTier({ ...base, rank: 13, tier: "godlike" })).toBeNull();
  });
  it("writes the rank-up ceremony", () => {
    const copy = dialogCopy("rank-up", { ...base, rank: 5, level: 36 });
    expect(copy.title).toBe("ASCEND TO GOLD");
    expect(copy.body).toBe("Level 36 · Silver → Gold");
    expect(copy.confirm).toBe("Rank Up");
  });
  it("writes the prestige ceremony", () => {
    const copy = dialogCopy("prestige", { ...base, rank: 13, tier: "godlike", prestige: 1 });
    expect(copy.title).toBe("PRESTIGE II");
    expect(copy.body).toBe(
      "The curve steepens. Surplus output carries forward into Prestige 2.",
    );
    expect(copy.confirm).toBe("Prestige");
  });
});
