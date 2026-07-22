import { describe, expect, it } from "vitest";
import { actionKind, badgeSlots, levelLabel, tierDisplayName, xpBarFraction } from "./progress-view";

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
  it("caps badge slots at ten", () => {
    expect(badgeSlots(0)).toEqual([]);
    expect(badgeSlots(3)).toEqual([1, 2, 3]);
    expect(badgeSlots(12)).toEqual([1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
  });
  it("picks the top-right action", () => {
    expect(actionKind(base)).toBeNull();
    expect(actionKind({ ...base, rank_up_eligible: true })).toBe("rank-up");
    expect(actionKind({ ...base, rank: 13, prestige_eligible: true })).toBe("prestige");
  });
});
