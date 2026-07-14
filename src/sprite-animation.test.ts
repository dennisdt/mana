import { describe, expect, it } from "vitest";
import mainSource from "./main.ts?raw";
import { SPRITE_FRAME_DURATION_MS, spriteFrameAt } from "./sprite-animation";

describe("sprite frame timing", () => {
  it.each([
    ["idle", 575],
    ["working", 340],
    ["hover", 410],
  ] as const)("advances the %s row through four frames", (state, frameDuration) => {
    expect(SPRITE_FRAME_DURATION_MS[state]).toBe(frameDuration);
    expect(spriteFrameAt(0, state, false)).toBe(0);
    expect(spriteFrameAt(frameDuration - 0.001, state, false)).toBe(0);
    expect(spriteFrameAt(frameDuration, state, false)).toBe(1);
    expect(spriteFrameAt(frameDuration * 2, state, false)).toBe(2);
    expect(spriteFrameAt(frameDuration * 3, state, false)).toBe(3);
    expect(spriteFrameAt(frameDuration * 4, state, false)).toBe(0);
  });

  it("freezes every state on frame zero for reduced motion", () => {
    expect(spriteFrameAt(999, "idle", true)).toBe(0);
    expect(spriteFrameAt(999, "working", true)).toBe(0);
    expect(spriteFrameAt(999, "hover", true)).toBe(0);
  });

  it("uses idle timing for missing or invalid DOM state", () => {
    expect(spriteFrameAt(575, undefined, false)).toBe(1);
    expect(spriteFrameAt(575, "unknown", false)).toBe(1);
  });
});

describe("sprite animation runtime", () => {
  it("schedules DOM frame updates independently of CSS animation", () => {
    expect(mainSource).toContain(
      'window.matchMedia("(prefers-reduced-motion: reduce)")',
    );
    expect(mainSource).toContain("element.dataset.frame = frame");
    expect(mainSource).toContain("setInterval(updateSpriteFrames, SPRITE_TICK_MS)");
  });

  it("supports older WebKit motion preference listeners", () => {
    expect(mainSource).toContain(
      'typeof spriteMotionPreference.addEventListener === "function"',
    );
    expect(mainSource).toContain("spriteMotionPreference.addListener");
  });
});
