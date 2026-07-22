import { describe, expect, it, vi } from "vitest";
import tauriConfig from "../src-tauri/tauri.conf.json";
import {
  INITIAL_ROSTER_HEIGHT,
  MAX_SCALE,
  MIN_SCALE,
  ROSTER_WIDTH,
  createSerialQueue,
  DEFAULT_SCALE,
  restoreScale,
  rosterHeight,
  rosterOrigin,
  scaleForWidth,
  scaledRosterSize,
} from "./window-layout";

describe("permanent roster geometry", () => {
  it("uses the wider expanded roster from startup", () => {
    expect({ width: ROSTER_WIDTH, height: INITIAL_ROSTER_HEIGHT }).toEqual({
      width: 440,
      height: 175,
    });
    const mainWindow = tauriConfig.app.windows.find(
      ({ label }) => label === "main",
    );
    // The config window opens at the default scale so first paint matches
    // what the frontend immediately resizes to for a fresh install.
    expect(mainWindow).toMatchObject({
      width: Math.round(ROSTER_WIDTH * DEFAULT_SCALE),
      height: Math.ceil(INITIAL_ROSTER_HEIGHT * DEFAULT_SCALE),
    });
  });

  it("keeps the main webview rendering while the panel is inactive", () => {
    const mainWindow = tauriConfig.app.windows.find(
      ({ label }) => label === "main",
    );

    expect(mainWindow).toMatchObject({
      backgroundThrottling: "disabled",
    });
  });

  it("keeps an origin that fits the active work area", () => {
    expect(
      rosterOrigin(
        { x: 500, y: 80 },
        { width: ROSTER_WIDTH, height: 175 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 500, y: 80 });
  });

  it("clamps both axes for a saved compact position near the edge", () => {
    expect(
      rosterOrigin(
        { x: 2300, y: 1700 },
        { width: ROSTER_WIDTH, height: 210 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 2000, y: 1380 });
  });

  it("reclamps an old 420px right-edge position for the wider roster", () => {
    expect(
      rosterOrigin(
        { x: 2040, y: 80 },
        { width: ROSTER_WIDTH, height: 210 },
        { x: 0, y: 0, width: 2880, height: 1800 },
        2,
      ),
    ).toEqual({ x: 2000, y: 80 });
  });
});

it("rounds measured card height plus the root border", () => {
  expect(rosterHeight(207.2)).toBe(210);
});

describe("drag-resize scaling", () => {
  it("declares a resizable window bounded by the scale limits", () => {
    const mainWindow = tauriConfig.app.windows.find(
      ({ label }) => label === "main",
    );
    expect(mainWindow).toMatchObject({
      resizable: true,
      minWidth: ROSTER_WIDTH * MIN_SCALE,
      maxWidth: ROSTER_WIDTH * MAX_SCALE,
    });
  });

  it("derives scale from the dragged width against the base roster width", () => {
    expect(scaleForWidth(ROSTER_WIDTH)).toBe(1);
    expect(scaleForWidth(660)).toBe(1.5);
  });

  it("clamps scale between the minimum and maximum", () => {
    expect(scaleForWidth(100)).toBe(MIN_SCALE);
    expect(scaleForWidth(4400)).toBe(MAX_SCALE);
  });

  it("scales the roster size to whole logical pixels", () => {
    expect(scaledRosterSize(207.2, 1)).toEqual({ width: 440, height: 210 });
    expect(scaledRosterSize(207.2, 1.5)).toEqual({ width: 660, height: 315 });
    expect(scaledRosterSize(173, 0.5)).toEqual({ width: 220, height: 88 });
  });

  it("restores a persisted scale, clamped, defaulting to a comfortable 1.2", () => {
    expect(restoreScale("1.5")).toBe(1.5);
    expect(restoreScale("9")).toBe(MAX_SCALE);
    expect(restoreScale("0.1")).toBe(MIN_SCALE);
    expect(restoreScale(null)).toBe(DEFAULT_SCALE);
    expect(restoreScale("garbage")).toBe(DEFAULT_SCALE);
    expect(restoreScale("-2")).toBe(DEFAULT_SCALE);
    expect(DEFAULT_SCALE).toBe(1.2);
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
