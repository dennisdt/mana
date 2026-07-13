import { afterEach, describe, expect, it, vi } from "vitest";
import {
  COLLAPSE_DELAY_MS,
  collapsedOriginFromExpanded,
  createHoverIntent,
  createRequestRevision,
  createSerialQueue,
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

it("recovers the collapsed origin from a shifted expanded window", () => {
  expect(collapsedOriginFromExpanded({ x: 1_000, y: 80 }, -60)).toEqual({
    x: 1_060,
    y: 80,
  });
});

it("serializes work and continues after a rejected operation", async () => {
  const events: string[] = [];
  const errors: unknown[] = [];
  const enqueue = createSerialQueue((error) => errors.push(error));
  let releaseFirst: () => void = () => undefined;
  const firstBlocked = new Promise<void>((resolve) => {
    releaseFirst = resolve;
  });
  const first = enqueue(async () => {
    events.push("first:start");
    await firstBlocked;
    events.push("first:end");
    throw new Error("window call failed");
  });
  const second = enqueue(async () => {
    events.push("second");
  });

  await vi.waitFor(() => expect(events).toEqual(["first:start"]));
  releaseFirst();
  await Promise.all([first, second]);

  expect(events).toEqual(["first:start", "first:end", "second"]);
  expect(errors).toHaveLength(1);
});

it("distinguishes an older request from the latest intent", () => {
  const revisions = createRequestRevision();
  const first = revisions.issue();
  const latest = revisions.issue();

  expect(revisions.isCurrent(first)).toBe(false);
  expect(revisions.isCurrent(latest)).toBe(true);
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

it("defers collapse until movement settles", () => {
  vi.useFakeTimers();
  let moving = true;
  const expandedStates: boolean[] = [];
  const intent = createHoverIntent(
    () => undefined,
    (value) => expandedStates.push(value),
    () => moving,
  );

  intent.enter();
  intent.leave();
  vi.advanceTimersByTime(COLLAPSE_DELAY_MS);
  expect(expandedStates).toEqual([true]);

  moving = false;
  intent.movementSettled();
  expect(expandedStates).toEqual([true, false]);
});

it("cancels a movement-deferred collapse on re-entry", () => {
  vi.useFakeTimers();
  let moving = true;
  const expandedStates: boolean[] = [];
  const intent = createHoverIntent(
    () => undefined,
    (value) => expandedStates.push(value),
    () => moving,
  );

  intent.enter();
  intent.leave();
  vi.advanceTimersByTime(COLLAPSE_DELAY_MS);
  intent.enter();
  moving = false;
  intent.movementSettled();

  expect(expandedStates).toEqual([true, true]);
});
