import { describe, expect, it } from "vitest";
import { fmtAge, fmtCountdown, manaLeft, planLabel } from "./format";

describe("manaLeft", () => {
  it("inverts used into remaining", () => {
    expect(manaLeft(26)).toBe(74);
  });
  it("clamps", () => {
    expect(manaLeft(120)).toBe(0);
    expect(manaLeft(-5)).toBe(100);
  });
});

describe("fmtCountdown", () => {
  const now = 1_783_712_399_000; // ms
  it("hours and minutes", () => {
    expect(fmtCountdown(1_783_712_399 + 2 * 3600 + 14 * 60, now)).toBe("2h 14m");
  });
  it("minutes only", () => {
    expect(fmtCountdown(1_783_712_399 + 14 * 60, now)).toBe("14m");
  });
  it("under a minute", () => {
    expect(fmtCountdown(1_783_712_399 + 30, now)).toBe("<1m");
  });
  it("past reset", () => {
    expect(fmtCountdown(1_783_712_399 - 10, now)).toBe("now");
  });
  it("unknown", () => {
    expect(fmtCountdown(null, now)).toBe("");
  });
});

describe("fmtAge", () => {
  const now = 1_783_712_399_000;
  it("fresh", () => {
    expect(fmtAge(1_783_712_399 - 30, now)).toBe("just now");
  });
  it("minutes", () => {
    expect(fmtAge(1_783_712_399 - 190, now)).toBe("3m ago");
  });
});

describe("planLabel", () => {
  it("maps known identifiers", () => {
    expect(planLabel("prolite")).toBe("Pro Lite");
    expect(planLabel("max")).toBe("Max");
    expect(planLabel("plus")).toBe("Plus");
    expect(planLabel("pro")).toBe("Pro");
  });
  it("falls back verbatim and handles null", () => {
    expect(planLabel("team_supreme")).toBe("team_supreme");
    expect(planLabel(null)).toBe("");
  });
});
