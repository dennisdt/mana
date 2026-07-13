import { afterEach, describe, expect, it, vi } from "vitest";
import {
  COLLAPSE_DELAY_MS,
  createHoverIntent,
  expandedHeight,
  expandedOrigin,
} from "./window-layout";

afterEach(() => vi.useRealTimers());

describe("expandedOrigin", () => {
  it("keeps an origin that fits the active work area", () => {
    expect(expandedOrigin({ x: 500, y: 80 }, { x: 0, y: 0, width: 2880, height: 1800 }, 2))
      .toEqual({ x: 500, y: 80 });
  });

  it("shifts left only enough to keep 420 logical pixels visible", () => {
    expect(expandedOrigin({ x: 2100, y: 80 }, { x: 0, y: 0, width: 2880, height: 1800 }, 2))
      .toEqual({ x: 2040, y: 80 });
  });
});

it("rounds measured card height plus the root border", () => {
  expect(expandedHeight(207.2)).toBe(210);
});

it("delays collapse and cancels it on re-entry", () => {
  vi.useFakeTimers();
  const hoverStates: boolean[] = [];
  const expandedStates: boolean[] = [];
  const intent = createHoverIntent(
    (value) => hoverStates.push(value),
    (value) => expandedStates.push(value),
  );
  intent.enter();
  intent.leave();
  vi.advanceTimersByTime(COLLAPSE_DELAY_MS - 1);
  expect(expandedStates).toEqual([true]);
  intent.enter();
  vi.advanceTimersByTime(COLLAPSE_DELAY_MS);
  expect(hoverStates).toEqual([true, false, true]);
  expect(expandedStates).toEqual([true, true]);
});
