import { describe, expect, it } from "vitest";
import mainSource from "./main.ts?raw";
import { resolveAura } from "./aura-assets";
import { auraFrameAt, auraFrameDelayAt } from "./aura-animation";

describe("irregular aura timing", () => {
  const claude = resolveAura("claude", "godlike", 0)!;
  const codex = resolveAura("codex", "godlike", 0)!;

  it("advances Claude on every exact authored boundary", () => {
    const boundaries = [0, 352, 736, 1_120, 1_504, 1_888, 2_272, 2_656];
    for (const [frame, boundary] of boundaries.entries()) {
      expect(auraFrameAt(boundary, claude, false), `frame ${frame}`).toBe(frame);
      expect(auraFrameDelayAt(boundary, claude, false)).toBe(
        claude.frameHoldsMs[frame],
      );
      if (boundary > 0) {
        expect(auraFrameAt(boundary - 0.001, claude, false)).toBe(frame - 1);
        expect(auraFrameDelayAt(boundary - 0.001, claude, false)).toBeCloseTo(
          0.001,
          8,
        );
      }
    }
    expect(auraFrameAt(3_200, claude, false)).toBe(0);
    expect(auraFrameDelayAt(3_200, claude, false)).toBe(352);
  });

  it("applies Codex's independent phase to frames and deadlines", () => {
    expect(auraFrameAt(0, codex, false)).toBe(2);
    expect(auraFrameDelayAt(0, codex, false)).toBe(6);
    expect(auraFrameAt(6, codex, false)).toBe(3);
    expect(auraFrameDelayAt(6, codex, false)).toBe(401);
    expect(auraFrameAt(2_270, codex, false)).toBe(0);
    expect(auraFrameDelayAt(2_270, codex, false)).toBe(511);
  });

  it("normalizes negative phase-adjusted elapsed time", () => {
    expect(auraFrameAt(-1, claude, false)).toBe(7);
    expect(auraFrameDelayAt(-1, claude, false)).toBe(1);
    expect(auraFrameAt(-1_380, codex, false)).toBe(0);
    expect(auraFrameDelayAt(-1_380, codex, false)).toBe(511);
  });

  it("keeps exact boundaries stable after very large elapsed times", () => {
    const loops = 1_000_000_000;
    expect(auraFrameAt(loops * 3_200 + 352, claude, false)).toBe(1);
    expect(auraFrameDelayAt(loops * 3_200 + 352, claude, false)).toBe(384);
    expect(auraFrameAt(loops * 3_650 - 1_380, codex, false)).toBe(0);
    expect(auraFrameDelayAt(loops * 3_650 - 1_380, codex, false)).toBe(511);
  });

  it("treats invalid elapsed values as time zero", () => {
    for (const elapsed of [Number.NaN, Number.POSITIVE_INFINITY, Number.NEGATIVE_INFINITY]) {
      expect(auraFrameAt(elapsed, claude, false)).toBe(0);
      expect(auraFrameDelayAt(elapsed, claude, false)).toBe(352);
      expect(auraFrameAt(elapsed, codex, false)).toBe(2);
      expect(auraFrameDelayAt(elapsed, codex, false)).toBe(6);
    }
  });

  it("freezes on quiet frame zero and returns no deadline under reduced motion", () => {
    expect(auraFrameAt(9_999, claude, true)).toBe(0);
    expect(auraFrameAt(9_999, codex, true)).toBe(0);
    expect(auraFrameDelayAt(9_999, claude, true)).toBeUndefined();
    expect(auraFrameDelayAt(9_999, codex, true)).toBeUndefined();
  });
});

describe("aura animation runtime", () => {
  it("owns one deadline timer and schedules the earliest visible aura", () => {
    expect(mainSource.match(/let auraFrameTimer:/g)).toHaveLength(1);
    expect(mainSource).toContain("Math.min(...delays)");
    expect(mainSource).toContain("setTimeout(runAuraFrameUpdate, delay)");
    expect(mainSource).not.toMatch(/setInterval\([^)]*Aura/i);
  });

  it("uses the shared clock without resetting provider phases after redraws", () => {
    expect(mainSource).toContain("auraFrameAt(now, descriptor,");
    expect(mainSource).toContain("auraFrameDelayAt(now, descriptor,");
    expect(mainSource).toMatch(/applyAuras\(\);\s*syncAuraFrames\(\);/s);
    expect(mainSource).toMatch(/renderProgress[\s\S]*applyAuras\(\);/);
    expect(mainSource).not.toMatch(/aura(?:Start|Epoch|PhaseStart)\s*=/);
  });

  it("stops aura scheduling when reduced motion is active", () => {
    expect(mainSource).toMatch(
      /if \(auraMotionPreference\.matches\) return;/,
    );
    expect(mainSource).toContain("clearTimeout(auraFrameTimer)");
  });
});
